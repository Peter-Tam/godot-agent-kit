use super::*;

// Public observation stamps are historical evidence, not newly received frames.
fn stamp(
    value: StampIn,
    session: &SessionId,
    elapsed: u64,
    editor: Option<bool>,
) -> Result<CollectionStamp, RoutingFailure> {
    let is_editor = value.clock_id.strip_prefix("editor:") == Some(session.as_str());
    if (!is_editor && value.clock_id != "caller")
        || editor.is_some_and(|expected| expected != is_editor)
        || value.received_elapsed_us > elapsed
    {
        return Err(bad(INPUT_STAGE));
    }
    CollectionStamp::new(
        if is_editor {
            ClockId::Editor(session.clone())
        } else {
            ClockId::Caller
        },
        decimal(value.started_tick_us, INPUT_STAGE)?,
        decimal(value.finished_tick_us, INPUT_STAGE)?,
        value.received_elapsed_us,
    )
    .map_err(|_| bad(INPUT_STAGE))
}
fn stale(
    value: StalenessIn,
    target: &ResolvedTarget,
    elapsed: u64,
) -> Result<Staleness, RoutingFailure> {
    match (value.state.as_str(), value.evidence) {
        ("unknown", None) => Ok(Staleness::unknown()),
        ("known_stale", Some(items)) => {
            let evidence = items
                .0
                .into_iter()
                .map(|e| {
                    StalenessEvidence::unapplied_change(
                        decimal(e.incorporated_version, INPUT_STAGE)?,
                        e.changed_authority,
                        decimal(e.changed_version, INPUT_STAGE)?,
                        stamp(
                            e.collection,
                            target.session_id(),
                            elapsed,
                            Some(e.changed_authority != Authority::D),
                        )?,
                        e.witness.domain(target.script_path(), INPUT_STAGE)?,
                    )
                    .map_err(|_| bad(INPUT_STAGE))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Staleness::known_stale(evidence).map_err(|_| bad(INPUT_STAGE))
        }
        _ => Err(bad(INPUT_STAGE)),
    }
}
pub(super) fn source(
    value: SourceIn,
    authority: Authority,
    target: &ResolvedTarget,
    elapsed: u64,
) -> Result<SourceObservation, RoutingFailure> {
    if value.authority != authority {
        return Err(bad(INPUT_STAGE));
    }
    let observed = |text, collection, witness: WitnessIn, staleness| {
        let TextIn::Within(text) = text else {
            return Err(bad(INPUT_STAGE));
        };
        Ok(SourceObservation::observed(
            authority,
            text,
            stamp(
                collection,
                target.session_id(),
                elapsed,
                Some(authority != Authority::D),
            )?,
            witness.domain(target.script_path(), INPUT_STAGE)?,
            stale(staleness, target, elapsed)?,
        ))
    };
    match value.availability {
        AvailabilityIn::Observed
            if value.reason.is_none() && value.invalidated_evidence.is_none() =>
        {
            observed(
                value.text.ok_or_else(|| bad(INPUT_STAGE))?,
                value.collection.ok_or_else(|| bad(INPUT_STAGE))?,
                value.witness.ok_or_else(|| bad(INPUT_STAGE))?,
                value.staleness.ok_or_else(|| bad(INPUT_STAGE))?,
            )
        }
        AvailabilityIn::Unavailable
            if value.text.is_none()
                && value.collection.is_none()
                && value.witness.is_none()
                && value.staleness.is_none() =>
        {
            let reason = source_reason(value.reason.ok_or_else(|| bad(INPUT_STAGE))?, INPUT_STAGE)?;
            if let Some(old) = value.invalidated_evidence {
                let old_reason = source_reason(old.reason, INPUT_STAGE)?;
                let mut source = observed(old.text, old.collection, old.witness, old.staleness)?;
                source
                    .invalidate(old_reason)
                    .map_err(|_| bad(INPUT_STAGE))?;
                source.invalidate(reason).map_err(|_| bad(INPUT_STAGE))?;
                Ok(source)
            } else {
                SourceObservation::unavailable(authority, reason).map_err(|_| bad(INPUT_STAGE))
            }
        }
        AvailabilityIn::NotApplicable
            if authority == Authority::B
                && value.text.is_none()
                && value.collection.is_none()
                && value.witness.is_none()
                && value.staleness.is_none()
                && value.invalidated_evidence.is_none()
                && source_reason(value.reason.ok_or_else(|| bad(INPUT_STAGE))?, INPUT_STAGE)?
                    == SourceReason::DocumentNotOpen =>
        {
            Ok(SourceObservation::closed_buffer())
        }
        _ => Err(bad(INPUT_STAGE)),
    }
}
fn fact<T>(
    value: FactIn<T>,
    session: &SessionId,
    elapsed: u64,
    editor: Option<bool>,
) -> Result<DocumentFact<T>, RoutingFailure> {
    match (
        value.value,
        value.collection,
        value.reason,
        value.invalidated_evidence,
    ) {
        (Some(v), Some(collection), None, None) => Ok(DocumentFact::observed(
            v,
            stamp(collection, session, elapsed, editor)?,
        )),
        (None, None, Some(reason), old) if reason.action == reason.code.action() => {
            if let Some(old) = old {
                if old.reason.action != old.reason.code.action() {
                    return Err(bad(INPUT_STAGE));
                }
                let mut fact = DocumentFact::observed(
                    old.value,
                    stamp(old.collection, session, elapsed, editor)?,
                );
                fact.invalidate(old.reason.code);
                fact.invalidate(reason.code);
                Ok(fact)
            } else {
                Ok(DocumentFact::unknown(reason.code))
            }
        }
        _ => Err(bad(INPUT_STAGE)),
    }
}
pub(super) fn document(
    value: DocumentIn,
    target: &ResolvedTarget,
    elapsed: u64,
) -> Result<DocumentState, RoutingFailure> {
    Ok(DocumentState::new(
        value
            .identity
            .map(|v| v.domain(target.script_path(), INPUT_STAGE))
            .transpose()?,
        fact(value.validity, target.session_id(), elapsed, None)?,
        fact(value.open_state, target.session_id(), elapsed, Some(true))?,
    ))
}
pub(super) fn dirty(
    value: DirtyIn,
    target: &ResolvedTarget,
    elapsed: u64,
) -> Result<DirtyObservation, RoutingFailure> {
    let observed = |state, collection, witness: WitnessIn| {
        Ok(DirtyObservation::observed(
            state,
            stamp(collection, target.session_id(), elapsed, Some(true))?,
            witness.domain(target.script_path(), INPUT_STAGE)?,
        ))
    };
    match value.availability {
        AvailabilityIn::Observed
            if value.reason.is_none() && value.invalidated_evidence.is_none() =>
        {
            let state = match value.state.as_str() {
                "dirty" => DirtyState::Dirty,
                "clean" => DirtyState::Clean,
                _ => return Err(bad(INPUT_STAGE)),
            };
            observed(
                state,
                value.collection.ok_or_else(|| bad(INPUT_STAGE))?,
                value.witness.ok_or_else(|| bad(INPUT_STAGE))?,
            )
        }
        AvailabilityIn::Unavailable
            if value.state == "unknown"
                && value.collection.is_none()
                && value.witness.is_none() =>
        {
            let reason = value.reason.ok_or_else(|| bad(INPUT_STAGE))?;
            if reason.action != reason.code.action() {
                return Err(bad(INPUT_STAGE));
            }
            if let Some(old) = value.invalidated_evidence {
                if old.reason.action != old.reason.code.action() {
                    return Err(bad(INPUT_STAGE));
                }
                let mut dirty = observed(old.state, old.collection, old.witness)?;
                dirty
                    .invalidate(old.reason.code)
                    .map_err(|_| bad(INPUT_STAGE))?;
                dirty
                    .invalidate(reason.code)
                    .map_err(|_| bad(INPUT_STAGE))?;
                Ok(dirty)
            } else {
                DirtyObservation::unavailable(reason.code).map_err(|_| bad(INPUT_STAGE))
            }
        }
        AvailabilityIn::NotApplicable
            if value.state == "not_applicable"
                && value.collection.is_none()
                && value.witness.is_none()
                && value.invalidated_evidence.is_none()
                && value.reason.is_some_and(|r| {
                    r.code == DirtyReason::DocumentNotOpen && r.action == r.code.action()
                }) =>
        {
            Ok(DirtyObservation::closed_document())
        }
        _ => Err(bad(INPUT_STAGE)),
    }
}
