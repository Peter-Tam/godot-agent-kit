//! Public edit-v1 result: borrowed summaries, never source text or native handles.
use super::*;
use crate::script_edit::Stage as EditStage;
use crate::script_edit::*;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

macro_rules! enum_name {
    ($name:ident,$ty:ident,{$($variant:ident => $value:literal),+ $(,)?})=>{
        fn $name(value:$ty)->&'static str {match value {$($ty::$variant=>$value,)+}}
    };
}
enum_name!(outcome_name,EditOutcomeKind,{VerifiedChanged=>"verified_changed",VerifiedUnchanged=>"verified_unchanged",Refused=>"refused",AppliedUnverified=>"applied_unverified",ApplicationUnknown=>"application_unknown"});
enum_name!(application_name,Application,{NotApplied=>"not_applied",Applied=>"applied",PartlyApplied=>"partly_applied",Unknown=>"unknown"});
enum_name!(stage_name,EditStage,{Accepted=>"accepted",Selected=>"selected",Prepared=>"prepared",Preflight=>"preflight",Authorized=>"authorized",Applying=>"applying",Persisting=>"persisting",Finalizing=>"finalizing",Validating=>"validating",Verifying=>"verifying"});
enum_name!(reason_name,Reason,{Complete=>"complete",Busy=>"busy",DirtyConflict=>"dirty_conflict",RevisionMismatch=>"revision_mismatch",IdentityChanged=>"identity_changed",SessionChanged=>"session_changed",Divergence=>"divergence",KnownStaleResource=>"known_stale_resource",KnownStaleBuffer=>"known_stale_buffer",UnavailableObservation=>"unavailable_observation",ClosedTarget=>"closed_target",UnsupportedTarget=>"unsupported_target",AmbiguousTarget=>"ambiguous_target",DeniedAccess=>"denied_access",UnsupportedEngine=>"unsupported_engine",UnsupportedEffect=>"unsupported_effect",UnsupportedRepresentation=>"unsupported_representation",SaveWouldReformat=>"save_would_reformat",PersistenceFailure=>"persistence_failure",PersistenceUnknown=>"persistence_unknown",PartialFinalization=>"partial_finalization",ParseError=>"parse_error",DependencyError=>"dependency_error",ValidationUnavailable=>"validation_unavailable",EvidenceLimit=>"evidence_limit",Deadline=>"deadline",Cancellation=>"cancellation",Disconnection=>"disconnection",ProtocolFailure=>"protocol_failure",MissingBasis=>"missing_basis",IncompleteVerification=>"incomplete_verification"});
enum_name!(step_name,StepState,{NotStarted=>"not_started",Entered=>"entered",Completed=>"completed",Failed=>"failed",Unknown=>"unknown"});
enum_name!(history_name,History,{NotParticipated=>"not_participated",NativeComplexEdit=>"native_complex_edit",Unknown=>"unknown"});
enum_name!(finalization_name,FinalizationStatus,{Complete=>"complete",Rejected=>"rejected",PartialOrUnknown=>"partial_or_unknown"});
enum_name!(validation_name,ValidationStatus,{Valid=>"valid",Invalid=>"invalid",Unavailable=>"unavailable"});
enum_name!(purpose_name,ValidationPurpose,{Preflight=>"preflight",PostChange=>"post_change",Unchanged=>"unchanged"});
enum_name!(origin_name,DiagnosticOrigin,{Root=>"root",Dependency=>"dependency"});

// Keep field selection at this adapter without allocating a second JSON tree.
macro_rules! fields {
    ($serializer:expr, {$($key:literal => $value:expr),+ $(,)?}) => {{
        let mut map = $serializer.serialize_map(None)?;
        $(map.serialize_entry($key, &$value)?;)+
        map.end()
    }};
}
struct Hex<'a>(&'a [u8; 32]);
impl Serialize for Hex<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut text = [0; 64];
        for (n, byte) in self.0.iter().enumerate() {
            text[n * 2] = DIGITS[(byte >> 4) as usize];
            text[n * 2 + 1] = DIGITS[(byte & 15) as usize];
        }
        s.serialize_str(std::str::from_utf8(&text).map_err(serde::ser::Error::custom)?)
    }
}
struct Digest<'a>(&'a SourceDigest);
impl Serialize for Digest<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        fields!(s,{"sha256"=>Hex(&self.0.sha256),"utf8_bytes"=>self.0.utf8_bytes})
    }
}
struct StepRecord<'a>(&'a Step);
impl Serialize for StepRecord<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"state"=>step_name(v.state),"reason"=>v.reason.map(reason_name),"collection"=>v.collection.as_ref().map(stamp_out)})
    }
}
struct Progress<'a>(&'a AttemptProgress);
impl Serialize for Progress<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"buffer_application"=>StepRecord(&v.buffer_application),"resource_sync"=>StepRecord(&v.resource_sync),
            "persistence"=>StepRecord(&v.persistence),"finalization"=>StepRecord(&v.finalization),
            "validation"=>StepRecord(&v.validation),"verification"=>StepRecord(&v.verification)})
    }
}
struct InvalidSurface<'a>(&'a InvalidatedSurface);
impl Serialize for InvalidSurface<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"source_sha256"=>Hex(&v.source.sha256),"utf8_bytes"=>v.source.utf8_bytes,
            "collection"=>stamp_out(&v.collection),"identity"=>witness_out(&v.identity),
            "staleness"=>stale_out(&v.staleness),"reason"=>SourceReason(v.reason)})
    }
}
struct SurfaceRecord<'a>(&'a SurfaceEvidence);
impl Serialize for SurfaceRecord<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"authority"=>Authority(v.authority),"availability"=>Availability(v.availability),
            "source_sha256"=>v.source.as_ref().map(|x|Hex(&x.sha256)),"utf8_bytes"=>v.source.as_ref().map(|x|x.utf8_bytes),
            "collection"=>v.collection.as_ref().map(stamp_out),"identity"=>v.identity.as_ref().map(witness_out),
            "staleness"=>v.staleness.as_ref().map(stale_out),"reason"=>v.reason.map(SourceReason),
            "invalidated_evidence"=>v.invalidated.as_ref().map(InvalidSurface)})
    }
}
struct Saved<'a>(&'a SavedStateEvidence);
impl Serialize for Saved<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"request_id"=>v.request_id.as_str(),"session_id"=>v.session_id.as_str(),
            "document"=>identity_out(&v.document),"collection"=>stamp_out(&v.collection),
            "current_version"=>v.current_version.as_str(),"saved_version"=>v.saved_version.as_str(),
            "resource_edited"=>v.resource_edited,"save_profile"=>Hex(&v.save_profile),
            "original_preserved"=>v.original_preserved,"desired_preserved"=>v.desired_preserved})
    }
}
struct EditSources<'a>(&'a [SurfaceEvidence; 3]);
impl Serialize for EditSources<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        fields!(s,{"D"=>SurfaceRecord(&self.0[0]),"R"=>SurfaceRecord(&self.0[1]),"B"=>SurfaceRecord(&self.0[2])})
    }
}
struct Disk<'a>(&'a DiskMetadata);
impl Serialize for Disk<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"identity"=>file_out(&v.identity),"mtime"=>v.mtime.as_str(),"collection"=>stamp_out(&v.collection)})
    }
}
struct Evidence<'a>(&'a EditEvidence);
impl Serialize for Evidence<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        let comparisons = ComparisonsOut {
            disk_resource: Comparison(v.comparisons.disk_resource()),
            disk_buffer: Comparison(v.comparisons.disk_buffer()),
            resource_buffer: Comparison(v.comparisons.resource_buffer()),
        };
        let consistency = ConsistencyOut {
            atomic: v.consistency.atomic(),
            checks: Checks(v.consistency.checks()),
            stability: Stability(v.consistency.stability()),
            detected_changes: ChangesOut(v.consistency.detected_changes()),
            recheck_reason: v.consistency.recheck_reason().map(RecheckReason),
        };
        fields!(s,{"document"=>document_out(&v.document),"sources"=>EditSources(&v.sources),
            "dirty"=>dirty_out(&v.dirty),"saved_state"=>v.saved_state.as_ref().map(Saved),
            "saved_state_reason"=>v.saved_state_reason.map(reason_name),"comparisons"=>comparisons,
            "agreement"=>Agreement(v.agreement),"consistency"=>consistency,"disk_metadata"=>v.disk_metadata.as_ref().map(Disk)})
    }
}
struct Persistence<'a>(&'a PersistenceReceipt);
impl Serialize for Persistence<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"request_id"=>v.request_id.as_str(),"session_id"=>v.session_id.as_str(),
            "document"=>identity_out(&v.document),"intended"=>Digest(&v.intended),
            "project_id"=>file_out(&v.project_id),"file_id"=>file_out(&v.file_id),"collection"=>stamp_out(&v.collection),
            "write_started"=>v.write_started,"bytes_written"=>v.bytes_written,"truncated"=>v.truncated,"flushed"=>v.flushed,
            "readback_matches"=>v.readback_matches,"attached"=>v.attached,"descriptor_open"=>v.descriptor_open,
            "interference"=>v.interference,"original_mtime"=>v.original_mtime.as_ref().map(DecimalCounter::as_str),
            "mtime"=>v.mtime.as_ref().map(DecimalCounter::as_str),"restore_attempted"=>v.restore_attempted,
            "restore_errno"=>v.restore_errno,"restored"=>v.restored,"reason"=>v.reason.map(reason_name)})
    }
}
struct Finalization<'a>(&'a FinalizationResult);
impl Serialize for Finalization<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        #[derive(Serialize)]
        struct Steps {
            resource_edited: bool,
            saved_version: bool,
        }
        fields!(s,{"status"=>finalization_name(v.status),"reason"=>v.reason.map(reason_name),
            "request_id"=>v.request_id.as_str(),"session_id"=>v.session_id.as_str(),"document"=>identity_out(&v.document),
            "collection"=>stamp_out(&v.collection),"before_source"=>v.before_source.as_ref().map(Digest),
            "after_source"=>v.after_source.as_ref().map(Digest),"before_current"=>v.before_current.as_ref().map(DecimalCounter::as_str),
            "after_current"=>v.after_current.as_ref().map(DecimalCounter::as_str),"before_saved"=>v.before_saved.as_ref().map(DecimalCounter::as_str),
            "after_saved"=>v.after_saved.as_ref().map(DecimalCounter::as_str),"before_resource_edited"=>v.before_resource_edited,
            "after_resource_edited"=>v.after_resource_edited,"steps"=>Steps{resource_edited:v.steps.resource_edited,saved_version:v.steps.saved_version}})
    }
}
struct Dependencies<'a>(&'a [DependencyWitness]);
impl Serialize for Dependencies<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Dependency<'a> {
            path: &'a str,
            identity: FileOut<'a>,
            source: Digest<'a>,
        }
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&Dependency {
                path: v.path.as_str(),
                identity: file_out(&v.identity),
                source: Digest(&v.source),
            })?;
        }
        seq.end()
    }
}
struct Fences<'a>(&'a [ValidationSourceFence]);
impl Serialize for Fences<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Fence<'a> {
            path: &'a str,
            source: Digest<'a>,
            diagnostics_completed: bool,
            symbols_completed: bool,
        }
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&Fence {
                path: v.path.as_str(),
                source: Digest(&v.source),
                diagnostics_completed: v.diagnostics_completed,
                symbols_completed: v.symbols_completed,
            })?;
        }
        seq.end()
    }
}
struct ValidationDiagnostics<'a>(&'a [ValidationDiagnostic]);
impl Serialize for ValidationDiagnostics<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Diagnostic<'a> {
            origin: &'static str,
            path: Option<&'a str>,
            path_reason: Option<&'static str>,
            source: Option<Digest<'a>>,
            line: Option<u32>,
            column: Option<u32>,
            message: &'a str,
        }
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&Diagnostic {
                origin: origin_name(v.origin),
                path: v.path.as_ref().map(ResourcePath::as_str),
                path_reason: v.path_reason.map(reason_name),
                source: v.source.as_ref().map(Digest),
                line: v.line,
                column: v.column,
                message: &v.message,
            })?;
        }
        seq.end()
    }
}
struct Validation<'a>(&'a ValidationResult);
impl Serialize for Validation<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"status"=>validation_name(v.status),"reason"=>v.reason.map(reason_name),"purpose"=>purpose_name(v.purpose),
            "request_id"=>v.request_id.as_str(),"session_id"=>v.session_id.as_str(),"document"=>identity_out(&v.document),
            "source_path"=>v.source_path.as_str(),"input"=>Digest(&v.input),"collection"=>stamp_out(&v.collection),
            "context"=>Hex(&v.context),"context_current"=>v.context_current,"dependencies_current"=>v.dependencies_current,
            "cleanup_confirmed"=>v.cleanup_confirmed,"sources"=>Fences(&v.sources),"dependencies"=>Dependencies(&v.dependencies),
            "diagnostics"=>ValidationDiagnostics(&v.diagnostics)})
    }
}
struct Validations<'a>(&'a [ValidationResult]);
impl Serialize for Validations<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            seq.serialize_element(&Validation(value))?;
        }
        seq.end()
    }
}
struct Context<'a>(&'a ContextRecheck);
impl Serialize for Context<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        fields!(s,{"request_id"=>v.request_id.as_str(),"session_id"=>v.session_id.as_str(),"document"=>identity_out(&v.document),
            "collection"=>stamp_out(&v.collection),"context"=>Hex(&v.context),"current"=>v.current,"dependencies"=>Dependencies(&v.dependencies)})
    }
}
fn interval(value: &ObservationInterval) -> IntervalOut {
    IntervalOut {
        started_unix_ms: value.started_unix_ms(),
        finished_unix_ms: value.finished_unix_ms(),
        elapsed_us: value.elapsed_us(),
    }
}
struct Expected<'a>(&'a ExpectedRevisionBasis);
impl Serialize for Expected<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        let w = v.witnesses();
        let c = v.collections();
        fields!(s,{"prior_request_id"=>v.prior_request_id().as_str(),"prior_interval"=>interval(v.prior_interval()),
            "target"=>target_out(v.target()),"document"=>identity_out(v.document()),
            "witnesses"=>[witness_out(&w[0]),witness_out(&w[1]),witness_out(&w[2])],
            "collections"=>[stamp_out(&c[0]),stamp_out(&c[1]),stamp_out(&c[2])],
            "prior_document"=>document_out(v.prior_document()),"prior_dirty"=>dirty_out(v.prior_dirty()),
            "source"=>Digest(v.source()),"current_version"=>v.current_version().as_str()})
    }
}
struct EditDiagnostics<'a>(&'a [EditDiagnostic]);
impl Serialize for EditDiagnostics<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Diagnostic {
            stage: &'static str,
            reason: &'static str,
        }
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&Diagnostic {
                stage: stage_name(v.stage),
                reason: reason_name(v.reason),
            })?;
        }
        seq.end()
    }
}
struct Outcome<'a>(&'a EditOutcome);
impl Serialize for Outcome<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = self.0;
        #[derive(Serialize)]
        struct Requested<'a> {
            project_root: &'a str,
            session_id: Option<&'a str>,
            script_path: &'a str,
        }
        let requested = Requested {
            project_root: v.requested_target.0.as_str(),
            session_id: v.requested_target.1.as_ref().map(SessionId::as_str),
            script_path: v.requested_target.2.as_str(),
        };
        fields!(s,{"schema_version"=>v.schema_version(),"operation"=>v.operation(),"request_id"=>v.request_id.as_str(),
            "requested_target"=>requested,"resolved_target"=>v.resolved_target.as_ref().map(target_out),"interval"=>interval(&v.interval),
            "outcome"=>outcome_name(v.outcome),"reason"=>reason_name(v.reason),"stage"=>stage_name(v.stage),"application"=>application_name(v.application),
            "progress"=>Progress(&v.progress),"expected"=>v.expected.as_ref().map(Expected),"before"=>v.before.as_ref().map(Evidence),
            "after"=>v.after.as_ref().map(Evidence),"persistence"=>v.persistence.as_ref().map(Persistence),"finalization"=>v.finalization.as_ref().map(Finalization),
            "validation"=>Validations(&v.validation),"context_recheck"=>v.context_recheck.as_ref().map(Context),"history"=>history_name(v.history),
            "diagnostics"=>EditDiagnostics(&v.diagnostics),"selection"=>v.selection.as_ref().map(selection_out),"safe_next_action"=>v.safe_next_action})
    }
}
/// Serialize the reducer's bounded evidence without copying it into a JSON value tree.
///
/// # Errors
/// Returns a redacted protocol error if serialization or the result-size bound fails.
pub fn encode_outcome(value: &EditOutcome) -> Result<Vec<u8>, RoutingFailure> {
    let bytes = serde_json::to_vec(&Outcome(value))
        .map_err(|_| bad(crate::observation::Stage::Finalize))?;
    if bytes.len() > RESULT_LIMIT {
        return Err(bad(crate::observation::Stage::Finalize));
    }
    Ok(bytes)
}
pub fn exit_code(outcome: &EditOutcome) -> i32 {
    match outcome.outcome {
        EditOutcomeKind::VerifiedChanged | EditOutcomeKind::VerifiedUnchanged => 0,
        EditOutcomeKind::AppliedUnverified => 2,
        EditOutcomeKind::Refused
            if matches!(
                outcome.reason,
                Reason::Deadline
                    | Reason::Cancellation
                    | Reason::Disconnection
                    | Reason::ProtocolFailure
            ) =>
        {
            4
        }
        EditOutcomeKind::Refused => 3,
        EditOutcomeKind::ApplicationUnknown => 4,
    }
}
