//! Private mutually authenticated version-three editor bridge and observation wire boundary.
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

use ring::{
    hmac,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::observation::{DiagnosticCode, EngineVersion, OutcomeKind, ProjectRoot, Stage};
use crate::target::RoutingFailure;
pub mod wire;

const FRAME_LIMIT: usize = 4096;
pub(crate) const GODOT_VERSION: &str = "4.7.2.stable.official.ed1daf0bf";
pub(crate) const ENGINE_HASH: &str = "ed1daf0bf001b61586d9930840f2f1394092c079";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub observe_gdscript: bool,
    pub open_enumeration: bool,
    pub buffer_attribution: bool,
    pub unsaved_paths: bool,
    pub cached_resource_lookup: bool,
    pub edit_open_gdscript: bool,
    pub open_gdscript: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Descriptor {
    v: u32,
    pub session_id: String,
    pub project_root: String,
    pub godot_version: String,
    pub engine_hash: String,
    host: String,
    port: u16,
    token: String,
}

impl Descriptor {
    pub(crate) fn parse(bytes: &[u8], filename: &str) -> Result<Self, RoutingFailure> {
        let descriptor: Self = serde_json::from_slice(bytes).map_err(|_| invalid_frame())?;
        if descriptor.v != 3
            || descriptor.session_id.len() != 32
            || !lower_hex(&descriptor.session_id, 32)
            || filename != format!("{}.json", descriptor.session_id)
            || ProjectRoot::new(descriptor.project_root.clone()).is_err()
            || descriptor.host != "127.0.0.1"
            || !(49152..=65535).contains(&descriptor.port)
            || !lower_hex(&descriptor.token, 64)
            || EngineVersion::new(&descriptor.godot_version, &descriptor.engine_hash).is_err()
        {
            return Err(invalid_frame());
        }
        Ok(descriptor)
    }
    pub(crate) fn supported(&self) -> bool {
        self.godot_version == GODOT_VERSION && self.engine_hash == ENGINE_HASH
    }
}

fn present_proof<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofMessage {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    godot_version: String,
    engine_hash: String,
    capabilities: Capabilities,
    native_api_revision: u32,
    native_build_id: String,
    client_nonce: String,
    server_nonce: String,
    #[serde(default, deserialize_with = "present_proof")]
    server_proof: Option<String>,
    #[serde(default, deserialize_with = "present_proof")]
    finish_proof: Option<String>,
}

pub(crate) struct Authenticated {
    // Kept alive until the selected session is dropped or handed to the collector.
    pub socket: TcpStream,
    pub capabilities: Capabilities,
    pub native_api_revision: u32,
    pub native_build_id: String,
}

fn error(outcome: OutcomeKind, code: DiagnosticCode) -> RoutingFailure {
    RoutingFailure::new(outcome, code, Stage::Authenticate)
}
fn invalid_frame() -> RoutingFailure {
    error(OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame)
}
fn auth_failed() -> RoutingFailure {
    error(
        OutcomeKind::DeniedAccess,
        DiagnosticCode::AuthenticationFailed,
    )
}
fn timeout() -> RoutingFailure {
    error(OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded)
}
fn unavailable() -> RoutingFailure {
    error(
        OutcomeKind::EditorUnavailable,
        DiagnosticCode::EditorUnavailable,
    )
}
fn network_error(err: std::io::Error) -> RoutingFailure {
    match err.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => timeout(),
        std::io::ErrorKind::InvalidData => invalid_frame(),
        _ => unavailable(),
    }
}
fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn decode_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if !lower_hex(value, N * 2) {
        return None;
    }
    let mut bytes = [0; N];
    for (n, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[n * 2..n * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}
fn encode_hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 15) as usize] as char);
    }
    text
}
fn field(buffer: &mut Vec<u8>, value: &[u8]) {
    buffer.extend_from_slice(&(value.len() as u32).to_be_bytes());
    buffer.extend_from_slice(value);
}
fn transcript(
    descriptor: &Descriptor,
    request: &str,
    caps: &Capabilities,
    native_api_revision: u32,
    native_build_id: &str,
    client: &[u8; 32],
    server: &[u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512);
    field(&mut bytes, b"godot-agent-kit/editor-bridge/v3");
    field(&mut bytes, request.as_bytes());
    field(
        &mut bytes,
        &decode_hex::<16>(&descriptor.session_id).expect("validated descriptor"),
    );
    field(&mut bytes, descriptor.project_root.as_bytes());
    field(&mut bytes, descriptor.godot_version.as_bytes());
    field(&mut bytes, descriptor.engine_hash.as_bytes());
    for value in [
        caps.observe_gdscript,
        caps.open_enumeration,
        caps.buffer_attribution,
        caps.unsaved_paths,
        caps.cached_resource_lookup,
        caps.edit_open_gdscript,
        caps.open_gdscript,
    ] {
        field(&mut bytes, &[u8::from(value)]);
    }
    field(&mut bytes, &native_api_revision.to_be_bytes());
    field(&mut bytes, native_build_id.as_bytes());
    field(&mut bytes, client);
    field(&mut bytes, server);
    bytes
}
fn role_transcript(role: &[u8], transcript: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(role.len() + transcript.len() + 4);
    field(&mut bytes, role);
    bytes.extend_from_slice(transcript);
    bytes
}
fn verify(key: &hmac::Key, role: &[u8], transcript: &[u8], proof: &str) -> bool {
    let Some(decoded) = decode_hex::<32>(proof) else {
        return false;
    };
    hmac::verify(key, &role_transcript(role, transcript), &decoded).is_ok()
}
fn remaining(deadline: Instant) -> Result<Duration, RoutingFailure> {
    let time = deadline.saturating_duration_since(Instant::now());
    if time.is_zero() {
        Err(timeout())
    } else {
        Ok(time)
    }
}
fn read_exact(
    stream: &mut TcpStream,
    data: &mut [u8],
    deadline: Instant,
) -> Result<(), RoutingFailure> {
    let mut offset = 0;
    while offset < data.len() {
        stream
            .set_read_timeout(Some(remaining(deadline)?))
            .map_err(network_error)?;
        match stream.read(&mut data[offset..]) {
            Ok(0) => return Err(unavailable()),
            Ok(count) => offset += count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(network_error(e)),
        }
    }
    Ok(())
}
fn write_all(stream: &mut TcpStream, data: &[u8], deadline: Instant) -> Result<(), RoutingFailure> {
    let mut offset = 0;
    while offset < data.len() {
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(network_error)?;
        match stream.write(&data[offset..]) {
            Ok(0) => return Err(unavailable()),
            Ok(count) => offset += count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(network_error(e)),
        }
    }
    Ok(())
}
fn send<T: Serialize>(
    stream: &mut TcpStream,
    frame: &T,
    deadline: Instant,
) -> Result<(), RoutingFailure> {
    let bytes = serde_json::to_vec(frame).map_err(|_| invalid_frame())?;
    if bytes.len() > FRAME_LIMIT {
        return Err(invalid_frame());
    }
    write_all(stream, &(bytes.len() as u32).to_be_bytes(), deadline)?;
    write_all(stream, &bytes, deadline)
}
fn receive(stream: &mut TcpStream, deadline: Instant) -> Result<ProofMessage, RoutingFailure> {
    let mut prefix = [0; 4];
    read_exact(stream, &mut prefix, deadline)?;
    let length = u32::from_be_bytes(prefix) as usize;
    if length == 0 || length > FRAME_LIMIT {
        return Err(invalid_frame());
    }
    let mut bytes = vec![0; length];
    read_exact(stream, &mut bytes, deadline)?;
    serde_json::from_slice(&bytes).map_err(|_| invalid_frame())
}
fn check_message(
    message: &ProofMessage,
    descriptor: &Descriptor,
    request: &str,
    nonce: &str,
    kind: &str,
) -> Result<(), RoutingFailure> {
    if message.v != 3
        || message.kind != kind
        || message.request_id != request
        || message.session_id != descriptor.session_id
        || message.project_root != descriptor.project_root
        || message.godot_version != descriptor.godot_version
        || message.engine_hash != descriptor.engine_hash
        || message.client_nonce != nonce
        || !lower_hex(&message.server_nonce, 64)
        || !matches!(message.native_api_revision, 0 | 2)
        || ((message.capabilities.edit_open_gdscript || message.capabilities.open_gdscript)
            && message.native_api_revision != 2)
        || if message.native_api_revision == 0 {
            !message.native_build_id.is_empty()
        } else {
            !lower_hex(&message.native_build_id, 64)
        }
    {
        return Err(auth_failed());
    }
    Ok(())
}

/// Authenticates a single potentially matching advertised session, without observing source.
pub(crate) fn authenticate(
    descriptor: &Descriptor,
    request: &str,
    deadline: Instant,
) -> Result<Authenticated, RoutingFailure> {
    let secret = decode_hex::<32>(&descriptor.token).ok_or_else(invalid_frame)?;
    let mut nonce = [0; 32];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| auth_failed())?;
    let nonce_hex = encode_hex(&nonce);
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, descriptor.port));
    let mut socket =
        TcpStream::connect_timeout(&address, remaining(deadline)?).map_err(network_error)?;
    socket.set_nodelay(true).map_err(network_error)?;
    send(
        &mut socket,
        &(
            3,
            "hello",
            request,
            &descriptor.session_id,
            &descriptor.project_root,
            &nonce_hex,
        ),
        deadline,
    )?;
    let challenge = receive(&mut socket, deadline)?;
    check_message(&challenge, descriptor, request, &nonce_hex, "challenge")?;
    if challenge.finish_proof.is_some() || challenge.server_proof.is_none() {
        return Err(invalid_frame());
    }
    let server_nonce = decode_hex::<32>(&challenge.server_nonce).ok_or_else(invalid_frame)?;
    if server_nonce == nonce {
        return Err(auth_failed());
    }
    let bytes = transcript(
        descriptor,
        request,
        &challenge.capabilities,
        challenge.native_api_revision,
        &challenge.native_build_id,
        &nonce,
        &server_nonce,
    );
    let key = hmac::Key::new(hmac::HMAC_SHA256, &secret);
    if !verify(
        &key,
        b"server",
        &bytes,
        challenge.server_proof.as_deref().unwrap_or(""),
    ) {
        return Err(auth_failed());
    }
    let client_proof = encode_hex(hmac::sign(&key, &role_transcript(b"client", &bytes)).as_ref());
    send(
        &mut socket,
        &(
            3,
            "authenticate",
            request,
            &descriptor.session_id,
            &descriptor.project_root,
            &nonce_hex,
            &challenge.server_nonce,
            &client_proof,
        ),
        deadline,
    )?;
    let finished = receive(&mut socket, deadline).map_err(|failure| {
        if failure.outcome == OutcomeKind::EditorUnavailable {
            auth_failed()
        } else {
            failure
        }
    })?;
    check_message(&finished, descriptor, request, &nonce_hex, "hello")?;
    if finished.capabilities != challenge.capabilities
        || finished.native_api_revision != challenge.native_api_revision
        || finished.native_build_id != challenge.native_build_id
        || finished.server_nonce != challenge.server_nonce
        || finished.server_proof.is_some()
        || finished.finish_proof.is_none()
    {
        return Err(invalid_frame());
    }
    if !verify(
        &key,
        b"finish",
        &bytes,
        finished.finish_proof.as_deref().unwrap_or(""),
    ) {
        return Err(auth_failed());
    }
    Ok(Authenticated {
        socket,
        capabilities: finished.capabilities,
        native_api_revision: finished.native_api_revision,
        native_build_id: finished.native_build_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agreed_proof_vectors() {
        let descriptor = Descriptor::parse(br#"{"v":3,"session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","godot_version":"4.7.2.stable.official.ed1daf0bf","engine_hash":"ed1daf0bf001b61586d9930840f2f1394092c079","host":"127.0.0.1","port":53124,"token":"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"}"#, "00112233445566778899aabbccddeeff.json").unwrap();
        let caps = Capabilities {
            observe_gdscript: true,
            open_enumeration: true,
            buffer_attribution: true,
            unsaved_paths: true,
            cached_resource_lookup: true,
            edit_open_gdscript: true,
            open_gdscript: false,
        };
        let client: [u8; 32] = std::array::from_fn(|i| (i + 32) as u8);
        let server: [u8; 32] = std::array::from_fn(|i| (i + 64) as u8);
        let bytes = transcript(
            &descriptor,
            "example-1",
            &caps,
            2,
            &"a".repeat(64),
            &client,
            &server,
        );
        let key = hmac::Key::new(
            hmac::HMAC_SHA256,
            &decode_hex::<32>(&descriptor.token).unwrap(),
        );
        for (role, expected) in [
            (
                b"server".as_slice(),
                "551fab39e1b856768c16a9722e38c9deb8c73a027220d1209a7c2c4653709d02",
            ),
            (
                b"client",
                "c1ee473fd4f970e1c1eb2c35d2fd204c402672c865dc8182d95b2721d7517fa1",
            ),
            (
                b"finish",
                "1abd54c16694a3158b96f59882fca5d16050f2fe54f49b0c07965de77b527053",
            ),
        ] {
            assert_eq!(
                encode_hex(hmac::sign(&key, &role_transcript(role, &bytes)).as_ref()),
                expected
            );
        }
    }

    #[test]
    fn native_metadata_cannot_advertise_an_unmatched_family() {
        let descriptor = Descriptor {
            v: 3,
            session_id: "00112233445566778899aabbccddeeff".into(),
            project_root: "/fixture/project".into(),
            godot_version: GODOT_VERSION.into(),
            engine_hash: ENGINE_HASH.into(),
            host: "127.0.0.1".into(),
            port: 53124,
            token: "00".repeat(32),
        };
        let mut message = ProofMessage {
            v: 3,
            kind: "challenge".into(),
            request_id: "metadata-check".into(),
            session_id: descriptor.session_id.clone(),
            project_root: descriptor.project_root.clone(),
            godot_version: descriptor.godot_version.clone(),
            engine_hash: descriptor.engine_hash.clone(),
            capabilities: Capabilities {
                observe_gdscript: true,
                open_enumeration: true,
                buffer_attribution: true,
                unsaved_paths: true,
                cached_resource_lookup: true,
                edit_open_gdscript: false,
                open_gdscript: false,
            },
            native_api_revision: 0,
            native_build_id: String::new(),
            client_nonce: "20".repeat(32),
            server_nonce: "40".repeat(32),
            server_proof: None,
            finish_proof: None,
        };
        let accepted = |message: &ProofMessage| {
            check_message(
                message,
                &descriptor,
                "metadata-check",
                &"20".repeat(32),
                "challenge",
            )
            .is_ok()
        };
        assert!(accepted(&message));
        for revision in [1, 3, u32::MAX] {
            message.native_api_revision = revision;
            assert!(!accepted(&message));
        }
        message.native_api_revision = 0;
        message.capabilities.edit_open_gdscript = true;
        assert!(!accepted(&message));
        message.capabilities.edit_open_gdscript = false;
        message.capabilities.open_gdscript = true;
        assert!(!accepted(&message));
        message.capabilities.open_gdscript = false;
        message.native_build_id = "a".repeat(64);
        assert!(!accepted(&message));
        message.native_api_revision = 2;
        assert!(accepted(&message));
        message.capabilities.edit_open_gdscript = true;
        assert!(accepted(&message));
        message.capabilities.open_gdscript = true;
        assert!(accepted(&message));
        for build in [String::new(), "a".repeat(63), "A".repeat(64)] {
            message.native_build_id = build;
            assert!(!accepted(&message));
        }
        message.native_build_id = "a".repeat(64);
        for version in [1, 2, 4] {
            message.v = version;
            assert!(!accepted(&message));
        }
    }

    #[test]
    fn capability_record_requires_all_seven_exact_boolean_fields() {
        let record = serde_json::json!({
            "observe_gdscript": true, "open_enumeration": true, "buffer_attribution": true,
            "unsaved_paths": true, "cached_resource_lookup": true,
            "edit_open_gdscript": true, "open_gdscript": false
        });
        for name in [
            "observe_gdscript",
            "open_enumeration",
            "buffer_attribution",
            "unsaved_paths",
            "cached_resource_lookup",
            "edit_open_gdscript",
            "open_gdscript",
        ] {
            let mut missing = record.clone();
            missing.as_object_mut().unwrap().remove(name);
            assert!(serde_json::from_value::<Capabilities>(missing).is_err());
            let mut wrong_type = record.clone();
            wrong_type[name] = serde_json::json!(0);
            assert!(serde_json::from_value::<Capabilities>(wrong_type).is_err());
        }
        let mut unknown = record;
        unknown["unrecognized_operation"] = serde_json::json!(false);
        assert!(serde_json::from_value::<Capabilities>(unknown).is_err());
    }
}
