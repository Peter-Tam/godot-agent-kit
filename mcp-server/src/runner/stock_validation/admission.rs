use super::*;

fn root_path(request: &WireRequest) -> Result<ProjectRoot, &'static str> {
    ProjectRoot::new(request.project_root.clone()).map_err(|_| "unsafe_project")
}
fn locator(value: &str) -> Result<ResourcePath, &'static str> {
    let path = ResourcePath::new(value.to_owned()).map_err(|_| "unsafe_path")?;
    if !value.starts_with("res://")
        || !value.ends_with(".gd")
        || value.contains("::")
        || value.len() > 2048
    {
        return Err("unsupported_source");
    }
    Ok(path)
}
fn capture(root: &ProjectRoot, path: &ResourcePath) -> Result<Option<Captured>, &'static str> {
    confined::source(root, path).map_err(|e| match e {
        confined::CaptureError::TooLarge => "source_limit",
        confined::CaptureError::Changed => "source_changed",
        confined::CaptureError::NonUtf8 => "source_non_utf8",
        confined::CaptureError::Unreadable => "source_unreadable",
        confined::CaptureError::Unsafe => "unsafe_path",
    })
}

#[derive(Clone)]
pub(super) struct Source {
    pub(super) path: ResourcePath,
    pub(super) captured: Captured,
    pub(super) proposed: bool,
}
pub(super) struct Missing {
    pub(super) referring: String,
    pub(super) path: String,
    pub(super) literal: String,
    pub(super) line: u32,
    pub(super) parents: Vec<(u64, u64, u32)>,
}
struct Dependency {
    path: String,
    literal: String,
    line: u32,
}
pub(super) struct Closure {
    pub(super) sources: Vec<Source>,
    pub(super) missing: Vec<Missing>,
    pub(super) settings: Captured,
    pub(super) warnings: WarningSettings,
    pub(super) global_classes: BTreeSet<String>,
    pub(super) baseline: Option<Captured>,
}

pub(super) fn attribute(result: &mut ValidationResult, source: &Source) {
    result.sources.push(SourceEvidence {
        path: source.path.as_str().into(),
        sha256: source.captured.sha256.clone(),
        utf8_bytes: source.captured.bytes,
        device: Some(source.captured.device),
        inode: Some(source.captured.inode),
        diagnostics_completed: false,
        symbols_completed: false,
    });
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Token<'a> {
    Ident(&'a str),
    Str(&'a str, usize),
    Punct(char),
}
// Scanner only locates links and excludes uncertain syntax. The stock parser alone
// decides syntax/types; no scanner decision can produce `valid` or `invalid`.
pub(super) fn tokens(text: &str) -> Result<Vec<Token<'_>>, &'static str> {
    let mut out = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        if c == '#' {
            for (_, n) in chars.by_ref() {
                if n == '\n' {
                    break;
                }
            }
            continue;
        }
        if c == '"' || c == '\'' {
            let content_start = start + c.len_utf8();
            let mut end = None;
            for (offset, n) in chars.by_ref() {
                if n == c {
                    end = Some(offset);
                    break;
                }
                if n == '\\' || n == '\n' || n == '\r' || n.is_control() {
                    return Err("ambiguous_literal");
                }
            }
            let end = end.ok_or("ambiguous_literal")?;
            if chars.peek().is_some_and(|(_, n)| *n == c) {
                return Err("ambiguous_literal");
            }
            out.push(Token::Str(&text[content_start..end], start));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            while chars
                .peek()
                .is_some_and(|(_, n)| n.is_alphanumeric() || *n == '_')
            {
                chars.next();
            }
            let end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            out.push(Token::Ident(&text[start..end]));
        } else if c.is_ascii_digit() {
            while chars
                .peek()
                .is_some_and(|(_, n)| n.is_ascii_alphanumeric() || *n == '_')
            {
                chars.next();
            }
        } else if c.is_ascii_punctuation() {
            out.push(Token::Punct(c));
        } else {
            return Err("unsupported_lexical_context");
        }
        if out.len() > SOURCE_LIMIT_BYTES / 2 {
            return Err("source_limit");
        }
    }
    Ok(out)
}
pub(super) fn resolve(referrer: &str, literal: &str) -> Result<String, &'static str> {
    if literal.is_empty()
        || literal.len() > 2048
        || literal.contains('\\')
        || literal.contains('?')
        || literal.contains('#')
        || literal.starts_with('/')
        || literal.contains("://") && !literal.starts_with("res://")
    {
        return Err("unsafe_link");
    }
    let base = if let Some(remainder) = literal.strip_prefix("res://") {
        remainder.to_owned()
    } else {
        let parent = referrer
            .strip_prefix("res://")
            .ok_or("unsafe_link")?
            .rsplit_once('/')
            .map(|(p, _)| p)
            .unwrap_or("");
        if parent.is_empty() {
            literal.to_owned()
        } else {
            format!("{parent}/{literal}")
        }
    };
    let mut parts = Vec::new();
    for component in base.split('/') {
        match component {
            "" => return Err("unsafe_link"),
            "." => continue,
            ".." => {
                if parts.pop().is_none() {
                    return Err("unsafe_link");
                }
            }
            other if other.contains(':') || other.chars().any(char::is_control) => {
                return Err("unsafe_link")
            }
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        return Err("unsafe_link");
    }
    Ok(format!("res://{}", parts.join("/")))
}
fn links(
    root: &ProjectRoot,
    source: &Source,
    global_classes: &BTreeSet<String>,
) -> Result<Vec<Dependency>, &'static str> {
    let toks = tokens(&source.captured.text)?;
    let declared: BTreeSet<&str> = toks
        .windows(2)
        .filter_map(|pair| match pair {
            [Token::Ident(keyword), Token::Ident(name)]
                if matches!(
                    *keyword,
                    "class" | "const" | "var" | "func" | "signal" | "enum"
                ) =>
            {
                Some(*name)
            }
            _ => None,
        })
        .collect();
    let mut dependencies = Vec::new();
    for (i, token) in toks.iter().enumerate() {
        if let Token::Str(literal, _) = token {
            if literal.is_empty() {
                continue;
            }
            if literal.ends_with(".uid")
                || literal.ends_with(".remap")
                || literal.ends_with(".gdextension")
            {
                return Err("unsupported_context");
            }
            // Document links probe every literal, not only explicit preload/extends.
            // Do not probe a caller-chosen absolute, UID or external path.
            let link = resolve(source.path.as_str(), literal)?;
            let path = ResourcePath::new(link).map_err(|_| "unsafe_link")?;
            let relationship = matches!(i.checked_sub(1).and_then(|j| toks.get(j)),
                Some(Token::Ident(name)) if *name == "extends")
                || (i >= 2
                    && matches!(toks.get(i - 2),
                    Some(Token::Ident(name)) if *name == "preload")
                    && toks.get(i - 1) == Some(&Token::Punct('(')));
            if !relationship || !path.as_str().ends_with(".gd") {
                // Plain strings only request a confined existence probe. Their
                // contents are neither a dependency nor staged source.
                match confined::capture(root, &path, 0) {
                    Ok(_) | Err(confined::CaptureError::TooLarge) => {}
                    Err(_) => return Err("unsafe_link"),
                }
            }
        }
        if let Token::Ident(name) = token {
            if global_classes.contains(*name) {
                // The clone deliberately excludes selected-project class registrations.
                // Even a local declaration with this name may change shadowing/errors.
                return Err("unreproducible_global_class");
            }
            if name.chars().next().is_some_and(char::is_uppercase)
                && !declared.contains(name)
                && !native_base(name)
                && !builtin_name(name)
                && !(i > 0 && toks.get(i - 1) == Some(&Token::Punct('.')))
            {
                return Err("unreproducible_global_class");
            }
            if matches!(
                *name,
                "class_name" | "load" | "ResourceLoader" | "GDExtensionManager" | "EditorInterface"
            ) {
                return Err("unsupported_context");
            }
            if matches!(*name, "icon" | "static_unload")
                && i > 0
                && toks.get(i - 1) == Some(&Token::Punct('@'))
            {
                return Err("unsupported_context");
            }
            if name.starts_with("export") && i > 0 && toks.get(i - 1) == Some(&Token::Punct('@')) {
                return Err("unsupported_context");
            }
            if *name == "preload" {
                let [Some(Token::Punct('(')), Some(Token::Str(literal, offset)), Some(Token::Punct(')'))] =
                    [toks.get(i + 1), toks.get(i + 2), toks.get(i + 3)]
                else {
                    return Err("computed_dependency");
                };
                let dep = resolve(source.path.as_str(), literal)?;
                if !dep.ends_with(".gd") {
                    return Err("non_gd_dependency");
                }
                dependencies.push(Dependency {
                    path: dep,
                    literal: (*literal).into(),
                    line: source.captured.text[..*offset]
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count() as u32,
                });
            }
            if *name == "extends" {
                match toks.get(i + 1) {
                    Some(Token::Str(literal, offset)) => {
                        let dep = resolve(source.path.as_str(), literal)?;
                        if !dep.ends_with(".gd") {
                            return Err("non_gd_dependency");
                        }
                        dependencies.push(Dependency {
                            path: dep,
                            literal: (*literal).into(),
                            line: source.captured.text[..*offset]
                                .bytes()
                                .filter(|b| *b == b'\n')
                                .count() as u32,
                        });
                    }
                    Some(Token::Ident(native)) if native_base(native) => {}
                    _ => return Err("unresolved_base"),
                }
            }
        }
    }
    Ok(dependencies)
}
fn native_base(name: &str) -> bool {
    matches!(
        name,
        "Object"
            | "RefCounted"
            | "Resource"
            | "Node"
            | "Node2D"
            | "Node3D"
            | "Control"
            | "CanvasItem"
            | "CharacterBody2D"
            | "CharacterBody3D"
            | "Area2D"
            | "Area3D"
            | "Sprite2D"
            | "Panel"
            | "Button"
            | "Label"
            | "TextureRect"
            | "EditorPlugin"
            | "EditorScript"
            | "SceneTree"
            | "MainLoop"
            | "HTTPRequest"
            | "Timer"
            | "AnimatedSprite2D"
            | "RigidBody2D"
            | "StaticBody2D"
    )
}
fn builtin_name(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "StringName"
            | "NodePath"
            | "Array"
            | "Dictionary"
            | "Variant"
            | "Color"
            | "Vector2"
            | "Vector2i"
            | "Vector3"
            | "Vector3i"
            | "Vector4"
            | "Vector4i"
            | "Rect2"
            | "Rect2i"
            | "Transform2D"
            | "Transform3D"
            | "Basis"
            | "Plane"
            | "Quaternion"
            | "Projection"
            | "AABB"
            | "RID"
            | "Callable"
            | "Signal"
            | "PackedByteArray"
            | "PackedInt32Array"
            | "PackedInt64Array"
            | "PackedFloat32Array"
            | "PackedFloat64Array"
            | "PackedStringArray"
            | "PackedVector2Array"
            | "PackedVector3Array"
            | "PackedColorArray"
            | "GDScript"
            | "FileAccess"
            | "DirAccess"
            | "OS"
            | "Time"
            | "Engine"
            | "Input"
            | "Math"
            | "PI"
            | "TAU"
            | "INF"
            | "NAN"
            | "TYPE_NIL"
            | "TYPE_INT"
            | "TYPE_STRING"
    )
}

fn context(request: &WireRequest, root: &ProjectRoot) -> Result<Captured, &'static str> {
    // The worker cannot independently query a live editor; its authenticated
    // integration must supply this selected-editor public-API snapshot. This
    // binding prevents substituting another session/project's effective policy.
    if request.warnings.provenance.project_root != request.project_root
        || request.warnings.provenance.session_id != request.session_id
    {
        return Err("context_mismatch");
    }
    if request.warnings.levels.len() != ACTIVE_WARNINGS.len()
        || ACTIVE_WARNINGS
            .iter()
            .any(|key| !request.warnings.levels.contains_key(*key))
    {
        return Err("incomplete_editor_context");
    }
    if request.global_classes.len() > 256
        || request.global_classes.iter().any(|name| {
            name.is_empty()
                || name.len() > 128
                || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
        })
        || request.global_classes.iter().collect::<BTreeSet<_>>().len()
            != request.global_classes.len()
    {
        return Err("incomplete_editor_context");
    }
    let path = ResourcePath::new("res://project.godot").map_err(|_| "unsafe_project")?;
    let config = confined::capture(root, &path, 64 * 1024)
        .map_err(|_| "unsupported_context")?
        .ok_or("unsupported_context")?;
    if !config.text.lines().any(|l| l.trim() == "config_version=5") {
        return Err("unsupported_context");
    }
    let mut section = "";
    for line in config.text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line;
        }
        // Editor plugins are not staged. They cannot supply a stock native
        // ClassDB name; unknown project classes and non-GD resource loads
        // are excluded by the source-only admission below.
        if matches!(section, "[autoload]" | "[gdextension]")
            && !line.is_empty()
            && !line.starts_with('[')
            && !line.starts_with(';')
        {
            return Err("unsupported_context");
        }
        if section == "[debug]" {
            if line.starts_with("gdscript/warnings/exclude_addons=") {
                return Err("unsupported_context");
            }
            if let Some((key, value)) = line
                .strip_prefix("gdscript/warnings/")
                .and_then(|setting| setting.split_once('='))
            {
                if key == "renamed_in_godot_4_hint" {
                    if value != "true" {
                        return Err("unsupported_context");
                    }
                } else if matches!(key, "enable" | "directory_rules") {
                    // Effective selected-editor overrides supersede on-disk bases.
                } else {
                    let base = key.split_once('.').map_or(key, |(base, _)| base);
                    if !ACTIVE_WARNINGS.contains(&base)
                        && !matches!(
                            base,
                            "property_used_as_function"
                                | "constant_used_as_function"
                                | "function_used_as_property"
                                | "enable"
                                | "directory_rules"
                        )
                    {
                        return Err("unsupported_context");
                    }
                    // The captured project.godot is a change witness; the selected
                    // editor's effective snapshot is the policy reproduced in stage.
                }
            }
        }
    }
    if request.warnings.directory_rules.len() > 32 {
        return Err("context_limit");
    }
    for (key, level) in &request.warnings.levels {
        if key.len() > 64 || *level > 2 {
            return Err("unsupported_context");
        }
    }
    for (dir, decision) in &request.warnings.directory_rules {
        if *decision > 1
            || dir.len() > 2048
            || !dir.starts_with("res://")
            || dir.contains("..")
            || dir.contains('"')
            || dir.contains('\\')
            || dir.contains('\n')
            || dir.contains('\r')
        {
            return Err("unsupported_context");
        }
    }
    Ok(config)
}
pub(super) fn capture_closure(
    request: &WireRequest,
    result: &mut ValidationResult,
    deadline_at: Instant,
) -> Result<Closure, &'static str> {
    let root = root_path(request)?;
    let root_locator = locator(&request.root_path)?;
    if (request.purpose == Purpose::Preflight) != request.source.is_some() {
        return Err("wrong_purpose");
    }
    if request
        .source
        .as_ref()
        .is_some_and(|source| source.len() > SOURCE_LIMIT_BYTES)
    {
        return Err("source_limit");
    }
    let settings = context(request, &root)?;
    result.context_sha256 = Some(confined::hex_sha256(
        format!(
            "{}:{}:{}",
            settings.sha256,
            serde_json::to_string(&request.warnings).map_err(|_| "unsupported_context")?,
            serde_json::to_string(&request.global_classes).map_err(|_| "unsupported_context")?
        )
        .as_bytes(),
    ));
    let selected_root = capture(&root, &root_locator)?.ok_or("root_missing")?;
    let (root_source, baseline) = match &request.source {
        Some(text) => {
            let proposed = Captured {
                text: text.clone(),
                sha256: confined::hex_sha256(text.as_bytes()),
                bytes: text.len(),
                device: selected_root.device,
                inode: selected_root.inode,
                metadata: selected_root.metadata,
            };
            (proposed, Some(selected_root))
        }
        None => (selected_root, None),
    };
    let mut sources = vec![Source {
        path: root_locator,
        captured: root_source,
        proposed: request.source.is_some(),
    }];
    attribute(result, &sources[0]);
    let global_classes: BTreeSet<String> = request.global_classes.iter().cloned().collect();
    let mut known = BTreeSet::from([request.root_path.clone()]);
    let mut missing = Vec::new();
    let mut total = 0usize;
    let mut index = 0usize;
    while index < sources.len() {
        deadline(deadline_at)?;
        let paths = links(&root, &sources[index], &global_classes)?;
        for link in paths {
            let dep = locator(&link.path)?;
            if known.contains(&link.path) {
                continue;
            }
            match capture(&root, &dep)? {
                Some(captured) => {
                    if sources.iter().any(|existing| {
                        existing.captured.device == captured.device
                            && existing.captured.inode == captured.inode
                    }) {
                        return Err("source_alias");
                    }
                    if sources.len() > 32 || total + captured.bytes > 4 * 1024 * 1024 {
                        return Err("dependency_limit");
                    }
                    known.insert(link.path);
                    total += captured.bytes;
                    sources.push(Source {
                        path: dep,
                        captured,
                        proposed: false,
                    });
                    attribute(result, sources.last().expect("added"));
                }
                None => {
                    if missing.len() == 64 {
                        return Err("dependency_limit");
                    }
                    if !missing.iter().any(|m: &Missing| {
                        m.referring == sources[index].path.as_str()
                            && m.path == link.path
                            && m.line == link.line
                    }) {
                        let parents = confined::missing_parent(&root, &dep)
                            .map_err(|_| "missing_unattributed")?;
                        missing.push(Missing {
                            referring: sources[index].path.as_str().into(),
                            path: link.path,
                            literal: link.literal,
                            line: link.line,
                            parents,
                        });
                    }
                }
            }
        }
        index += 1;
    }
    Ok(Closure {
        sources,
        missing,
        settings,
        warnings: request.warnings.clone(),
        global_classes,
        baseline,
    })
}
pub(super) fn recheck(
    request: &WireRequest,
    closure: &Closure,
    deadline_at: Instant,
) -> Result<(), &'static str> {
    deadline(deadline_at)?;
    if request.warnings != closure.warnings
        || request.global_classes.len() != closure.global_classes.len()
        || !request
            .global_classes
            .iter()
            .all(|name| closure.global_classes.contains(name))
    {
        return Err("context_changed");
    }
    let root = root_path(request)?;
    let config = context(request, &root)?;
    if config != closure.settings {
        return Err("context_changed");
    }
    for source in &closure.sources {
        deadline(deadline_at)?;
        let now = capture(&root, &source.path)?.ok_or("source_changed")?;
        if source.proposed {
            // The proposed root is not yet on disk; exact original bytes and
            // metadata must still be attached at this name.
            if Some(&now) != closure.baseline.as_ref() {
                return Err("source_changed");
            }
        } else if now != source.captured {
            return Err("source_changed");
        }
    }
    for missing in &closure.missing {
        let path = locator(&missing.path)?;
        if capture(&root, &path)?.is_some()
            || confined::missing_parent(&root, &path).map_err(|_| "dependency_changed")?
                != missing.parents
        {
            return Err("dependency_changed");
        }
    }
    Ok(())
}

pub(super) fn missing_attributed(
    closure: &Closure,
    diagnostics: &[ErrorDiagnostic],
) -> Result<(), &'static str> {
    if diagnostics.len() > 64 {
        return Err("diagnostic_limit");
    }
    let mut used = [false; 64];
    for missing in &closure.missing {
        let referring = closure
            .sources
            .iter()
            .find(|source| source.path.as_str() == missing.referring)
            .ok_or("missing_unattributed")?;
        let matched = diagnostics
            .iter()
            .enumerate()
            .find(|(index, diagnostic)| {
                !used[*index]
                    && diagnostic.path == missing.referring
                    && diagnostic.source_sha256 == referring.captured.sha256
                    && diagnostic.line == missing.line
                    && ['"', '\''].iter().any(|quote| {
                        diagnostic
                            .message
                            .split(*quote)
                            .any(|part| part == missing.literal || part == missing.path)
                    })
            })
            .map(|(index, _)| index)
            .ok_or("missing_unattributed")?;
        used[matched] = true;
    }
    Ok(())
}
