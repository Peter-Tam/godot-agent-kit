//! Closed, private current-document guard records. No general JSON canonicalizer.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningContext {
    kind: ContextKind,
    reason: Option<String>,
    projection: Option<Projection>,
    source: Option<String>,
    sha256: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContextKind {
    NoSourceEditor,
    CurrentGdscript,
    Unavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    request_id: String,
    session_id: String,
    project_root: String,
    project_device: String,
    project_inode: String,
    path: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
    source_sha256: String,
    source_length: String,
    version: String,
    saved_version: String,
    dirty: bool,
    resource_edited: bool,
    has_undo: bool,
    has_redo: bool,
    tool: bool,
    external_editor: bool,
    script_base_id: String,
    properties: Vec<Property>,
    methods: Vec<String>,
    warnings: Warnings,
    global_classes: Vec<String>,
    autoloads: Vec<String>,
    bindings: Vec<Binding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Property {
    name: String,
    r#type: u64,
    hint: u64,
    hint_string: String,
    usage: u64,
    class_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    name: String,
    api_type: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Warnings {
    enable: bool,
    levels: BTreeMap<String, u8>,
    directory_rules: BTreeMap<String, u8>,
}

/// Checked, source-free attribution for the actual private fixture consumer.
/// Fields are not independently constructible authorization tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningValidationBinding {
    request_id: String,
    session_id: String,
    project_root: String,
    project_device: String,
    project_inode: String,
    path: String,
    script_id: String,
    editor_id: String,
    buffer_id: String,
    source_sha256: String,
    source_length: String,
    guard_sha256: String,
}

fn decimal(value: &str) -> Result<u64, &'static str> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("opening_context_mismatch");
    }
    value.parse().map_err(|_| "opening_context_mismatch")
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
fn sorted_names(values: &[String], limit: usize) -> bool {
    values.len() <= limit
        && values.iter().all(|value| name(value))
        && values
            .windows(2)
            .all(|pair| pair[0].as_bytes() < pair[1].as_bytes())
}
fn dynamic(name: &str) -> bool {
    matches!(name, "_get" | "_set" | "_get_property_list")
}

impl OpeningContext {
    fn current(&self) -> Result<(&Projection, &str), &'static str> {
        if self.kind != ContextKind::CurrentGdscript {
            return Err("opening_context_unavailable");
        }
        let projection = self.projection.as_ref().ok_or("opening_context_mismatch")?;
        let source = self.source.as_deref().ok_or("opening_context_mismatch")?;
        if self.reason.is_some() || source.len() > SOURCE_LIMIT_BYTES {
            return Err("opening_context_mismatch");
        }
        projection.check()?;
        if projection.source_sha256 != confined::hex_sha256(source.as_bytes())
            || decimal(&projection.source_length)? != source.len() as u64
            || self.sha256.as_deref() != Some(projection.fingerprint()?.as_str())
        {
            return Err("opening_context_mismatch");
        }
        source_profile(source, projection)?;
        Ok((projection, source))
    }

    /// Independently recompute the private guard, retaining only source-free
    /// attribution while the consuming validator owns the copied source.
    pub fn checked_binding(&self) -> Option<OpeningValidationBinding> {
        let (projection, _) = self.current().ok()?;
        Some(projection.binding(self.sha256.as_deref()?))
    }
}
impl OpeningValidationBinding {
    /// Only completed valid evidence for this exact checked context may release
    /// source/guard hashes to the private native fixture.
    pub fn validated_hashes<'a>(&self, result: &'a ValidationResult) -> Option<(&'a str, &'a str)> {
        let actual = result.opening_binding.as_ref()?;
        if actual != self || !completed(self, result) {
            return None;
        }
        Some((&actual.source_sha256, &actual.guard_sha256))
    }
}
impl Projection {
    fn check(&self) -> Result<(), &'static str> {
        RequestId::new(self.request_id.clone()).map_err(|_| "opening_context_mismatch")?;
        SessionId::new(self.session_id.clone()).map_err(|_| "opening_context_mismatch")?;
        ProjectRoot::new(self.project_root.clone()).map_err(|_| "opening_context_mismatch")?;
        admission::locator(&self.path)?;
        for counter in [
            &self.project_device,
            &self.project_inode,
            &self.script_id,
            &self.editor_id,
            &self.buffer_id,
            &self.source_length,
            &self.version,
            &self.saved_version,
            &self.script_base_id,
        ] {
            decimal(counter)?;
        }
        if [&self.script_id, &self.editor_id, &self.buffer_id]
            .iter()
            .any(|id| id.as_str() == "0")
            || !hash(&self.source_sha256)
            || self.script_base_id != "0"
            || self.tool
            || self.external_editor
        {
            return Err("unsupported_opening_context");
        }
        if !sorted_names(&self.methods, 64)
            || self.methods.iter().any(|method| dynamic(method))
            || !sorted_names(&self.global_classes, 64)
            || !sorted_names(&self.autoloads, 64)
            || self.bindings.len() > 256
            || self
                .bindings
                .iter()
                .any(|binding| !name(&binding.name) || binding.api_type != 0)
            || !self
                .bindings
                .windows(2)
                .all(|pair| pair[0].name.as_bytes() < pair[1].name.as_bytes())
            || self.properties.len() > 64
            || !self
                .properties
                .windows(2)
                .all(|pair| pair[0].name.as_bytes() < pair[1].name.as_bytes())
        {
            return Err("unsupported_opening_context");
        }
        for property in &self.properties {
            let display = property.usage & (64 | 128 | 256) != 0 && property.usage & 4096 == 0;
            if !name(&property.name)
                || property.r#type >= 39
                || property.hint_string.len() > 2048
                || property.class_name.len() > 256
                || (!display
                    && (property.r#type == 24
                        || property.hint != 0
                        || !property.hint_string.is_empty()
                        || !property.class_name.is_empty()
                        || (property.usage & 4096 != 0 && property.usage & 4 != 0)))
            {
                return Err("unsupported_opening_context");
            }
        }
        if self.warnings.levels.len() != ACTIVE_WARNINGS.len()
            || ACTIVE_WARNINGS
                .iter()
                .any(|key| !self.warnings.levels.contains_key(*key))
            || self.warnings.levels.values().any(|value| *value > 2)
            || self.warnings.directory_rules.len() > 32
            || self.warnings.directory_rules.iter().any(|(path, value)| {
                *value > 1
                    || path.len() > 2048
                    || !path.starts_with("res://")
                    || path.contains("..")
                    || path.contains(['"', '\\', '\n', '\r'])
            })
        {
            return Err("unsupported_opening_context");
        }
        Ok(())
    }
    fn binding(&self, guard: &str) -> OpeningValidationBinding {
        OpeningValidationBinding {
            request_id: self.request_id.clone(),
            session_id: self.session_id.clone(),
            project_root: self.project_root.clone(),
            project_device: self.project_device.clone(),
            project_inode: self.project_inode.clone(),
            path: self.path.clone(),
            script_id: self.script_id.clone(),
            editor_id: self.editor_id.clone(),
            buffer_id: self.buffer_id.clone(),
            source_sha256: self.source_sha256.clone(),
            source_length: self.source_length.clone(),
            guard_sha256: guard.into(),
        }
    }
    fn fingerprint(&self) -> Result<String, &'static str> {
        let mut guard = Guard::new();
        guard.frame("godot-agent-kit/open-context/v1");
        guard.object(26);
        guard.key("autoloads");
        guard.strings(&self.autoloads);
        guard.key("bindings");
        guard.array(self.bindings.len());
        for binding in &self.bindings {
            guard.object(2);
            guard.key("api_type");
            guard.integer(binding.api_type);
            guard.key("name");
            guard.string(&binding.name);
        }
        guard.key("buffer_id");
        guard.string(&self.buffer_id);
        guard.key("dirty");
        guard.boolean(self.dirty);
        guard.key("editor_id");
        guard.string(&self.editor_id);
        guard.key("external_editor");
        guard.boolean(self.external_editor);
        guard.key("global_classes");
        guard.strings(&self.global_classes);
        guard.key("has_redo");
        guard.boolean(self.has_redo);
        guard.key("has_undo");
        guard.boolean(self.has_undo);
        guard.key("methods");
        guard.strings(&self.methods);
        guard.key("path");
        guard.string(&self.path);
        guard.key("project_device");
        guard.string(&self.project_device);
        guard.key("project_inode");
        guard.string(&self.project_inode);
        guard.key("project_root");
        guard.string(&self.project_root);
        guard.key("properties");
        guard.array(self.properties.len());
        for property in &self.properties {
            guard.object(6);
            guard.key("class_name");
            guard.string(&property.class_name);
            guard.key("hint");
            guard.integer(property.hint);
            guard.key("hint_string");
            guard.string(&property.hint_string);
            guard.key("name");
            guard.string(&property.name);
            guard.key("type");
            guard.integer(property.r#type);
            guard.key("usage");
            guard.integer(property.usage);
        }
        guard.key("request_id");
        guard.string(&self.request_id);
        guard.key("resource_edited");
        guard.boolean(self.resource_edited);
        guard.key("saved_version");
        guard.string(&self.saved_version);
        guard.key("script_base_id");
        guard.string(&self.script_base_id);
        guard.key("script_id");
        guard.string(&self.script_id);
        guard.key("session_id");
        guard.string(&self.session_id);
        guard.key("source_length");
        guard.string(&self.source_length);
        guard.key("source_sha256");
        guard.string(&self.source_sha256);
        guard.key("tool");
        guard.boolean(self.tool);
        guard.key("version");
        guard.string(&self.version);
        guard.key("warnings");
        guard.object(3);
        guard.key("directory_rules");
        guard.levels(&self.warnings.directory_rules);
        guard.key("enable");
        guard.boolean(self.warnings.enable);
        guard.key("levels");
        guard.levels(&self.warnings.levels);
        if guard.bytes > 256 * 1024 {
            return Err("opening_context_limit");
        }
        let mut hash = String::with_capacity(64);
        for byte in guard.digest.finish().as_ref() {
            use std::fmt::Write as _;
            let _ = write!(hash, "{byte:02x}");
        }
        Ok(hash)
    }
}

// Streaming encoding of this one closed typed projection; never JSON values.
struct Guard {
    digest: Context,
    bytes: usize,
}
impl Guard {
    fn new() -> Self {
        Self {
            digest: Context::new(&SHA256),
            bytes: 0,
        }
    }
    fn write(&mut self, bytes: &[u8]) {
        self.digest.update(bytes);
        self.bytes += bytes.len();
    }
    fn frame(&mut self, value: &str) {
        self.write(&(value.len() as u32).to_be_bytes());
        self.write(value.as_bytes());
    }
    fn key(&mut self, value: &str) {
        self.frame(value);
    }
    fn object(&mut self, count: usize) {
        self.write(b"o");
        self.write(&(count as u32).to_be_bytes());
    }
    fn array(&mut self, count: usize) {
        self.write(b"a");
        self.write(&(count as u32).to_be_bytes());
    }
    fn string(&mut self, value: &str) {
        self.write(b"s");
        self.frame(value);
    }
    fn integer(&mut self, value: u64) {
        self.write(b"u");
        self.write(&value.to_be_bytes());
    }
    fn boolean(&mut self, value: bool) {
        self.write(&[b'b', u8::from(value)]);
    }
    fn strings(&mut self, values: &[String]) {
        self.array(values.len());
        for value in values {
            self.string(value);
        }
    }
    fn levels(&mut self, values: &BTreeMap<String, u8>) {
        self.object(values.len());
        for (name, value) in values {
            self.key(name);
            self.integer(u64::from(*value));
        }
    }
}

fn source_profile(source: &str, projection: &Projection) -> Result<(), &'static str> {
    if source.starts_with('\u{feff}')
        || source
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err("unsupported_opening_source");
    }
    let mut chars = source.char_indices().peekable();
    let mut identifiers = BTreeSet::new();
    let mut extends = false;
    while let Some((start, character)) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        if character == '#' {
            for (_, c) in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
            continue;
        }
        if character == '\'' || character == '"' {
            if extends {
                return Err("unsupported_opening_source");
            }
            let _ = opening_literals::quoted(source, start, character, &mut chars, false)
                .map_err(|_| "unsupported_opening_source")?;
            continue;
        }
        if character == '@' || !character.is_ascii() {
            return Err("unsupported_opening_source");
        }
        if character.is_ascii_alphabetic() || character == '_' {
            while chars
                .peek()
                .is_some_and(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
            {
                chars.next();
            }
            let end = chars.peek().map_or(source.len(), |(offset, _)| *offset);
            let identifier = &source[start..end];
            identifiers.insert(identifier);
            if identifiers.len() > 256
                || matches!(
                    identifier,
                    "class_name" | "static" | "const" | "load" | "preload"
                )
                || dynamic(identifier)
                || projection
                    .global_classes
                    .iter()
                    .any(|name| name == identifier)
                || projection.autoloads.iter().any(|name| name == identifier)
            {
                return Err("unsupported_opening_source");
            }
            if extends {
                if !projection
                    .bindings
                    .iter()
                    .any(|binding| binding.name == identifier && binding.api_type == 0)
                {
                    return Err("unsupported_opening_source");
                }
                if chars.peek().is_some_and(|(_, c)| *c == '.') {
                    return Err("unsupported_opening_source");
                }
            }
            extends = identifier == "extends";
        } else if extends {
            return Err("unsupported_opening_source");
        }
    }
    if extends {
        return Err("unsupported_opening_source");
    }
    Ok(())
}

pub(super) fn source(request: &WireRequest) -> Result<Option<&str>, &'static str> {
    if request
        .source
        .as_ref()
        .is_some_and(|source| source.len() > SOURCE_LIMIT_BYTES)
    {
        return Err("source_limit");
    }
    match request.purpose {
        Purpose::OpenContext => {
            if request.source.is_some() {
                return Err("wrong_purpose");
            }
            let context = request
                .open_context
                .as_ref()
                .ok_or("opening_context_unavailable")?;
            let (projection, source) = context.current()?;
            if projection.request_id != request.request_id
                || projection.session_id != request.session_id
                || projection.project_root != request.project_root
                || projection.path != request.root_path
                || projection.global_classes != request.global_classes
                || projection.warnings.enable != request.warnings.enable
                || projection.warnings.levels != request.warnings.levels
                || projection.warnings.directory_rules != request.warnings.directory_rules
            {
                return Err("opening_context_mismatch");
            }
            recheck_project(request)?;
            Ok(Some(source))
        }
        Purpose::Preflight | Purpose::PostChange | Purpose::Unchanged => {
            if request.open_context.is_some()
                || (request.purpose == Purpose::Preflight) != request.source.is_some()
            {
                return Err("wrong_purpose");
            }
            Ok(request.source.as_deref())
        }
    }
}
pub(super) fn recheck_project(request: &WireRequest) -> Result<(), &'static str> {
    if request.purpose == Purpose::OpenContext {
        let projection = request
            .open_context
            .as_ref()
            .and_then(|context| context.projection.as_ref())
            .ok_or("opening_context_unavailable")?;
        let metadata =
            fs::symlink_metadata(&request.project_root).map_err(|_| "opening_context_changed")?;
        if !metadata.is_dir()
            || metadata.dev() != decimal(&projection.project_device)?
            || metadata.ino() != decimal(&projection.project_inode)?
        {
            return Err("opening_context_changed");
        }
    }
    Ok(())
}
fn completed(projection: &OpeningValidationBinding, result: &ValidationResult) -> bool {
    result.status == "valid"
        && result.reason.is_none()
        && result.purpose == Purpose::OpenContext
        && result.request_id == projection.request_id
        && result.session_id == projection.session_id
        && result.root_path == projection.path
        && result.diagnostics.is_empty()
        && result.cleanup_confirmed
        && result.child_spawned == Some(true)
        && result.child_reaped == Some(true)
        && result.child_pid.is_some_and(|pid| pid != 0)
        && result.clone_extra_files == Some(false)
        && result.clone_log_files == Some(false)
        && result.context_sha256.as_deref().is_some_and(hash)
        && result.finished_unix_ms >= result.started_unix_ms
        && result.elapsed_us <= 9_500_000
        && result.sources.len() == 1
        && result.sources.first().is_some_and(|source| {
            source.path == projection.path
                && source.sha256 == projection.source_sha256
                && source.utf8_bytes as u64
                    == decimal(&projection.source_length).unwrap_or(u64::MAX)
                && source.device.is_some()
                && source.inode.is_some()
                && source.diagnostics_completed
                && source.symbols_completed
        })
}
pub(super) fn complete(request: &WireRequest, result: &mut ValidationResult) {
    if request.purpose != Purpose::OpenContext {
        return;
    }
    let Some(context) = &request.open_context else {
        return;
    };
    let (Some(projection), Some(hash)) = (&context.projection, &context.sha256) else {
        return;
    };
    // Admission already checked this immutable worker-local record.
    let binding = projection.binding(hash);
    if completed(&binding, result) {
        result.opening_binding = Some(binding);
    }
}
pub(super) fn check_receipt(request: &WireRequest, result: &mut ValidationResult) {
    if request.purpose == Purpose::OpenContext {
        if result.status == "valid"
            && !request
                .open_context
                .as_ref()
                .and_then(OpeningContext::checked_binding)
                .is_some_and(|binding| binding.validated_hashes(result).is_some())
        {
            result.unavailable("opening_receipt_mismatch");
        } else if result.status != "valid" {
            result.opening_binding = None;
        }
    } else if result.opening_binding.is_some() {
        result.unavailable("wrong_purpose");
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{admission, project};
    use super::*;

    const CURRENT: &str = "extends RefCounted\nvar value: int = 1 # café\n";

    fn context(request: &WireRequest) -> OpeningContext {
        let identity = fs::symlink_metadata(&request.project_root).unwrap();
        let projection = Projection {
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            project_root: request.project_root.clone(),
            project_device: identity.dev().to_string(),
            project_inode: identity.ino().to_string(),
            path: request.root_path.clone(),
            script_id: "101".into(),
            editor_id: "102".into(),
            buffer_id: "103".into(),
            source_sha256: confined::hex_sha256(CURRENT.as_bytes()),
            source_length: CURRENT.len().to_string(),
            version: "17".into(),
            saved_version: "13".into(),
            dirty: true,
            resource_edited: true,
            has_undo: true,
            has_redo: false,
            tool: false,
            external_editor: false,
            script_base_id: "0".into(),
            properties: vec![Property {
                name: "value".into(),
                r#type: 2,
                hint: 0,
                hint_string: String::new(),
                usage: 4096,
                class_name: String::new(),
            }],
            methods: vec!["value".into()],
            warnings: Warnings {
                enable: request.warnings.enable,
                levels: request.warnings.levels.clone(),
                directory_rules: request.warnings.directory_rules.clone(),
            },
            global_classes: request.global_classes.clone(),
            autoloads: Vec::new(),
            bindings: vec![Binding {
                name: "RefCounted".into(),
                api_type: 0,
            }],
        };
        OpeningContext {
            kind: ContextKind::CurrentGdscript,
            reason: None,
            sha256: Some(projection.fingerprint().unwrap()),
            projection: Some(projection),
            source: Some(CURRENT.into()),
        }
    }
    fn opening() -> (PrivateClone, WireRequest) {
        let (private, mut request) = project();
        request.purpose = Purpose::OpenContext;
        request.open_context = Some(context(&request));
        (private, request)
    }
    fn rehash(context: &mut OpeningContext) {
        context.sha256 = Some(context.projection.as_ref().unwrap().fingerprint().unwrap());
    }

    #[test]
    fn guard_fingerprint_matches_independent_typed_cross_language_vector() {
        let (_private, request) = project();
        let mut context = context(&request);
        let projection = context.projection.as_mut().unwrap();
        projection.request_id = "vector".into();
        projection.project_root = "/private/open-vector".into();
        projection.project_device = "7".into();
        projection.project_inode = "11".into();
        projection
            .warnings
            .levels
            .values_mut()
            .for_each(|value| *value = 1);
        projection.global_classes = vec!["Glob".into()];
        projection.autoloads = vec!["Auto".into()];
        assert_eq!(
            projection.fingerprint().unwrap(),
            "c7cb0ae8c98286c2da33a3154941135dea9cc58b0e61e36afd77428ac67ec18f"
        );
        projection.version = "18".into();
        assert_ne!(
            projection.fingerprint().unwrap(),
            "c7cb0ae8c98286c2da33a3154941135dea9cc58b0e61e36afd77428ac67ec18f"
        );
    }

    #[test]
    fn private_current_source_is_not_a_proposal_and_need_not_equal_d() {
        let (_private, request) = opening();
        let closure = admission(&request).unwrap();
        assert_eq!(closure.sources[0].captured.text, CURRENT);
        assert!(!closure.sources[0].proposed);
        assert_eq!(
            closure.baseline.as_ref().unwrap().text,
            "extends RefCounted\n"
        );
        let limit = Instant::now() + Duration::from_secs(2);
        recheck(&request, &closure, limit).unwrap();
        fs::write(
            Path::new(&request.project_root).join("scripts/main.gd"),
            "extends RefCounted\n# newly saved human source\n",
        )
        .unwrap();
        assert_eq!(recheck(&request, &closure, limit), Err("source_changed"));
    }

    #[test]
    fn display_category_hint_is_not_an_object_property_or_export() {
        let (_private, mut request) = opening();
        let context = request.open_context.as_mut().unwrap();
        context.projection.as_mut().unwrap().properties.insert(
            0,
            Property {
                name: "main.gd".into(),
                r#type: 0,
                hint: 0,
                hint_string: "res://scripts/main.gd".into(),
                usage: 128,
                class_name: String::new(),
            },
        );
        rehash(context);
        assert_eq!(
            admission(&request).map(|closure| closure.sources[0].captured.text.clone()),
            Ok(CURRENT.into())
        );

        let context = request.open_context.as_mut().unwrap();
        let property = &mut context.projection.as_mut().unwrap().properties[0];
        property.usage |= 4096;
        property.r#type = 24;
        rehash(context);
        assert_eq!(
            admission(&request).err(),
            Some("unsupported_opening_context")
        );
    }

    #[test]
    fn current_file_replacement_and_settings_change_invalidate_completed_work() {
        let (_private, request) = opening();
        let closure = admission(&request).unwrap();
        let path = Path::new(&request.project_root).join("scripts/main.gd");
        let bytes = fs::read(&path).unwrap();
        fs::rename(&path, path.with_extension("retained")).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("source_changed")
        );
        let closure = admission(&request).unwrap();
        fs::write(
            Path::new(&request.project_root).join("project.godot"),
            "config_version=5\n[debug]\ngdscript/warnings/enable=false\n",
        )
        .unwrap();
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("context_changed")
        );
    }

    #[test]
    fn proposal_disguise_missing_context_and_wrong_purpose_refuse_before_engine() {
        let (_private, mut request) = opening();
        request.source = Some(CURRENT.into());
        assert_eq!(admission(&request).err(), Some("wrong_purpose"));
        request.source = None;
        request.purpose = Purpose::Unchanged;
        assert_eq!(admission(&request).err(), Some("wrong_purpose"));
        request.purpose = Purpose::OpenContext;
        request.open_context = None;
        let result = collect(&request);
        assert_eq!(
            result.reason.as_deref(),
            Some("opening_context_unavailable")
        );
        assert_eq!(result.child_spawned, Some(false));
        assert!(result.opening_binding.is_none());
    }

    #[test]
    fn source_identity_warning_and_guard_substitution_refuse_admission() {
        for field in ["source", "identity", "warnings", "fingerprint", "project"] {
            let (_private, mut request) = opening();
            let context = request.open_context.as_mut().unwrap();
            match field {
                "source" => context.source.as_mut().unwrap().push_str("# substituted\n"),
                "identity" => context.projection.as_mut().unwrap().buffer_id = "104".into(),
                "warnings" => request.warnings.enable = false,
                "fingerprint" => context.sha256 = Some("0".repeat(64)),
                "project" => {
                    let projection = context.projection.as_mut().unwrap();
                    projection.project_inode =
                        (decimal(&projection.project_inode).unwrap() + 1).to_string();
                    rehash(context);
                }
                _ => unreachable!(),
            }
            let result = collect(&request);
            assert_eq!(result.status, "unavailable", "{field}");
            assert_eq!(result.child_spawned, Some(false), "{field}");
            assert!(result.opening_binding.is_none(), "{field}");
        }
    }

    #[test]
    fn plain_escaped_triple_and_symlink_literals_keep_the_original_confinement_fence() {
        let (private, mut request) = opening();
        let outside = private.0.join("outside.gd");
        fs::write(&outside, "extends RefCounted\n# outside private project\n").unwrap();
        std::os::unix::fs::symlink(
            &outside,
            Path::new(&request.project_root).join("scripts/escape.gd"),
        )
        .unwrap();
        for literal in [
            serde_json::to_string(outside.to_str().unwrap()).unwrap(),
            "\"../../outside.gd\"".into(),
            "'''../../outside.gd'''".into(),
            "\"\\u002e\\u002e/\\u002e\\u002e/outside.gd\"".into(),
            "\"res://scripts/escape.gd\"".into(),
        ] {
            let text = format!("extends RefCounted\nvar text = {literal}\n");
            let context = request.open_context.as_mut().unwrap();
            let projection = context.projection.as_mut().unwrap();
            projection.source_sha256 = confined::hex_sha256(text.as_bytes());
            projection.source_length = text.len().to_string();
            context.source = Some(text);
            rehash(context);
            let result = collect(&request);
            assert_eq!(result.status, "unavailable", "{literal}");
            assert_eq!(result.reason.as_deref(), Some("unsafe_link"), "{literal}");
            assert_eq!(result.child_spawned, Some(false), "{literal}");
            assert!(result.opening_binding.is_none(), "{literal}");
        }
        let text = "extends RefCounted\n# var s = \"../../outside.gd\"\n";
        let context = request.open_context.as_mut().unwrap();
        let projection = context.projection.as_mut().unwrap();
        projection.source_sha256 = confined::hex_sha256(text.as_bytes());
        projection.source_length = text.len().to_string();
        context.source = Some(text.into());
        rehash(context);
        assert_eq!(collect(&request).reason.as_deref(), Some("wrong_binary"));
    }

    #[test]
    fn closed_typed_metadata_rejects_missing_unknown_unsorted_and_incompatible_fields() {
        let (_private, request) = opening();
        let native = serde_json::to_value(request.open_context.as_ref().unwrap()).unwrap();
        let mut missing = native.clone();
        missing["projection"]
            .as_object_mut()
            .unwrap()
            .remove("has_undo");
        assert!(serde_json::from_value::<OpeningContext>(missing).is_err());
        let mut unknown = native.clone();
        unknown["projection"]["collection_tick_us"] = json!("42");
        assert!(serde_json::from_value::<OpeningContext>(unknown).is_err());
        for (field, value) in [
            ("methods", json!(["z", "a"])),
            ("buffer_id", json!("0103")),
            ("script_base_id", json!("2")),
            ("tool", json!(true)),
            (
                "properties",
                json!([{"name":"value","type":24,"hint":0,"hint_string":"",
                                   "usage":4096,"class_name":""}]),
            ),
            ("bindings", json!([{"name":"RefCounted","api_type":2}])),
        ] {
            let mut changed = native.clone();
            changed["projection"][field] = value;
            let mut context: OpeningContext = serde_json::from_value(changed).unwrap();
            rehash(&mut context);
            assert!(context.checked_binding().is_none(), "{field}");
        }
    }

    #[test]
    fn standalone_profile_ignores_literal_tokens_but_never_follows_dependencies() {
        let (_private, request) = opening();
        let projection = request
            .open_context
            .as_ref()
            .unwrap()
            .projection
            .as_ref()
            .unwrap();
        assert!(source_profile(
            "extends RefCounted\n# @tool preload\nvar s = \"load(\\\"x\\\")\"\n",
            projection
        )
        .is_ok());
        assert!(source_profile("var s = '''preload(\"x\")\n@tool'''\n", projection).is_ok());
        for text in [
            "@tool\n",
            "extends \"base.gd\"\n",
            "const X = 1\n",
            "func _get_property_list(): pass\n",
            "var s = preload(\"base.gd\")\n",
            "extends RefCounted.Base\n",
            "var café = 1\n",
            "var s = \"unterminated\n",
        ] {
            assert_eq!(
                source_profile(text, projection),
                Err("unsupported_opening_source"),
                "{text}"
            );
        }
    }

    #[test]
    fn incomplete_invalid_wrong_purpose_or_unclean_receipts_never_release_hashes() {
        let (_private, request) = opening();
        let context = request.open_context.as_ref().unwrap();
        let binding = context.checked_binding().unwrap();
        let closure = admission(&request).unwrap();
        let mut incomplete = ValidationResult::new(&request);
        attribute(&mut incomplete, &closure.sources[0]);
        incomplete.status = "valid".into();
        incomplete.context_sha256 = Some("a".repeat(64));
        incomplete.opening_binding = Some(binding.clone());
        assert!(binding.validated_hashes(&incomplete).is_none());
        // Exercise completion/purpose/replay refusal independently of parser
        // validity, which is proved by the actual stock acceptance fixture.
        incomplete.sources[0].diagnostics_completed = true;
        incomplete.sources[0].symbols_completed = true;
        incomplete.child_spawned = Some(true);
        incomplete.child_reaped = Some(true);
        incomplete.child_pid = Some(42);
        incomplete.clone_extra_files = Some(false);
        incomplete.clone_log_files = Some(false);
        assert!(completed(&binding, &incomplete));
        for field in [
            "invalid", "purpose", "cleanup", "logs", "effects", "session", "buffer", "source",
            "guard", "missing",
        ] {
            let mut changed = incomplete.clone();
            match field {
                "invalid" => changed.status = "invalid".into(),
                "purpose" => changed.purpose = Purpose::Preflight,
                "cleanup" => changed.cleanup_confirmed = false,
                "logs" => changed.clone_log_files = Some(true),
                "effects" => changed.clone_extra_files = None,
                "session" => changed.session_id = "fedcba9876543210fedcba9876543210".into(),
                "buffer" => changed.opening_binding.as_mut().unwrap().buffer_id = "999".into(),
                "source" => changed.sources[0].sha256 = "0".repeat(64),
                "guard" => changed.opening_binding.as_mut().unwrap().guard_sha256 = "0".repeat(64),
                "missing" => changed.opening_binding = None,
                _ => unreachable!(),
            }
            assert!(binding.validated_hashes(&changed).is_none(), "{field}");
        }
    }
}
