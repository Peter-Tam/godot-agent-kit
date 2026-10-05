//! Source-free, one-generation native documentation preparation. No compiler survives readiness.
use super::*;
use cap_std::fs::{OpenOptionsExt as _, PermissionsExt as _};
use std::sync::{Arc, Mutex, Weak};

pub const INTERNAL_FLAG: &str = "--internal-validator-readiness-worker";
const STARTUP: Duration = Duration::from_secs(10);
const MAX_ARTIFACT: u64 = 8 * 1024 * 1024;
const ARTIFACT: &str = "editor_doc_cache-4.7.res";
const COMPONENTS: [&str; 4] = ["home", "Library", "Caches", "Godot"];
const CONTROL_LIMIT: usize = 16 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Descriptor {
    directory: String,
    size: u64,
    sha256: [u8; 32],
}

pub(crate) struct PreparedDocuments {
    _private: PrivateClone,
    descriptor: Descriptor,
    elapsed_us: u64,
}
impl PreparedDocuments {
    pub(crate) fn elapsed_us(&self) -> u64 {
        self.elapsed_us
    }
    pub(super) fn descriptor(&self) -> Descriptor {
        self.descriptor.clone()
    }
}

enum Registration {
    Never,
    Preparing,
    Ready(Weak<PreparedDocuments>),
    Failed,
}
static REGISTERED: Mutex<Registration> = Mutex::new(Registration::Never);
fn borrow_registered(
    registration: &Registration,
) -> Result<Option<Arc<PreparedDocuments>>, &'static str> {
    match registration {
        Registration::Never => Ok(None),
        Registration::Ready(weak) => weak.upgrade().map(Some).ok_or("prepared_documents_lost"),
        Registration::Preparing | Registration::Failed => Err("prepared_documents_unavailable"),
    }
}
pub(super) fn current() -> Result<Option<Arc<PreparedDocuments>>, &'static str> {
    let registered = REGISTERED
        .lock()
        .map_err(|_| "prepared_documents_unavailable")?;
    borrow_registered(&registered)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparationRequest {
    binary: String,
    directory: String,
    remaining_us: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum PreparationReply {
    Ready { descriptor: Descriptor },
    Unavailable,
}

// poll does not consume MCP framing bytes while detecting a pipe's closed writer.
#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}
#[cfg(target_os = "macos")]
type PollCount = u32;
#[cfg(not(target_os = "macos"))]
type PollCount = usize;
extern "C" {
    fn poll(fds: *mut PollFd, count: PollCount, timeout: i32) -> i32;
    fn recv(fd: i32, buffer: *mut u8, length: usize, flags: i32) -> isize;
}
fn startup_cancelled(shutdown: &AtomicBool) -> bool {
    if shutdown.load(Ordering::Relaxed) {
        return true;
    }
    let mut fd = PollFd {
        fd: 0,
        events: 1,
        revents: 0,
    };
    // SAFETY: one initialized repr(C) pollfd, the platform's nfds_t width,
    // and a zero timeout. No framing reader exists during preparation.
    if unsafe { poll(&mut fd, 1, 0) } <= 0 {
        return false;
    }
    if fd.revents & (8 | 16 | 32) != 0 {
        return true;
    }
    if fd.revents & 1 != 0 {
        let mut byte = 0;
        // MSG_PEEK: socket EOF is readable without POLLHUP. Pipes return ENOTSOCK.
        // SAFETY: the one-byte buffer is valid. This is the sole stdin consumer;
        // poll established readability, and MSG_PEEK never removes protocol data.
        if unsafe { recv(0, &mut byte, 1, 2) } == 0 {
            return true;
        }
    }
    false
}
fn startup_frame(
    stream: &mut UnixStream,
    at: Instant,
    shutdown: &AtomicBool,
) -> Result<Vec<u8>, &'static str> {
    fn exact(
        stream: &mut UnixStream,
        bytes: &mut [u8],
        at: Instant,
        shutdown: &AtomicBool,
    ) -> Result<(), &'static str> {
        let mut used = 0;
        while used < bytes.len() {
            deadline(at)?;
            if startup_cancelled(shutdown) {
                return Err("cancelled");
            }
            match stream.read(&mut bytes[used..]) {
                Ok(0) => return Err("worker_lost"),
                Ok(n) => used += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted
                            | io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return Err("worker_lost"),
            }
        }
        Ok(())
    }
    let mut header = [0; 4];
    exact(stream, &mut header, at, shutdown)?;
    let size = u32::from_be_bytes(header) as usize;
    if size == 0 || size > CONTROL_LIMIT {
        return Err("protocol_limit");
    }
    let mut bytes = vec![0; size];
    exact(stream, &mut bytes, at, shutdown)?;
    Ok(bytes)
}

pub(crate) fn prepare(
    binary: &Path,
    shutdown: &AtomicBool,
) -> Result<Arc<PreparedDocuments>, &'static str> {
    let started = Instant::now();
    let at = started + STARTUP;
    {
        let mut registered = REGISTERED
            .lock()
            .map_err(|_| "prepared_documents_unavailable")?;
        if !matches!(*registered, Registration::Never) {
            return Err("prepared_documents_already_configured");
        }
        *registered = Registration::Preparing;
    }
    let result = prepare_owned(binary, shutdown, started, at);
    let mut registered = REGISTERED
        .lock()
        .map_err(|_| "prepared_documents_unavailable")?;
    *registered = match &result {
        Ok(owner) => Registration::Ready(Arc::downgrade(owner)),
        Err(_) => Registration::Failed,
    };
    result
}
fn prepare_owned(
    binary: &Path,
    shutdown: &AtomicBool,
    started: Instant,
    at: Instant,
) -> Result<Arc<PreparedDocuments>, &'static str> {
    if !cfg!(target_os = "macos") {
        return Err("prepared_documents_platform");
    }
    if startup_cancelled(shutdown) {
        return Err("cancelled");
    }
    let directory = private_name()?;
    // The owned worker canonicalizes/verifies the directory. Do not perform
    // potentially blocking filesystem access on this supervisor deadline path.
    let cleanup = std::env::temp_dir().join(&directory);
    let request = PreparationRequest {
        binary: binary.to_str().ok_or("invalid_engine")?.to_owned(),
        directory: directory.clone(),
        remaining_us: at
            .saturating_duration_since(Instant::now())
            .as_micros()
            .min(10_000_000) as u64,
    };
    let body = serde_json::to_vec(&request).map_err(|_| "invalid_request")?;
    if body.len() > CONTROL_LIMIT {
        return Err("protocol_limit");
    }
    let (mut parent, child) = UnixStream::pair().map_err(|_| "worker_unavailable")?;
    parent
        .set_read_timeout(Some(Duration::from_millis(50)))
        .map_err(|_| "worker_unavailable")?;
    parent
        .set_write_timeout(Some(Duration::from_millis(50)))
        .map_err(|_| "worker_unavailable")?;
    let input: OwnedFd = child.try_clone().map_err(|_| "worker_unavailable")?.into();
    let output: OwnedFd = child.into();
    let mut command = Command::new(std::env::current_exe().map_err(|_| "worker_unavailable")?);
    command
        .arg(INTERNAL_FLAG)
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(output))
        .stderr(Stdio::null());
    // SAFETY: the post-fork closure only creates the child's own process group
    // via setpgid(0, 0); no allocation, shared locks or inherited state mutation.
    unsafe {
        command.pre_exec(|| {
            if setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        });
    }
    let mut worker = command.spawn().map_err(|_| "worker_unavailable")?;
    drop(command);
    let receipt = (|| {
        let reply_at = at - CLEANUP_RESERVE;
        send_request(&mut parent, &body, reply_at, shutdown)?;
        let bytes = startup_frame(&mut parent, reply_at, shutdown)?;
        let reply: PreparationReply = serde_json::from_slice(&bytes).map_err(|_| "worker_lost")?;
        let descriptor = match reply {
            PreparationReply::Ready { descriptor } => descriptor,
            PreparationReply::Unavailable => return Err("prepared_documents_unavailable"),
        };
        if descriptor.directory != directory
            || descriptor.size == 0
            || descriptor.size > MAX_ARTIFACT
        {
            return Err("worker_lost");
        }
        Ok(descriptor)
    })();
    drop(parent);
    kill_owned_group(worker.id());
    let mut reaped = false;
    while receipt.is_ok() && Instant::now() < at {
        if matches!(worker.try_wait(), Ok(Some(_))) {
            reaped = true;
            break;
        }
        thread::sleep(TICK);
    }
    if !reaped || receipt.is_err() {
        let cleaned = super::super::reap_detached(worker, Some(cleanup))
            .and_then(|receipt| {
                receipt
                    .recv_timeout(at.saturating_duration_since(Instant::now()))
                    .ok()
            })
            .unwrap_or(false);
        return Err(if cleaned {
            match receipt {
                Err(reason) => reason,
                Ok(_) => "cleanup_unavailable",
            }
        } else {
            "cleanup_unavailable"
        });
    }
    drop(worker); // try_wait above already reaped this process.
    let private = PrivateClone(cleanup);
    let descriptor = receipt?;
    if startup_cancelled(shutdown) {
        return Err("cancelled");
    }
    deadline(at)?;
    Ok(Arc::new(PreparedDocuments {
        _private: private,
        descriptor,
        elapsed_us: started.elapsed().as_micros().min(u64::MAX as u128) as u64,
    }))
}

pub fn worker_main() -> Option<i32> {
    let input: OwnedFd = io::stdin().as_fd().try_clone_to_owned().ok()?;
    let output: OwnedFd = io::stdout().as_fd().try_clone_to_owned().ok()?;
    let mut input = UnixStream::from(input);
    let mut output = UnixStream::from(output);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let started = Instant::now();
    input
        .set_read_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    output
        .set_write_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    let bytes = receive_frame(
        &mut input,
        CONTROL_LIMIT,
        started + STARTUP - CLEANUP_RESERVE,
        None,
    )
    .ok()?;
    let request: PreparationRequest = match serde_json::from_slice(&bytes) {
        Ok(request) => request,
        Err(_) => return Some(1),
    };
    if request.remaining_us == 0 || request.remaining_us > 10_000_000 || !cfg!(target_os = "macos")
    {
        return Some(1);
    }
    let at = started + Duration::from_micros(request.remaining_us) - CLEANUP_RESERVE;
    let result = prepare_worker(&request, at);
    match result {
        Ok((private, descriptor)) => {
            let reply = PreparationReply::Ready { descriptor };
            let sent = serde_json::to_vec(&reply)
                .ok()
                .is_some_and(|bytes| send_frame(&mut output, &bytes).is_ok());
            if sent {
                // Host already knows this exact namespace and owns failure cleanup even if delivery is lost.
                std::mem::forget(private);
                Some(0)
            } else {
                Some(1)
            }
        }
        Err(_) => {
            let reply = PreparationReply::Unavailable;
            if let Ok(bytes) = serde_json::to_vec(&reply) {
                let _ = send_frame(&mut output, &bytes);
            }
            Some(1)
        }
    }
}
fn prepare_worker(
    request: &PreparationRequest,
    at: Instant,
) -> Result<(PrivateClone, Descriptor), &'static str> {
    official(&request.binary, at)?;
    let private = private_clone_named(&request.directory)?;
    let project = stage_project(&private, None)?;
    let (child, _, port) = launch(&request.binary, &private, &project)?;
    let pid = child.id();
    let mut child = OwnedGodot(Some(child));
    let mut stream = connect_owned(pid, port, at)?;
    initialize(&mut stream, &project, at)?;
    drop(stream);
    if !child.reap(at.min(Instant::now() + Duration::from_millis(500))) {
        return Err("cleanup_unavailable");
    }
    deadline(at)?;
    // Bind the finished native bytes, then move (rather than copy) that one
    // artifact aside while discarding every engine scratch output.
    let (size, sha256) = copy_artifact(&private, &mut io::sink(), None, at)?;
    let mut artifact = private.0.clone();
    artifact.extend(COMPONENTS);
    artifact.push(ARTIFACT);
    fs::rename(artifact, private.0.join("snapshot.res"))
        .map_err(|_| "prepared_documents_unavailable")?;
    for name in ["project", "home", "config", "data", "cache", "tmp"] {
        deadline(at)?;
        fs::remove_dir_all(private.0.join(name)).map_err(|_| "prepared_documents_unavailable")?;
    }
    let mut path = private.0.clone();
    for component in COMPONENTS {
        path.push(component);
        mkdir(&path)?;
    }
    fs::rename(private.0.join("snapshot.res"), path.join(ARTIFACT))
        .map_err(|_| "prepared_documents_unavailable")?;
    fs::set_permissions(path.join(ARTIFACT), fs::Permissions::from_mode(0o600))
        .map_err(|_| "prepared_documents_unavailable")?;
    deadline(at)?;
    Ok((
        private,
        Descriptor {
            directory: request.directory.clone(),
            size,
            sha256,
        },
    ))
}

// Bound all opens to verified directory handles, rejecting symlinks, hardlinks and writable aliases.
fn artifact_parent(private: &PrivateClone, create: bool) -> Result<Dir, &'static str> {
    verify_private_dir(&private.0)?;
    let mut dir = Dir::open_ambient_dir(&private.0, ambient_authority())
        .map_err(|_| "prepared_documents_unsafe")?;
    for component in COMPONENTS {
        if create {
            match dir.create_dir(component) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err("prepared_documents_unsafe"),
            }
        }
        let before = dir
            .symlink_metadata(component)
            .map_err(|_| "prepared_documents_missing")?;
        if !before.is_dir()
            || before.permissions().mode() & 0o022 != 0
            || !project_fs::validation_owner(before.uid())
        {
            return Err("prepared_documents_unsafe");
        }
        let child = dir
            .open_dir(component)
            .map_err(|_| "prepared_documents_unsafe")?;
        let after = child
            .dir_metadata()
            .map_err(|_| "prepared_documents_unsafe")?;
        if before.dev() != after.dev()
            || before.ino() != after.ino()
            || !project_fs::validation_acl_safe(&child)
        {
            return Err("prepared_documents_unsafe");
        }
        dir = child;
    }
    Ok(dir)
}
fn file_identity(meta: &cap_std::fs::Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    (
        meta.dev(),
        meta.ino(),
        meta.len(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
    )
}
fn copy_artifact(
    private: &PrivateClone,
    output: &mut impl Write,
    expected: Option<&Descriptor>,
    at: Instant,
) -> Result<(u64, [u8; 32]), &'static str> {
    deadline(at)?;
    let parent = artifact_parent(private, false)?;
    let before = parent
        .symlink_metadata(ARTIFACT)
        .map_err(|_| "prepared_documents_missing")?;
    if !before.is_file()
        || before.nlink() != 1
        || before.permissions().mode() & 0o022 != 0
        || !project_fs::validation_owner(before.uid())
    {
        return Err("prepared_documents_unsafe");
    }
    if before.len() == 0
        || before.len() > MAX_ARTIFACT
        || expected.is_some_and(|d| d.size != before.len())
    {
        return Err("prepared_documents_size");
    }
    // O_NONBLOCK prevents a raced FIFO open from blocking; O_NOFOLLOW rejects
    // a replaced final symlink. The opened and named identities are rechecked.
    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0x4 | 0x100;
    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x800 | 0x20000;
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).custom_flags(FLAGS);
    let mut file = parent
        .open_with(ARTIFACT, &options)
        .map_err(|_| "prepared_documents_unsafe")?;
    let opened = file.metadata().map_err(|_| "prepared_documents_unsafe")?;
    if file_identity(&before) != file_identity(&opened)
        || opened.nlink() != 1
        || !project_fs::validation_acl_safe(&file)
    {
        return Err("prepared_documents_changed");
    }
    let mut hash = Context::new(&SHA256);
    let mut buffer = [0u8; 64 * 1024];
    let mut size = 0u64;
    loop {
        deadline(at)?;
        let count = file
            .read(&mut buffer)
            .map_err(|_| "prepared_documents_unreadable")?;
        if count == 0 {
            break;
        }
        size += count as u64;
        if size > before.len() || size > MAX_ARTIFACT {
            return Err("prepared_documents_size");
        }
        hash.update(&buffer[..count]);
        output
            .write_all(&buffer[..count])
            .map_err(|_| "prepared_documents_unavailable")?;
    }
    let after = file.metadata().map_err(|_| "prepared_documents_changed")?;
    let named = parent
        .symlink_metadata(ARTIFACT)
        .map_err(|_| "prepared_documents_changed")?;
    if size != before.len()
        || file_identity(&before) != file_identity(&after)
        || file_identity(&after) != file_identity(&named)
        || after.nlink() != 1
        || named.nlink() != 1
    {
        return Err("prepared_documents_changed");
    }
    let digest = hash.finish();
    let mut sha256 = [0u8; 32];
    sha256.copy_from_slice(digest.as_ref());
    if expected.is_some_and(|d| d.sha256 != sha256) {
        return Err("prepared_documents_digest");
    }
    Ok((size, sha256))
}
pub(super) fn seed(
    private: &PrivateClone,
    descriptor: &Descriptor,
    at: Instant,
) -> Result<(), &'static str> {
    deadline(at)?;
    let suffix = descriptor
        .directory
        .strip_prefix("godot-validation-")
        .ok_or("prepared_documents_unsafe")?;
    if suffix.len() != 32
        || !suffix.bytes().all(|b| b.is_ascii_hexdigit())
        || descriptor.size == 0
        || descriptor.size > MAX_ARTIFACT
    {
        return Err("prepared_documents_unsafe");
    }
    let root = fs::canonicalize(std::env::temp_dir())
        .map_err(|_| "prepared_documents_unsafe")?
        .join(&descriptor.directory);
    if root == private.0 {
        return Err("prepared_documents_unsafe");
    }
    // This is a borrowed namespace, not a new owner: never remove it on worker exit.
    let source = std::mem::ManuallyDrop::new(PrivateClone(root));
    let destination = artifact_parent(private, true)?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut output = destination
        .open_with(ARTIFACT, &options)
        .map_err(|_| "prepared_documents_unsafe")?;
    let result = copy_artifact(&source, &mut output, Some(descriptor), at);
    drop(output);
    if result.is_err() {
        let _ = destination.remove_file(ARTIFACT);
    }
    result.map(|_| ())
}

#[cfg(test)]
#[path = "native_docs_tests.rs"]
mod tests;
