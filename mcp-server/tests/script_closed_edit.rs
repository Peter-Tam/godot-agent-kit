//! Revision-based editing retains the standalone-script mutation boundary.
use godot_agent_kit::observation::{ObservationRequest, ProjectRoot, RequestId, ResourcePath};
use godot_agent_kit::script_edit::ReplacementSource;
use godot_agent_kit::script_read::{ScriptEditRequest, ScriptRevision};

#[test]
fn revision_intent_cannot_admit_embedded_or_non_gdscript_targets() {
    for path in [
        "res://scene.tscn::GDScript_1",
        "res://resource.tres",
        "res://script.cs",
    ] {
        let observation = ObservationRequest::new(
            RequestId::new("edit-intent").unwrap(),
            ProjectRoot::new("/fixture/project").unwrap(),
            None,
            ResourcePath::new(path).unwrap(),
        );
        let revision = ScriptRevision::new(format!("sr1:{}", "a".repeat(64))).unwrap();
        assert!(ScriptEditRequest::new(
            observation,
            revision,
            ReplacementSource::new("extends Node\n".to_owned()).unwrap(),
        )
        .is_err());
    }
}
