//! The actual public binary/worker/authenticated framing, with independent on-disk D.
use super::*;
use godot_agent_kit::observation::{DirtyReason, SourceReason};
use std::os::unix::fs::MetadataExt;
#[derive(Clone, Copy)]
enum OpenMode {
    RecognitionDirty,
    RecognitionLimited,
    RecognitionChanged,
    RecognitionDiskReplaced,
    Verified,
    InvalidParse,
    DivergentReadback,
    DirtySampleEof,
    EditedSampleEof,
    DivergentSampleEof,
    SourceChangedRecheckFailure,
    NativeRecheckFailure,
    PartialRecheckChanges,
    SlowVerification,
    LostAuthorization,
    LostAfterBinding,
    Discarded,
    WrongEnvelope,
    Oversized,
    UnavailableInitialSample,
    MissingDocumentSample,
    StallBegin,
    StallAfterBinding,
}
const PATH: &str = "res://scripts/subject.gd";
const SOURCE: &str = "extends Node\n";
fn sha(source: &str) -> String {
    hex(ring::digest::digest(&ring::digest::SHA256, source.as_bytes()).as_ref())
}
fn stamp(hello: &Value, tick: u64) -> Value {
    json!({"clock_id":format!("editor:{}",hello[3].as_str().unwrap()),"started_tick_us":tick.to_string(),"finished_tick_us":(tick+10).to_string(),"received_elapsed_us":0})
}
fn sample(hello: &Value, tick: u64, open: bool, dirty: bool, resource: &str) -> Value {
    let mut s = observation_sample(hello);
    let c = stamp(hello, tick);
    s["collection"] = c.clone();
    s["document"]["validity"]["collection"] = c.clone();
    s["document"]["open_state"]["collection"] = c.clone();
    s["R"]["collection"] = c.clone();
    s["B"]["collection"] = c.clone();
    s["dirty"]["collection"] = c;
    s["R"]["text"] = json!(resource);
    s["B"]["text"] = json!(SOURCE);
    s["dirty"]["state"] = json!(if dirty { "dirty" } else { "clean" });
    if !open {
        s["document"]["open_state"]["value"] = json!("not_open");
        s["document"]["identity"]["script_instance_id"] = Value::Null;
        s["document"]["identity"]["editor_instance_id"] = Value::Null;
        s["document"]["identity"]["buffer_instance_id"] = Value::Null;
        s["R"] = unavailable_source("R", "resource_not_loaded", SourceReason::ResourceNotLoaded);
        s["B"] = unavailable_source("B", "document_not_open", SourceReason::DocumentNotOpen);
        s["B"]["availability"] = json!("not_applicable");
        s["dirty"] = json!({"availability":"not_applicable","state":"not_applicable","collection":null,"witness":null,"reason":{"code":"document_not_open","action":DirtyReason::DocumentNotOpen.action()},"invalidated_evidence":null});
    }
    s
}
fn facts(hello: &Value, stage: &str, next: &str, recognition: bool, invalid: bool) -> Value {
    let bound = matches!(stage, "bind" | "compile" | "open");
    let compiled = matches!(stage, "compile" | "open");
    let opened = stage == "open";
    json!({"request_id":hello[2],"stage":stage,"next_stage":next,"cache_binding":if bound{"new_resource_published"}else{"not_started"},"initial_compilation":if compiled{if invalid{"completed_invalid"}else{"completed_valid"}}else{"not_started"},"document_open":if opened{"association_obtained"}else{"not_started"},"target_parse_code":if compiled{Some(if invalid{43}else{0})}else{None},"script_instance_id":if bound||recognition{Some("9007199254740993")}else{None},"editor_instance_id":if opened||recognition{Some("9007199254740994")}else{None},"buffer_instance_id":if opened||recognition{Some("9007199254740995")}else{None},"target_open":opened||recognition,"terminal_discard":false,"entered":true,"mode":if stage=="inspected"{None}else{Some("cold")},"protection":null,"selection":if opened||recognition{"requested_target"}else{"no_source"}})
}
fn reply(hello: &Value, kind: &str, status: &str, tick: u64, native: Value) -> Value {
    json!({"v":3,"kind":kind,"request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":PATH,"collection":stamp(hello,tick),"status":status,"reason":"","native":native,"resource_edited":false,"expiry_tick_us":"8000000"})
}
fn serve_open(f: &Fixture, mode: OpenMode) -> thread::JoinHandle<()> {
    let listener = listener();
    f.descriptor(ID1, listener.local_addr().unwrap().port());
    let project = f.project.clone();
    thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let hello = read_frame(&mut socket);
        let caps = json!({"observe_gdscript":true,"open_enumeration":true,"buffer_attribution":true,"unsaved_paths":true,"cached_resource_lookup":true,"edit_open_gdscript":true,"open_gdscript":true});
        frame(
            &mut socket,
            &json!({"v":3,"kind":"challenge","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":2,"native_build_id":NATIVE_BUILD_ID,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"server_proof":proof(b"server",&hello,&caps,SECRET)}),
        );
        let authentication = read_frame(&mut socket);
        assert_eq!(authentication[7], proof(b"client", &hello, &caps, SECRET));
        frame(
            &mut socket,
            &json!({"v":3,"kind":"hello","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":2,"native_build_id":NATIVE_BUILD_ID,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"finish_proof":proof(b"finish",&hello,&caps,SECRET)}),
        );
        let begin = read_frame(&mut socket);
        assert_eq!(begin.as_array().unwrap().len(), 7);
        assert_eq!(begin[1], "open_begin");
        assert!((1..=9000).contains(&begin[6].as_u64().unwrap()));
        if matches!(mode, OpenMode::StallBegin) {
            thread::sleep(Duration::from_secs(11));
            return;
        }
        if matches!(mode, OpenMode::Oversized) {
            socket
                .write_all(&((wire::RESULT_LIMIT + 1) as u32).to_be_bytes())
                .unwrap();
            return;
        }
        let recognition = matches!(
            mode,
            OpenMode::RecognitionDirty
                | OpenMode::RecognitionLimited
                | OpenMode::RecognitionChanged
                | OpenMode::RecognitionDiskReplaced
        );
        let invalid = matches!(mode, OpenMode::InvalidParse);
        let mut state = reply(
            &hello,
            "open_state",
            "inspected",
            100,
            facts(
                &hello,
                "inspected",
                if recognition { "" } else { "prepare" },
                recognition,
                invalid,
            ),
        );
        let resource = if matches!(mode, OpenMode::RecognitionLimited) {
            "x".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES + 1)
        } else if matches!(mode, OpenMode::RecognitionChanged) {
            "RECOGNIZED_RESOURCE".to_owned()
        } else {
            SOURCE.to_owned()
        };
        state["sample"] = sample(&hello, 100, recognition, recognition, &resource);
        state["resource_edited"] = if recognition {
            json!(true)
        } else {
            Value::Null
        };
        state["cache"] = if recognition {
            json!({"state":"present","script_instance_id":"9007199254740993","sha256":if matches!(mode,OpenMode::RecognitionLimited){None}else{Some(sha(&resource))},"utf8_bytes":if matches!(mode,OpenMode::RecognitionLimited){None}else{Some(resource.len().to_string())},"reason":if matches!(mode,OpenMode::RecognitionLimited){Some("resource_source_unavailable")}else{None}})
        } else {
            json!({"state":"absent","script_instance_id":null,"sha256":null,"utf8_bytes":null,"reason":null})
        };
        if matches!(
            mode,
            OpenMode::UnavailableInitialSample | OpenMode::MissingDocumentSample
        ) {
            if matches!(mode, OpenMode::MissingDocumentSample) {
                state["sample"]["document"]["validity"]["value"] = json!("missing");
            } else {
                state["sample"] = Value::Null;
            }
            frame(&mut socket, &state);
            return;
        }
        if matches!(mode, OpenMode::WrongEnvelope) {
            state["script_path"] = json!("res://unrelated.gd");
            state["sample"]["R"]["text"] = json!("UNRELATED_SOURCE_SENTINEL");
            frame(&mut socket, &state);
            return;
        }
        frame(&mut socket, &state);
        if !recognition {
            let prepare = read_frame(&mut socket);
            assert_eq!(prepare.as_array().unwrap().len(), 15);
            assert_eq!(prepare[1], "open_prepare");
            assert_eq!(prepare[14], SOURCE);
            assert_eq!(prepare[12], sha(SOURCE));
            assert_eq!(prepare[10], "absent");
            assert_eq!(prepare[11], "");
            let root = fs::metadata(&project).unwrap();
            let file = fs::metadata(project.join("scripts/subject.gd")).unwrap();
            assert_eq!(prepare[6], root.dev().to_string());
            assert_eq!(prepare[7], root.ino().to_string());
            assert_eq!(prepare[8], file.dev().to_string());
            assert_eq!(prepare[9], file.ino().to_string());
            let mut prepared = reply(
                &hello,
                "open_prepared",
                "prepared",
                200,
                facts(&hello, "prepared", "bind", false, invalid),
            );
            prepared["mode"] = json!("cold");
            prepared["sample"] = sample(&hello, 200, false, false, SOURCE);
            prepared["resource_edited"] = Value::Null;
            prepared["context"] = json!({"kind":"no_source_editor","reason":null,"projection":null,"source":null,"sha256":null});
            prepared["validation"] = Value::Null;
            prepared["project_device"] = json!(root.dev().to_string());
            prepared["project_inode"] = json!(root.ino().to_string());
            prepared["target_device"] = json!(file.dev().to_string());
            prepared["target_inode"] = json!(file.ino().to_string());
            frame(&mut socket, &prepared);
            for (stage, next, tick) in [
                ("bind", "compile", 300),
                ("compile", "open", 400),
                ("open", "", 500),
            ] {
                let advance = read_frame(&mut socket);
                assert_eq!(advance.as_array().unwrap().len(), 9);
                assert_eq!(advance[1], "open_advance");
                assert_eq!(advance[6], stage);
                assert_eq!(advance[7], "");
                assert_eq!(advance[8], "");
                if matches!(mode, OpenMode::LostAuthorization) && stage == "bind" {
                    return;
                }
                if matches!(mode, OpenMode::LostAfterBinding) && stage == "compile" {
                    return;
                }
                if matches!(mode, OpenMode::StallAfterBinding) && stage == "compile" {
                    thread::sleep(Duration::from_secs(11));
                    let late = serde_json::to_vec(&reply(
                        &hello,
                        "open_progress",
                        "ready",
                        400,
                        facts(&hello, "compile", "open", false, invalid),
                    ))
                    .unwrap();
                    let _ = socket
                        .write_all(&(late.len() as u32).to_be_bytes())
                        .and_then(|_| socket.write_all(&late));
                    return;
                }
                if matches!(mode, OpenMode::Discarded) && stage == "bind" {
                    let mut n = facts(&hello, "prepared", "", false, invalid);
                    n["terminal_discard"] = json!(true);
                    let mut result = reply(&hello, "open_progress", "refused", 300, n);
                    result["reason"] = json!("current_context_changed");
                    frame(&mut socket, &result);
                    return;
                }
                if matches!(mode, OpenMode::SlowVerification) && stage == "open" {
                    thread::sleep(Duration::from_millis(250));
                }
                let result = reply(
                    &hello,
                    "open_progress",
                    "ready",
                    tick,
                    facts(&hello, stage, next, false, invalid),
                );
                frame(&mut socket, &result);
            }
        }
        let verify = read_frame(&mut socket);
        let purpose = if recognition {
            "recognition"
        } else {
            "post_open"
        };
        assert_eq!(
            verify,
            json!([
                3,
                "open_verify",
                hello[2],
                hello[3],
                hello[4],
                PATH,
                purpose
            ])
        );
        let nstage = if recognition { "inspected" } else { "open" };
        let mut n = facts(&hello, nstage, "", recognition, invalid);
        n["protection"] = json!(if recognition {
            "not_applicable"
        } else {
            "unchanged"
        });
        if matches!(mode, OpenMode::SlowVerification) {
            thread::sleep(Duration::from_millis(50));
        }
        let mut observed = reply(&hello, "open_sample", "observed", 600, n.clone());
        observed["purpose"] = json!(purpose);
        observed["sample"] = sample(
            &hello,
            600,
            true,
            recognition || matches!(mode, OpenMode::DirtySampleEof),
            if matches!(
                mode,
                OpenMode::DivergentReadback | OpenMode::DivergentSampleEof
            ) {
                "ACKNOWLEDGMENT_IS_NOT_READBACK"
            } else {
                &resource
            },
        );
        observed["resource_edited"] =
            json!(recognition || matches!(mode, OpenMode::EditedSampleEof));
        observed["protection"] = n["protection"].clone();
        observed["selection"] = json!("requested_target");
        frame(&mut socket, &observed);
        if matches!(
            mode,
            OpenMode::DirtySampleEof | OpenMode::EditedSampleEof | OpenMode::DivergentSampleEof
        ) {
            return;
        }
        let recheck = read_frame(&mut socket);
        assert_eq!(
            recheck,
            json!([
                3,
                "open_recheck",
                hello[2],
                hello[3],
                hello[4],
                PATH,
                purpose
            ])
        );
        if matches!(mode, OpenMode::SlowVerification) {
            thread::sleep(Duration::from_millis(50));
        }
        let mut checked = reply(&hello, "open_rechecked", "unchanged", 700, n);
        checked["purpose"] = json!(purpose);
        checked["resource_edited"] = json!(recognition);
        checked["protection"] = json!(if recognition {
            "not_applicable"
        } else {
            "unchanged"
        });
        checked["selection"] = json!("requested_target");
        checked["recheck"] = json!({"v":3,"kind":"recheck","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":PATH,"collection":stamp(&hello,700),"checks":"performed","detected_changes":[],"reason":null});
        if matches!(mode, OpenMode::RecognitionChanged) {
            checked["recheck"]["detected_changes"] =
                json!([{"surface":"R","code":"source_changed"}]);
        }
        if matches!(mode, OpenMode::PartialRecheckChanges) {
            checked["recheck"]["checks"] = json!("unavailable");
            checked["recheck"]["reason"] = json!("unavailable");
            checked["recheck"]["detected_changes"] = json!([
                {"surface":"R","code":"source_changed"},
                {"surface":"B","code":"source_changed"},
                {"surface":"dirty","code":"source_changed"}
            ]);
        }
        if matches!(
            mode,
            OpenMode::SourceChangedRecheckFailure | OpenMode::NativeRecheckFailure
        ) {
            checked["status"] = json!("partial");
            checked["reason"] = json!(if matches!(mode, OpenMode::SourceChangedRecheckFailure) {
                "resource_source_changed"
            } else {
                "native_failure"
            });
            checked["recheck"]["detected_changes"] = json!([
                {"surface":"R","code":"source_changed"},
                {"surface":"B","code":"source_changed"},
                {"surface":"dirty","code":"source_changed"}
            ]);
        }
        if matches!(mode, OpenMode::RecognitionDiskReplaced) {
            let replacement = project.join("scripts/replacement.gd");
            fs::write(&replacement, "NEWER_HUMAN_DISK_SOURCE").unwrap();
            fs::rename(replacement, project.join("scripts/subject.gd")).unwrap();
        }
        frame(&mut socket, &checked);
    })
}
fn invoke_open(f: &Fixture) -> (i32, Value, Duration) {
    let started = Instant::now();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_open-gdscript"))
        .args([
            "--registry",
            f.registry.to_str().unwrap(),
            "--project",
            f.project.to_str().unwrap(),
            "--session",
            ID1,
            "--script",
            PATH,
        ])
        .output()
        .unwrap();
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    let value = serde_json::from_str(&stdout).unwrap();
    (output.status.code().unwrap(), value, started.elapsed())
}
fn fixture() -> Fixture {
    let f = Fixture::new();
    fs::create_dir(f.project.join("scripts")).unwrap();
    fs::write(f.project.join("scripts/subject.gd"), SOURCE).unwrap();
    f
}
#[test]
fn missing_closed_disk_is_distinct_from_unavailable_editor_evidence() {
    for (missing, mode) in [
        (true, OpenMode::UnavailableInitialSample),
        (false, OpenMode::UnavailableInitialSample),
        (true, OpenMode::MissingDocumentSample),
    ] {
        let f = fixture();
        if missing {
            fs::remove_file(f.project.join("scripts/subject.gd")).unwrap();
        }
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        assert_eq!(code, 3);
        assert_eq!(result["outcome"], "refused");
        assert_eq!(result["application"], "not_applied");
        assert_eq!(
            result["reason"],
            if missing {
                "missing_script"
            } else {
                "evidence_unavailable"
            }
        );
        assert!(result["observation"].is_null());
        assert_eq!(
            result["progress"]["resource_binding"]["state"],
            "not_started"
        );
    }
}
#[test]
fn actual_open_caller_distinguishes_receipts_from_independent_readback() {
    for mode in [
        OpenMode::Verified,
        OpenMode::InvalidParse,
        OpenMode::DivergentReadback,
    ] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, elapsed) = invoke_open(&f);
        peer.join().unwrap();
        assert!(elapsed < Duration::from_secs(10));
        assert_eq!(result["application"], "applied");
        if matches!(mode, OpenMode::DivergentReadback) {
            assert_eq!(code, 4);
            assert_eq!(result["outcome"], "applied_unverified");
            assert_eq!(result["reason"], "verification_failed");
        } else {
            assert_eq!(code, 0);
            assert_eq!(result["outcome"], "verified_newly_opened");
            assert_eq!(
                result["target_parse"]["state"],
                if matches!(mode, OpenMode::InvalidParse) {
                    "invalid"
                } else {
                    "valid"
                }
            );
        }
        assert_eq!(
            fs::read_to_string(f.project.join("scripts/subject.gd")).unwrap(),
            SOURCE
        );
    }
}
#[test]
fn incomplete_or_failed_recheck_retains_valid_ordinary_invalidation_and_its_original_cause() {
    for mode in [
        OpenMode::SourceChangedRecheckFailure,
        OpenMode::NativeRecheckFailure,
        OpenMode::PartialRecheckChanges,
    ] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        assert_eq!(code, 4);
        assert_eq!(result["outcome"], "applied_unverified");
        assert_eq!(result["application"], "applied");
        assert_eq!(
            result["reason"],
            if matches!(mode, OpenMode::NativeRecheckFailure) {
                "native_failure"
            } else {
                "source_changed"
            }
        );
        assert_eq!(result["progress"]["resource_binding"]["mode"], "created");
        assert_eq!(result["progress"]["document_open"]["state"], "completed");
        let snapshot = &result["observation"]["snapshot"];
        assert_eq!(snapshot["consistency"]["checks"], "unavailable");
        assert_eq!(
            snapshot["consistency"]["detected_changes"],
            json!([
                {"surface":"R","code":"source_changed"},
                {"surface":"B","code":"source_changed"},
                {"surface":"dirty","code":"source_changed"}
            ])
        );
        for authority in ["R", "B"] {
            assert_eq!(
                snapshot["sources"][authority]["availability"],
                "unavailable"
            );
            assert_eq!(snapshot["sources"][authority]["text"], Value::Null);
            assert_eq!(
                snapshot["sources"][authority]["invalidated_evidence"]["text"],
                SOURCE
            );
        }
        assert_eq!(snapshot["dirty"]["availability"], "unavailable");
        assert_eq!(snapshot["dirty"]["invalidated_evidence"]["state"], "clean");
        assert_eq!(
            fs::read_to_string(f.project.join("scripts/subject.gd")).unwrap(),
            SOURCE
        );
    }
}
#[test]
fn decisive_observation_failure_precedes_later_editor_eof() {
    for mode in [
        OpenMode::DirtySampleEof,
        OpenMode::EditedSampleEof,
        OpenMode::DivergentSampleEof,
    ] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        let expected = if matches!(mode, OpenMode::DivergentSampleEof) {
            "verification_failed"
        } else {
            "dirty_conflict"
        };
        assert_eq!(code, 4);
        assert_eq!(result["outcome"], "applied_unverified");
        assert_eq!(result["application"], "applied");
        assert_eq!(result["reason"], expected);
        assert_eq!(result["progress"]["verification"]["state"], "failed");
        assert_eq!(result["progress"]["verification"]["reason"], expected);
        assert_eq!(result["progress"]["document_open"]["state"], "completed");
        let snapshot = &result["observation"]["snapshot"];
        assert_eq!(result["observation"]["purpose"], "verification");
        // EOF invalidates editor-local observations without erasing their
        // acquired evidence or replacing the earlier decisive failure.
        assert_eq!(snapshot["sources"]["B"]["availability"], "unavailable");
        assert_eq!(snapshot["sources"]["B"]["reason"]["code"], "session_ended");
        assert_eq!(
            snapshot["sources"]["B"]["invalidated_evidence"]["text"],
            SOURCE
        );
        if matches!(mode, OpenMode::DirtySampleEof) {
            assert_eq!(snapshot["dirty"]["invalidated_evidence"]["state"], "dirty");
        }
        if matches!(mode, OpenMode::EditedSampleEof) {
            assert_eq!(result["resource_edited"]["value"], true);
        }
        if matches!(mode, OpenMode::DivergentSampleEof) {
            assert_eq!(
                snapshot["sources"]["R"]["invalidated_evidence"]["text"],
                "ACKNOWLEDGMENT_IS_NOT_READBACK"
            );
        }
        assert_eq!(
            fs::read_to_string(f.project.join("scripts/subject.gd")).unwrap(),
            SOURCE
        );
    }
}
#[test]
fn latest_snapshot_times_only_its_acquisition_with_operation_relative_source_stamps() {
    let f = fixture();
    let peer = serve_open(&f, OpenMode::SlowVerification);
    let (code, result, _) = invoke_open(&f);
    peer.join().unwrap();
    assert_eq!(code, 0);
    assert_eq!(result["outcome"], "verified_newly_opened");
    let operation = &result["interval"];
    let snapshot = &result["observation"]["snapshot"];
    let acquisition = &snapshot["interval"];
    let operation_elapsed = operation["elapsed_us"].as_u64().unwrap();
    let acquisition_elapsed = acquisition["elapsed_us"].as_u64().unwrap();
    assert!(operation_elapsed >= acquisition_elapsed + 250_000);
    assert!(acquisition_elapsed >= 50_000);
    assert!(
        acquisition["started_unix_ms"].as_u64().unwrap()
            >= operation["started_unix_ms"].as_u64().unwrap() + 200
    );
    assert!(
        acquisition["finished_unix_ms"].as_u64().unwrap()
            <= operation["finished_unix_ms"].as_u64().unwrap()
    );
    let disk = &snapshot["sources"]["D"]["collection"];
    assert_eq!(disk["clock_id"], "caller");
    let disk_start = disk["started_tick_us"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let disk_finish = disk["finished_tick_us"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!(disk_start >= 250_000);
    assert!(disk_start <= disk_finish);
    assert!(disk_finish <= operation_elapsed);
    assert!(disk["received_elapsed_us"].as_u64().unwrap() <= operation_elapsed);
    assert_eq!(
        snapshot["sources"]["R"]["collection"]["clock_id"],
        format!("editor:{ID1}")
    );
}
#[test]
fn dirty_limited_and_invalidated_recognition_never_issue_effect_stages() {
    for mode in [
        OpenMode::RecognitionDirty,
        OpenMode::RecognitionLimited,
        OpenMode::RecognitionChanged,
    ] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        assert_eq!(code, 0);
        assert_eq!(result["outcome"], "already_open_unchanged");
        assert_eq!(result["application"], "not_applied");
        assert_eq!(result["observation"]["snapshot"]["dirty"]["state"], "dirty");
        assert_eq!(result["resource_edited"]["value"], true);
        if matches!(mode, OpenMode::RecognitionLimited) {
            assert_eq!(
                result["observation"]["snapshot"]["sources"]["R"]["reason"]["code"],
                "too_large"
            );
            assert_eq!(
                result["observation"]["snapshot"]["sources"]["B"]["text"],
                SOURCE
            );
        }
        if matches!(mode, OpenMode::RecognitionChanged) {
            let snapshot = &result["observation"]["snapshot"];
            assert_eq!(
                snapshot["consistency"]["detected_changes"],
                json!([{"surface":"R","code":"source_changed"}])
            );
            assert_eq!(snapshot["sources"]["R"]["availability"], "unavailable");
            assert_eq!(
                snapshot["sources"]["R"]["invalidated_evidence"]["text"],
                "RECOGNIZED_RESOURCE"
            );
            assert_eq!(snapshot["sources"]["B"]["text"], SOURCE);
        }
    }
}
#[test]
fn recognition_retains_disk_invalidation_without_requiring_new_open_coherence() {
    let f = fixture();
    let peer = serve_open(&f, OpenMode::RecognitionDiskReplaced);
    let (code, result, _) = invoke_open(&f);
    peer.join().unwrap();
    assert_eq!(code, 0);
    assert_eq!(result["outcome"], "already_open_unchanged");
    assert_eq!(result["application"], "not_applied");
    let snapshot = &result["observation"]["snapshot"];
    assert_eq!(snapshot["sources"]["D"]["availability"], "unavailable");
    assert_eq!(
        snapshot["sources"]["D"]["invalidated_evidence"]["text"],
        SOURCE
    );
    assert_eq!(snapshot["sources"]["B"]["text"], SOURCE);
    assert_eq!(snapshot["document"]["open_state"]["value"], "open");
    assert!(snapshot["consistency"]["detected_changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| { change["surface"] == "D" && change["code"] == "identity_changed" }));
    assert_eq!(
        fs::read_to_string(f.project.join("scripts/subject.gd")).unwrap(),
        "NEWER_HUMAN_DISK_SOURCE"
    );
}
#[test]
fn effect_sensitive_loss_retains_known_binding_and_accepts_only_irrevocable_no_effect_proof() {
    for mode in [
        OpenMode::LostAuthorization,
        OpenMode::LostAfterBinding,
        OpenMode::Discarded,
    ] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        match mode {
            OpenMode::LostAuthorization => {
                assert_eq!(code, 4);
                assert_eq!(result["outcome"], "effects_unknown");
                assert_eq!(result["application"], "unknown");
            }
            OpenMode::LostAfterBinding => {
                assert_eq!(code, 4);
                assert_eq!(result["outcome"], "applied_unverified");
                assert_eq!(result["application"], "partly_applied");
                assert_eq!(result["progress"]["resource_binding"]["mode"], "created");
            }
            OpenMode::Discarded => {
                assert_eq!(code, 3);
                assert_eq!(result["outcome"], "refused");
                assert_eq!(result["application"], "not_applied");
                assert_eq!(result["reason"], "context_changed");
            }
            _ => unreachable!(),
        }
    }
}
#[test]
fn malformed_pre_effect_frames_never_disclose_unauthorized_source() {
    for mode in [OpenMode::WrongEnvelope, OpenMode::Oversized] {
        let f = fixture();
        let peer = serve_open(&f, mode);
        let (code, result, _) = invoke_open(&f);
        peer.join().unwrap();
        assert_eq!(code, 3);
        assert_eq!(result["application"], "not_applied");
        assert_eq!(result["observation"], Value::Null);
        assert!(!result.to_string().contains("UNRELATED_SOURCE_SENTINEL"));
    }
}
#[test]
fn stalled_begin_delivers_a_no_effect_result_within_ten_seconds() {
    let f = fixture();
    let peer = serve_open(&f, OpenMode::StallBegin);
    let (code, result, elapsed) = invoke_open(&f);
    assert_eq!(code, 3);
    assert_eq!(result["reason"], "timeout");
    assert_eq!(result["application"], "not_applied");
    assert!(elapsed <= Duration::from_secs(10));
    peer.join().unwrap();
}
#[test]
fn a_late_compilation_result_cannot_upgrade_a_delivered_known_partial_timeout() {
    let f = fixture();
    let peer = serve_open(&f, OpenMode::StallAfterBinding);
    let (code, result, elapsed) = invoke_open(&f);
    assert_eq!(code, 4);
    assert_eq!(result["outcome"], "applied_unverified");
    assert_eq!(result["application"], "partly_applied");
    assert_eq!(result["reason"], "timeout");
    assert_eq!(result["progress"]["resource_binding"]["mode"], "created");
    assert_eq!(result["target_parse"]["state"], "unavailable");
    assert!(elapsed <= Duration::from_secs(10));
    peer.join().unwrap();
}
#[test]
fn read_only_target_disk_is_eligible_for_opening_without_a_write_descriptor() {
    let f = fixture();
    let path = f.project.join("scripts/subject.gd");
    let mut mode = fs::metadata(&path).unwrap().permissions();
    mode.set_readonly(true);
    fs::set_permissions(&path, mode).unwrap();
    let peer = serve_open(&f, OpenMode::Verified);
    let (code, result, _) = invoke_open(&f);
    peer.join().unwrap();
    assert_eq!(code, 0);
    assert_eq!(result["outcome"], "verified_newly_opened");
    assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
}
#[test]
fn ambiguous_authenticated_sessions_do_not_capture_any_target_source() {
    let f = fixture();
    fs::write(
        f.project.join("scripts/subject.gd"),
        "SELECTED_SOURCE_BEFORE_AUTH_SENTINEL",
    )
    .unwrap();
    let first = attach(&f, ID1, Mode::ValidNative);
    let second = attach(&f, ID2, Mode::ValidNative);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_open-gdscript"))
        .args([
            "--registry",
            f.registry.to_str().unwrap(),
            "--project",
            f.project.to_str().unwrap(),
            "--script",
            PATH,
        ])
        .output()
        .unwrap();
    first.join().unwrap();
    second.join().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["reason"], "ambiguous_session");
    assert_eq!(result["resolved_target"], Value::Null);
    assert_eq!(result["before"], Value::Null);
    assert_eq!(result["observation"], Value::Null);
    for bytes in [&output.stdout, &output.stderr] {
        assert!(!String::from_utf8_lossy(bytes).contains("SELECTED_SOURCE_BEFORE_AUTH_SENTINEL"));
    }
}
#[test]
fn an_ended_explicit_session_never_contacts_a_replacement_session() {
    let f = fixture();
    let other = listener();
    f.descriptor(ID2, other.local_addr().unwrap().port());
    let (code, result, _) = invoke_open(&f);
    assert_eq!(code, 3);
    assert_eq!(result["reason"], "session_ended");
    assert_eq!(result["resolved_target"], Value::Null);
    assert_eq!(result["observation"], Value::Null);
    other.set_nonblocking(true).unwrap();
    assert_eq!(
        other.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
