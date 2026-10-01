use super::*;
use crate::script_discovery::events::{ContextReason, ContextStatus, ScopeFacts};
fn request() -> DiscoveryRequest {
    DiscoveryRequest::new(
        RequestId::new("discovery-test").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new("00112233445566778899aabbccddeeff").unwrap()),
    )
}
fn target(r: &DiscoveryRequest) -> DiscoveryTarget {
    DiscoveryTarget::for_request(
        r,
        r.project_root().clone(),
        FileIdentity::new(
            crate::observation::DecimalCounter::new("1").unwrap(),
            crate::observation::DecimalCounter::new("2").unwrap(),
        ),
        r.session_id().unwrap().clone(),
        EngineVersion::new(crate::bridge::GODOT_VERSION, crate::bridge::ENGINE_HASH).unwrap(),
    )
    .unwrap()
}
fn context(receipt: u64, start: u64, end: u64) -> Context {
    Context {
        collection: Stamp {
            clock_id: "editor:00112233445566778899aabbccddeeff".into(),
            started_tick_us: start.to_string(),
            finished_tick_us: end.to_string(),
            received_elapsed_us: receipt,
        },
        status: ContextStatus::Observed,
        reason: None,
        context: Some(ScopeFacts {
            policy: "godot_project_files_v1".into(),
            project_data_directory: "res://.godot".into(),
            filesystem_epoch: "0".into(),
            scanning: false,
            importing: false,
        }),
        expiry_tick_us: Some("9000000".into()),
    }
}
fn scoped() -> (Attempt, Binding) {
    let r = request();
    let t = target(&r);
    let b = Binding::target(&t);
    let mut a = Attempt::new(r.clone());
    a.accept(
        Event::Selected {
            binding: Binding::request(&r),
            target: t,
            capability: true,
        },
        10,
    )
    .unwrap();
    a.accept(
        Event::Scope {
            binding: b.clone(),
            context: context(20, 100, 110),
        },
        30,
    )
    .unwrap();
    (a, b)
}
fn batch(b: &Binding, ordinal: u32, paths: &[&str], count: u32) -> Event {
    Event::Entries {
        binding: b.clone(),
        ordinal,
        collection: Stamp::caller(
            20 + ordinal as u64 * 10,
            25 + ordinal as u64 * 10,
            25 + ordinal as u64 * 10,
        ),
        entries: paths.iter().map(|s| (*s).into()).collect(),
        visited_entries: count,
        visited_directories: 1,
    }
}
fn recheck(b: &Binding, count: u32, dirs: u32, exhausted: bool, gaps: Vec<Gap>) -> Event {
    Event::Rechecked {
        binding: b.clone(),
        context: context(500, 120, 130),
        exhausted,
        namespace_checked: true,
        changed: false,
        visited_entries: count,
        visited_directories: dirs,
        gaps,
        omitted_gaps: 0,
    }
}
fn close(mut a: Attempt, b: &Binding) -> DiscoveryOutcome {
    a.accept(Event::Done { binding: b.clone() }, 600).unwrap();
    a.finish(ObservationInterval::new(1000, 1001, 700))
}
#[test]
fn true_empty_requires_exhaustion_and_both_rechecks() {
    let (mut a, b) = scoped();
    a.accept(recheck(&b, 0, 1, true, vec![]), 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "complete_listing");
    let i = o.inventory.unwrap();
    assert!(i.entries.is_empty());
    assert_eq!(i.coverage, "complete");
    assert_eq!(i.consistency.recheck, "completed");
    assert!(!i.consistency.atomic);
    let (a, _) = scoped();
    let o = a.finish(ObservationInterval::new(0, 1, 700));
    assert_eq!(o.outcome(), "limited_listing");
    assert_ne!(o.inventory.unwrap().coverage, "complete");
    let (mut a, b) = scoped();
    a.accept(recheck(&b, 0, 1, false, vec![]), 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    assert_eq!(o.inventory.unwrap().coverage, "partial");
}
#[test]
fn parent_receipt_never_compares_against_worker_clock_start() {
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://é.GD"], 1), 300).unwrap();
    a.accept(recheck(&b, 1, 1, true, vec![]), 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "complete_listing");
    let i = o.inventory.unwrap();
    assert_eq!(i.collection.started_tick_us, "20");
    assert_eq!(i.collection.finished_tick_us, "500");
    assert_eq!(i.collection.received_elapsed_us, 500);
}
#[test]
fn local_gap_retains_sorted_exact_names_and_never_deduplicates_by_inode() {
    let (mut a, b) = scoped();
    a.accept(
        batch(
            &b,
            0,
            &[
                "res://z.GD",
                "res://a.gd",
                "res://space name.gd",
                "res://café.gd",
            ],
            4,
        ),
        40,
    )
    .unwrap();
    a.accept(
        recheck(
            &b,
            4,
            1,
            true,
            vec![Gap {
                code: Reason::DirectoryUnreadable,
                stage: Stage::Enumerate,
                scope: Some("res://private".into()),
            }],
        ),
        500,
    )
    .unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    let i = o.inventory.unwrap();
    assert_eq!(
        i.entries,
        vec![
            "res://a.gd",
            "res://café.gd",
            "res://space name.gd",
            "res://z.GD"
        ]
    );
    assert_eq!(i.coverage, "partial");
}
#[test]
fn cancelled_and_protocol_loss_preserve_only_earlier_checked_evidence() {
    for reason in [
        Reason::Cancelled,
        Reason::Timeout,
        Reason::DisconnectedEditor,
        Reason::ProtocolError,
    ] {
        let (mut a, b) = scoped();
        a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
        a.fail(reason);
        let o = a.finish(ObservationInterval::new(0, 1, 700));
        assert_eq!(o.outcome(), "interrupted");
        let i = o.inventory.unwrap();
        assert_eq!(i.entries, vec!["res://safe.gd"]);
        assert_eq!(i.validity, "earlier_observation");
        assert_eq!(i.consistency.recheck, "unavailable");
        assert_ne!(i.coverage, "complete");
    }
}
#[test]
fn request_wide_denial_and_replacement_suppress_paths_and_scopes() {
    for reason in [
        Reason::DeniedAccess,
        Reason::UnsafeRegistry,
        Reason::AuthenticationFailed,
        Reason::OutOfProject,
        Reason::ProjectIdentityChanged,
        Reason::SessionReplaced,
        Reason::ScopeChanged,
    ] {
        let (mut a, b) = scoped();
        a.accept(batch(&b, 0, &["res://private-sentinel.gd"], 1), 40)
            .unwrap();
        a.fail(reason);
        let o = a.finish(ObservationInterval::new(0, 1, 700));
        assert!(o.inventory.is_none());
        let json = String::from_utf8(o.to_json_line().unwrap()).unwrap();
        assert!(!json.contains("private-sentinel"));
        assert!(o.diagnostics.iter().all(|d| d.scope.is_none()));
    }
}
#[test]
fn simultaneous_failure_precedence_is_independent_of_arrival_order() {
    let reasons = [
        Reason::Timeout,
        Reason::DisconnectedEditor,
        Reason::SessionReplaced,
        Reason::Cancelled,
        Reason::ProtocolError,
        Reason::UnsupportedCapability,
        Reason::AmbiguousTarget,
        Reason::DeniedAccess,
    ];
    for reverse in [false, true] {
        let (mut a, _) = scoped();
        let iter: Box<dyn Iterator<Item = Reason>> = if reverse {
            Box::new(reasons.into_iter().rev())
        } else {
            Box::new(reasons.into_iter())
        };
        for reason in iter {
            a.fail(reason);
        }
        let o = a.finish(ObservationInterval::new(0, 1, 700));
        assert_eq!(o.outcome(), "refused");
        assert_eq!(o.diagnostics.len(), 8);
        assert!(o.inventory.is_none());
        assert!(o.resolved_target.is_none());
    }
}
#[test]
fn invalidated_subtree_is_removed_and_cannot_be_resurrected() {
    let (mut a, b) = scoped();
    a.accept(
        batch(&b, 0, &["res://gone/x.gd", "res://gone_elsewhere.gd"], 2),
        40,
    )
    .unwrap();
    a.accept(
        Event::Invalidate {
            binding: b.clone(),
            reason: Reason::NamespaceChanged,
            prefix: Some("res://gone".into()),
        },
        45,
    )
    .unwrap();
    a.accept(recheck(&b, 2, 1, true, vec![]), 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    let i = o.inventory.unwrap();
    assert_eq!(i.entries, vec!["res://gone_elsewhere.gd"]);
    assert_eq!(i.consistency.stability, "changed");
    let (mut a, b) = scoped();
    a.accept(
        Event::Invalidate {
            binding: b.clone(),
            reason: Reason::NamespaceChanged,
            prefix: Some("res://gone".into()),
        },
        35,
    )
    .unwrap();
    assert_eq!(
        a.accept(batch(&b, 0, &["res://gone/x.gd"], 1), 40),
        Err(Reason::ProtocolError)
    );
    assert!(a
        .finish(ObservationInterval::new(0, 1, 700))
        .inventory
        .unwrap()
        .entries
        .is_empty());
}
#[test]
fn every_event_requires_state_order_and_immutable_binding() {
    let r = request();
    let t = target(&r);
    let b = Binding::target(&t);
    let mut a = Attempt::new(r.clone());
    assert_eq!(
        a.accept(batch(&b, 0, &["res://injected.gd"], 1), 40),
        Err(Reason::ProtocolError)
    );
    assert!(a
        .finish(ObservationInterval::new(0, 1, 700))
        .inventory
        .is_none());
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
    assert_eq!(
        a.accept(batch(&b, 0, &["res://injected.gd"], 2), 50),
        Err(Reason::ProtocolError)
    );
    assert_eq!(
        a.finish(ObservationInterval::new(0, 1, 700))
            .inventory
            .unwrap()
            .entries,
        vec!["res://safe.gd"]
    );
    for field in 0..4 {
        let (mut a, mut b) = scoped();
        a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
        match field {
            0 => b.request_id = "foreign".into(),
            1 => b.session_id = Some("ffeeddccbbaa99887766554433221100".into()),
            2 => b.project_root = "/other".into(),
            _ => b.project_file_id.as_mut().unwrap().inode = "3".into(),
        };
        assert_eq!(
            a.accept(batch(&b, 1, &["res://injected.gd"], 2), 50),
            Err(Reason::ProtocolError)
        );
        let o = a.finish(ObservationInterval::new(0, 1, 700));
        assert_eq!(o.outcome(), "interrupted");
        if field == 0 {
            assert_eq!(o.inventory.unwrap().entries, vec!["res://safe.gd"]);
        } else {
            assert!(o.inventory.is_none());
        }
    }
}
#[test]
fn no_late_evidence_or_post_terminal_upgrade() {
    let (mut a, b) = scoped();
    assert_eq!(
        a.accept(batch(&b, 0, &["res://late.gd"], 1), CUTOFF_US),
        Err(Reason::Timeout)
    );
    let o = a.finish(ObservationInterval::new(0, 1, CUTOFF_US));
    assert_eq!(o.outcome(), "interrupted");
    assert!(o.inventory.unwrap().entries.is_empty());
    let (mut a, b) = scoped();
    a.accept(recheck(&b, 0, 1, true, vec![]), 500).unwrap();
    a.accept(Event::Done { binding: b.clone() }, 600).unwrap();
    assert_eq!(
        a.accept(batch(&b, 0, &["res://late.gd"], 1), 700),
        Err(Reason::ProtocolError)
    );
    assert_eq!(
        a.finish(ObservationInterval::new(0, 1, 800)).outcome(),
        "interrupted"
    );
}
#[test]
fn changed_editor_scope_suppresses_inventory_but_epoch_change_retains_limited() {
    for data_change in [true, false] {
        let (mut a, b) = scoped();
        a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
        let mut e = recheck(&b, 1, 1, true, vec![]);
        if let Event::Rechecked { context, .. } = &mut e {
            let f = context.context.as_mut().unwrap();
            if data_change {
                f.project_data_directory = "res://godot".into();
            } else {
                f.filesystem_epoch = "1".into();
            }
        }
        a.accept(e, 500).unwrap();
        let o = close(a, &b);
        assert_eq!(o.outcome(), "limited_listing");
        if data_change {
            assert!(o.inventory.is_none());
        } else {
            let i = o.inventory.unwrap();
            assert_eq!(i.entries, vec!["res://safe.gd"]);
            assert_eq!(i.consistency.stability, "changed");
        }
    }
}
#[test]
fn unsupported_and_busy_contexts_do_not_invent_empty_scope() {
    for reason in [
        ContextReason::Busy,
        ContextReason::UnsupportedDiscovery,
        ContextReason::UnsupportedVisibilityPolicy,
        ContextReason::UnavailableEditorContext,
    ] {
        let r = request();
        let t = target(&r);
        let b = Binding::target(&t);
        let mut a = Attempt::new(r.clone());
        a.accept(
            Event::Selected {
                binding: Binding::request(&r),
                target: t,
                capability: true,
            },
            10,
        )
        .unwrap();
        let mut c = context(20, 100, 110);
        c.context = None;
        c.reason = Some(reason);
        c.status = if reason == ContextReason::UnavailableEditorContext {
            ContextStatus::Unavailable
        } else {
            ContextStatus::Refused
        };
        if matches!(
            reason,
            ContextReason::Busy | ContextReason::UnsupportedDiscovery
        ) {
            c.expiry_tick_us = None;
        }
        a.accept(
            Event::Scope {
                binding: b.clone(),
                context: c,
            },
            30,
        )
        .unwrap();
        let o = close(a, &b);
        assert!(o.inventory.is_none());
        assert_ne!(o.outcome(), "complete_listing");
    }
    let r = request();
    let t = target(&r);
    let b = Binding::target(&t);
    let mut a = Attempt::new(r.clone());
    a.accept(
        Event::Selected {
            binding: Binding::request(&r),
            target: t,
            capability: true,
        },
        10,
    )
    .unwrap();
    let mut c = context(20, 100, 110);
    c.context.as_mut().unwrap().scanning = true;
    a.accept(
        Event::Scope {
            binding: b.clone(),
            context: c,
        },
        30,
    )
    .unwrap();
    let mut e = recheck(&b, 0, 0, false, vec![]);
    if let Event::Rechecked {
        namespace_checked, ..
    } = &mut e
    {
        *namespace_checked = false;
    }
    a.accept(e, 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    assert_eq!(o.inventory.unwrap().coverage, "not_started");
}
#[test]
fn metadata_and_editor_clocks_are_checked_separately() {
    for defect in 0..5 {
        let (mut a, b) = scoped();
        let mut e = batch(&b, 0, &["res://bad.gd"], 1);
        if let Event::Entries { collection, .. } = &mut e {
            match defect {
                0 => collection.clock_id = "editor:00112233445566778899aabbccddeeff".into(),
                1 => collection.started_tick_us = "26".into(),
                2 => collection.finished_tick_us = "50".into(),
                3 => collection.started_tick_us = "020".into(),
                _ => collection.received_elapsed_us = 41,
            }
        }
        assert_eq!(a.accept(e, 40), Err(Reason::ProtocolError));
        assert!(a
            .finish(ObservationInterval::new(0, 1, 700))
            .inventory
            .unwrap()
            .entries
            .is_empty());
    }
    for defect in 0..4 {
        let (mut a, b) = scoped();
        let mut e = recheck(&b, 0, 1, true, vec![]);
        if let Event::Rechecked { context, .. } = &mut e {
            match defect {
                0 => context.collection.clock_id = "caller".into(),
                1 => context.collection.started_tick_us = "109".into(),
                2 => context.expiry_tick_us = Some("8000000".into()),
                _ => context.collection.received_elapsed_us = 501,
            }
        }
        assert_eq!(a.accept(e, 500), Err(Reason::ProtocolError));
    }
}
#[test]
fn unsafe_or_noneligible_paths_never_enter_retained_evidence() {
    for p in [
        "res://a/../x.gd",
        "res://a//x.gd",
        "res://x.gd?query",
        "res://x.gd#fragment",
        "res://x\\y.gd",
        "res://.hidden/x.gd",
        "res://.godot/x.gd",
        "res://x.tscn::GDScript_1",
        "res://x.txt",
        "res://colon:name.gd",
        "res://new\nline.gd",
    ] {
        let (mut a, b) = scoped();
        assert_eq!(
            a.accept(batch(&b, 0, &[p], 1), 40),
            Err(Reason::ProtocolError),
            "{p}"
        );
        assert!(a
            .finish(ObservationInterval::new(0, 1, 700))
            .inventory
            .unwrap()
            .entries
            .is_empty());
    }
    let (mut a, b) = scoped();
    assert_eq!(
        a.accept(batch(&b, 0, &["res://same.gd", "res://same.gd"], 2), 40),
        Err(Reason::ProtocolError)
    );
    let (mut a, b) = scoped();
    a.accept(
        batch(
            &b,
            0,
            &["res://Case.gd", "res://case.gd", "res://é.gd", "res://é.gd"],
            4,
        ),
        40,
    )
    .unwrap();
    a.accept(recheck(&b, 4, 1, true, vec![]), 500).unwrap();
    assert_eq!(
        close(a, &b).inventory.unwrap().entries,
        vec!["res://Case.gd", "res://case.gd", "res://é.gd", "res://é.gd"]
    );
}
#[test]
fn exact_batch_entry_work_directory_and_depth_bounds_are_not_false_overflow() {
    for over in [false, true] {
        let (mut a, b) = scoped();
        let paths: Vec<String> = (0..BATCH_LIMIT + usize::from(over))
            .map(|n| format!("res://{n}.gd"))
            .collect();
        let e = Event::Entries {
            binding: b,
            ordinal: 0,
            collection: Stamp::caller(20, 25, 25),
            entries: paths,
            visited_entries: 65,
            visited_directories: 1,
        };
        assert_eq!(a.accept(e, 40).is_err(), over);
    }
    let (mut a, b) = scoped();
    for n in 0..16 {
        let paths: Vec<String> = (n * 64..(n + 1) * 64)
            .map(|i| format!("res://{i}.gd"))
            .collect();
        a.accept(
            Event::Entries {
                binding: b.clone(),
                ordinal: n,
                collection: Stamp::caller(
                    20 + n as u64 * 10,
                    25 + n as u64 * 10,
                    25 + n as u64 * 10,
                ),
                entries: paths,
                visited_entries: (n + 1) * 64,
                visited_directories: 1,
            },
            40 + n as u64 * 10,
        )
        .unwrap();
    }
    assert_eq!(a.entries.len(), ENTRY_LIMIT);
    a.accept(recheck(&b, WORK_LIMIT, DIRECTORY_LIMIT, true, vec![]), 500)
        .unwrap();
    assert_eq!(close(a, &b).outcome(), "complete_listing");
    for over in [0, 1] {
        for dirs in [false, true] {
            let (mut a, b) = scoped();
            let e = recheck(
                &b,
                WORK_LIMIT + if dirs { 0 } else { over },
                DIRECTORY_LIMIT + if dirs { over } else { 0 },
                true,
                vec![],
            );
            assert_eq!(a.accept(e, 500).is_err(), over == 1);
        }
    }
    for depth in [DEPTH_LIMIT, DEPTH_LIMIT + 1] {
        let (mut a, b) = scoped();
        let path = format!("res://{}x.gd", "a/".repeat(depth));
        assert_eq!(
            a.accept(batch(&b, 0, &[&path], 1), 40).is_err(),
            depth > DEPTH_LIMIT
        );
    }
}
#[test]
fn first_entry_over_limit_is_rejected_and_explicit_worker_limit_is_limited() {
    let (mut a, b) = scoped();
    for n in 0..16 {
        let paths: Vec<String> = (n * 64..(n + 1) * 64)
            .map(|i| format!("res://{i}.gd"))
            .collect();
        a.accept(
            Event::Entries {
                binding: b.clone(),
                ordinal: n,
                collection: Stamp::caller(
                    20 + n as u64 * 10,
                    25 + n as u64 * 10,
                    25 + n as u64 * 10,
                ),
                entries: paths,
                visited_entries: (n + 1) * 64,
                visited_directories: 1,
            },
            40 + n as u64 * 10,
        )
        .unwrap();
    }
    assert_eq!(
        a.accept(batch(&b, 16, &["res://one-over.gd"], 1025), 210),
        Err(Reason::ProtocolError)
    );
    let o = a.finish(ObservationInterval::new(0, 1, 700));
    assert_eq!(o.outcome(), "interrupted");
    assert_eq!(o.inventory.unwrap().entries.len(), 1024);
    for code in [
        Reason::EntryLimit,
        Reason::DirectoryLimit,
        Reason::DepthLimit,
        Reason::WorkLimit,
    ] {
        let (mut a, b) = scoped();
        a.accept(
            recheck(
                &b,
                0,
                1,
                false,
                vec![Gap {
                    code,
                    stage: Stage::Enumerate,
                    scope: Some("res://".into()),
                }],
            ),
            500,
        )
        .unwrap();
        assert_eq!(close(a, &b).outcome(), "limited_listing");
    }
}
#[test]
fn diagnostics_overflow_has_one_bounded_aggregate_and_never_restores_complete() {
    let (mut a, b) = scoped();
    for n in 0..65 {
        a.accept(
            Event::Invalidate {
                binding: b.clone(),
                reason: Reason::NamespaceChanged,
                prefix: Some(format!("res://gone{n}")),
            },
            40 + n,
        )
        .unwrap();
    }
    let mut e = recheck(&b, 0, 1, true, vec![]);
    if let Event::Rechecked { omitted_gaps, .. } = &mut e {
        *omitted_gaps = 7;
    }
    a.accept(e, 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    assert_eq!(o.diagnostics.len(), 64);
    let aggregate = o.diagnostics.last().unwrap();
    assert_eq!(aggregate.code, Reason::AdditionalGaps);
    assert_eq!(aggregate.omitted_count, Some(9));
}
#[test]
fn terminal_size_is_checked_without_upgrading_precedence_or_splitting_entries() {
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
    a.accept(recheck(&b, 1, 1, true, vec![]), 500).unwrap();
    let mut o = close(a, &b);
    // A private oversized projection exercises the serialization guard although
    // legal bounded paths cannot normally fill six MiB.
    o.inventory.as_mut().unwrap().entries = vec!["x".repeat(RESULT_LIMIT), "res://safe.gd".into()];
    for original in ["complete_listing", "interrupted"] {
        let mut value = o.clone();
        value.outcome = original;
        let value = value.bounded();
        assert_eq!(
            value.outcome(),
            if original == "complete_listing" {
                "limited_listing"
            } else {
                original
            }
        );
        assert!(value.inventory.as_ref().unwrap().entries.is_empty());
        let bytes = value.to_json_line().unwrap();
        assert!(bytes.len() <= RESULT_LIMIT + 1);
        assert_eq!(value.diagnostics.last().unwrap().code, Reason::ResultLimit);
    }
    let mut exact = o.clone();
    exact.inventory.as_mut().unwrap().entries = vec![String::new()];
    let base = serde_json::to_vec(&exact).unwrap().len();
    exact.inventory.as_mut().unwrap().entries[0] = "x".repeat(RESULT_LIMIT - base);
    assert_eq!(exact.to_json_line().unwrap().len(), RESULT_LIMIT + 1);
    exact.inventory.as_mut().unwrap().entries[0].push('x');
    assert!(exact.to_json_line().is_err());
}
#[test]
fn missing_begin_policy_refuses_but_missing_final_context_is_a_limited_observation() {
    let r = request();
    let t = target(&r);
    let b = Binding::target(&t);
    let mut a = Attempt::new(r.clone());
    a.accept(
        Event::Selected {
            binding: Binding::request(&r),
            target: t,
            capability: true,
        },
        10,
    )
    .unwrap();
    let mut c = context(20, 100, 110);
    c.context = None;
    c.status = ContextStatus::Unavailable;
    c.reason = Some(ContextReason::UnavailableEditorContext);
    a.accept(
        Event::Scope {
            binding: b.clone(),
            context: c,
        },
        30,
    )
    .unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "refused");
    assert!(o.inventory.is_none());
    assert_eq!(o.diagnostics[0].code, Reason::UnsupportedVisibilityPolicy);
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
    let mut e = recheck(&b, 1, 1, true, vec![]);
    if let Event::Rechecked { context, .. } = &mut e {
        context.context = None;
        context.status = ContextStatus::Unavailable;
        context.reason = Some(ContextReason::UnavailableEditorContext);
    }
    a.accept(e, 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    let i = o.inventory.unwrap();
    assert_eq!(i.entries, vec!["res://safe.gd"]);
    assert_eq!(i.consistency.recheck, "unavailable");
}
#[test]
fn losing_supported_policy_records_admitted_root_without_disclosing_inventory() {
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://private-sentinel.gd"], 1), 40)
        .unwrap();
    let mut e = recheck(&b, 1, 1, true, vec![]);
    if let Event::Rechecked { context, .. } = &mut e {
        context.context = None;
        context.status = ContextStatus::Refused;
        context.reason = Some(ContextReason::UnsupportedVisibilityPolicy);
    }
    a.accept(e, 500).unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    assert!(o.inventory.is_none());
    assert!(o
        .diagnostics
        .iter()
        .any(|d| d.code == Reason::ScopeChanged && d.scope.as_deref() == Some("res://")));
    assert!(!String::from_utf8(o.to_json_line().unwrap())
        .unwrap()
        .contains("private-sentinel"));
}
#[test]
fn final_namespace_gap_removes_affected_entries_without_a_prior_invalidation_frame() {
    let (mut a, b) = scoped();
    a.accept(
        batch(&b, 0, &["res://changed/a.gd", "res://safe.gd"], 2),
        40,
    )
    .unwrap();
    a.accept(
        recheck(
            &b,
            2,
            1,
            true,
            vec![Gap {
                code: Reason::NamespaceChanged,
                stage: Stage::Recheck,
                scope: Some("res://changed".into()),
            }],
        ),
        500,
    )
    .unwrap();
    let o = close(a, &b);
    assert_eq!(o.outcome(), "limited_listing");
    let i = o.inventory.unwrap();
    assert_eq!(i.entries, vec!["res://safe.gd"]);
    assert_eq!(i.consistency.stability, "changed");
}
#[test]
fn exact_locator_byte_bound_is_preserved_and_one_over_is_not_truncated() {
    for length in [2048, 2049] {
        let path = format!("res://{}.gd", "x".repeat(length - 9));
        let (mut a, b) = scoped();
        assert_eq!(
            a.accept(batch(&b, 0, &[&path], 1), 40).is_err(),
            length > 2048
        );
    }
}
#[test]
fn negative_authenticated_binding_fact_suppresses_earlier_paths_without_relabeling_error() {
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://private-sentinel.gd"], 1), 40)
        .unwrap();
    a.accept(
        Event::Invalidate {
            binding: b.clone(),
            reason: Reason::ProtocolError,
            prefix: None,
        },
        45,
    )
    .unwrap();
    a.accept(
        Event::Failed {
            binding: b,
            reason: Reason::ProtocolError,
            selection: None,
        },
        50,
    )
    .unwrap();
    let o = a.finish(ObservationInterval::new(0, 1, 700));
    assert_eq!(o.outcome(), "interrupted");
    assert!(o.inventory.is_none());
    assert!(o
        .diagnostics
        .iter()
        .all(|d| d.code == Reason::ProtocolError && d.scope.is_none()));
    assert!(!String::from_utf8(o.to_json_line().unwrap())
        .unwrap()
        .contains("private-sentinel"));
    let (mut a, b) = scoped();
    a.accept(batch(&b, 0, &["res://safe.gd"], 1), 40).unwrap();
    assert_eq!(
        a.accept(
            Event::Invalidate {
                binding: b,
                reason: Reason::ProtocolError,
                prefix: Some("res://partial".into())
            },
            45
        ),
        Err(Reason::ProtocolError)
    );
    assert_eq!(
        a.finish(ObservationInterval::new(0, 1, 700))
            .inventory
            .unwrap()
            .entries,
        vec!["res://safe.gd"]
    );
}
