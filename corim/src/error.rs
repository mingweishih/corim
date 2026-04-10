// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Error types for the corim crate.

use thiserror::Error;

/// Errors from CBOR encoding.
#[derive(Debug, Error)]
pub enum EncodeError {
    /// CBOR serialization failed.
    #[error("CBOR serialization failed: {0}")]
    Serialization(String),
}

/// Errors from CBOR decoding.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// CBOR deserialization failed.
    #[error("CBOR deserialization failed: {0}")]
    Deserialization(String),

    /// Expected a specific CBOR tag but found a different one.
    #[error("expected CBOR tag {expected}, found {found}")]
    UnexpectedTag {
        /// The tag number that was expected.
        expected: u64,
        /// The tag number that was found.
        found: u64,
    },

    /// The decoded structure is invalid.
    #[error("invalid structure: {0}")]
    InvalidStructure(String),
}

/// Errors from the builder API.
#[derive(Debug, Error)]
pub enum BuilderError {
    /// A required field was not set.
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    /// The triples map is empty.
    #[error("triples map is empty — at least one triple type must be populated")]
    EmptyTriples,

    /// No CoMID tags were added to the CoRIM.
    #[error("at least one CoMID tag is required")]
    NoTags,

    /// An encoding error occurred during building.
    #[error("encoding error: {0}")]
    Encode(#[from] EncodeError),
}

/// Errors from validation / appraisal.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// A decode error occurred during validation.
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),

    /// The CoRIM has expired.
    #[error("CoRIM has expired (not-after is in the past)")]
    Expired,

    /// The CoMID tag-identity is missing tag-id.
    #[error("CoMID tag-identity is missing tag-id")]
    MissingTagId,

    /// The CoMID triples map is empty.
    #[error("CoMID triples map is empty")]
    EmptyTriples,

    /// A non-empty constraint was violated.
    #[error("non-empty constraint violated: {0}")]
    NonEmpty(String),

    /// No common digest algorithms between reference and evidence.
    #[error("no common digest algorithms between reference and evidence")]
    NoCommonAlgorithms,

    /// Digest values do not match for a given algorithm.
    #[error("digest mismatch for algorithm {alg}")]
    DigestMismatch {
        /// The algorithm identifier where the mismatch occurred.
        alg: i64,
    },

    /// SVN values do not match.
    #[error("SVN mismatch: expected {expected}, got {actual}")]
    SvnMismatch {
        /// The expected SVN value.
        expected: u64,
        /// The actual SVN value.
        actual: u64,
    },

    /// Conditional endorsement series entries use inconsistent mkeys.
    #[error("conditional-endorsement-series entries use inconsistent mkeys")]
    InconsistentMkeys,
}
