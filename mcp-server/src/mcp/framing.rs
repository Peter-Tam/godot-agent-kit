use crate::runner::AttemptClock;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::fmt;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;

pub(super) const FRAME_LIMIT: usize = 16 * 1024 * 1024;
pub(super) const RESPONSE_LIMIT: usize = 64 * 1024 * 1024;
pub(super) enum Input {
    Frame(Vec<u8>, AttemptClock),
    End,
    Failure,
}

// A blocking stdin reader is deliberately not a runtime blocking task. It cannot be
// joined when an inherited pipe remains open; executable teardown exits that thread.
pub(super) fn reader(partial: Arc<Mutex<Option<Instant>>>) -> io::Result<mpsc::Receiver<Input>> {
    let (tx, rx) = mpsc::channel(1);
    std::thread::Builder::new()
        .name("mcp-input".into())
        .spawn(move || {
            let mut stdin = io::stdin().lock();
            let mut frame = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let count = match stdin.read(&mut buffer) {
                    Ok(0) => {
                        let _ = tx.blocking_send(if frame.is_empty() {
                            Input::End
                        } else {
                            Input::Failure
                        });
                        return;
                    }
                    Ok(count) => count,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        let _ = tx.blocking_send(Input::Failure);
                        return;
                    }
                };
                for &byte in &buffer[..count] {
                    if frame.is_empty() {
                        match partial.lock() {
                            Ok(mut partial) => *partial = Some(Instant::now()),
                            Err(_) => {
                                let _ = tx.blocking_send(Input::Failure);
                                return;
                            }
                        }
                    }
                    if byte == b'\n' {
                        let clock = AttemptClock::start();
                        match partial.lock() {
                            Ok(mut partial) => *partial = None,
                            Err(_) => {
                                let _ = tx.blocking_send(Input::Failure);
                                return;
                            }
                        }
                        if tx
                            .blocking_send(Input::Frame(std::mem::take(&mut frame), clock))
                            .is_err()
                        {
                            return;
                        }
                    } else {
                        if frame.len() == FRAME_LIMIT - 1 {
                            let _ = tx.blocking_send(Input::Failure);
                            return;
                        }
                        frame.push(byte);
                    }
                }
            }
        })?;
    Ok(rx)
}

struct Strict(usize);
impl<'de> DeserializeSeed<'de> for Strict {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Strict {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON")
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid number"))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }
    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        if self.0 >= 64 {
            return Err(de::Error::custom("depth limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(Strict(self.0 + 1))? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        if self.0 >= 64 {
            return Err(de::Error::custom("depth limit"));
        }
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate key"));
            }
            values.insert(key, map.next_value_seed(Strict(self.0 + 1))?);
        }
        Ok(Value::Object(values))
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<Value, ()> {
    let text = std::str::from_utf8(bytes).map_err(|_| ())?;
    let mut decoder = serde_json::Deserializer::from_str(text);
    let value = Strict(0).deserialize(&mut decoder).map_err(|_| ())?;
    decoder.end().map_err(|_| ())?;
    Ok(value)
}

pub(super) struct CappedBytes {
    pub bytes: Vec<u8>,
    limit: usize,
}
impl CappedBytes {
    pub fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}
impl io::Write for CappedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("response limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) struct ByteCount {
    count: usize,
    limit: usize,
}
impl ByteCount {
    pub fn new(limit: usize) -> Self {
        Self { count: 0, limit }
    }
}
impl io::Write for ByteCount {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.count) {
            return Err(io::Error::other("object limit"));
        }
        self.count += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicates_and_depth_are_strict() {
        assert!(parse(br#"{"a":{"b":1,"b":2}}"#).is_err());
        let bounded = format!("{}0{}", "[".repeat(64), "]".repeat(64));
        assert!(parse(bounded.as_bytes()).is_ok());
        let excessive = format!("{}0{}", "[".repeat(65), "]".repeat(65));
        assert!(parse(excessive.as_bytes()).is_err());
    }
    #[test]
    fn escaped_representation_and_complete_response_bounds_are_checked() {
        let source = Value::String("\"\\\n".repeat(16));
        // JSON escaping exceeds the decoded UTF-8 source size.
        assert!(serde_json::to_writer(ByteCount::new(64), &source).is_err());
        assert!(serde_json::to_writer(ByteCount::new(128), &source).is_ok());
        let mut object = ByteCount::new(FRAME_LIMIT);
        let block = [b'x'; 4096];
        for _ in 0..FRAME_LIMIT / block.len() {
            io::Write::write_all(&mut object, &block).unwrap();
        }
        assert!(io::Write::write_all(&mut object, b"x").is_err());
        let mut response = ByteCount::new(RESPONSE_LIMIT - 1);
        for _ in 0..RESPONSE_LIMIT / block.len() - 1 {
            io::Write::write_all(&mut response, &block).unwrap();
        }
        io::Write::write_all(&mut response, &block[..4095]).unwrap();
        assert!(io::Write::write_all(&mut response, b"x").is_err());
    }
}
