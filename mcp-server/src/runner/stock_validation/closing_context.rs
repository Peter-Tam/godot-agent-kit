//! Private remaining-document validation for guarded native closing.
use super::opening_context::{
    bounded, context_record, ContextDecode, ContextSeed, DecodeBudget, Guard,
    OpeningValidationBinding, Records,
};
use super::*;

fn roster<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<DocumentIdentity>, D::Error> {
    bounded::<_, _, 8>(d)
}
fn protected<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<ProtectedIdentity>, D::Error> {
    bounded::<_, _, 7>(d)
}

pub struct CloseContext {
    projection: CloseProjection,
    documents: Vec<OpeningContext>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseProjection {
    request_id: String,
    session_id: String,
    project_root: String,
    project_device: String,
    project_inode: String,
    target_path: String,
    target: CloseTarget,
    selected_script_id: String,
    selected_editor_id: String,
    selected_buffer_id: String,
    idle_parse_delay_us: String,
    idle_parse_error_delay_us: String,
    effective_sha256: String,
    #[serde(deserialize_with = "roster")]
    roster: Vec<DocumentIdentity>,
    #[serde(deserialize_with = "protected")]
    protected: Vec<ProtectedIdentity>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseTarget {
    target_device: String,
    target_inode: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
    source_sha256: String,
    source_length: String,
    version: String,
    saved_version: String,
    dirty: bool,
    resource_edited: bool,
    guard_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentIdentity {
    path: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedIdentity {
    path: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
    guard_sha256: String,
}
impl<'de> Deserialize<'de> for CloseContext {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::decode(d, &mut DecodeBudget::new())
    }
}
context_record!(CloseContext {
    projection: CloseProjection => |v| v,
    documents: Records<OpeningContext, 7> => |v: Records<OpeningContext, 7>| v.0,
});
context_record!(CloseProjection {
    request_id: String => |v| v, session_id: String => |v| v,
    project_root: String => |v| v, project_device: String => |v| v, project_inode: String => |v| v,
    target_path: String => |v| v, target: CloseTarget => |v| v,
    selected_script_id: String => |v| v, selected_editor_id: String => |v| v,
    selected_buffer_id: String => |v| v, idle_parse_delay_us: String => |v| v,
    idle_parse_error_delay_us: String => |v| v,
    effective_sha256: String => |v| v,
    roster: Records<DocumentIdentity, 8> => |v: Records<DocumentIdentity, 8>| v.0,
    protected: Records<ProtectedIdentity, 7> => |v: Records<ProtectedIdentity, 7>| v.0,
});
context_record!(CloseTarget {
    target_device: String => |v| v, target_inode: String => |v| v,
    script_id: String => |v| v, editor_id: String => |v| v, buffer_id: String => |v| v,
    source_sha256: String => |v| v, source_length: String => |v| v,
    version: String => |v| v, saved_version: String => |v| v,
    dirty: bool => |v| v, resource_edited: bool => |v| v, guard_sha256: String => |v| v,
});
context_record!(DocumentIdentity {
    path: String => |v| v, script_id: String => |v| v, editor_id: String => |v| v, buffer_id: String => |v| v,
});
context_record!(ProtectedIdentity {
    path: String => |v| v, script_id: String => |v| v, editor_id: String => |v| v,
    buffer_id: String => |v| v, guard_sha256: String => |v| v,
});
#[derive(Serialize)]
pub struct CloseValidation {
    status: String,
    reason: Option<String>,
    guard_sha256: Option<String>,
    receipt_bindings: Vec<CloseReceiptBinding>,
    results: Vec<ValidationResult>,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CloseReceiptBinding {
    request_id: String,
    session_id: String,
    target_path: String,
    document_path: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
    source_sha256: String,
    utf8_bytes: usize,
    guard_sha256: String,
    context_sha256: String,
}
fn decimal(value: &str) -> bool {
    !value.is_empty()
        && (value.len() == 1 || !value.starts_with('0'))
        && value.bytes().all(|b| b.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn ids(script: &str, editor: &str, buffer: &str) -> bool {
    [script, editor, buffer]
        .iter()
        .all(|s| decimal(s) && *s != "0")
}
fn identity(
    g: &mut Guard,
    path: &str,
    script: &str,
    editor: &str,
    buffer: &str,
    guard: Option<&str>,
) {
    g.object(if guard.is_some() { 5 } else { 4 });
    g.key("buffer_id");
    g.string(buffer);
    g.key("editor_id");
    g.string(editor);
    if let Some(guard) = guard {
        g.key("guard_sha256");
        g.string(guard);
    }
    g.key("path");
    g.string(path);
    g.key("script_id");
    g.string(script);
}
impl CloseProjection {
    fn encoding(&self) -> Guard {
        let mut g = Guard::new();
        g.frame("godot-agent-kit/close-context/v1");
        g.object(15);
        g.key("effective_sha256");
        g.string(&self.effective_sha256);
        g.key("idle_parse_delay_us");
        g.string(&self.idle_parse_delay_us);
        g.key("idle_parse_error_delay_us");
        g.string(&self.idle_parse_error_delay_us);
        g.key("project_device");
        g.string(&self.project_device);
        g.key("project_inode");
        g.string(&self.project_inode);
        g.key("project_root");
        g.string(&self.project_root);
        g.key("protected");
        g.array(self.protected.len());
        for p in &self.protected {
            identity(
                &mut g,
                &p.path,
                &p.script_id,
                &p.editor_id,
                &p.buffer_id,
                Some(&p.guard_sha256),
            );
        }
        g.key("request_id");
        g.string(&self.request_id);
        g.key("roster");
        g.array(self.roster.len());
        for p in &self.roster {
            identity(
                &mut g,
                &p.path,
                &p.script_id,
                &p.editor_id,
                &p.buffer_id,
                None,
            );
        }
        g.key("selected_buffer_id");
        g.string(&self.selected_buffer_id);
        g.key("selected_editor_id");
        g.string(&self.selected_editor_id);
        g.key("selected_script_id");
        g.string(&self.selected_script_id);
        g.key("session_id");
        g.string(&self.session_id);
        g.key("target");
        g.object(12);
        g.key("buffer_id");
        g.string(&self.target.buffer_id);
        g.key("dirty");
        g.boolean(self.target.dirty);
        g.key("editor_id");
        g.string(&self.target.editor_id);
        g.key("guard_sha256");
        g.string(&self.target.guard_sha256);
        g.key("resource_edited");
        g.boolean(self.target.resource_edited);
        g.key("saved_version");
        g.string(&self.target.saved_version);
        g.key("script_id");
        g.string(&self.target.script_id);
        g.key("source_length");
        g.string(&self.target.source_length);
        g.key("source_sha256");
        g.string(&self.target.source_sha256);
        g.key("target_device");
        g.string(&self.target.target_device);
        g.key("target_inode");
        g.string(&self.target.target_inode);
        g.key("version");
        g.string(&self.target.version);
        g.key("target_path");
        g.string(&self.target_path);
        g
    }
}
impl CloseContext {
    fn checked(
        &self,
        request: &RequestId,
        session: &SessionId,
        root: &ProjectRoot,
        target: &ResourcePath,
    ) -> Result<(String, Vec<OpeningValidationBinding>), &'static str> {
        let p = &self.projection;
        if p.request_id != request.as_str()
            || p.session_id != session.as_str()
            || p.project_root != root.as_str()
            || p.target_path != target.as_str()
            || !decimal(&p.project_device)
            || !decimal(&p.project_inode)
            || !decimal(&p.idle_parse_delay_us)
            || !decimal(&p.idle_parse_error_delay_us)
            || !hash(&p.effective_sha256)
            || !ids(
                &p.target.script_id,
                &p.target.editor_id,
                &p.target.buffer_id,
            )
            || [
                &p.target.target_device,
                &p.target.target_inode,
                &p.target.source_length,
                &p.target.version,
                &p.target.saved_version,
            ]
            .iter()
            .any(|s| !decimal(s))
            || p.target
                .source_length
                .parse::<usize>()
                .unwrap_or(usize::MAX)
                > SOURCE_LIMIT_BYTES
            || !hash(&p.target.source_sha256)
            || !hash(&p.target.guard_sha256)
            || p.target.dirty
            || p.target.resource_edited
            || p.target.version != p.target.saved_version
            || p.roster.is_empty()
            || p.roster.len() > 8
            || p.protected.len() > 7
            || self.documents.len() != p.protected.len()
            || self.documents.len() + 1 != p.roster.len()
        {
            return Err("close_context_mismatch");
        }
        let metadata = fs::symlink_metadata(root.as_str()).map_err(|_| "close_context_changed")?;
        if !metadata.is_dir()
            || metadata.dev().to_string() != p.project_device
            || metadata.ino().to_string() != p.project_inode
        {
            return Err("close_context_changed");
        }
        let mut paths = BTreeSet::new();
        let mut scripts = BTreeSet::new();
        let mut editors = BTreeSet::new();
        let mut buffers = BTreeSet::new();
        for d in &p.roster {
            admission::locator(&d.path)?;
            if !ids(&d.script_id, &d.editor_id, &d.buffer_id)
                || !paths.insert(&d.path)
                || !scripts.insert(&d.script_id)
                || !editors.insert(&d.editor_id)
                || !buffers.insert(&d.buffer_id)
            {
                return Err("close_context_mismatch");
            }
        }
        if !p
            .roster
            .windows(2)
            .all(|v| v[0].path.as_bytes() < v[1].path.as_bytes())
            || !p
                .protected
                .windows(2)
                .all(|v| v[0].path.as_bytes() < v[1].path.as_bytes())
            || !p.roster.iter().any(|d| {
                d.path == p.target_path
                    && d.script_id == p.target.script_id
                    && d.editor_id == p.target.editor_id
                    && d.buffer_id == p.target.buffer_id
            })
            || !([
                &p.selected_script_id,
                &p.selected_editor_id,
                &p.selected_buffer_id,
            ]
            .iter()
            .all(|s| s.as_str() == "0")
                || p.roster.iter().any(|d| {
                    d.script_id == p.selected_script_id
                        && d.editor_id == p.selected_editor_id
                        && d.buffer_id == p.selected_buffer_id
                }))
        {
            return Err("close_context_mismatch");
        }
        let guard = p.encoding();
        let mut metadata_bytes = guard.bytes - 4 - "godot-agent-kit/close-context/v1".len();
        let mut source_bytes = 0usize;
        let mut bindings = Vec::with_capacity(self.documents.len());
        for (document, protected) in self.documents.iter().zip(&p.protected) {
            let (b, bytes, effective) = document.close_facts().ok_or("close_document_mismatch")?;
            if effective != p.effective_sha256 {
                return Err("close_effective_mismatch");
            }
            source_bytes = source_bytes
                .checked_add(
                    b.source_length
                        .parse::<usize>()
                        .map_err(|_| "close_context_limit")?,
                )
                .ok_or("close_context_limit")?;
            metadata_bytes = metadata_bytes
                .checked_add(bytes)
                .ok_or("close_context_limit")?;
            if source_bytes > SOURCE_LIMIT_BYTES || metadata_bytes > 256 * 1024 {
                return Err("close_context_limit");
            }
            if b.request_id != p.request_id
                || b.session_id != p.session_id
                || b.project_root != p.project_root
                || b.project_device != p.project_device
                || b.project_inode != p.project_inode
                || b.path == p.target_path
                || b.path != protected.path
                || b.script_id != protected.script_id
                || b.editor_id != protected.editor_id
                || b.buffer_id != protected.buffer_id
                || b.guard_sha256 != protected.guard_sha256
                || !p.roster.iter().any(|d| {
                    d.path == b.path
                        && d.script_id == b.script_id
                        && d.editor_id == b.editor_id
                        && d.buffer_id == b.buffer_id
                })
            {
                return Err("close_document_mismatch");
            }
            bindings.push(b);
        }
        if metadata_bytes > 256 * 1024 {
            return Err("close_context_limit");
        }
        Ok((guard.finish(), bindings))
    }
}

/// Validate a captured protection set with actual children under one deadline.
///
/// `request` names the closing target and the actual validator environment. It
/// must use `CloseContext` without a proposal or an individual source context;
/// each admitted protected document supplies that context independently.
pub fn validate_close_context(
    context: CloseContext,
    expected_guard: &str,
    request: ValidationRequest,
    remaining_budget_ms: u64,
) -> CloseValidation {
    let clock = AttemptClock::start();
    let ValidationRequest {
        request_id,
        session_id,
        project_root: project,
        root_path: target,
        source,
        purpose,
        open_context,
        warnings,
        global_classes,
        official_binary: binary,
    } = request;
    let mut output = CloseValidation {
        status: "unavailable".into(),
        reason: None,
        guard_sha256: None,
        receipt_bindings: Vec::new(),
        results: Vec::new(),
    };
    let work = (|| -> Result<(), &'static str> {
        if purpose != Purpose::CloseContext || source.is_some() || open_context.is_some() {
            return Err("close_validation_request_mismatch");
        }
        if !(1..=9000).contains(&remaining_budget_ms) {
            return Err("close_budget_invalid");
        }
        let deadline_at = clock.started + Duration::from_millis(remaining_budget_ms);
        let (guard, bindings) = context.checked(&request_id, &session_id, &project, &target)?;
        if guard != expected_guard {
            return Err("close_guard_mismatch");
        }
        output.guard_sha256 = Some(guard);
        for (document, binding) in context.documents.into_iter().zip(bindings) {
            deadline(deadline_at)?;
            let input = ValidationRequest {
                request_id: request_id.clone(),
                session_id: session_id.clone(),
                project_root: project.clone(),
                root_path: ResourcePath::new(binding.path.clone())
                    .map_err(|_| "close_document_mismatch")?,
                source: None,
                purpose: Purpose::CloseContext,
                open_context: Some(document),
                warnings: warnings.clone(),
                global_classes: global_classes.clone(),
                official_binary: binary.clone(),
            };
            let result = validate_until(input, clock, &AtomicBool::new(false), deadline_at);
            let valid = binding.close_completed(&result);
            if !valid && binding.close_invalid_completed(&result) {
                output.status = "invalid".into();
            }
            if valid {
                output.receipt_bindings.push(CloseReceiptBinding {
                    request_id: binding.request_id,
                    session_id: binding.session_id,
                    target_path: target.as_str().into(),
                    document_path: binding.path,
                    script_id: binding.script_id,
                    editor_id: binding.editor_id,
                    buffer_id: binding.buffer_id,
                    source_sha256: binding.source_sha256,
                    utf8_bytes: binding
                        .source_length
                        .parse()
                        .map_err(|_| "close_document_mismatch")?,
                    guard_sha256: binding.guard_sha256,
                    context_sha256: result
                        .context_sha256
                        .clone()
                        .ok_or("close_receipt_mismatch")?,
                });
            }
            output.results.push(result);
            if !valid {
                return Err("close_validation_failed");
            }
        }
        deadline(deadline_at)?;
        Ok(())
    })();
    match work {
        Ok(()) => output.status = "valid".into(),
        Err(reason) => {
            output.reason = Some(reason.into());
            output.receipt_bindings.clear();
        }
    }
    output
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TargetCapture {
    #[serde(deserialize_with = "super::opening_context::captured_source")]
    source: String,
    project_device: String,
    project_inode: String,
    target_device: String,
    target_inode: String,
    source_sha256: String,
    source_length: String,
}
/// Confined, bounded D capture; never runs validation or loads a Script.
pub fn capture_close_target(
    project: &ProjectRoot,
    target: &ResourcePath,
) -> Result<TargetCapture, &'static str> {
    admission::locator(target.as_str())?;
    let before = fs::symlink_metadata(project.as_str()).map_err(|_| "close_disk_unavailable")?;
    let source = confined::source(project, target)
        .map_err(|_| "close_disk_unavailable")?
        .ok_or("close_disk_missing")?;
    let after = fs::symlink_metadata(project.as_str()).map_err(|_| "close_disk_unavailable")?;
    if !before.is_dir() || before.dev() != after.dev() || before.ino() != after.ino() {
        return Err("close_disk_changed");
    }
    Ok(TargetCapture {
        source: source.text,
        project_device: before.dev().to_string(),
        project_inode: before.ino().to_string(),
        target_device: source.device.to_string(),
        target_inode: source.inode.to_string(),
        source_sha256: source.sha256,
        source_length: source.bytes.to_string(),
    })
}
pub fn recheck_close_target(
    project: &ProjectRoot,
    target: &ResourcePath,
    expected: &TargetCapture,
) -> Result<(bool, TargetCapture), &'static str> {
    let current = capture_close_target(project, target)?;
    Ok((&current == expected, current))
}

#[cfg(test)]
mod tests {
    use super::super::tests::project;
    use super::*;
    fn private_record(source: &str, reason: &str) -> String {
        format!(
            r#"{{"kind":"unavailable","reason":{},"projection":null,"source":{},"sha256":null}}"#,
            serde_json::to_string(reason).unwrap(),
            serde_json::to_string(source).unwrap(),
        )
    }
    #[test]
    fn later_source_is_rejected_at_the_remaining_budget_before_later_fields() {
        let first = private_record(&"a".repeat(SOURCE_LIMIT_BYTES - 1), "");
        let exact = format!("[{first},{}]", private_record("b", ""));
        let mut d = serde_json::Deserializer::from_str(&exact);
        let mut budget = DecodeBudget::new();
        let records = Records::<OpeningContext, 7>::decode(&mut d, &mut budget).unwrap();
        assert_eq!(records.0.len(), 2);
        assert_eq!(budget.source, 0);
        // The malformed projection must not be reached: source rejection occurs
        // before the string is retained and before subsequent fields decode.
        let over = format!(r#"[{first},{{"source":"bc","projection":false}}]"#);
        let mut d = serde_json::Deserializer::from_str(&over);
        let mut budget = DecodeBudget::new();
        let error = Records::<OpeningContext, 7>::decode(&mut d, &mut budget)
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"));
        assert_eq!(budget.source, 1);
    }
    #[test]
    fn later_metadata_is_rejected_before_retaining_the_oversized_value() {
        let empty = private_record("", "");
        let mut d = serde_json::Deserializer::from_str(&empty);
        let mut budget = DecodeBudget::new();
        OpeningContext::decode(&mut d, &mut budget).unwrap();
        let record_overhead = 256 * 1024 - budget.metadata;
        let first = private_record("", &"a".repeat(256 * 1024 - 4 - 2 * record_overhead - 1));
        let exact = format!("[{first},{}]", private_record("", "b"));
        let mut d = serde_json::Deserializer::from_str(&exact);
        let mut budget = DecodeBudget::new();
        assert!(Records::<OpeningContext, 7>::decode(&mut d, &mut budget).is_ok());
        assert_eq!(budget.metadata, 0);
        let over = format!(
            r#"[{first},{{"kind":"unavailable","projection":null,"source":"","sha256":null,"reason":"bc"}}]"#
        );
        let mut d = serde_json::Deserializer::from_str(&over);
        let mut budget = DecodeBudget::new();
        let error = Records::<OpeningContext, 7>::decode(&mut d, &mut budget)
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"));
        assert_eq!(budget.metadata, 1);
        assert_eq!(budget.source, SOURCE_LIMIT_BYTES);
    }
    #[test]
    fn later_protected_property_metadata_is_bounded_before_the_unread_tail() {
        let first = private_record("", &"a".repeat(128 * 1024));
        let input = format!(
            r#"[{first},{{"projection":{{"properties":[{{"hint_string":"{}","type":"#,
            "b".repeat(128 * 1024),
        );
        let mut d = serde_json::Deserializer::from_str(&input);
        let mut budget = DecodeBudget::new();
        let error = Records::<OpeningContext, 7>::decode(&mut d, &mut budget)
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"), "{error}");
        assert!(budget.metadata < 128 * 1024);
        assert_eq!(budget.source, SOURCE_LIMIT_BYTES);
    }
    #[test]
    fn eighth_document_is_rejected_without_decoding_its_fields() {
        let record = private_record("", "");
        let first_seven = std::iter::repeat_n(record.as_str(), 7)
            .collect::<Vec<_>>()
            .join(",");
        let exact = format!("[{first_seven}]");
        let mut d = serde_json::Deserializer::from_str(&exact);
        assert_eq!(
            Records::<OpeningContext, 7>::decode(&mut d, &mut DecodeBudget::new())
                .unwrap()
                .0
                .len(),
            7
        );
        let over = format!(r#"[{first_seven},{{"source":false}}]"#);
        let mut d = serde_json::Deserializer::from_str(&over);
        let error = Records::<OpeningContext, 7>::decode(&mut d, &mut DecodeBudget::new())
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"));
    }
    #[test]
    fn ordered_context_fields_preserve_required_unknown_and_duplicate_checks() {
        let record = private_record("x", "");
        let reversed =
            r#"{"sha256":null,"source":"x","projection":null,"reason":"","kind":"unavailable"}"#;
        let mut budget = DecodeBudget::new();
        let mut d = serde_json::Deserializer::from_str(reversed);
        assert!(OpeningContext::decode(&mut d, &mut budget).is_ok());
        assert_eq!(budget.source, SOURCE_LIMIT_BYTES - 1);
        assert!(serde_json::from_str::<OpeningContext>(&record).is_ok());
        for (input, message) in [
            (r#"{"source":"x","source":false}"#, "duplicate field"),
            (r#"{"unknown":false,"source":false}"#, "unknown field"),
            (
                r#"{"kind":"unavailable","reason":null,"projection":null,"source":null}"#,
                "missing field",
            ),
        ] {
            let error = serde_json::from_str::<OpeningContext>(input).err().unwrap();
            assert!(error.to_string().contains(message), "{error}");
        }
        // CloseContext does not buffer a projection before decoding documents.
        let input = format!(
            r#"{{"documents":[{},{{"source":"xx","projection":false}}],"projection":false}}"#,
            private_record(&"a".repeat(SOURCE_LIMIT_BYTES - 1), "")
        );
        let error = serde_json::from_str::<CloseContext>(&input).err().unwrap();
        assert!(error.to_string().contains("close_context_limit"));
        for (input, message) in [
            (r#"{"documents":[],"documents":false}"#, "duplicate field"),
            (r#"{"documents":[],"extra":false}"#, "unknown field"),
        ] {
            let error = serde_json::from_str::<CloseContext>(input).err().unwrap();
            assert!(error.to_string().contains(message), "{error}");
        }
    }
    #[test]
    fn aggregate_framing_matches_independent_cross_language_vector() {
        let mut p: CloseProjection = serde_json::from_value(json!({
            "request_id":"test","session_id":"0123456789abcdef0123456789abcdef",
            "project_root":"/fixture/project","project_device":"1","project_inode":"2",
            "target_path":"res://scripts/main.gd",
            "target":{"target_device":"3","target_inode":"4","script_id":"5","editor_id":"6",
                "buffer_id":"7","source_sha256":"c".repeat(64),"source_length":"19","version":"8",
                "saved_version":"8","dirty":false,"resource_edited":false,"guard_sha256":"a".repeat(64)},
            "selected_script_id":"5","selected_editor_id":"6","selected_buffer_id":"7",
            "idle_parse_delay_us":"1000000","idle_parse_error_delay_us":"250000","effective_sha256":"b".repeat(64),
            "roster":[{"path":"res://scripts/main.gd","script_id":"5","editor_id":"6","buffer_id":"7"}],
            "protected":[]
        })).unwrap();
        let expected = "f558e1b26512002df2110d4d4e1629927774d3c00ca65cb7bd8c550bd6427094";
        assert_eq!(p.encoding().finish(), expected);
        p.idle_parse_delay_us = "1000001".into();
        assert_ne!(p.encoding().finish(), expected);
        p.idle_parse_delay_us = "1000000".into();
        p.idle_parse_error_delay_us = "250001".into();
        assert_ne!(p.encoding().finish(), expected);
        p.idle_parse_error_delay_us = "250000".into();
        p.target.saved_version = "9".into();
        assert_ne!(p.encoding().finish(), expected);
    }
    fn empty(request: &WireRequest) -> CloseContext {
        let root = ProjectRoot::new(request.project_root.clone()).unwrap();
        let path = ResourcePath::new(request.root_path.clone()).unwrap();
        let captured = capture_close_target(&root, &path).unwrap();
        serde_json::from_value(json!({
            "projection": {
                "request_id":request.request_id, "session_id":request.session_id,
                "project_root":request.project_root, "project_device":captured.project_device,
                "project_inode":captured.project_inode, "target_path":request.root_path,
                "target":{"target_device":captured.target_device,"target_inode":captured.target_inode,
                    "script_id":"1","editor_id":"2","buffer_id":"3","source_sha256":captured.source_sha256,
                    "source_length":captured.source_length,"version":"4","saved_version":"4",
                    "dirty":false,"resource_edited":false,"guard_sha256":"a".repeat(64)},
                "selected_script_id":"1","selected_editor_id":"2","selected_buffer_id":"3",
                "idle_parse_delay_us":"1000000","idle_parse_error_delay_us":"250000","effective_sha256":"b".repeat(64),
                "roster":[{"path":request.root_path,"script_id":"1","editor_id":"2","buffer_id":"3"}],
                "protected":[]
            }, "documents":[]
        })).unwrap()
    }
    fn validate(
        context: CloseContext,
        guard: &str,
        request: &WireRequest,
        budget: u64,
    ) -> CloseValidation {
        validate_close_context(
            context,
            guard,
            ValidationRequest {
                request_id: RequestId::new(request.request_id.clone()).unwrap(),
                session_id: SessionId::new(request.session_id.clone()).unwrap(),
                project_root: ProjectRoot::new(request.project_root.clone()).unwrap(),
                root_path: ResourcePath::new(request.root_path.clone()).unwrap(),
                source: None,
                purpose: Purpose::CloseContext,
                open_context: None,
                warnings: request.warnings.clone(),
                global_classes: request.global_classes.clone(),
                official_binary: PathBuf::from("/fixture/no-validator"),
            },
            budget,
        )
    }
    #[test]
    fn empty_remaining_set_never_validates_even_a_syntax_invalid_target() {
        let (_private, request) = project();
        fs::write(
            Path::new(&request.project_root).join("scripts/main.gd"),
            "func :\\n",
        )
        .unwrap();
        let context = empty(&request);
        let guard = context.projection.encoding().finish();
        let result = validate(context, &guard, &request, 9000);
        assert_eq!(result.status, "valid");
        assert!(result.results.is_empty());
        assert!(result.receipt_bindings.is_empty());
    }
    #[test]
    fn dirty_or_cross_request_aggregate_cannot_authorize_a_child_or_close() {
        let (_private, request) = project();
        for change in [
            "dirty",
            "edited",
            "request",
            "session",
            "selection",
            "noncanonical",
            "target_identity",
            "guard",
        ] {
            let mut context = empty(&request);
            match change {
                "dirty" => context.projection.target.dirty = true,
                "edited" => context.projection.target.resource_edited = true,
                "request" => context.projection.request_id = "other".into(),
                "session" => context.projection.session_id = "f".repeat(32),
                "selection" => context.projection.selected_buffer_id = "9".into(),
                "noncanonical" => context.projection.idle_parse_delay_us = "01".into(),
                "target_identity" => context.projection.target.editor_id = "9".into(),
                "guard" => (),
                _ => unreachable!(),
            }
            let expected = if change == "guard" {
                "0".repeat(64)
            } else {
                context.projection.encoding().finish()
            };
            let result = validate(context, &expected, &request, 9000);
            assert_eq!(result.status, "unavailable", "{change}");
            assert!(result.results.is_empty(), "{change}");
            assert!(result.receipt_bindings.is_empty(), "{change}");
        }
    }
    #[test]
    fn invalid_original_budget_cannot_become_a_fresh_lease() {
        let (_private, request) = project();
        for budget in [0, 9001, u64::MAX] {
            let context = empty(&request);
            let guard = context.projection.encoding().finish();
            let result = validate(context, &guard, &request, budget);
            assert_eq!(result.reason.as_deref(), Some("close_budget_invalid"));
            assert!(result.results.is_empty());
        }
    }
    #[test]
    fn bounded_roster_deserialization_rejects_one_over_without_silent_omission() {
        let entry = json!({"path":"res://x.gd","script_id":"1","editor_id":"2","buffer_id":"3"});
        let exact: Vec<DocumentIdentity> = roster(serde_json::json!([
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone()
        ]))
        .unwrap();
        assert_eq!(exact.len(), 8);
        let one_over = serde_json::json!([
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry
        ]);
        assert!(roster(one_over).is_err());
    }
    #[test]
    fn final_disk_reacquisition_detects_same_text_inode_replacement_and_symlink() {
        let (_private, request) = project();
        let root = ProjectRoot::new(request.project_root.clone()).unwrap();
        let target = ResourcePath::new(request.root_path.clone()).unwrap();
        let captured = capture_close_target(&root, &target).unwrap();
        assert!(recheck_close_target(&root, &target, &captured).unwrap().0);
        let path = Path::new(root.as_str()).join("scripts/main.gd");
        let replacement = path.with_extension("replacement");
        fs::write(&replacement, &captured.source).unwrap();
        fs::rename(&replacement, &path).unwrap();
        let (unchanged, fresh) = recheck_close_target(&root, &target, &captured).unwrap();
        assert!(!unchanged);
        assert_eq!(fresh.source_sha256, captured.source_sha256);
        assert_ne!(fresh.target_inode, captured.target_inode);
        fs::rename(&path, &replacement).unwrap();
        std::os::unix::fs::symlink(&replacement, &path).unwrap();
        assert!(capture_close_target(&root, &target).is_err());
    }
}
