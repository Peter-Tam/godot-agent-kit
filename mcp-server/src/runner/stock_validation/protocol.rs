use super::*;

pub(super) fn uri(path: &Path) -> Result<String, &'static str> {
    let path = path.to_str().ok_or("unsafe_path")?;
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(path.len() + 7);
    result.push_str("file://");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            result.push(byte as char);
        } else {
            result.push('%');
            result.push(HEX[(byte >> 4) as usize] as char);
            result.push(HEX[(byte & 15) as usize] as char);
        }
    }
    Ok(result)
}
pub(super) fn send_lsp(stream: &mut TcpStream, value: &Value) -> Result<(), &'static str> {
    let bytes = serde_json::to_vec(value).map_err(|_| "protocol_loss")?;
    if bytes.len() > CONTROL_MAX {
        return Err("protocol_limit");
    }
    write!(stream, "Content-Length: {}\r\n\r\n", bytes.len()).map_err(|_| "protocol_loss")?;
    stream.write_all(&bytes).map_err(|_| "protocol_loss")
}
fn read_byte(stream: &mut TcpStream, deadline_at: Instant) -> Result<u8, &'static str> {
    loop {
        deadline(deadline_at)?;
        let mut byte = [0u8];
        match stream.read(&mut byte) {
            Ok(1) => return Ok(byte[0]),
            Ok(0) => return Err("protocol_loss"),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return Err("protocol_loss"),
            _ => unreachable!(),
        }
    }
}
pub(super) fn read_lsp(
    stream: &mut TcpStream,
    deadline_at: Instant,
) -> Result<Value, &'static str> {
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= 1024 {
            return Err("protocol_limit");
        }
        header.push(read_byte(stream, deadline_at)?);
    }
    let text = std::str::from_utf8(&header).map_err(|_| "protocol_loss")?;
    let mut length = None;
    for row in text[..text.len() - 4].split("\r\n") {
        let value = row
            .strip_prefix("Content-Length: ")
            .ok_or("protocol_loss")?;
        if length.is_some() || value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err("protocol_loss");
        }
        length = Some(value.parse::<usize>().map_err(|_| "protocol_limit")?);
    }
    let length = length.ok_or("protocol_loss")?;
    if length == 0 || length > LSP_MAX {
        return Err("protocol_limit");
    }
    let mut body = vec![0u8; length];
    let mut used = 0;
    while used < length {
        deadline(deadline_at)?;
        match stream.read(&mut body[used..]) {
            Ok(0) => return Err("protocol_loss"),
            Ok(n) => used += n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return Err("protocol_loss"),
        }
    }
    let value: Value = serde_json::from_slice(&body).map_err(|_| "protocol_loss")?;
    if value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") || !value.is_object() {
        return Err("protocol_loss");
    }
    Ok(value)
}
pub(super) fn response(
    stream: &mut TcpStream,
    id: u64,
    deadline_at: Instant,
) -> Result<Value, &'static str> {
    loop {
        let message = read_lsp(stream, deadline_at)?;
        if message.get("id").is_some() {
            if message.get("id").and_then(Value::as_u64) != Some(id)
                || message.get("error").is_some()
            {
                return Err("protocol_loss");
            }
            return message.get("result").cloned().ok_or("protocol_loss");
        }
        // Before didOpen no notification is a source/version fence.
    }
}
pub(super) fn source_fence(
    stream: &mut TcpStream,
    source: &Source,
    index: usize,
    project: &Path,
    result: &mut ValidationResult,
    deadline_at: Instant,
) -> Result<(), &'static str> {
    let relative = source
        .path
        .as_str()
        .strip_prefix("res://")
        .ok_or("unsafe_path")?;
    let source_uri = uri(&project.join(relative))?;
    send_lsp(
        stream,
        &json!({"jsonrpc":"2.0","method":"textDocument/didOpen",
        "params":{"textDocument":{"uri":source_uri,"languageId":"gdscript","version":1,
            "text":source.captured.text}}}),
    )?;
    let id = index as u64 + 2;
    send_lsp(
        stream,
        &json!({"jsonrpc":"2.0","id":id,"method":"textDocument/documentSymbol",
        "params":{"textDocument":{"uri":source_uri}}}),
    )?;
    let mut diagnostics = false;
    let mut symbols = false;
    while !diagnostics || !symbols {
        let message = read_lsp(stream, deadline_at)?;
        if let Some(reply_id) = message.get("id") {
            if reply_id.as_u64() != Some(id)
                || symbols
                || !diagnostics
                || message.get("error").is_some()
            {
                return Err("protocol_loss");
            }
            let array = message
                .get("result")
                .and_then(Value::as_array)
                .ok_or("incomplete_symbols")?;
            let filename = relative.rsplit('/').next().ok_or("incomplete_symbols")?;
            if !array.iter().any(|sym| {
                sym.get("kind").and_then(Value::as_u64) == Some(5)
                    && sym.get("name").and_then(Value::as_str) == Some(filename)
                    && sym
                        .pointer("/range/start/line")
                        .and_then(Value::as_u64)
                        .is_some()
                    && sym
                        .pointer("/range/end/line")
                        .and_then(Value::as_u64)
                        .is_some()
            }) {
                return Err("incomplete_symbols");
            }
            symbols = true;
            result.sources[index].symbols_completed = true;
        } else if message.get("method").and_then(Value::as_str)
            == Some("textDocument/publishDiagnostics")
        {
            let params = message.get("params").ok_or("protocol_loss")?;
            if params.get("uri").and_then(Value::as_str) != Some(&source_uri) {
                // Unopened/previous URI notifications are not this owner URI's fence.
                continue;
            }
            if diagnostics {
                return Err("protocol_loss");
            }
            let entries = params
                .get("diagnostics")
                .and_then(Value::as_array)
                .ok_or("protocol_loss")?;
            for item in entries {
                let severity = item
                    .get("severity")
                    .and_then(Value::as_u64)
                    .filter(|level| (1..=4).contains(level))
                    .ok_or("protocol_loss")?;
                let message = item
                    .get("message")
                    .and_then(Value::as_str)
                    .ok_or("protocol_loss")?;
                if message.len() > 2048 {
                    return Err("diagnostic_limit");
                }
                let line = item
                    .pointer("/range/start/line")
                    .and_then(Value::as_u64)
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or("protocol_loss")?;
                let column = item
                    .pointer("/range/start/character")
                    .and_then(Value::as_u64)
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or("protocol_loss")?;
                for field in ["/range/end/line", "/range/end/character"] {
                    item.pointer(field)
                        .and_then(Value::as_u64)
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or("protocol_loss")?;
                }
                if severity != 1 {
                    continue;
                }
                if result.diagnostics.len() == 64 {
                    return Err("diagnostic_limit");
                }
                result.diagnostics.push(ErrorDiagnostic {
                    path: source.path.as_str().into(),
                    source_sha256: source.captured.sha256.clone(),
                    line,
                    column,
                    message: message.to_owned(),
                });
            }
            diagnostics = true;
            result.sources[index].diagnostics_completed = true;
        } else if message.get("method").is_none() {
            return Err("protocol_loss");
        }
    }
    Ok(())
}
