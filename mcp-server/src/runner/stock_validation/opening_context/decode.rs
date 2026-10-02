//! Closed-schema context decoding with pre-retention budgets.
use super::{Binding, ContextKind, OpeningContext, Projection, Property, Warnings};
use crate::observation::SOURCE_LIMIT_BYTES;
use serde::Deserialize;
use std::collections::BTreeMap;

// Decode the closed context schema with shared remaining budgets. Checks happen
// before retaining strings or growing collections; serde_json's escaped-string
// scratch remains covered by the outer transport frame limit.
pub(in super::super) struct DecodeBudget {
    pub(in super::super) source: usize,
    pub(in super::super) metadata: usize,
}
impl DecodeBudget {
    pub(in super::super) fn new() -> Self {
        Self {
            source: SOURCE_LIMIT_BYTES,
            metadata: 256 * 1024,
        }
    }
    pub(in super::super) fn charge<E: serde::de::Error>(&mut self, bytes: usize) -> Result<(), E> {
        self.metadata = self
            .metadata
            .checked_sub(bytes)
            .ok_or_else(|| E::custom("close_context_limit"))?;
        Ok(())
    }
}
pub(in super::super) trait ContextDecode: Sized {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        budget: &mut DecodeBudget,
    ) -> Result<Self, D::Error>;
}
pub(in super::super) struct ContextSeed<'a, T>(
    pub(in super::super) &'a mut DecodeBudget,
    pub(in super::super) std::marker::PhantomData<T>,
);
impl<'de, T: ContextDecode> serde::de::DeserializeSeed<'de> for ContextSeed<'_, T> {
    type Value = T;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        T::decode(d, self.0)
    }
}
fn retained_string<'de, D: serde::Deserializer<'de>>(
    d: D,
    remaining: &mut usize,
) -> Result<String, D::Error> {
    struct Text<'a>(&'a mut usize);
    impl<'de> serde::de::Visitor<'de> for Text<'_> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded context string")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            if value.len() > *self.0 {
                return Err(E::custom("close_context_limit"));
            }
            *self.0 -= value.len();
            Ok(value.to_owned())
        }
        fn visit_string<E: serde::de::Error>(self, value: String) -> Result<String, E> {
            if value.len() > *self.0 {
                return Err(E::custom("close_context_limit"));
            }
            *self.0 -= value.len();
            Ok(value)
        }
    }
    d.deserialize_string(Text(remaining))
}
impl ContextDecode for String {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        b: &mut DecodeBudget,
    ) -> Result<Self, D::Error> {
        b.charge::<D::Error>(4)?;
        retained_string(d, &mut b.metadata)
    }
}
macro_rules! scalar_context {
    ($($ty:ty),*) => {$(
        impl ContextDecode for $ty {
            fn decode<'de, D: serde::Deserializer<'de>>(d: D, b: &mut DecodeBudget) -> Result<Self, D::Error> {
                b.charge::<D::Error>(std::mem::size_of::<Self>())?;
                Self::deserialize(d)
            }
        }
    )*};
}
scalar_context!(bool, u64, u8, ContextKind);
impl<T: ContextDecode> ContextDecode for Option<T> {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        b: &mut DecodeBudget,
    ) -> Result<Self, D::Error> {
        struct Nullable<'a, T>(&'a mut DecodeBudget, std::marker::PhantomData<T>);
        impl<'de, T: ContextDecode> serde::de::Visitor<'de> for Nullable<'_, T> {
            type Value = Option<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("required nullable context field")
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }
            fn visit_some<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                T::decode(d, self.0).map(Some)
            }
        }
        d.deserialize_option(Nullable::<T>(b, std::marker::PhantomData))
    }
}
struct SourceText(Option<String>);
impl ContextDecode for SourceText {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        b: &mut DecodeBudget,
    ) -> Result<Self, D::Error> {
        struct Source<'a>(&'a mut usize);
        impl<'de> serde::de::Visitor<'de> for Source<'_> {
            type Value = SourceText;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("bounded nullable source")
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(SourceText(None))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(SourceText(None))
            }
            fn visit_some<D: serde::Deserializer<'de>>(
                self,
                d: D,
            ) -> Result<Self::Value, D::Error> {
                retained_string(d, self.0).map(|s| SourceText(Some(s)))
            }
        }
        d.deserialize_option(Source(&mut b.source))
    }
}
pub(in super::super) struct Records<T, const N: usize>(pub(in super::super) Vec<T>);
impl<T: ContextDecode, const N: usize> ContextDecode for Records<T, N> {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        b: &mut DecodeBudget,
    ) -> Result<Self, D::Error> {
        b.charge::<D::Error>(4)?;
        struct Items<'a, T, const N: usize>(&'a mut DecodeBudget, std::marker::PhantomData<T>);
        impl<'de, T: ContextDecode, const N: usize> serde::de::Visitor<'de> for Items<'_, T, N> {
            type Value = Records<T, N>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "at most {N} context records")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut out = Vec::new();
                while out.len() < N {
                    let Some(value) =
                        seq.next_element_seed(ContextSeed::<T>(self.0, std::marker::PhantomData))?
                    else {
                        return Ok(Records(out));
                    };
                    out.push(value);
                }
                struct Reject;
                impl<'de> serde::de::DeserializeSeed<'de> for Reject {
                    type Value = ();
                    fn deserialize<D: serde::Deserializer<'de>>(
                        self,
                        _: D,
                    ) -> Result<(), D::Error> {
                        Err(serde::de::Error::custom("close_context_limit"))
                    }
                }
                // Reject the eighth record without traversing or retaining it.
                seq.next_element_seed(Reject)?;
                Ok(Records(out))
            }
        }
        d.deserialize_seq(Items::<T, N>(b, std::marker::PhantomData))
    }
}
impl ContextDecode for BTreeMap<String, u8> {
    fn decode<'de, D: serde::Deserializer<'de>>(
        d: D,
        b: &mut DecodeBudget,
    ) -> Result<Self, D::Error> {
        b.charge::<D::Error>(4)?;
        struct Key<'a>(&'a mut DecodeBudget, bool);
        impl<'de> serde::de::DeserializeSeed<'de> for Key<'_> {
            type Value = String;
            fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
                if self.1 {
                    return Err(serde::de::Error::custom("invalid guard warning map"));
                }
                self.0.charge::<D::Error>(4)?;
                let mut remaining = self.0.metadata.min(2048);
                let key = retained_string(d, &mut remaining)?;
                self.0.charge::<D::Error>(key.len())?;
                Ok(key)
            }
        }
        struct Levels<'a>(&'a mut DecodeBudget);
        impl<'de> serde::de::Visitor<'de> for Levels<'_> {
            type Value = BTreeMap<String, u8>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("bounded unique warning map")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut out = BTreeMap::new();
                while let Some(key) = map.next_key_seed(Key(self.0, out.len() == 128))? {
                    if out.contains_key(&key) {
                        return Err(serde::de::Error::custom("invalid guard warning map"));
                    }
                    let value =
                        map.next_value_seed(ContextSeed::<u8>(self.0, std::marker::PhantomData))?;
                    out.insert(key, value);
                }
                Ok(out)
            }
        }
        d.deserialize_map(Levels(b))
    }
}
// Typed fields, not a generic JSON tree: retain the existing projection types
// and require every field exactly once, regardless of input order.
macro_rules! context_record {
    ($ty:ident { $($field:ident : $decode:ty => $convert:expr),* $(,)? }) => {
        impl ContextDecode for $ty {
            fn decode<'de, D: serde::Deserializer<'de>>(d: D, b: &mut DecodeBudget) -> Result<Self, D::Error> {
                struct Field(&'static str);
                impl<'de> Deserialize<'de> for Field {
                    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                        struct Key;
                        impl<'de> serde::de::Visitor<'de> for Key {
                            type Value = Field;
                            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("known context field") }
                            fn visit_str<E: serde::de::Error>(self, key: &str) -> Result<Field, E> {
                                $(if key == stringify!($field).trim_start_matches("r#") {
                                    return Ok(Field(stringify!($field)));
                                })*
                                Err(E::unknown_field(key, &[$(stringify!($field)),*]))
                            }
                        }
                        d.deserialize_identifier(Key)
                    }
                }
                struct Record<'a>(&'a mut DecodeBudget);
                impl<'de> serde::de::Visitor<'de> for Record<'_> {
                    type Value = $ty;
                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(stringify!($ty)) }
                    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<$ty, A::Error> {
                        self.0.charge::<A::Error>(4)?;
                        $(let mut $field = None;)*
                        while let Some(key) = map.next_key::<Field>()? {
                            match key.0 {
                                $(key if key == stringify!($field) => {
                                    if $field.is_some() { return Err(serde::de::Error::duplicate_field(stringify!($field))); }
                                    self.0.charge::<A::Error>(4 + stringify!($field).trim_start_matches("r#").len())?;
                                    $field = Some(map.next_value_seed(ContextSeed::<$decode>(self.0, std::marker::PhantomData))?);
                                }),*
                                _ => unreachable!(),
                            }
                        }
                        Ok($ty {
                            $($field: {
                                let convert = $convert;
                                convert($field.ok_or_else(|| serde::de::Error::missing_field(stringify!($field)))?)
                            },)*
                        })
                    }
                }
                d.deserialize_struct(stringify!($ty), &[$(stringify!($field)),*], Record(b))
            }
        }
    };
}
pub(in super::super) use context_record;
context_record!(OpeningContext {
    kind: ContextKind => |v| v, reason: Option<String> => |v| v,
    projection: Option<Projection> => |v| v, source: SourceText => |v: SourceText| v.0,
    sha256: Option<String> => |v| v,
});
context_record!(Projection {
    request_id: String => |v| v, session_id: String => |v| v, project_root: String => |v| v,
    project_device: String => |v| v, project_inode: String => |v| v, path: String => |v| v,
    script_id: String => |v| v, editor_id: String => |v| v, buffer_id: String => |v| v,
    source_sha256: String => |v| v, source_length: String => |v| v,
    version: String => |v| v, saved_version: String => |v| v,
    dirty: bool => |v| v, resource_edited: bool => |v| v, has_undo: bool => |v| v,
    has_redo: bool => |v| v, tool: bool => |v| v, external_editor: bool => |v| v,
    script_base_id: String => |v| v,
    properties: Records<Property, 64> => |v: Records<Property, 64>| v.0,
    methods: Records<String, 64> => |v: Records<String, 64>| v.0,
    warnings: Warnings => |v| v,
    global_classes: Records<String, 64> => |v: Records<String, 64>| v.0,
    autoloads: Records<String, 64> => |v: Records<String, 64>| v.0,
    bindings: Records<Binding, 256> => |v: Records<Binding, 256>| v.0,
});
context_record!(Property {
    name: String => |v| v, r#type: u64 => |v| v, hint: u64 => |v| v,
    hint_string: String => |v| v, usage: u64 => |v| v, class_name: String => |v| v,
});
context_record!(Binding { name: String => |v| v, api_type: u64 => |v| v });
context_record!(Warnings {
    enable: bool => |v| v, levels: BTreeMap<String, u8> => |v| v,
    directory_rules: BTreeMap<String, u8> => |v| v,
});

impl<'de> Deserialize<'de> for OpeningContext {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::decode(d, &mut DecodeBudget::new())
    }
}
pub(in super::super) fn captured_source<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<String, D::Error> {
    let mut remaining = SOURCE_LIMIT_BYTES;
    retained_string(d, &mut remaining)
}

pub(in super::super) fn bounded<
    'de,
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
    const N: usize,
>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Sequence<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Sequence<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "at most {N} private context records")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<T>, A::Error> {
            if seq.size_hint().is_some_and(|n| n > N) {
                return Err(serde::de::Error::custom("context_limit"));
            }
            let mut records = Vec::new();
            while records.len() < N {
                let Some(record) = seq.next_element()? else {
                    return Ok(records);
                };
                records.push(record);
            }
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("context_limit"));
            }
            Ok(records)
        }
    }
    d.deserialize_seq(Sequence::<T, N>(std::marker::PhantomData))
}
pub(super) fn properties<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<Property>, D::Error> {
    bounded::<_, _, 64>(d)
}
pub(super) fn names<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    bounded::<_, _, 64>(d)
}
pub(super) fn bindings<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Binding>, D::Error> {
    bounded::<_, _, 256>(d)
}

pub(super) fn unique_levels<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, u8>, D::Error> {
    struct Levels;
    impl<'de> serde::de::Visitor<'de> for Levels {
        type Value = BTreeMap<String, u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded unique guard warning levels")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, u8>()? {
                if out.len() == 128 || key.len() > 2048 || out.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("invalid guard warning map"));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_map(Levels)
}
