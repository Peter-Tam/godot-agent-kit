use super::*;

pub(super) struct PrivateClone(pub(super) PathBuf);
impl Drop for PrivateClone {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub(super) fn private_name() -> Result<String, &'static str> {
    let mut random = [0u8; 16];
    SystemRandom::new()
        .fill(&mut random)
        .map_err(|_| "host_unavailable")?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut name = String::from("godot-validation-");
    for byte in random {
        name.push(HEX[(byte >> 4) as usize] as char);
        name.push(HEX[(byte & 15) as usize] as char);
    }
    Ok(name)
}
#[cfg(test)]
pub(super) fn private_clone() -> Result<PrivateClone, &'static str> {
    private_clone_named(&private_name()?)
}
pub(super) fn private_clone_named(name: &str) -> Result<PrivateClone, &'static str> {
    let suffix = name
        .strip_prefix("godot-validation-")
        .ok_or("host_unavailable")?;
    if suffix.len() != 32 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("host_unavailable");
    }
    let parent = fs::canonicalize(std::env::temp_dir()).map_err(|_| "host_unavailable")?;
    if !project_fs::validation_ancestors(&parent) {
        return Err("host_unavailable");
    }
    let path = parent.join(name);
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder.create(&path).map_err(|_| "host_unavailable")?;
    let private = PrivateClone(path);
    verify_private_dir(&private.0)?;
    Ok(private)
}
fn verify_private_dir(path: &Path) -> Result<(), &'static str> {
    let parent = fs::canonicalize(std::env::temp_dir()).map_err(|_| "host_unavailable")?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("host_unavailable")?;
    let suffix = name
        .strip_prefix("godot-validation-")
        .ok_or("host_unavailable")?;
    if path.parent() != Some(parent.as_path())
        || suffix.len() != 32
        || !suffix.bytes().all(|b| b.is_ascii_hexdigit())
        || fs::canonicalize(path).ok().as_deref() != Some(path)
        || !project_fs::validation_ancestors(path)
    {
        return Err("host_unavailable");
    }
    let before = fs::symlink_metadata(path).map_err(|_| "host_unavailable")?;
    if !before.is_dir()
        || before.permissions().mode() & 0o777 != 0o700
        || !project_fs::validation_owner(before.uid())
    {
        return Err("host_unavailable");
    }
    let dir = Dir::open_ambient_dir(path, ambient_authority()).map_err(|_| "host_unavailable")?;
    let after = dir.dir_metadata().map_err(|_| "host_unavailable")?;
    if after.dev() != before.dev()
        || after.ino() != before.ino()
        || !project_fs::validation_acl_safe(&dir)
    {
        return Err("host_unavailable");
    }
    Ok(())
}
pub(super) fn mkdir(path: &Path) -> Result<(), &'static str> {
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).map_err(|_| "host_unavailable")
}
pub(super) fn stage(
    closure: &Closure,
    clone: &PrivateClone,
    warnings: &WarningSettings,
) -> Result<PathBuf, &'static str> {
    let project = clone.0.join("project");
    mkdir(&project)?;
    for name in ["home", "config", "data", "cache", "tmp"] {
        mkdir(&clone.0.join(name))?;
    }
    // No inherited feature list, selected addons, UID remaps, extensions or old .godot.
    let mut text = String::from("config_version=5\n\n[application]\nconfig/name=\"Private GDScript Validation\"\n\n[debug]\n");
    text.push_str("gdscript/warnings/enable=");
    text.push_str(if warnings.enable { "true\n" } else { "false\n" });
    use std::fmt::Write as _;
    for (key, value) in &warnings.levels {
        writeln!(text, "gdscript/warnings/{key}={value}").expect("String accepts formatting");
    }
    if !warnings.directory_rules.is_empty() {
        text.push_str("gdscript/warnings/directory_rules={");
        for (index, (dir, decision)) in warnings.directory_rules.iter().enumerate() {
            if index != 0 {
                text.push_str(", ");
            }
            write!(text, "\"{dir}\": {decision}").expect("String accepts formatting");
        }
        text.push_str("}\n");
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(project.join("project.godot"))
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|_| "host_unavailable")?;
    for source in &closure.sources {
        let relative = source
            .path
            .as_str()
            .strip_prefix("res://")
            .ok_or("unsafe_path")?;
        let destination = project.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|_| "host_unavailable")?;
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|_| "source_alias")?
            .write_all(source.captured.text.as_bytes())
            .map_err(|_| "host_unavailable")?;
    }
    fs::canonicalize(project).map_err(|_| "host_unavailable")
}
pub(super) fn official(binary: &str, deadline_at: Instant) -> Result<(), &'static str> {
    deadline(deadline_at)?;
    let path = Path::new(binary);
    if !path.is_absolute() || fs::canonicalize(path).ok().as_deref() != Some(path) {
        return Err("wrong_binary");
    }
    let before = fs::symlink_metadata(path).map_err(|_| "wrong_binary")?;
    if !before.is_file() || before.permissions().mode() & 0o022 != 0 {
        return Err("wrong_binary");
    }
    let mut file = File::open(path).map_err(|_| "wrong_binary")?;
    let opened = file.metadata().map_err(|_| "wrong_binary")?;
    if opened.dev() != before.dev() || opened.ino() != before.ino() {
        return Err("wrong_binary");
    }
    let mut digest = Context::new(&SHA256);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        deadline(deadline_at)?;
        let len = file.read(&mut buffer).map_err(|_| "wrong_binary")?;
        if len == 0 {
            break;
        }
        digest.update(&buffer[..len]);
    }
    let after = file.metadata().map_err(|_| "wrong_binary")?;
    if after.dev() != before.dev()
        || after.ino() != before.ino()
        || after.len() != before.len()
        || after.mtime() != before.mtime()
        || after.mtime_nsec() != before.mtime_nsec()
        || fs::symlink_metadata(path)
            .map_err(|_| "wrong_binary")?
            .ino()
            != before.ino()
    {
        return Err("wrong_binary");
    }
    let fingerprint = digest.finish();
    let hex = OFFICIAL_SHA256.as_bytes();
    const DIGIT: &[u8; 16] = b"0123456789abcdef";
    if !fingerprint
        .as_ref()
        .iter()
        .enumerate()
        .all(|(index, byte)| {
            hex[2 * index] == DIGIT[(byte >> 4) as usize]
                && hex[2 * index + 1] == DIGIT[(byte & 15) as usize]
        })
    {
        return Err("wrong_binary");
    }
    Ok(())
}

pub(super) struct OwnedGodot(pub(super) Option<Child>);
impl OwnedGodot {
    pub(super) fn reap(&mut self, deadline_at: Instant) -> bool {
        let Some(mut child) = self.0.take() else {
            return true;
        };
        if matches!(child.try_wait(), Ok(Some(_))) {
            return true;
        }
        // Graceful termination is bounded; no JSON-RPC shutdown (stock returns -32601).
        #[cfg(unix)]
        unsafe {
            kill(child.id() as i32, 15);
        }
        while Instant::now() < deadline_at {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return true;
            }
            thread::sleep(TICK);
        }
        let _ = child.kill();
        for _ in 0..30 {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return true;
            }
            thread::sleep(TICK);
        }
        self.0 = Some(child);
        false
    }
}
impl Drop for OwnedGodot {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
#[cfg(unix)]
extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    pub(super) fn setpgid(pid: i32, pgid: i32) -> i32;
}

pub(super) fn kill_owned_group(pid: u32) {
    // The worker is our process-group leader. Its exit does not imply that its
    // editor descendants have exited.
    unsafe {
        kill(-(pid as i32), 9);
    }
}

// lsof reports the PID owning this exact listening socket, not a connected
// peer's PID. A second trusted client is not an ownership failure.
fn listener_owned(pid: u32, port: u16, deadline_at: Instant) -> Result<bool, &'static str> {
    #[cfg(target_os = "macos")]
    let lsof = "/usr/sbin/lsof";
    #[cfg(not(target_os = "macos"))]
    let lsof = "/usr/bin/lsof";
    let pid_string = pid.to_string();
    let mut probe = Command::new(lsof)
        .args([
            "-nP",
            "-a",
            "-p",
            &pid_string,
            &format!("-iTCP@127.0.0.1:{port}"),
            "-sTCP:LISTEN",
            "-Fp",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "endpoint_unverified")?;
    let checked = (|| -> Result<bool, &'static str> {
        while Instant::now() < deadline_at {
            if let Some(status) = probe.try_wait().map_err(|_| "endpoint_unverified")? {
                if !status.success() {
                    return Ok(false);
                }
                let mut data = Vec::new();
                probe
                    .stdout
                    .take()
                    .ok_or("endpoint_unverified")?
                    .take(1024)
                    .read_to_end(&mut data)
                    .map_err(|_| "endpoint_unverified")?;
                let text = String::from_utf8(data).map_err(|_| "endpoint_unverified")?;
                return Ok(text
                    .lines()
                    .any(|line| line.strip_prefix('p') == Some(pid_string.as_str())));
            }
            thread::sleep(TICK);
        }
        Err("deadline")
    })();
    if probe.try_wait().ok().flatten().is_none() {
        let _ = probe.kill();
    }
    let _ = probe.wait();
    checked
}
fn ownership_diagnostic(pid: u32, label: &str, start: Instant) {
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true)
        .open("/tmp/gak-validation-diagnostic.log")
    {
        let text = format!("owner {pid} {label} {}\n", start.elapsed().as_micros());
        let _ = file.write_all(text.as_bytes());
    }
}

pub(super) fn connect_owned(
    pid: u32,
    port: u16,
    deadline_at: Instant,
) -> Result<TcpStream, &'static str> {
    let diagnostic_start = Instant::now();
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    loop {
        deadline(deadline_at)?;
        if let Ok(connection) = TcpStream::connect_timeout(&addr.into(), Duration::from_millis(50))
        {
            ownership_diagnostic(pid, "tcp_connected", diagnostic_start);
            // Check the owner after connection succeeds: a listener can start
            // between an earlier lsof snapshot and connect. No source is sent
            // until this connected endpoint is attributed to our child.
            if !listener_owned(pid, port, deadline_at)? {
                return Err("wrong_owner");
            }
            ownership_diagnostic(pid, "owner_verified", diagnostic_start);
            connection
                .set_read_timeout(Some(Duration::from_millis(50)))
                .map_err(|_| "protocol_loss")?;
            connection
                .set_write_timeout(Some(Duration::from_millis(50)))
                .map_err(|_| "protocol_loss")?;
            return Ok(connection);
        }
        thread::sleep(Duration::from_millis(25));
    }
}
pub(super) fn inspect_clone(
    private: &PrivateClone,
    closure: &Closure,
) -> Result<(bool, bool), &'static str> {
    let expected: BTreeSet<_> = closure
        .sources
        .iter()
        .map(|source| {
            PathBuf::from("project").join(
                source
                    .path
                    .as_str()
                    .strip_prefix("res://")
                    .expect("admitted"),
            )
        })
        .chain([PathBuf::from("project/project.godot")])
        .collect();
    let mut pending = vec![PathBuf::new()];
    let mut count = 0usize;
    let (mut extra, mut logs) = (false, false);
    while let Some(relative) = pending.pop() {
        if relative.components().count() > 16 {
            return Err("privacy_unverified");
        }
        let entries = fs::read_dir(private.0.join(&relative)).map_err(|_| "privacy_unverified")?;
        for entry in entries {
            count += 1;
            if count > 2048 {
                return Err("privacy_unverified");
            }
            let entry = entry.map_err(|_| "privacy_unverified")?;
            let next = relative.join(entry.file_name());
            let metadata =
                fs::symlink_metadata(private.0.join(&next)).map_err(|_| "privacy_unverified")?;
            if next
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("log"))
            {
                logs = true;
            }
            let project_file = next.starts_with("project") && next != Path::new("project");
            let generated_uid = next
                .to_str()
                .and_then(|name| name.strip_suffix(".uid"))
                .is_some_and(|name| name.ends_with(".gd") && expected.contains(Path::new(name)));
            let known = expected.contains(&next)
                || generated_uid
                || (metadata.is_dir() && expected.iter().any(|item| item.starts_with(&next)))
                || next.starts_with("project/.godot");
            if project_file && !known {
                extra = true;
            }
            if next.starts_with("project/.godot") && next.extension().is_some_and(|e| e == "txt") {
                extra = true;
            }
            if metadata.file_type().is_symlink() {
                extra = true;
                continue;
            }
            if metadata.is_dir() {
                pending.push(next);
            }
        }
    }
    Ok((extra, logs))
}
