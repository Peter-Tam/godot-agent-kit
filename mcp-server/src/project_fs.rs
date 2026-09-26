//! Owner-private registry and source-free project/locator validation.
use cap_std::ambient_authority;
use cap_std::fs::{Dir, MetadataExt as CapMetadataExt, PermissionsExt as CapPermissionsExt};
use std::fs::{self, DirBuilder};
use std::io::{self, Read};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use crate::observation::{
    DecimalCounter, DiagnosticCode, FileIdentity, ObservationRequest, OutcomeKind, ProjectRoot,
    ResourcePath, Stage,
};
use crate::target::RoutingFailure;

// POSIX unistd.h declares geteuid() with a uid_t return; uid_t is u32 on
// the supported Unix targets. Keep the C ABI call inside effective_uid().
extern "C" {
    fn geteuid() -> u32;
}

fn effective_uid() -> u32 {
    // SAFETY: geteuid takes no arguments and returns the process's effective Unix UID.
    unsafe { geteuid() }
}

#[cfg(target_os = "macos")]
mod acl {
    use std::ffi::c_void;
    use std::io;
    use std::os::fd::AsRawFd;
    use std::os::raw::c_int;
    use std::ptr;

    // Darwin sys/acl.h: ACL_TYPE_EXTENDED, ACL_FIRST_ENTRY, ACL_NEXT_ENTRY,
    // ACL_EXTENDED_DENY. Other tags, including inherited allows, fail closed.
    const EXTENDED: c_int = 0x100;
    const FIRST: c_int = 0;
    const NEXT: c_int = -1;
    const DENY: c_int = 2;

    // Darwin sys/acl.h declares these functions with opaque ACL/entry pointers
    // and C-int enums. ACL entry pointers borrow their parent acl_t.
    #[link(name = "System")]
    extern "C" {
        fn acl_get_fd_np(fd: c_int, acl_type: c_int) -> *mut c_void;
        fn acl_get_entry(acl: *mut c_void, index: c_int, entry: *mut *mut c_void) -> c_int;
        fn acl_get_tag_type(entry: *mut c_void, tag: *mut c_int) -> c_int;
        fn acl_free(acl: *mut c_void) -> c_int;
    }

    struct OwnedAcl(*mut c_void);
    impl Drop for OwnedAcl {
        fn drop(&mut self) {
            // SAFETY: only a nonnull allocation returned by acl_get_fd_np enters OwnedAcl;
            // no entry pointer escapes its lifetime and acl_free is called exactly once.
            unsafe { acl_free(self.0) };
        }
    }

    pub(super) fn denies_only(fd: &impl AsRawFd) -> bool {
        // SAFETY: fd is borrowed and remains open throughout ACL inspection; Darwin's
        // acl_get_fd_np returns an independently owned allocation or null with errno.
        let raw = unsafe { acl_get_fd_np(fd.as_raw_fd(), EXTENDED) };
        if raw.is_null() {
            // ENOENT means no extended ACL is installed (verified against Darwin's
            // empty-ACL behavior); all other failures mean metadata is unverifiable.
            return io::Error::last_os_error().raw_os_error() == Some(2);
        }
        let acl = OwnedAcl(raw);
        let mut entry = ptr::null_mut();
        let mut index = FIRST;
        loop {
            // SAFETY: acl remains live; entry is an out-pointer. The returned entry is
            // borrowed from acl and accessed only before OwnedAcl drops.
            match unsafe { acl_get_entry(acl.0, index, &mut entry) } {
                0 if !entry.is_null() => {
                    let mut tag = 0;
                    // SAFETY: entry was returned by acl_get_entry on the still-live ACL,
                    // and tag points to initialized writable storage for Darwin's enum.
                    if unsafe { acl_get_tag_type(entry, &mut tag) } != 0 || tag != DENY {
                        return false;
                    }
                    index = NEXT;
                }
                // Darwin returns 0 for an entry and EINVAL after the last entry,
                // unlike the POSIX draft's 1/0 iteration convention. Only NEXT
                // after a valid entry can establish exhaustion of this owned ACL.
                -1 if index == NEXT && io::Error::last_os_error().raw_os_error() == Some(22) => {
                    return true;
                }
                _ => return false,
            }
        }
    }
}

// Unimplemented ACL observability must refuse, not silently become mode-only isolation.
#[cfg(not(target_os = "macos"))]
mod acl {
    use std::os::fd::AsRawFd;
    pub(super) fn denies_only(_fd: &impl AsRawFd) -> bool {
        false
    }
}

fn failure(outcome: OutcomeKind, code: DiagnosticCode) -> RoutingFailure {
    RoutingFailure::new(outcome, code, Stage::ResolveTarget)
}
fn unsafe_registry() -> RoutingFailure {
    failure(OutcomeKind::DeniedAccess, DiagnosticCode::UnsafeRegistry)
}
fn out_of_project() -> RoutingFailure {
    failure(OutcomeKind::DeniedAccess, DiagnosticCode::OutOfProject)
}

fn check_ancestors(path: &Path) -> Result<(), RoutingFailure> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(unsafe_registry());
    }
    let uid = effective_uid();
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component);
        let meta = fs::symlink_metadata(&prefix).map_err(|_| unsafe_registry())?;
        if !meta.file_type().is_dir() || (meta.uid() != uid && meta.uid() != 0) {
            return Err(unsafe_registry());
        }
        let mode = meta.permissions().mode();
        if (mode & 0o022) != 0 && !(meta.uid() == 0 && mode & 0o1000 != 0) {
            return Err(unsafe_registry());
        }
        let opened =
            Dir::open_ambient_dir(&prefix, ambient_authority()).map_err(|_| unsafe_registry())?;
        let checked = opened.dir_metadata().map_err(|_| unsafe_registry())?;
        if checked.dev() != meta.dev()
            || checked.ino() != meta.ino()
            || checked.uid() != meta.uid()
            || checked.permissions().mode() != mode
            || !acl::denies_only(&opened)
        {
            return Err(unsafe_registry());
        }
    }
    Ok(())
}

/// Creates or verifies a dedicated owner-private registry; never repairs unsafe metadata.
/// # Errors
/// Refuses missing, non-directory, non-owner-private, ACL-granted or unverifiable
/// registry ancestry/metadata; never changes an existing object's permissions.
pub fn init_registry(path: &Path) -> Result<PathBuf, RoutingFailure> {
    let parent = path.parent().ok_or_else(unsafe_registry)?;
    check_ancestors(parent)?;
    match DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(unsafe_registry()),
    }
    open_registry(path)?;
    fs::canonicalize(path).map_err(|_| unsafe_registry())
}
pub(crate) fn open_registry(path: &Path) -> Result<Dir, RoutingFailure> {
    check_ancestors(path)?;
    let before = fs::symlink_metadata(path).map_err(|_| unsafe_registry())?;
    if !before.is_dir()
        || before.uid() != effective_uid()
        || before.permissions().mode() & 0o7777 != 0o700
    {
        return Err(unsafe_registry());
    }
    let dir = Dir::open_ambient_dir(path, ambient_authority()).map_err(|_| unsafe_registry())?;
    let opened = dir.dir_metadata().map_err(|_| unsafe_registry())?;
    if opened.dev() != before.dev()
        || opened.ino() != before.ino()
        || opened.uid() != before.uid()
        || opened.permissions().mode() & 0o7777 != 0o700
        || !acl::denies_only(&dir)
    {
        return Err(unsafe_registry());
    }
    Ok(dir)
}

pub(crate) fn descriptor_bytes(dir: &Dir, filename: &str) -> Result<Vec<u8>, RoutingFailure> {
    let before = dir
        .symlink_metadata(filename)
        .map_err(|_| unsafe_registry())?;
    if !before.is_file()
        || before.uid() != effective_uid()
        || before.permissions().mode() & 0o7777 != 0o600
        || before.len() > 4096
    {
        return Err(unsafe_registry());
    }
    let file = dir.open(filename).map_err(|_| unsafe_registry())?;
    let opened = file.metadata().map_err(|_| unsafe_registry())?;
    if !opened.is_file()
        || opened.dev() != before.dev()
        || opened.ino() != before.ino()
        || opened.uid() != before.uid()
        || opened.permissions().mode() & 0o7777 != 0o600
        || opened.len() > 4096
        || !acl::denies_only(&file)
    {
        return Err(unsafe_registry());
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| unsafe_registry())?;
    if bytes.len() > 4096 {
        return Err(unsafe_registry());
    }
    Ok(bytes)
}

pub(crate) struct ProjectIdentity {
    pub root: ProjectRoot,
    pub file_id: FileIdentity,
    pub directory: Dir,
    dev: u64,
    ino: u64,
}

fn identity(path: &Path) -> Result<(PathBuf, u64, u64), RoutingFailure> {
    let canonical = fs::canonicalize(path).map_err(|_| out_of_project())?;
    check_ancestors(&canonical).map_err(|_| out_of_project())?;
    let meta = fs::metadata(&canonical).map_err(|_| out_of_project())?;
    if !meta.is_dir() || meta.permissions().mode() & 0o022 != 0 {
        return Err(out_of_project());
    }
    Ok((canonical, meta.dev(), meta.ino()))
}

pub(crate) fn project(request: &ObservationRequest) -> Result<ProjectIdentity, RoutingFailure> {
    let (path, dev, ino) = identity(Path::new(request.project_root().as_str()))?;
    let root = ProjectRoot::new(path.to_str().ok_or_else(out_of_project)?.to_owned())
        .map_err(|_| out_of_project())?;
    let directory =
        Dir::open_ambient_dir(&path, ambient_authority()).map_err(|_| out_of_project())?;
    let opened = directory.dir_metadata().map_err(|_| out_of_project())?;
    if opened.dev() != dev || opened.ino() != ino || !acl::denies_only(&directory) {
        return Err(out_of_project());
    }
    validate_locator(&directory, request.script_path())?;
    Ok(ProjectIdentity {
        root,
        dev,
        ino,
        file_id: FileIdentity::new(
            DecimalCounter::new(dev.to_string()).map_err(|_| out_of_project())?,
            DecimalCounter::new(ino.to_string()).map_err(|_| out_of_project())?,
        ),
        directory,
    })
}

/// Validates an exact locator without opening or reading any source or container.
/// # Errors
/// Refuses a locator that escapes project confinement, traverses a symlink, or
/// encounters a cross-user-writable component. Missing source is left for the
/// later independent source/document observation.
pub fn validate_locator(directory: &Dir, locator: &ResourcePath) -> Result<(), RoutingFailure> {
    let relative = locator
        .as_str()
        .strip_prefix("res://")
        .ok_or_else(out_of_project)?
        .split("::")
        .next()
        .ok_or_else(out_of_project)?;
    let mut prefix = PathBuf::new();
    let mut components = relative.split('/').peekable();
    while let Some(component) = components.next() {
        if component.is_empty() || component == "." || component == ".." {
            return Err(out_of_project());
        }
        prefix.push(component);
        let last = components.peek().is_none();
        match directory.symlink_metadata(&prefix) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(out_of_project()),
            Ok(meta) if meta.permissions().mode() & 0o022 != 0 => return Err(out_of_project()),
            Ok(meta) if !last => {
                if !meta.is_dir() {
                    return Err(out_of_project());
                }
                // Inspect the same opened directory, not a path-only ACL that may
                // follow a replaced component; never open the source/container.
                let opened = directory.open_dir(&prefix).map_err(|_| out_of_project())?;
                let checked = opened.dir_metadata().map_err(|_| out_of_project())?;
                if checked.dev() != meta.dev()
                    || checked.ino() != meta.ino()
                    || checked.permissions().mode() != meta.permissions().mode()
                    || !acl::denies_only(&opened)
                {
                    return Err(out_of_project());
                }
            }
            Ok(meta) if !meta.is_file() => return Err(out_of_project()),
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => break,
            Err(_) => return Err(out_of_project()),
        }
    }
    Ok(())
}

pub(crate) fn matching_project(
    advertised: &str,
    project: &ProjectIdentity,
) -> Result<bool, RoutingFailure> {
    let root = ProjectRoot::new(advertised.to_owned()).map_err(|_| unsafe_registry())?;
    let (_, dev, ino) = identity(Path::new(root.as_str()))?;
    Ok(project.dev == dev && project.ino == ino)
}
