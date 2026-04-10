// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Ciborium-backed serde impls for `Value` and `Tagged<T>`.

use super::{Tagged, Value};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Value <-> ciborium::value::Value conversion
// ---------------------------------------------------------------------------

impl From<ciborium::value::Value> for Value {
    fn from(cv: ciborium::value::Value) -> Self {
        match cv {
            ciborium::value::Value::Integer(i) => Value::Integer(i.into()),
            ciborium::value::Value::Bytes(b) => Value::Bytes(b),
            ciborium::value::Value::Text(t) => Value::Text(t),
            ciborium::value::Value::Array(a) => {
                Value::Array(a.into_iter().map(Value::from).collect())
            }
            ciborium::value::Value::Map(m) => Value::Map(
                m.into_iter()
                    .map(|(k, v)| (Value::from(k), Value::from(v)))
                    .collect(),
            ),
            ciborium::value::Value::Tag(t, v) => Value::Tag(t, Box::new(Value::from(*v))),
            ciborium::value::Value::Bool(b) => Value::Bool(b),
            ciborium::value::Value::Null => Value::Null,
            ciborium::value::Value::Float(f) => Value::Float(f),
            _ => Value::Null, // forward-compat for future ciborium variants
        }
    }
}

impl From<Value> for ciborium::value::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Integer(i) => {
                // ciborium::value::Integer only supports i64 range via From
                ciborium::value::Value::Integer(ciborium::value::Integer::try_from(i).unwrap_or_else(|_| {
                    // Fallback: clamp to i64 range (should not happen for CoRIM data)
                    ciborium::value::Integer::from(i as i64)
                }))
            }
            Value::Bytes(b) => ciborium::value::Value::Bytes(b),
            Value::Text(t) => ciborium::value::Value::Text(t),
            Value::Array(a) => {
                ciborium::value::Value::Array(a.into_iter().map(Into::into).collect())
            }
            Value::Map(m) => ciborium::value::Value::Map(
                m.into_iter()
                    .map(|(k, v)| (Into::into(k), Into::into(v)))
                    .collect(),
            ),
            Value::Tag(t, v) => {
                ciborium::value::Value::Tag(t, Box::new((*v).into()))
            }
            Value::Bool(b) => ciborium::value::Value::Bool(b),
            Value::Null => ciborium::value::Value::Null,
            Value::Float(f) => ciborium::value::Value::Float(f),
        }
    }
}

// ---------------------------------------------------------------------------
// Value serde
// ---------------------------------------------------------------------------

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let cv: ciborium::value::Value = self.clone().into();
        cv.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let cv = ciborium::value::Value::deserialize(d)?;
        Ok(Value::from(cv))
    }
}

// ---------------------------------------------------------------------------
// Tagged<T> serde — delegates to ciborium::tag::Required
// ---------------------------------------------------------------------------

impl<T: Serialize> Serialize for Tagged<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Build a ciborium Value::Tag around the inner serialization
        // We serialize T to a ciborium::value::Value first, then wrap in a tag.
        let inner_value = ciborium::value::Value::serialized(&self.value)
            .map_err(serde::ser::Error::custom)?;
        let tagged = ciborium::value::Value::Tag(self.tag, Box::new(inner_value));
        tagged.serialize(s)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Tagged<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Deserialize as a dynamic value, check tag, then deserialize inner
        let cv = ciborium::value::Value::deserialize(d)?;
        match cv {
            ciborium::value::Value::Tag(tag, inner) => {
                let value: T = inner
                    .deserialized()
                    .map_err(|e| de::Error::custom(format!("inner value: {}", e)))?;
                Ok(Tagged { tag, value })
            }
            _ => Err(de::Error::custom("expected CBOR tag")),
        }
    }
}

/// Serialize a value wrapped in a specific CBOR tag number.
///
/// This is a convenience for one-off tagged serialization without constructing
/// a `Tagged<T>`.
pub fn serialize_tagged<T: Serialize, S: Serializer>(
    tag: u64,
    value: &T,
    s: S,
) -> Result<S::Ok, S::Error> {
    let inner_value = ciborium::value::Value::serialized(value)
        .map_err(serde::ser::Error::custom)?;
    let tagged = ciborium::value::Value::Tag(tag, Box::new(inner_value));
    tagged.serialize(s)
}

/// Serialize bytes wrapped in a specific CBOR tag number.
pub fn serialize_tagged_bytes<S: Serializer>(
    tag: u64,
    bytes: &[u8],
    s: S,
) -> Result<S::Ok, S::Error> {
    let inner = ciborium::value::Value::Bytes(bytes.to_vec());
    let tagged = ciborium::value::Value::Tag(tag, Box::new(inner));
    tagged.serialize(s)
}
