// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Ciborium-backed CBOR codec.

use crate::cbor::CborCodec;
use crate::error::{DecodeError, EncodeError};
use serde::{de::DeserializeOwned, Serialize};

/// CBOR codec implementation using [ciborium](https://docs.rs/ciborium).
pub struct CiboriumCodec;

impl CborCodec for CiboriumCodec {
    fn encode_deterministic<T: Serialize>(value: &T) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        ciborium::into_writer(value, &mut buf)
            .map_err(|e| EncodeError::Serialization(e.to_string()))?;
        Ok(buf)
    }

    fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
        ciborium::from_reader(bytes).map_err(|e| DecodeError::Deserialization(e.to_string()))
    }
}
