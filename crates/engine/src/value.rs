use std::collections::HashMap;

use serde::{Deserialize, Serialize, de::Visitor};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub enum Value {
    String(String),
    SignedInt(i64),
    UnsignedInt(u64),
    FloatingPoint(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Null,
}
impl Value {
    /// Attempts to merge the incoming value with the current value. Currently this feature is only
    /// supported with in-memory operations. Attempting to merge against a location on disk will
    /// result in loading the entire location into memory, and operating on it there.
    pub fn merge(self, value: &Value) -> Value {
        /// Merges the patch onto the target value, following the algorithm laid out in RFC7396
        fn merge_inner(target: Value, patch: &Value) -> Value {
            if let Value::Object(patch) = patch {
                let mut target = if let Value::Object(target) = target {
                    target.into_iter().collect::<HashMap<_, _>>()
                } else {
                    HashMap::new()
                };
                for (name, value) in patch {
                    let old_value = target.remove(name);
                    if *value != Value::Null {
                        target.insert(
                            name.to_string(),
                            merge_inner(old_value.unwrap_or(Value::Null), value),
                        );
                    }
                }
                Value::Object(target.into_iter().collect())
            } else {
                patch.clone()
            }
        }
        merge_inner(self, value)
    }
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::String(value) => {
                value.hash(state);
            }
            Value::SignedInt(value) => {
                value.hash(state);
            }
            Value::UnsignedInt(value) => {
                value.hash(state);
            }
            Value::FloatingPoint(value) => {
                value.to_bits().hash(state);
            }
            Value::Boolean(value) => {
                value.hash(state);
            }
            Value::Array(values) => {
                values.hash(state);
            }
            Value::Object(items) => {
                items.hash(state);
            }
            Value::Null => {
                ().hash(state);
            }
        }
    }
}
impl<'a> Deserialize<'a> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}
pub struct ValueVisitor;

impl<'a> Visitor<'a> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Boolean(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::SignedInt(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::UnsignedInt(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::FloatingPoint(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.to_owned().into()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bytes(v),
            &self,
        ))
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(&v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::NewtypeStruct,
            &self,
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut buffer = Vec::new();
        while let Some(value) = seq.next_element::<Value>()? {
            buffer.push(value);
        }
        Ok(Value::Array(buffer.into()))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'a>,
    {
        let mut buffer = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            buffer.push((key, value));
        }
        Ok(Value::Object(buffer))
    }
}
