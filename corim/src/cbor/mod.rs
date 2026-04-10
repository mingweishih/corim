// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! CBOR encoding/decoding abstraction layer.
//!
//! Defines a [`CborCodec`] trait with deterministic encoding, plus a
//! backend-agnostic [`value::Value`] enum and [`value::Tagged`] wrapper.
//! The default implementation uses ciborium (behind the `cbor-ciborium`
//! feature gate). To swap backends, add a new feature and implement
//! `Serialize`/`Deserialize` for `Value` and `Tagged<T>`.
//!
//! At least one CBOR backend feature must be enabled.

#[cfg(not(feature = "cbor-ciborium"))]
compile_error!("At least one CBOR backend must be enabled. Enable the `cbor-ciborium` feature.");

#[cfg(feature = "cbor-ciborium")]
mod ciborium_backend;

pub mod value;

use crate::error::{DecodeError, EncodeError};
use serde::{de::DeserializeOwned, Serialize};

/// Trait abstracting CBOR encode/decode operations.
///
/// Only deterministic encoding is provided. Map keys are emitted in ascending
/// integer order by the `CborSerialize` derive macro, satisfying RFC 8949
/// §4.2.1 (CBOR Core Deterministic Encoding).
pub trait CborCodec {
    /// Encode a value as deterministic CBOR bytes.
    fn encode_deterministic<T: Serialize>(value: &T) -> Result<Vec<u8>, EncodeError>;

    /// Decode a value from CBOR bytes.
    fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError>;
}

/// The default codec selected by feature flags.
#[cfg(feature = "cbor-ciborium")]
pub type DefaultCodec = ciborium_backend::CiboriumCodec;

/// Convenience: encode using the default codec.
#[cfg(feature = "cbor-ciborium")]
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, EncodeError> {
    DefaultCodec::encode_deterministic(value)
}

/// Convenience: decode using the default codec.
#[cfg(feature = "cbor-ciborium")]
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
    DefaultCodec::decode(bytes)
}
