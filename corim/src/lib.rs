// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! CoRIM (Concise Reference Integrity Manifest) generation and validation.
//!
//! This crate provides Rust types for the CoRIM/CoMID CDDL schema
//! (draft-ietf-rats-corim-10), CBOR encoding/decoding (via a swappable
//! backend), a builder API for generating launch endorsements, and
//! validation per the specification.
//!
//! # Feature gates
//!
//! - `cbor-ciborium` (default) — use [ciborium](https://docs.rs/ciborium) as
//!   the CBOR backend.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod cbor;
pub mod error;
pub mod types;

pub mod builder;
pub mod validate;

pub use error::{BuilderError, DecodeError, EncodeError, ValidationError};
