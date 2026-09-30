use godot_agent_kit::observation::{ProjectRoot, RequestId, ResourcePath};
use godot_agent_kit::runner::stock_validation::OpeningContext;
use godot_agent_kit::script_open::OpenRequest;
use serde_json::json;
#[test]
fn checked_opening_intent_cannot_target_an_embedded_or_non_script_document() {
    for path in [
        "res://scene.tscn::GDScript_1",
        "res://scene.tscn",
        "res://subject.GD",
    ] {
        let result = OpenRequest::new(
            RequestId::new("open").unwrap(),
            ProjectRoot::new("/project").unwrap(),
            None,
            ResourcePath::new(path).unwrap(),
        );
        assert!(
            result.is_err(),
            "opening must not widen the external .gd scope: {path}"
        );
    }
    let request = OpenRequest::new(
        RequestId::new("open").unwrap(),
        ProjectRoot::new("/project").unwrap(),
        None,
        ResourcePath::new("res://scripts/exact spelling.gd").unwrap(),
    )
    .unwrap();
    assert_eq!(
        request.script_path().as_str(),
        "res://scripts/exact spelling.gd"
    );
}
#[test]
fn private_context_debug_never_discloses_even_rejected_current_source() {
    let sentinel = "CURRENT_SOURCE_NEVER_PUBLIC";
    let context:OpeningContext=serde_json::from_value(json!({"kind":"unavailable","reason":"PRIVATE_CURRENT_PATH","projection":null,"source":sentinel,"sha256":"PRIVATE_GUARD_HASH"})).unwrap();
    let debug = format!("{context:?}");
    for private in [sentinel, "PRIVATE_CURRENT_PATH", "PRIVATE_GUARD_HASH"] {
        assert!(!debug.contains(private));
    }
    assert!(context.checked_binding().is_none());
}
#[test]
fn private_context_requires_all_nullable_fields_and_closed_typed_shape() {
    let exact = json!({"kind":"no_source_editor","reason":null,"projection":null,"source":null,"sha256":null});
    for field in ["reason", "projection", "source", "sha256"] {
        let mut missing = exact.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<OpeningContext>(missing).is_err(),
            "missing {field}"
        );
    }
    let mut extra = exact;
    extra["current_path"] = json!("res://unrelated.gd");
    assert!(serde_json::from_value::<OpeningContext>(extra).is_err());
}
