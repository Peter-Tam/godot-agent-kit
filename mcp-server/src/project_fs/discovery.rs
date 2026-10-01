//! Fresh, bounded, metadata-only inventory through the authenticated root capability.
use super::*;
use crate::target::SelectedEditor;
use cap_std::fs::Metadata;
use std::collections::BTreeMap;

const ENTRY_LIMIT: usize = 1024;
const DIRECTORY_LIMIT: usize = 1024;
const DEPTH_LIMIT: usize = 64;
const WORK_LIMIT: u32 = 16384;
const BATCH_LIMIT: usize = 64;
const GAP_LIMIT: usize = 63;
#[cfg(target_os = "macos")]
const SAFE_FLAGS: i32 = 0x4 | 0x100;
#[cfg(not(target_os = "macos"))]
const SAFE_FLAGS: i32 = 0x800 | 0x20000;
#[cfg(target_os = "macos")]
const DIRECTORY_FLAG: i32 = 0x100000;
#[cfg(not(target_os = "macos"))]
const DIRECTORY_FLAG: i32 = 0x10000;

pub(crate) struct Gap {
    pub(crate) code: &'static str,
    pub(crate) scope: String,
    pub(crate) stage: &'static str,
}
pub(crate) enum Progress {
    Entries {
        entries: Vec<ResourcePath>,
        visited_entries: u32,
        visited_directories: u32,
    },
    Invalidate {
        scope: String,
    },
}
pub(crate) struct Collection {
    pub(crate) visited_entries: u32,
    pub(crate) visited_directories: u32,
    pub(crate) gaps: Vec<Gap>,
    pub(crate) omitted_gaps: u32,
    pub(crate) exhausted: bool,
    pub(crate) recheck_completed: bool,
    pub(crate) changed: bool,
}
struct Binding<'a> {
    root: &'a ProjectRoot,
    requested: &'a ProjectRoot,
    advertised: &'a str,
    file_id: &'a FileIdentity,
    dir: &'a Dir,
}
impl Binding<'_> {
    fn verify(&self) -> Result<(), RoutingFailure> {
        let recheck_identity = |path: &str| {
            root_identity(Path::new(path)).map_err(|error| {
                if error.outcome == OutcomeKind::InvalidTarget {
                    failure(OutcomeKind::DeniedAccess, DiagnosticCode::IdentityChanged)
                } else {
                    error
                }
            })
        };
        let (canonical, dev, ino) = recheck_identity(self.root.as_str())?;
        let (requested, requested_dev, requested_ino) = recheck_identity(self.requested.as_str())?;
        let (_, advertised_dev, advertised_ino) = recheck_identity(self.advertised)?;
        let opened = self.dir.dir_metadata().map_err(|_| root_access_denied())?;
        if canonical.to_str() != Some(self.root.as_str())
            || requested != canonical
            || requested_dev != dev
            || requested_ino != ino
            || advertised_dev != dev
            || advertised_ino != ino
            || self.file_id.device().as_str().parse::<u64>() != Ok(dev)
            || self.file_id.inode().as_str().parse::<u64>() != Ok(ino)
            || opened.dev() != dev
            || opened.ino() != ino
        {
            return Err(failure(
                OutcomeKind::DeniedAccess,
                DiagnosticCode::IdentityChanged,
            ));
        }
        if !safe(&opened) || !acl::denies_only(self.dir) {
            return Err(root_access_denied());
        }
        Ok(())
    }
}
struct DirectoryWitness {
    metadata: Metadata,
    markers: Option<[Marker; 2]>,
}
#[derive(Clone)]
enum Marker {
    Absent,
    Present(Metadata),
}
struct FileWitness {
    locator: ResourcePath,
    metadata: Metadata,
}
struct CheckFailure {
    code: &'static str,
    relative: String,
}
impl CheckFailure {
    fn new(code: &'static str, relative: &str) -> Self {
        Self {
            code,
            relative: relative.to_owned(),
        }
    }
}
struct Walker<'a, F> {
    binding: Binding<'a>,
    data_name: &'a str,
    deadline: Instant,
    callback: F,
    directories: BTreeMap<String, DirectoryWitness>,
    files: Vec<FileWitness>,
    pending: Vec<usize>,
    invalidated: Vec<String>,
    result: Collection,
    stop: bool,
    #[cfg(test)]
    before_batch: Option<Box<dyn FnMut()>>,
    #[cfg(test)]
    before_recheck: Option<Box<dyn FnMut()>>,
}
fn safe(meta: &Metadata) -> bool {
    (meta.uid() == effective_uid() || meta.uid() == 0) && meta.permissions().mode() & 0o022 == 0
}
fn same(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.uid() == b.uid()
        && a.permissions().mode() == b.permissions().mode()
        && a.is_dir() == b.is_dir()
        && a.is_file() == b.is_file()
        && a.file_type().is_symlink() == b.file_type().is_symlink()
        && safe(b)
}
fn membership(a: &Metadata, b: &Metadata) -> bool {
    same(a, b)
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}
fn scope(relative: &str) -> String {
    format!("res://{relative}")
}
fn affected(relative: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || relative == prefix
        || relative
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}
fn checked_open(
    parent: &Dir,
    name: &str,
    before: &Metadata,
) -> Result<cap_std::fs::File, &'static str> {
    if !safe(before) || before.file_type().is_symlink() {
        return Err("unsafe_entry");
    }
    let mut options = CapOpenOptions::new();
    options
        .read(true)
        .custom_flags(SAFE_FLAGS | if before.is_dir() { DIRECTORY_FLAG } else { 0 });
    let file = parent
        .open_with(name, &options)
        .map_err(|_| "entry_unreadable")?;
    let opened = file.metadata().map_err(|_| "entry_unreadable")?;
    let named = parent
        .symlink_metadata(name)
        .map_err(|_| "namespace_changed")?;
    if !same(before, &opened) || !same(before, &named) {
        return Err("namespace_changed");
    }
    if !acl::denies_only(&file) {
        return Err("unsafe_entry");
    }
    Ok(file)
}
fn marker(dir: &Dir, name: &str) -> Result<Marker, &'static str> {
    let meta = match dir.symlink_metadata(name) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Marker::Absent),
        Err(_) => return Err("unsupported_marker"),
    };
    if meta.file_type().is_symlink() || !safe(&meta) {
        return Err("unsupported_marker");
    }
    // Neither directory nor special-file markers establish regular-file exclusion.
    if meta.is_file() || meta.is_dir() {
        checked_open(dir, name, &meta).map_err(|_| "unsupported_marker")?;
    }
    Ok(Marker::Present(meta))
}
fn markers(dir: &Dir) -> Result<[Marker; 2], &'static str> {
    Ok([marker(dir, ".gdignore")?, marker(dir, "project.godot")?])
}
fn markers_same(a: &[Marker; 2], b: &[Marker; 2]) -> bool {
    a.iter().zip(b).all(|(a, b)| match (a, b) {
        (Marker::Absent, Marker::Absent) => true,
        (Marker::Present(a), Marker::Present(b)) => same(a, b),
        _ => false,
    })
}
fn excluded(markers: &[Marker; 2]) -> bool {
    markers
        .iter()
        .any(|m| matches!(m,Marker::Present(meta) if meta.is_file()))
}

pub(crate) fn collect(
    selected: &SelectedEditor,
    data_directory: &str,
    deadline: Instant,
    callback: impl FnMut(Progress) -> Result<(), RoutingFailure>,
) -> Result<Collection, RoutingFailure> {
    let binding = Binding {
        root: &selected.project_root,
        requested: &selected.requested_project_root,
        advertised: &selected.advertised_project_root,
        file_id: &selected.project_file_id,
        dir: &selected.project_directory,
    };
    Walker::new(binding, data_directory, deadline, callback)?.run()
}
impl<'a, F: FnMut(Progress) -> Result<(), RoutingFailure>> Walker<'a, F> {
    fn new(
        binding: Binding<'a>,
        data_directory: &'a str,
        deadline: Instant,
        callback: F,
    ) -> Result<Self, RoutingFailure> {
        let data_name = match data_directory {
            "res://.godot" => ".godot",
            "res://godot" => "godot",
            _ => return Err(out_of_project()),
        };
        Ok(Self {
            binding,
            data_name,
            deadline,
            callback,
            directories: BTreeMap::new(),
            files: Vec::new(),
            pending: Vec::with_capacity(BATCH_LIMIT),
            invalidated: Vec::new(),
            stop: false,
            result: Collection {
                visited_entries: 0,
                visited_directories: 0,
                gaps: Vec::new(),
                omitted_gaps: 0,
                exhausted: true,
                recheck_completed: true,
                changed: false,
            },
            #[cfg(test)]
            before_batch: None,
            #[cfg(test)]
            before_recheck: None,
        })
    }
    fn time(&self) -> Result<(), RoutingFailure> {
        if Instant::now() >= self.deadline {
            return Err(failure(
                OutcomeKind::Timeout,
                DiagnosticCode::DeadlineExceeded,
            ));
        }
        Ok(())
    }
    fn gap(&mut self, code: &'static str, relative: &str, stage: &'static str) {
        if self.result.gaps.len() < GAP_LIMIT {
            self.result.gaps.push(Gap {
                code,
                scope: scope(relative),
                stage,
            });
        } else {
            self.result.omitted_gaps = self.result.omitted_gaps.saturating_add(1);
        }
    }
    fn invalidate(
        &mut self,
        relative: &str,
        code: &'static str,
        stage: &'static str,
    ) -> Result<(), RoutingFailure> {
        self.result.changed = true;
        self.gap(code, relative, stage);
        if !self.invalidated.iter().any(|p| affected(relative, p)) {
            self.invalidated.push(relative.to_owned());
            (self.callback)(Progress::Invalidate {
                scope: scope(relative),
            })?;
        }
        Ok(())
    }
    // Reopen each named component; retained witnesses never retain traversal handles.
    fn reopen(&self, relative: &str) -> Result<Dir, CheckFailure> {
        let mut dir = self
            .binding
            .dir
            .try_clone()
            .map_err(|_| CheckFailure::new("recheck_unavailable", ""))?;
        let root = self
            .directories
            .get("")
            .ok_or_else(|| CheckFailure::new("recheck_unavailable", ""))?;
        let meta = dir
            .dir_metadata()
            .map_err(|_| CheckFailure::new("recheck_unavailable", ""))?;
        if !same(&root.metadata, &meta) || !acl::denies_only(&dir) {
            return Err(CheckFailure::new("namespace_changed", ""));
        }
        let mut prefix = String::new();
        for component in relative.split('/').filter(|c| !c.is_empty()) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            let witness = self
                .directories
                .get(&prefix)
                .ok_or_else(|| CheckFailure::new("recheck_unavailable", &prefix))?;
            let meta = dir
                .symlink_metadata(component)
                .map_err(|_| CheckFailure::new("namespace_changed", &prefix))?;
            if !same(&witness.metadata, &meta) || !meta.is_dir() {
                return Err(CheckFailure::new("namespace_changed", &prefix));
            }
            dir = Dir::from_std_file(
                checked_open(&dir, component, &meta)
                    .map_err(|code| CheckFailure::new(code, &prefix))?
                    .into_std(),
            );
            if let Some(expected) = &witness.markers {
                let current = markers(&dir).map_err(|code| CheckFailure::new(code, &prefix))?;
                if !markers_same(expected, &current) {
                    return Err(CheckFailure::new("namespace_changed", &prefix));
                }
            }
        }
        Ok(dir)
    }
    fn check_file(&self, index: usize) -> Result<(), CheckFailure> {
        let file = &self.files[index];
        let relative = &file.locator.as_str()[6..];
        let (parent, name) = relative.rsplit_once('/').unwrap_or(("", relative));
        let dir = self.reopen(parent)?;
        let meta = dir
            .symlink_metadata(name)
            .map_err(|_| CheckFailure::new("namespace_changed", relative))?;
        if !meta.is_file() || !same(&file.metadata, &meta) {
            return Err(CheckFailure::new("namespace_changed", relative));
        }
        checked_open(&dir, name, &meta).map_err(|code| CheckFailure::new(code, relative))?;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), RoutingFailure> {
        if self.pending.is_empty() {
            return Ok(());
        }
        self.time()?;
        #[cfg(test)]
        if let Some(hook) = &mut self.before_batch {
            hook();
        }
        self.binding.verify()?;
        let pending = std::mem::replace(&mut self.pending, Vec::with_capacity(BATCH_LIMIT));
        let mut entries = Vec::with_capacity(pending.len());
        for index in pending {
            let relative = self.files[index].locator.as_str()[6..].to_owned();
            if self.invalidated.iter().any(|p| affected(&relative, p)) {
                continue;
            }
            match self.check_file(index) {
                Ok(()) => entries.push(self.files[index].locator.clone()),
                Err(error) => self.invalidate(&error.relative, error.code, "enumerate")?,
            }
        }
        entries.retain(|entry| {
            !self
                .invalidated
                .iter()
                .any(|prefix| affected(&entry.as_str()[6..], prefix))
        });
        self.binding.verify()?;
        if !entries.is_empty() {
            (self.callback)(Progress::Entries {
                entries,
                visited_entries: self.result.visited_entries,
                visited_directories: self.result.visited_directories,
            })?;
        }
        Ok(())
    }
    fn visit(&mut self, dir: &Dir, relative: &str, depth: usize) -> Result<(), RoutingFailure> {
        self.time()?;
        self.result.visited_directories += 1;
        let before = match dir.dir_metadata() {
            Ok(meta) => meta,
            Err(_) => {
                self.gap("directory_unreadable", relative, "enumerate");
                return Ok(());
            }
        };
        let marker_evidence = if relative.is_empty() {
            None
        } else {
            match markers(dir) {
                Ok(markers) => Some(markers),
                Err(code) => {
                    self.gap(code, relative, "enumerate");
                    return Ok(());
                }
            }
        };
        let skip = marker_evidence.as_ref().is_some_and(excluded);
        self.directories.insert(
            relative.to_owned(),
            DirectoryWitness {
                metadata: before.clone(),
                markers: marker_evidence,
            },
        );
        let effective_data = if relative.is_empty() && !self.data_name.starts_with('.') {
            match dir.symlink_metadata(self.data_name) {
                Ok(meta) => Some(meta),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(_) => {
                    self.gap("entry_unreadable", "", "enumerate");
                    None
                }
            }
        } else {
            None
        };
        if !skip {
            let iterator = match dir.entries() {
                Ok(iterator) => iterator,
                Err(_) => {
                    self.gap("directory_unreadable", relative, "enumerate");
                    return Ok(());
                }
            };
            for entry in iterator {
                self.time()?;
                // Count even dot names, unsupported names and failed entries, before file_name allocation.
                if self.result.visited_entries == WORK_LIMIT {
                    self.gap("work_limit", relative, "enumerate");
                    self.result.exhausted = false;
                    self.stop = true;
                    break;
                }
                self.result.visited_entries += 1;
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => {
                        self.gap("entry_unreadable", relative, "enumerate");
                        continue;
                    }
                };
                let native = entry.file_name();
                use std::os::unix::ffi::OsStrExt;
                if native.as_bytes().first() == Some(&b'.') {
                    continue;
                }
                let name = match native.to_str() {
                    Some(name) => name,
                    None => {
                        self.gap("unsupported_path", relative, "enumerate");
                        continue;
                    }
                };
                if relative.is_empty() && name == self.data_name {
                    continue;
                }
                if name.contains(':') {
                    self.gap("unsupported_path", relative, "enumerate");
                    continue;
                }
                // Literal lookup inherits the selected filesystem's case behavior.
                // Compare identities rather than folding arbitrary project names.
                let meta = match dir.symlink_metadata(name) {
                    Ok(meta) => meta,
                    Err(error) => {
                        self.gap(
                            if error.kind() == io::ErrorKind::NotFound {
                                "namespace_changed"
                            } else {
                                "entry_unreadable"
                            },
                            relative,
                            "enumerate",
                        );
                        continue;
                    }
                };
                if let Some(data) = &effective_data {
                    if data.dev() == meta.dev() && data.ino() == meta.ino() && data.is_dir() {
                        continue;
                    }
                }
                let gdscript = meta.is_file()
                    && name
                        .rsplit_once('.')
                        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("gd"));
                if !meta.is_dir() && !meta.file_type().is_symlink() && !gdscript {
                    continue;
                }
                // Name storage is bounded by ResourcePath's byte cap before construction.
                if 6 + relative.len() + usize::from(!relative.is_empty()) + name.len() > 2048 {
                    self.gap("unsupported_path", relative, "enumerate");
                    continue;
                }
                let child = if relative.is_empty() {
                    name.to_owned()
                } else {
                    format!("{relative}/{name}")
                };
                let locator = match ResourcePath::new(scope(&child)) {
                    Ok(locator) => locator,
                    Err(_) => {
                        self.gap("unsupported_path", relative, "enumerate");
                        continue;
                    }
                };
                if meta.file_type().is_symlink() {
                    self.gap("unsafe_entry", &child, "enumerate");
                    continue;
                }
                if meta.is_dir() {
                    if depth == DEPTH_LIMIT {
                        self.gap("depth_limit", &child, "enumerate");
                        self.result.exhausted = false;
                        continue;
                    }
                    if self.result.visited_directories as usize == DIRECTORY_LIMIT {
                        self.gap("directory_limit", &child, "enumerate");
                        self.result.exhausted = false;
                        continue;
                    }
                    match checked_open(dir, name, &meta) {
                        Ok(file) => {
                            self.visit(&Dir::from_std_file(file.into_std()), &child, depth + 1)?
                        }
                        Err(code) => self.gap(
                            if code == "entry_unreadable" {
                                "directory_unreadable"
                            } else {
                                code
                            },
                            &child,
                            "enumerate",
                        ),
                    }
                    if self.stop {
                        break;
                    }
                } else if gdscript {
                    match checked_open(dir, name, &meta) {
                        Ok(_) => {
                            if self.files.len() == ENTRY_LIMIT {
                                self.gap("entry_limit", &child, "enumerate");
                                self.result.exhausted = false;
                                self.stop = true;
                                break;
                            }
                            self.pending.push(self.files.len());
                            self.files.push(FileWitness {
                                locator,
                                metadata: meta,
                            });
                            if self.pending.len() == BATCH_LIMIT {
                                self.flush()?;
                            }
                        }
                        Err(code) => self.gap(code, &child, "enumerate"),
                    }
                }
            }
        }
        match dir.dir_metadata() {
            Ok(after) if membership(&before, &after) => {}
            Ok(_) => self.invalidate(relative, "namespace_changed", "enumerate")?,
            Err(_) => self.invalidate(relative, "recheck_unavailable", "enumerate")?,
        }
        if let Some(expected) = self
            .directories
            .get(relative)
            .and_then(|w| w.markers.clone())
        {
            match markers(dir) {
                Ok(current) if markers_same(&expected, &current) => {}
                Ok(_) => self.invalidate(relative, "namespace_changed", "enumerate")?,
                Err(_) => self.invalidate(relative, "recheck_unavailable", "enumerate")?,
            }
        }
        Ok(())
    }
    fn run(mut self) -> Result<Collection, RoutingFailure> {
        self.time()?;
        self.binding.verify()?;
        let root = self.binding.dir.try_clone().map_err(|_| out_of_project())?;
        self.visit(&root, "", 0)?;
        self.flush()?;
        #[cfg(test)]
        if let Some(hook) = &mut self.before_recheck {
            hook();
        }
        self.time()?;
        self.binding.verify()?;
        // Bounded witness names, not live FDs. One reopened chain at a time.
        let names: Vec<String> = self.directories.keys().cloned().collect();
        for relative in names {
            self.time()?;
            let result = self.reopen(&relative).and_then(|dir| {
                let current = dir
                    .dir_metadata()
                    .map_err(|_| CheckFailure::new("recheck_unavailable", &relative))?;
                let expected = &self.directories[&relative].metadata;
                if !membership(expected, &current) {
                    return Err(CheckFailure::new("namespace_changed", &relative));
                }
                Ok(())
            });
            if let Err(error) = result {
                if error.code != "namespace_changed" {
                    self.result.recheck_completed = false;
                }
                self.invalidate(&error.relative, error.code, "recheck")?;
            }
        }
        for index in 0..self.files.len() {
            self.time()?;
            if let Err(error) = self.check_file(index) {
                if error.code != "namespace_changed" {
                    self.result.recheck_completed = false;
                }
                self.invalidate(&error.relative, error.code, "recheck")?;
            }
        }
        self.binding.verify()?;
        Ok(self.result)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture {
        home: PathBuf,
        path: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let home = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "gak-discovery-fs-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            DirBuilder::new().mode(0o700).create(&home).unwrap();
            let path = home.join("project");
            DirBuilder::new().mode(0o700).create(&path).unwrap();
            Self { home, path }
        }
        fn file(&self, relative: &str) {
            let path = self.path.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        fn project(&self) -> ProjectIdentity {
            project_root(&ProjectRoot::new(self.path.to_str().unwrap()).unwrap()).unwrap()
        }
        fn binding<'a>(&'a self, project: &'a ProjectIdentity) -> Binding<'a> {
            Binding {
                root: &project.root,
                requested: &project.root,
                advertised: project.root.as_str(),
                file_id: &project.file_id,
                dir: &project.directory,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.home).unwrap();
        }
    }
    #[derive(Default)]
    struct Receipt {
        entries: BTreeSet<String>,
        batches: Vec<usize>,
        invalidations: Vec<String>,
    }
    impl Receipt {
        fn receive(&mut self, progress: Progress) -> Result<(), RoutingFailure> {
            match progress {
                Progress::Entries {
                    entries,
                    visited_entries,
                    visited_directories,
                } => {
                    assert!(entries.len() <= BATCH_LIMIT);
                    assert!(visited_entries <= WORK_LIMIT);
                    assert!(visited_directories as usize <= DIRECTORY_LIMIT);
                    self.batches.push(entries.len());
                    for entry in entries {
                        assert!(self.entries.insert(entry.as_str().to_owned()));
                    }
                }
                Progress::Invalidate { scope } => {
                    self.entries
                        .retain(|entry| !affected(&entry[6..], &scope[6..]));
                    self.invalidations.push(scope);
                }
            }
            Ok(())
        }
    }
    fn run(f: &Fixture, data: &str) -> (Collection, Receipt) {
        let project = f.project();
        let mut receipt = Receipt::default();
        let result = Walker::new(
            f.binding(&project),
            data,
            Instant::now() + Duration::from_secs(120),
            |progress| receipt.receive(progress),
        )
        .unwrap()
        .run()
        .unwrap();
        (result, receipt)
    }
    fn complete(result: &Collection) {
        assert!(result.exhausted && result.recheck_completed && !result.changed);
        assert!(result.gaps.is_empty());
        assert_eq!(result.omitted_gaps, 0);
    }
    fn gap(result: &Collection, code: &str) {
        assert!(result.gaps.iter().any(|g| g.code == code));
    }

    #[test]
    fn exact_visible_names_hardlinks_and_root_markers_are_not_source_filters() {
        let f = Fixture::new();
        for name in [
            "project.godot",
            ".gdignore",
            "first/player.gd",
            "second/player.gd",
            "a space.GD",
            "cafe\u{301}.gd",
            "addons/tool.gd",
            "vcs_ignored.gd",
            "export_excluded.gd",
            ".hidden.gd",
            ".hidden/hidden.gd",
            "ignored/.gdignore",
            "ignored/no.gd",
            "nested/project.godot",
            "nested/no.gd",
            ".godot/no.gd",
            "godot/visible.gd",
            "non_script.txt",
        ] {
            f.file(name);
        }
        fs::hard_link(f.path.join("first/player.gd"), f.path.join("hardlink.gd")).unwrap();
        fs::set_permissions(
            f.path.join("second/player.gd"),
            fs::Permissions::from_mode(0o444),
        )
        .unwrap();
        fs::create_dir(f.path.join("directory.gd")).unwrap();
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        let expected = [
            "first/player.gd",
            "second/player.gd",
            "a space.GD",
            "cafe\u{301}.gd",
            "addons/tool.gd",
            "vcs_ignored.gd",
            "export_excluded.gd",
            "hardlink.gd",
            "godot/visible.gd",
        ]
        .into_iter()
        .map(scope)
        .collect::<BTreeSet<_>>();
        assert_eq!(receipt.entries, expected);
    }
    #[test]
    fn visible_effective_data_and_literal_marker_case_follow_native_lookup() {
        let f = Fixture::new();
        f.file("godot/no.gd");
        f.file("GoDoT/also_no.gd");
        f.file("child/.GDIGNORE");
        f.file("child/a.gd");
        f.file("nested/PROJECT.GODOT");
        f.file("nested/b.gd");
        let root = f.project();
        let ignored = root.directory.symlink_metadata("child/.gdignore").is_ok();
        let nested = root
            .directory
            .symlink_metadata("nested/project.godot")
            .is_ok();
        let alias = root.directory.symlink_metadata("godot").unwrap();
        let other = root.directory.symlink_metadata("GoDoT").unwrap();
        let (result, receipt) = run(&f, "res://godot");
        complete(&result);
        let mut expected = BTreeSet::new();
        if !ignored {
            expected.insert(scope("child/a.gd"));
        }
        if !nested {
            expected.insert(scope("nested/b.gd"));
        }
        if alias.ino() != other.ino() {
            expected.insert(scope("GoDoT/also_no.gd"));
        }
        assert_eq!(receipt.entries, expected);
    }
    #[test]
    fn empty_excluded_scope_is_complete_but_symlink_and_unrepresentable_names_are_gaps() {
        let f = Fixture::new();
        f.file(".hidden/a.gd");
        f.file("ignored/.gdignore");
        f.file("ignored/a.gd");
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(receipt.entries, BTreeSet::new());
        symlink(
            f.home.join("outside-private-sentinel"),
            f.path.join("redirect"),
        )
        .unwrap();
        f.file("bad:name.gd");
        match fs::write(
            f.path
                .join(std::ffi::OsString::from_vec(b"invalid-\xff.gd".to_vec())),
            b"",
        ) {
            Ok(()) => {}
            // APFS rejects ill-formed UTF-8 names before a directory entry exists.
            Err(error) if error.raw_os_error() == Some(92) => {}
            Err(error) => panic!("non-UTF8 fixture creation failed: {error}"),
        }
        f.file("safe.gd");
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "unsafe_entry");
        gap(&result, "unsupported_path");
        assert_eq!(receipt.entries, [scope("safe.gd")].into_iter().collect());
        assert!(result
            .gaps
            .iter()
            .all(|g| !g.scope.contains("outside-private-sentinel")));
    }
    #[test]
    fn nonregular_markers_do_not_exclude_but_unsafe_markers_do_not_guess_visibility() {
        let f = Fixture::new();
        fs::create_dir_all(f.path.join("ordinary/.gdignore")).unwrap();
        f.file("ordinary/a.gd");
        f.file("unsafe/a.gd");
        symlink(f.home.join("outside"), f.path.join("unsafe/.gdignore")).unwrap();
        f.file("writable/a.gd");
        fs::set_permissions(f.path.join("writable"), fs::Permissions::from_mode(0o777)).unwrap();
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "unsupported_marker");
        gap(&result, "unsafe_entry");
        assert_eq!(
            receipt.entries,
            [scope("ordinary/a.gd")].into_iter().collect()
        );
    }
    #[test]
    fn exact_entry_and_batch_bounds_require_actual_exhaustion() {
        let f = Fixture::new();
        for n in 0..ENTRY_LIMIT {
            f.file(&format!("script{n:04}.gd"));
        }
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(receipt.entries.len(), ENTRY_LIMIT);
        assert_eq!(
            receipt.batches,
            vec![BATCH_LIMIT; ENTRY_LIMIT / BATCH_LIMIT]
        );
        f.file("one-more.gd");
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "entry_limit");
        assert!(!result.exhausted);
        assert_eq!(receipt.entries.len(), ENTRY_LIMIT);
    }
    #[test]
    fn exact_directory_and_depth_bounds_do_not_fabricate_overflow() {
        let f = Fixture::new();
        for n in 0..DIRECTORY_LIMIT - 1 {
            fs::create_dir(f.path.join(format!("d{n:04}"))).unwrap();
        }
        let (result, _) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(result.visited_directories as usize, DIRECTORY_LIMIT);
        fs::create_dir(f.path.join("one-more")).unwrap();
        let (result, _) = run(&f, "res://.godot");
        gap(&result, "directory_limit");
        assert_eq!(result.visited_directories as usize, DIRECTORY_LIMIT);
        let f = Fixture::new();
        let chain = vec!["d"; DEPTH_LIMIT].join("/");
        f.file(&format!("{chain}/last.gd"));
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(
            receipt.entries,
            [scope(&format!("{chain}/last.gd"))].into_iter().collect()
        );
        f.file(&format!("{chain}/extra/no.gd"));
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "depth_limit");
        assert!(!receipt
            .entries
            .contains(&scope(&format!("{chain}/extra/no.gd"))));
    }
    #[test]
    fn exact_work_bound_counts_dot_names_before_filtering_and_gap_details_are_bounded() {
        let f = Fixture::new();
        f.file(".seed");
        for n in 1..WORK_LIMIT {
            fs::hard_link(f.path.join(".seed"), f.path.join(format!(".n{n}"))).unwrap();
        }
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(result.visited_entries, WORK_LIMIT);
        assert!(receipt.entries.is_empty());
        f.file(".extra");
        let (result, _) = run(&f, "res://.godot");
        gap(&result, "work_limit");
        assert!(!result.exhausted);
        assert_eq!(result.visited_entries, WORK_LIMIT);
        for count in [GAP_LIMIT, GAP_LIMIT + 1] {
            let f = Fixture::new();
            for n in 0..count {
                symlink(f.home.join("absent"), f.path.join(format!("link{n}"))).unwrap();
            }
            let (result, _) = run(&f, "res://.godot");
            assert_eq!(result.gaps.len(), GAP_LIMIT);
            assert_eq!(result.omitted_gaps, (count - GAP_LIMIT) as u32);
        }
    }
    #[test]
    fn source_only_changes_before_batch_and_final_recheck_do_not_invalidate_paths() {
        for before_batch in [true, false] {
            let f = Fixture::new();
            f.file("subject.gd");
            let project = f.project();
            let mut receipt = Receipt::default();
            let mut walker = Walker::new(
                f.binding(&project),
                "res://.godot",
                Instant::now() + Duration::from_secs(30),
                |progress| receipt.receive(progress),
            )
            .unwrap();
            let path = f.path.join("subject.gd");
            let hook: Box<dyn FnMut()> = Box::new(move || {
                fs::write(&path, b"changed human source").unwrap();
            });
            if before_batch {
                walker.before_batch = Some(hook);
            } else {
                walker.before_recheck = Some(hook);
            }
            let result = walker.run().unwrap();
            complete(&result);
            assert_eq!(receipt.entries, [scope("subject.gd")].into_iter().collect());
        }
    }
    #[test]
    fn marker_file_and_named_parent_replacement_remove_only_affected_subtrees() {
        for kind in ["marker", "file", "parent", "marker_removal"] {
            let f = Fixture::new();
            f.file("safe.gd");
            f.file("branch/subject/a.gd");
            if kind == "marker_removal" {
                f.file("branch/subject/.gdignore");
            }
            let project = f.project();
            let mut receipt = Receipt::default();
            let mut walker = Walker::new(
                f.binding(&project),
                "res://.godot",
                Instant::now() + Duration::from_secs(30),
                |progress| receipt.receive(progress),
            )
            .unwrap();
            let path = f.path.join("branch");
            let outside = f.home.join("retained-original");
            walker.before_recheck = Some(Box::new(move || match kind {
                "marker" => {
                    fs::write(path.join("subject/.gdignore"), b"").unwrap();
                }
                "marker_removal" => {
                    fs::remove_file(path.join("subject/.gdignore")).unwrap();
                }
                "file" => {
                    fs::rename(path.join("subject/a.gd"), &outside).unwrap();
                    fs::write(path.join("subject/a.gd"), b"").unwrap();
                }
                "parent" => {
                    fs::rename(path.join("subject"), &outside).unwrap();
                    fs::create_dir(path.join("subject")).unwrap();
                    fs::write(path.join("subject/a.gd"), b"").unwrap();
                }
                _ => unreachable!(),
            }));
            let result = walker.run().unwrap();
            gap(&result, "namespace_changed");
            assert!(result.changed);
            assert_eq!(receipt.entries, [scope("safe.gd")].into_iter().collect());
            assert!(receipt
                .invalidations
                .iter()
                .any(|s| s == "res://branch" || s == "res://branch/subject"));
        }
    }
    #[test]
    fn before_batch_marker_or_file_replacement_is_never_reintroduced_by_later_matching_markers() {
        for kind in ["marker", "file"] {
            let f = Fixture::new();
            f.file("safe.gd");
            f.file("subject/a.gd");
            let project = f.project();
            let mut receipt = Receipt::default();
            let mut walker = Walker::new(
                f.binding(&project),
                "res://.godot",
                Instant::now() + Duration::from_secs(30),
                |progress| receipt.receive(progress),
            )
            .unwrap();
            let subject = f.path.join("subject");
            let original = f.home.join("original-file");
            let restored = subject.clone();
            walker.before_batch = Some(Box::new(move || {
                if kind == "marker" {
                    fs::write(subject.join(".gdignore"), b"").unwrap();
                } else {
                    fs::rename(subject.join("a.gd"), &original).unwrap();
                    fs::write(subject.join("a.gd"), b"").unwrap();
                }
            }));
            if kind == "marker" {
                walker.before_recheck = Some(Box::new(move || {
                    fs::remove_file(restored.join(".gdignore")).unwrap();
                }));
            }
            let result = walker.run().unwrap();
            gap(&result, "namespace_changed");
            assert!(result.changed);
            assert_eq!(receipt.entries, [scope("safe.gd")].into_iter().collect());
            assert!(!receipt.entries.contains("res://subject/a.gd"));
        }
    }
    #[test]
    fn before_batch_parent_redirect_never_emits_a_path_and_root_replacement_refuses() {
        let f = Fixture::new();
        f.file("subject/a.gd");
        let project = f.project();
        let mut receipt = Receipt::default();
        let mut walker = Walker::new(
            f.binding(&project),
            "res://.godot",
            Instant::now() + Duration::from_secs(30),
            |progress| receipt.receive(progress),
        )
        .unwrap();
        let path = f.path.clone();
        let outside = f.home.join("original");
        walker.before_batch = Some(Box::new(move || {
            fs::rename(path.join("subject"), &outside).unwrap();
            symlink(&outside, path.join("subject")).unwrap();
        }));
        let result = walker.run().unwrap();
        gap(&result, "namespace_changed");
        assert!(receipt.entries.is_empty());
        assert!(receipt.batches.is_empty());
        for before_batch in [true, false] {
            let f = Fixture::new();
            f.file("a.gd");
            let project = f.project();
            let mut receipt = Receipt::default();
            let mut walker = Walker::new(
                f.binding(&project),
                "res://.godot",
                Instant::now() + Duration::from_secs(30),
                |progress| receipt.receive(progress),
            )
            .unwrap();
            let path = f.path.clone();
            let old = f.home.join("old-root");
            let hook: Box<dyn FnMut()> = Box::new(move || {
                fs::rename(&path, &old).unwrap();
                fs::create_dir(&path).unwrap();
            });
            if before_batch {
                walker.before_batch = Some(hook);
            } else {
                walker.before_recheck = Some(hook);
            }
            let failure = walker.run().err().unwrap();
            assert_eq!(failure.outcome, OutcomeKind::DeniedAccess);
            assert_eq!(failure.diagnostic.code(), DiagnosticCode::IdentityChanged);
            if before_batch {
                assert!(receipt.entries.is_empty());
            } else {
                assert_eq!(receipt.entries, [scope("a.gd")].into_iter().collect());
            }
            // The typed request-wide failure requires supervisor suppression of earlier entries.
        }
    }
    #[test]
    fn requested_alias_replacement_and_root_access_revocation_are_request_wide() {
        let f = Fixture::new();
        f.file("a.gd");
        let alias = f.home.join("alias");
        symlink(&f.path, &alias).unwrap();
        let requested = ProjectRoot::new(alias.to_str().unwrap()).unwrap();
        let project = f.project();
        let mut receipt = Receipt::default();
        let binding = Binding {
            requested: &requested,
            ..f.binding(&project)
        };
        let mut walker = Walker::new(
            binding,
            "res://.godot",
            Instant::now() + Duration::from_secs(30),
            |progress| receipt.receive(progress),
        )
        .unwrap();
        let other = f.home.join("other");
        fs::create_dir(&other).unwrap();
        walker.before_recheck = Some(Box::new(move || {
            fs::remove_file(&alias).unwrap();
            symlink(&other, &alias).unwrap();
        }));
        assert_eq!(
            walker.run().err().unwrap().outcome,
            OutcomeKind::DeniedAccess
        );
        let mut walker = Walker::new(
            f.binding(&project),
            "res://.godot",
            Instant::now() + Duration::from_secs(30),
            |_| Ok(()),
        )
        .unwrap();
        let path = f.path.clone();
        walker.before_recheck = Some(Box::new(move || {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o777)).unwrap();
        }));
        assert_eq!(
            walker.run().err().unwrap().outcome,
            OutcomeKind::DeniedAccess
        );
    }
    #[test]
    fn acl_grants_are_not_mode_only_safe_and_os_hidden_flags_are_not_scope_filters() {
        let f = Fixture::new();
        f.file("visible.gd");
        f.file("denied/a.gd");
        let flagged = std::process::Command::new("chflags")
            .args(["hidden"])
            .arg(f.path.join("visible.gd"))
            .status()
            .unwrap();
        assert!(flagged.success());
        let grant = std::process::Command::new("chmod")
            .args(["+a", "everyone allow read,write,delete"])
            .arg(f.path.join("denied"))
            .status()
            .unwrap();
        assert!(grant.success());
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "unsafe_entry");
        assert_eq!(receipt.entries, [scope("visible.gd")].into_iter().collect());
    }
    #[test]
    fn exact_path_byte_bound_preserves_identity_and_one_over_is_not_truncated() {
        let f = Fixture::new();
        let project = f.project();
        let component = "d".repeat(240);
        let mut dir = project.directory.try_clone().unwrap();
        for _ in 0..8 {
            dir.create_dir(&component).unwrap();
            dir = dir.open_dir(&component).unwrap();
        }
        let exact = format!("{}.gd", "s".repeat(111));
        let over = format!("{}.gd", "s".repeat(112));
        dir.write(&exact, b"").unwrap();
        let relative = format!("{}/{}", [component.as_str(); 8].join("/"), exact);
        assert_eq!(scope(&relative).len(), 2048);
        let (result, receipt) = run(&f, "res://.godot");
        complete(&result);
        assert_eq!(receipt.entries, [scope(&relative)].into_iter().collect());
        dir.write(&over, b"").unwrap();
        let (result, receipt) = run(&f, "res://.godot");
        gap(&result, "unsupported_path");
        assert_eq!(receipt.entries, [scope(&relative)].into_iter().collect());
        assert!(result.gaps.iter().all(|g| g.scope.len() <= 2048));
    }
    #[test]
    fn consumer_interruption_after_checked_batch_retains_only_earlier_evidence() {
        let f = Fixture::new();
        for n in 0..BATCH_LIMIT + 1 {
            f.file(&format!("script{n:03}.gd"));
        }
        let project = f.project();
        let mut receipt = Receipt::default();
        let walker = Walker::new(
            f.binding(&project),
            "res://.godot",
            Instant::now() + Duration::from_secs(30),
            |progress| {
                receipt.receive(progress)?;
                Err(failure(
                    OutcomeKind::Timeout,
                    DiagnosticCode::DeadlineExceeded,
                ))
            },
        )
        .unwrap();
        assert_eq!(walker.run().err().unwrap().outcome, OutcomeKind::Timeout);
        assert_eq!(receipt.entries.len(), BATCH_LIMIT);
        assert_eq!(receipt.batches, vec![BATCH_LIMIT]);
        assert!(receipt.invalidations.is_empty());
    }
    #[test]
    fn project_only_admission_distinguishes_invalid_root_from_unsafe_access_without_changing_script_refusal(
    ) {
        let f = Fixture::new();
        f.file("a.gd");
        for path in [f.path.join("absent"), f.path.join("a.gd")] {
            let requested = ProjectRoot::new(path.to_str().unwrap()).unwrap();
            let error = project_root(&requested).err().unwrap();
            assert_eq!(error.outcome, OutcomeKind::InvalidTarget);
            assert_eq!(error.diagnostic.code(), DiagnosticCode::InvalidTarget);
            let request = ObservationRequest::new(
                crate::observation::RequestId::new("root-syntax").unwrap(),
                requested,
                None,
                ResourcePath::new("res://a.gd").unwrap(),
            );
            let error = super::super::project(&request).err().unwrap();
            assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
            assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
        }
        fs::set_permissions(&f.path, fs::Permissions::from_mode(0o777)).unwrap();
        let requested = ProjectRoot::new(f.path.to_str().unwrap()).unwrap();
        let error = project_root(&requested).err().unwrap();
        assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
        assert_eq!(error.diagnostic.code(), DiagnosticCode::DeniedAccess);
        let request = ObservationRequest::new(
            crate::observation::RequestId::new("root-mode").unwrap(),
            requested,
            None,
            ResourcePath::new("res://a.gd").unwrap(),
        );
        assert_eq!(
            super::super::project(&request)
                .err()
                .unwrap()
                .diagnostic
                .code(),
            DiagnosticCode::OutOfProject
        );
    }
}
