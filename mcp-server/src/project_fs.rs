//! Owner-private registry, source-free routing, and selected-session confined D reads.
use crate::observation::{
    Authority, ClockId, CollectionStamp, DecimalCounter, DetectedChange, DiagnosticCode,
    FileIdentity, ObservationRequest, OutcomeKind, ProjectRoot, ResourcePath, ScriptKind,
    SourceObservation, SourceReason, Stage, Staleness, Surface, Validity, Witness,
    SOURCE_LIMIT_BYTES,
};
use crate::target::{RoutingFailure, SelectedSession};
use cap_std::ambient_authority;
use cap_std::fs::{
    Dir, MetadataExt as CapMetadataExt, OpenOptions as CapOpenOptions,
    OpenOptionsExt as CapOpenOptionsExt, PermissionsExt as CapPermissionsExt,
};
use std::fs::{self, DirBuilder};
use std::io::{self, Read};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

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

fn disk_denial(stage: Stage) -> RoutingFailure {
    let mut failure = out_of_project();
    failure.diagnostic =
        crate::observation::Diagnostic::new(DiagnosticCode::OutOfProject, stage, Some(Surface::D));
    failure
}

fn verify_selected_root(selected: &SelectedSession, stage: Stage) -> Result<(), RoutingFailure> {
    let (canonical, dev, ino) = identity(Path::new(selected.target().project_root().as_str()))
        .map_err(|_| disk_denial(stage))?;
    let opened = selected
        .project_directory
        .dir_metadata()
        .map_err(|_| disk_denial(stage))?;
    if canonical.to_str() != Some(selected.target().project_root().as_str())
        || selected
            .target()
            .project_file_id()
            .device()
            .as_str()
            .parse::<u64>()
            != Ok(dev)
        || selected
            .target()
            .project_file_id()
            .inode()
            .as_str()
            .parse::<u64>()
            != Ok(ino)
        || opened.dev() != dev
        || opened.ino() != ino
        || opened.permissions().mode() & 0o022 != 0
        || !acl::denies_only(&selected.project_directory)
    {
        return Err(disk_denial(stage));
    }
    Ok(())
}

// Each component is resolved from an already checked directory handle. Never open a
// symlink or pass a path containing another component to the final source open.
fn source_parent(selected: &SelectedSession, stage: Stage) -> Result<(Dir, &str), RoutingFailure> {
    verify_selected_root(selected, stage)?;
    let relative = selected
        .target()
        .script_path()
        .as_str()
        .strip_prefix("res://")
        .ok_or_else(|| disk_denial(stage))?;
    let mut components = relative.split('/').peekable();
    let mut parent = selected
        .project_directory
        .try_clone()
        .map_err(|_| disk_denial(stage))?;
    while let Some(component) = components.next() {
        if component.is_empty() || component == "." || component == ".." {
            return Err(disk_denial(stage));
        }
        if components.peek().is_none() {
            return Ok((parent, component));
        }
        let before = match parent.symlink_metadata(component) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok((parent, ""));
            }
            Err(_) => return Err(disk_denial(stage)),
        };
        if !before.is_dir() || before.permissions().mode() & 0o022 != 0 {
            return Err(disk_denial(stage));
        }
        let child = parent.open_dir(component).map_err(|_| disk_denial(stage))?;
        let after = child.dir_metadata().map_err(|_| disk_denial(stage))?;
        if before.dev() != after.dev()
            || before.ino() != after.ino()
            || before.uid() != after.uid()
            || before.permissions().mode() != after.permissions().mode()
            || !acl::denies_only(&child)
        {
            return Err(disk_denial(stage));
        }
        parent = child;
    }
    Err(disk_denial(stage))
}

fn disk_unavailable(reason: SourceReason) -> SourceObservation {
    SourceObservation::unavailable(Authority::D, reason)
        .expect("disk source reasons are valid for D")
}
pub(crate) struct DiskRead {
    pub(crate) source: SourceObservation,
    pub(crate) metadata: Option<DiskMetadata>,
}

/// Independent filesystem facts, never a source witness for unavailable text.
pub(crate) struct DiskMetadata {
    /// `None` is proven absence, not unavailable identity.
    pub(crate) file: Option<FileIdentity>,
    pub(crate) collection: CollectionStamp,
}

fn unavailable_read(reason: SourceReason) -> DiskRead {
    DiskRead {
        source: disk_unavailable(reason),
        metadata: None,
    }
}

fn caller_stamp(started: Instant, started_tick: u64) -> CollectionStamp {
    let finished_tick = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    CollectionStamp::new(
        ClockId::Caller,
        DecimalCounter::new(started_tick.to_string()).expect("caller tick is decimal"),
        DecimalCounter::new(finished_tick.to_string()).expect("caller tick is decimal"),
        finished_tick,
    )
    .expect("monotonic ticks are ordered")
}

fn missing_read(started: Instant, started_tick: u64) -> DiskRead {
    DiskRead {
        source: disk_unavailable(SourceReason::DiskMissing),
        metadata: Some(DiskMetadata {
            file: None,
            collection: caller_stamp(started, started_tick),
        }),
    }
}

// An attributable invalid/missing non-GDScript document must reach the reducer,
// but cannot acquire bytes through this GDScript-only observation operation.
pub(crate) fn non_gd_document_disk(
    selected: &SelectedSession,
    validity: Validity,
    stage: Stage,
) -> Result<DiskRead, RoutingFailure> {
    if selected.target().script_path().kind().is_some()
        || !matches!(validity, Validity::Missing | Validity::Invalid)
    {
        return Err(disk_denial(stage));
    }
    verify_selected_root(selected, stage)?;
    validate_locator(&selected.project_directory, selected.target().script_path())
        .map_err(|_| disk_denial(stage))?;
    verify_selected_root(selected, stage)?;
    Ok(unavailable_read(if validity == Validity::Missing {
        SourceReason::DiskMissing
    } else {
        SourceReason::DiskUnreadable
    }))
}

fn disk_identity(meta: &cap_std::fs::Metadata) -> FileIdentity {
    FileIdentity::new(
        DecimalCounter::new(meta.dev().to_string()).expect("device is decimal"),
        DecimalCounter::new(meta.ino().to_string()).expect("inode is decimal"),
    )
}

fn same_file_scope(a: &cap_std::fs::Metadata, b: &cap_std::fs::Metadata) -> bool {
    a.is_file()
        && b.is_file()
        && a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.uid() == b.uid()
        && a.permissions().mode() == b.permissions().mode()
        && a.permissions().mode() & 0o022 == 0
}

fn same_file_metadata(a: &cap_std::fs::Metadata, b: &cap_std::fs::Metadata) -> bool {
    same_file_scope(a, b)
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}

fn disk_source(
    selected: &SelectedSession,
    started: Instant,
    stage: Stage,
) -> Result<DiskRead, RoutingFailure> {
    match selected.target().script_path().kind() {
        Some(ScriptKind::BuiltinGdscript) => {
            // The container is only a locator. It is never a standalone D source.
            verify_selected_root(selected, stage)?;
            validate_locator(&selected.project_directory, selected.target().script_path())
                .map_err(|_| disk_denial(stage))?;
            verify_selected_root(selected, stage)?;
            return Ok(unavailable_read(SourceReason::NoStandaloneDiskSource));
        }
        Some(ScriptKind::ExternalGdscript) => {}
        None => {
            verify_selected_root(selected, stage)?;
            validate_locator(&selected.project_directory, selected.target().script_path())
                .map_err(|_| disk_denial(stage))?;
            return Err(RoutingFailure::new(
                OutcomeKind::UnsupportedObservation,
                DiagnosticCode::UnsupportedCapability,
                stage,
            ));
        }
    }
    let started_tick = Instant::now()
        .checked_duration_since(started)
        .unwrap_or_default()
        .as_micros() as u64;
    let (parent, name) = source_parent(selected, stage)?;
    if name.is_empty() {
        verify_selected_root(selected, stage)?;
        return Ok(missing_read(started, started_tick));
    }
    let before = match parent.symlink_metadata(name) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            verify_selected_root(selected, stage)?;
            return Ok(missing_read(started, started_tick));
        }
        Err(_) => {
            verify_selected_root(selected, stage)?;
            return Ok(unavailable_read(SourceReason::DiskUnreadable));
        }
    };
    if before.file_type().is_symlink() || before.permissions().mode() & 0o022 != 0 {
        return Err(disk_denial(stage));
    }
    if !before.is_file() {
        verify_selected_root(selected, stage)?;
        return Ok(unavailable_read(SourceReason::DiskUnreadable));
    }
    // O_NONBLOCK ensures a last-component FIFO substitution cannot stall open().
    // O_NOFOLLOW also closes the symlink-substitution window between lstat and open.
    #[cfg(target_os = "macos")]
    const SAFE_OPEN_FLAGS: i32 = 0x4 | 0x100;
    #[cfg(not(target_os = "macos"))]
    const SAFE_OPEN_FLAGS: i32 = 0x800 | 0x20000;
    let mut options = CapOpenOptions::new();
    options.read(true).custom_flags(SAFE_OPEN_FLAGS);
    let mut file = match parent.open_with(name, &options) {
        Ok(file) => file,
        Err(_) => {
            if parent
                .symlink_metadata(name)
                .ok()
                .as_ref()
                .is_some_and(|m| !same_file_scope(&before, m))
            {
                return Err(disk_denial(stage));
            }
            verify_selected_root(selected, stage)?;
            return Ok(unavailable_read(SourceReason::DiskUnreadable));
        }
    };
    let opened = file.metadata().map_err(|_| disk_denial(stage))?;
    if !same_file_scope(&before, &opened) || !acl::denies_only(&file) {
        return Err(disk_denial(stage));
    }
    let mut bytes = Vec::new();
    let read_result = if opened.len() > SOURCE_LIMIT_BYTES as u64 {
        Ok(0)
    } else {
        (&mut file)
            .take((SOURCE_LIMIT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
    };
    let after = file.metadata().map_err(|_| disk_denial(stage))?;
    let named_after = parent
        .symlink_metadata(name)
        .map_err(|_| disk_denial(stage))?;
    if !same_file_scope(&opened, &after)
        || !same_file_scope(&after, &named_after)
        || !acl::denies_only(&file)
    {
        return Err(disk_denial(stage));
    }
    // Revalidate all directory components, including the original project identity.
    // A safe file inode reached through a newly substituted parent is not attributable.
    let (checked_parent, checked_name) = source_parent(selected, stage)?;
    let checked = checked_parent
        .symlink_metadata(checked_name)
        .map_err(|_| disk_denial(stage))?;
    if !same_file_scope(&after, &checked) {
        return Err(disk_denial(stage));
    }
    if !same_file_metadata(&before, &opened)
        || !same_file_metadata(&opened, &after)
        || !same_file_metadata(&after, &named_after)
        || !same_file_metadata(&after, &checked)
    {
        return Ok(unavailable_read(SourceReason::SourceChanged));
    }
    let file_id = disk_identity(&after);
    let stamp = caller_stamp(started, started_tick);
    let unavailable = if read_result.is_err() {
        SourceReason::DiskUnreadable
    } else if opened.len() > SOURCE_LIMIT_BYTES as u64 || bytes.len() > SOURCE_LIMIT_BYTES {
        SourceReason::TooLarge
    } else {
        match String::from_utf8(bytes) {
            Ok(text) => {
                return Ok(DiskRead {
                    source: SourceObservation::observed(
                        Authority::D,
                        text,
                        stamp,
                        Witness::new(
                            selected.target().script_path().clone(),
                            None,
                            None,
                            None,
                            Some(file_id),
                            None,
                        ),
                        Staleness::unknown(),
                    ),
                    metadata: None,
                });
            }
            Err(_) => SourceReason::InvalidUtf8,
        }
    };
    Ok(DiskRead {
        source: disk_unavailable(unavailable),
        metadata: Some(DiskMetadata {
            file: Some(file_id),
            collection: stamp,
        }),
    })
}

/// Independently reads standalone D only after a unique authenticated selection.
/// Does not open a built-in script's scene/resource container.
/// # Errors
/// Refuses uncertain or unsafe scope; source-level limits remain D unavailability.
pub fn read_disk(
    selected: &SelectedSession,
    started: Instant,
) -> Result<SourceObservation, RoutingFailure> {
    disk_source(selected, started, Stage::ReadDisk).map(|reading| reading.source)
}

pub(crate) fn read_disk_with_metadata(
    selected: &SelectedSession,
    started: Instant,
) -> Result<DiskRead, RoutingFailure> {
    disk_source(selected, started, Stage::ReadDisk)
}

/// Freshly checks disk content, file identity and confinement against original D.
/// The returned changes invalidate original evidence; they do not replace it.
/// # Errors
/// Refuses uncertain or unsafe scope even if original D was unavailable.
pub fn recheck_disk(
    selected: &SelectedSession,
    initial: &SourceObservation,
    started: Instant,
) -> Result<Vec<DetectedChange>, RoutingFailure> {
    recheck_disk_inner(selected, initial, None, started)
}

pub(crate) fn recheck_disk_read(
    selected: &SelectedSession,
    initial: &DiskRead,
    started: Instant,
) -> Result<Vec<DetectedChange>, RoutingFailure> {
    recheck_disk_inner(
        selected,
        &initial.source,
        initial
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.file.as_ref()),
        started,
    )
}

fn recheck_disk_inner(
    selected: &SelectedSession,
    initial: &SourceObservation,
    initial_identity: Option<&FileIdentity>,
    started: Instant,
) -> Result<Vec<DetectedChange>, RoutingFailure> {
    if initial.authority() != Authority::D
        || initial.witness().is_some_and(|w| {
            w.resource_path() != selected.target().script_path() || w.disk_file_id().is_none()
        })
        || initial
            .invalidated_evidence()
            .is_some_and(|i| i.witness().resource_path() != selected.target().script_path())
    {
        return Err(RoutingFailure::new(
            OutcomeKind::ProtocolError,
            DiagnosticCode::InvalidFrame,
            Stage::Recheck,
        ));
    }
    let fresh = disk_source(selected, started, Stage::Recheck)?;
    let original_file =
        initial_identity.or_else(|| initial.witness().and_then(Witness::disk_file_id));
    let fresh_file = fresh
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.file.as_ref())
        .or_else(|| fresh.source.witness().and_then(Witness::disk_file_id));
    let mut changes = Vec::new();
    if original_file
        .zip(fresh_file)
        .is_some_and(|(before, after)| before != after)
        || (initial_identity.is_some() && fresh_file.is_none())
    {
        changes.push(DetectedChange::DiskIdentityReplaced);
    } else if initial.text() != fresh.source.text()
        || (initial.text().is_none() && initial.reason() != fresh.source.reason())
    {
        changes.push(DetectedChange::Source(Authority::D));
    }
    Ok(changes)
}
