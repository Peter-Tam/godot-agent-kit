//! No-follow, descriptor-bound capture for the disposable stock validator.
use cap_std::ambient_authority;
use cap_std::fs::{Dir, MetadataExt, OpenOptions, OpenOptionsExt, PermissionsExt};
use ring::digest::{digest, SHA256};
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::Path;

use crate::observation::{ProjectRoot, ResourcePath, SOURCE_LIMIT_BYTES};
use crate::project_fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureError {
    Unsafe,
    Changed,
    TooLarge,
    NonUtf8,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Captured {
    pub text: String,
    pub sha256: String,
    pub device: u64,
    pub inode: u64,
    pub bytes: usize,
    pub(crate) metadata: (u64, u64, i64, i64, i64, i64, u32),
}

fn metadata(meta: &cap_std::fs::Metadata) -> (u64, u64, i64, i64, i64, i64, u32) {
    (
        meta.dev(),
        meta.ino(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
        meta.permissions().mode(),
    )
}

pub(crate) fn hex_sha256(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for byte in digest(&SHA256, bytes).as_ref() {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

// Each component is inspected relative to a retained, verified directory. A pathname
// check without O_NOFOLLOW on the final open would permit a last-component race.
pub(crate) fn capture(
    root: &ProjectRoot,
    locator: &ResourcePath,
    limit: usize,
) -> Result<Option<Captured>, CaptureError> {
    let absolute = Path::new(root.as_str());
    if !project_fs::validation_ancestors(absolute) {
        return Err(CaptureError::Unsafe);
    }
    let canonical = fs::canonicalize(absolute).map_err(|_| CaptureError::Unsafe)?;
    if canonical != absolute {
        return Err(CaptureError::Unsafe);
    }
    let root_meta = fs::symlink_metadata(absolute).map_err(|_| CaptureError::Unsafe)?;
    if !root_meta.is_dir() || root_meta.permissions().mode() & 0o022 != 0 {
        return Err(CaptureError::Unsafe);
    }
    let root_dir =
        Dir::open_ambient_dir(absolute, ambient_authority()).map_err(|_| CaptureError::Unsafe)?;
    let opened_root = root_dir.dir_metadata().map_err(|_| CaptureError::Unsafe)?;
    if opened_root.dev() != root_meta.dev()
        || opened_root.ino() != root_meta.ino()
        || !project_fs::validation_acl_safe(&root_dir)
    {
        return Err(CaptureError::Unsafe);
    }
    let relative = locator
        .as_str()
        .strip_prefix("res://")
        .ok_or(CaptureError::Unsafe)?;
    if relative.contains("::") {
        return Err(CaptureError::Unsafe);
    }
    let components: Vec<_> = relative.split('/').collect();
    if components.is_empty()
        || components
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err(CaptureError::Unsafe);
    }
    let mut parent = root_dir.try_clone().map_err(|_| CaptureError::Unsafe)?;
    let mut parents = Vec::with_capacity(components.len());
    for part in &components[..components.len() - 1] {
        let before = match parent.symlink_metadata(part) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(CaptureError::Unsafe),
        };
        if !before.is_dir() || before.permissions().mode() & 0o022 != 0 {
            return Err(CaptureError::Unsafe);
        }
        let child = parent.open_dir(part).map_err(|_| CaptureError::Unsafe)?;
        let after = child.dir_metadata().map_err(|_| CaptureError::Unsafe)?;
        if metadata(&before) != metadata(&after) || !project_fs::validation_acl_safe(&child) {
            return Err(CaptureError::Changed);
        }
        parents.push((parent, (*part).to_owned(), metadata(&before)));
        parent = child;
    }
    let name = components[components.len() - 1];
    let before = match parent.symlink_metadata(name) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(CaptureError::Unsafe),
    };
    if !before.is_file() || before.permissions().mode() & 0o022 != 0 {
        return Err(CaptureError::Unsafe);
    }
    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0x4 | 0x100; // O_NONBLOCK | O_NOFOLLOW
    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x800 | 0x20000;
    let mut options = OpenOptions::new();
    options.read(true).custom_flags(FLAGS);
    let mut file = parent
        .open_with(name, &options)
        .map_err(|_| CaptureError::Unsafe)?;
    let opened = file.metadata().map_err(|_| CaptureError::Unsafe)?;
    if metadata(&before) != metadata(&opened) || !project_fs::validation_acl_safe(&file) {
        return Err(CaptureError::Changed);
    }
    if opened.len() > limit as u64 {
        return Err(CaptureError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    (&mut file)
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| CaptureError::Unreadable)?;
    let after = file.metadata().map_err(|_| CaptureError::Changed)?;
    let named = parent
        .symlink_metadata(name)
        .map_err(|_| CaptureError::Changed)?;
    if metadata(&before) != metadata(&after)
        || metadata(&after) != metadata(&named)
        || !project_fs::validation_acl_safe(&file)
    {
        return Err(CaptureError::Changed);
    }
    for (parent, name, before) in &parents {
        let current = parent
            .symlink_metadata(name)
            .map_err(|_| CaptureError::Changed)?;
        if metadata(&current) != *before {
            return Err(CaptureError::Changed);
        }
    }
    let current_root = fs::symlink_metadata(absolute).map_err(|_| CaptureError::Changed)?;
    if current_root.dev() != root_meta.dev() || current_root.ino() != root_meta.ino() {
        return Err(CaptureError::Changed);
    }
    if bytes.len() > limit {
        return Err(CaptureError::TooLarge);
    }
    let sha256 = hex_sha256(&bytes);
    let size = bytes.len();
    let text = String::from_utf8(bytes).map_err(|_| CaptureError::NonUtf8)?;
    Ok(Some(Captured {
        text,
        sha256,
        device: after.dev(),
        inode: after.ino(),
        bytes: size,
        metadata: metadata(&after),
    }))
}

pub(crate) fn source(
    root: &ProjectRoot,
    locator: &ResourcePath,
) -> Result<Option<Captured>, CaptureError> {
    capture(root, locator, SOURCE_LIMIT_BYTES)
}

/// A missing literal is attributable only while its entire existing parent
/// namespace remains attached. A missing intermediate directory is ambiguous
/// for the small source-only profile and is refused rather than guessed.
pub(crate) fn missing_parent(
    root: &ProjectRoot,
    locator: &ResourcePath,
) -> Result<Vec<(u64, u64, u32)>, CaptureError> {
    let absolute = Path::new(root.as_str());
    if !project_fs::validation_ancestors(absolute)
        || fs::canonicalize(absolute).ok().as_deref() != Some(absolute)
    {
        return Err(CaptureError::Unsafe);
    }
    let mut dir =
        Dir::open_ambient_dir(absolute, ambient_authority()).map_err(|_| CaptureError::Unsafe)?;
    let root_meta = dir.dir_metadata().map_err(|_| CaptureError::Unsafe)?;
    if !project_fs::validation_acl_safe(&dir) {
        return Err(CaptureError::Unsafe);
    }
    let mut identities = vec![(
        root_meta.dev(),
        root_meta.ino(),
        root_meta.permissions().mode(),
    )];
    let mut parents = Vec::new();
    let relative = locator
        .as_str()
        .strip_prefix("res://")
        .ok_or(CaptureError::Unsafe)?;
    let parts: Vec<_> = relative.split('/').collect();
    for part in &parts[..parts.len().saturating_sub(1)] {
        let before = dir
            .symlink_metadata(part)
            .map_err(|_| CaptureError::Unsafe)?;
        if !before.is_dir() || before.permissions().mode() & 0o022 != 0 {
            return Err(CaptureError::Unsafe);
        }
        let child = dir.open_dir(part).map_err(|_| CaptureError::Unsafe)?;
        let after = child.dir_metadata().map_err(|_| CaptureError::Unsafe)?;
        if metadata(&before) != metadata(&after) || !project_fs::validation_acl_safe(&child) {
            return Err(CaptureError::Changed);
        }
        identities.push((after.dev(), after.ino(), after.permissions().mode()));
        parents.push((
            dir.try_clone().map_err(|_| CaptureError::Changed)?,
            (*part).to_owned(),
            metadata(&before),
        ));
        dir = child;
    }
    match dir.symlink_metadata(parts.last().ok_or(CaptureError::Unsafe)?) {
        Ok(_) => return Err(CaptureError::Changed),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Err(CaptureError::Unsafe),
    }
    let current_root = fs::symlink_metadata(absolute).map_err(|_| CaptureError::Changed)?;
    if current_root.dev() != root_meta.dev() || current_root.ino() != root_meta.ino() {
        return Err(CaptureError::Changed);
    }
    for (parent, name, before) in &parents {
        let current = parent
            .symlink_metadata(name)
            .map_err(|_| CaptureError::Changed)?;
        if metadata(&current) != *before {
            return Err(CaptureError::Changed);
        }
    }
    Ok(identities)
}
