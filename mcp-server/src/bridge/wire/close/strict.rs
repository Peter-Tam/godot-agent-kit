use super::*;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
#[derive(Clone, Copy)]
struct Unique {
    public_input: bool,
    required: &'static [&'static str],
}
impl<'de> DeserializeSeed<'de> for Unique {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Unique {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a strict bounded JSON value")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let mut count = 0;
        while seq.next_element_seed(self)?.is_some() {
            count += 1;
            if count > 4096 {
                return Err(serde::de::Error::custom("collection bound"));
            }
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let mut keys = std::collections::BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            let required = if self.public_input {
                public_nullable(&key)
            } else {
                &[]
            };
            if key.len() > 2048 || keys.len() >= 1024 || !keys.insert(key) {
                return Err(serde::de::Error::custom("duplicate or excessive key"));
            }
            map.next_value_seed(Unique {
                public_input: self.public_input,
                required,
            })?;
        }
        if self.required.iter().any(|key| !keys.contains(*key)) {
            return Err(serde::de::Error::custom("missing required nullable key"));
        }
        Ok(())
    }
}

// Shared live DTOs accept omitted Option fields. Public observation-v1 input
// requires explicit nulls; enforce presence in the existing bounded key scan
// without copying source-bearing DTOs or changing other operations' decoders.
fn public_nullable(key: &str) -> &'static [&'static str] {
    match key {
        "document" => &["identity"],
        "identity" => &[
            "script_instance_id",
            "editor_instance_id",
            "buffer_instance_id",
            "disk_file_id",
        ],
        "validity" | "open_state" => &["value", "collection", "reason", "invalidated_evidence"],
        // Observation-v1 omits unavailable text/collection, rather than nulling them.
        "D" | "R" | "B" => &["witness", "staleness", "reason", "invalidated_evidence"],
        "dirty" => &["collection", "witness", "reason", "invalidated_evidence"],
        "witness" => &[
            "script_instance_id",
            "editor_instance_id",
            "buffer_instance_id",
            "disk_file_id",
            "source_version",
        ],
        "diagnostics" => &["surface"],
        _ => &[],
    }
}

pub(super) fn check(bytes: &[u8], stage: Stage) -> Result<(), RoutingFailure> {
    check_mode(bytes, stage, false)
}
pub(super) fn check_request(bytes: &[u8], stage: Stage) -> Result<(), RoutingFailure> {
    check_mode(bytes, stage, true)
}
fn check_mode(bytes: &[u8], stage: Stage, public_input: bool) -> Result<(), RoutingFailure> {
    if bytes.is_empty() || bytes.len() > RESULT_LIMIT {
        return Err(bad(stage));
    }
    json_depth(bytes, stage)?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    Unique {
        public_input,
        required: &[],
    }
    .deserialize(&mut deserializer)
    .map_err(|_| bad(stage))?;
    deserializer.end().map_err(|_| bad(stage))
}
