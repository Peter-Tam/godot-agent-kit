use godot_agent_kit::observation::*;
use godot_agent_kit::script_edit::ReplacementSource;
use godot_agent_kit::script_read::{ScriptEditRequest, ScriptRevision};

#[test]
fn revision_and_request_are_checked_at_public_boundary() {
    for invalid in [
        "sr1:",
        "sr1:ABC",
        "sr2:0000000000000000000000000000000000000000000000000000000000000000",
    ] {
        assert!(ScriptRevision::new(invalid).is_err());
    }
    let revision = ScriptRevision::new(format!("sr1:{}", "a".repeat(64))).unwrap();
    for (path, accepted) in [("res://subject.gd", true), ("res://subject.tscn", false)] {
        let request = ObservationRequest::new(
            RequestId::new("edit").unwrap(),
            ProjectRoot::new("/fixture/project").unwrap(),
            None,
            ResourcePath::new(path).unwrap(),
        );
        assert_eq!(
            ScriptEditRequest::new(
                request,
                revision.clone(),
                ReplacementSource::new("extends Node\n".to_owned()).unwrap()
            )
            .is_ok(),
            accepted
        );
    }
}
