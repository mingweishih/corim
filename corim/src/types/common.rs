// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Common types shared across CoRIM and CoMID structures.
//!
//! All custom serde impls use [`crate::cbor::value::Value`] for tag dispatch
//! rather than importing any specific CBOR backend directly.

use corim_derive::{CborDeserialize, CborSerialize};
use serde::{Deserialize, Serialize};

use crate::cbor::value::{self, Value};
use crate::types::measurement::Digest;

// ---------------------------------------------------------------------------
// tag-identity-map  { tag-id: 0, tag-version: 1 }
// ---------------------------------------------------------------------------

/// `tag-identity-map` — identifies a CoMID tag.
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct TagIdentity {
    /// `tag-id` (key 0): globally unique tag identifier.
    #[cbor(key = 0)]
    pub tag_id: TagIdChoice,
    /// `tag-version` (key 1): optional revision number.
    #[cbor(key = 1, optional)]
    pub tag_version: Option<u64>,
}

// ---------------------------------------------------------------------------
// validity-map  { not-before: 0, not-after: 1 }
// ---------------------------------------------------------------------------

/// `validity-map` — time window (epoch seconds).
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct ValidityMap {
    /// `not-before` (key 0): optional start of validity.
    #[cbor(key = 0, optional)]
    pub not_before: Option<i64>,
    /// `not-after` (key 1): end of validity.
    #[cbor(key = 1)]
    pub not_after: i64,
}

// ---------------------------------------------------------------------------
// entity-map  { entity-name: 0, reg-id: 1, role: 2 }
// ---------------------------------------------------------------------------

/// `entity-map` — describes an entity (creator, signer, etc.).
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct EntityMap {
    /// `entity-name` (key 0): name of the entity.
    #[cbor(key = 0)]
    pub entity_name: String,
    /// `reg-id` (key 1): optional URI for the organization.
    #[cbor(key = 1, optional)]
    pub reg_id: Option<String>,
    /// `role` (key 2): list of roles.
    #[cbor(key = 2)]
    pub role: Vec<i64>,
}

// ---------------------------------------------------------------------------
// version-map  { version: 0, version-scheme: 1 }
// ---------------------------------------------------------------------------

/// `version-map` — software version info.
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct VersionMap {
    /// `version` (key 0): the version string.
    #[cbor(key = 0)]
    pub version: String,
    /// `version-scheme` (key 1): optional versioning convention.
    #[cbor(key = 1, optional)]
    pub version_scheme: Option<i64>,
}

// ---------------------------------------------------------------------------
// Type-choice enums
// ---------------------------------------------------------------------------

/// `$tag-id-type-choice` — text string or UUID.
#[derive(Clone, Debug, PartialEq)]
pub enum TagIdChoice {
    /// A textual tag identifier.
    Text(String),
    /// A 16-byte UUID (CBOR tag 37).
    Uuid([u8; 16]),
}

impl Serialize for TagIdChoice {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            TagIdChoice::Text(t) => s.serialize_str(t),
            TagIdChoice::Uuid(u) => value::serialize_tagged_bytes(37, u.as_slice(), s),
        }
    }
}

impl<'de> Deserialize<'de> for TagIdChoice {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Text(t) => Ok(TagIdChoice::Text(t)),
            Value::Tag(37, inner) => {
                let b = inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 37 must wrap bytes"))?;
                let arr: [u8; 16] = b.try_into().map_err(|_| serde::de::Error::custom("UUID must be 16 bytes"))?;
                Ok(TagIdChoice::Uuid(arr))
            }
            _ => Err(serde::de::Error::custom("expected text or tagged UUID")),
        }
    }
}

/// `$class-id-type-choice` — OID, UUID, or generic bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum ClassIdChoice {
    /// OID (CBOR tag 111).
    Oid(Vec<u8>),
    /// UUID (CBOR tag 37).
    Uuid([u8; 16]),
    /// Generic tagged bytes (CBOR tag 560).
    Bytes(Vec<u8>),
}

impl Serialize for ClassIdChoice {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            ClassIdChoice::Oid(b) => value::serialize_tagged_bytes(111, b, s),
            ClassIdChoice::Uuid(u) => value::serialize_tagged_bytes(37, u, s),
            ClassIdChoice::Bytes(b) => value::serialize_tagged_bytes(560, b, s),
        }
    }
}

impl<'de> Deserialize<'de> for ClassIdChoice {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Tag(111, inner) => Ok(ClassIdChoice::Oid(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 111 must wrap bytes"))?)),
            Value::Tag(37, inner) => {
                let b = inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 37 must wrap bytes"))?;
                Ok(ClassIdChoice::Uuid(b.try_into().map_err(|_| serde::de::Error::custom("UUID must be 16 bytes"))?))
            }
            Value::Tag(560, inner) => Ok(ClassIdChoice::Bytes(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 560 must wrap bytes"))?)),
            _ => Err(serde::de::Error::custom("expected tagged OID, UUID, or bytes")),
        }
    }
}

/// `$instance-id-type-choice` — UEID, UUID, bytes, or crypto key types.
#[derive(Clone, Debug, PartialEq)]
pub enum InstanceIdChoice {
    /// UEID (CBOR tag 550).
    Ueid(Vec<u8>),
    /// UUID (CBOR tag 37).
    Uuid([u8; 16]),
    /// Generic tagged bytes (CBOR tag 560).
    Bytes(Vec<u8>),
    /// PEM SubjectPublicKeyInfo (CBOR tag 554).
    PkixBase64Key(String),
    /// PEM X.509 certificate (CBOR tag 555).
    PkixBase64Cert(String),
    /// CBOR-encoded COSE_Key (CBOR tag 558).
    CoseKey(Vec<u8>),
    /// Key thumbprint digest (CBOR tag 557).
    KeyThumbprint(Digest),
    /// Cert thumbprint digest (CBOR tag 559).
    CertThumbprint(Digest),
    /// ASN.1 DER X.509 certificate (CBOR tag 562).
    PkixAsn1DerCert(Vec<u8>),
}

impl Serialize for InstanceIdChoice {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            InstanceIdChoice::Ueid(b) => value::serialize_tagged_bytes(550, b, s),
            InstanceIdChoice::Uuid(u) => value::serialize_tagged_bytes(37, u, s),
            InstanceIdChoice::Bytes(b) => value::serialize_tagged_bytes(560, b, s),
            InstanceIdChoice::PkixBase64Key(t) => value::serialize_tagged(554, t, s),
            InstanceIdChoice::PkixBase64Cert(t) => value::serialize_tagged(555, t, s),
            InstanceIdChoice::CoseKey(b) => value::serialize_tagged_bytes(558, b, s),
            InstanceIdChoice::KeyThumbprint(d) => value::serialize_tagged(557, d, s),
            InstanceIdChoice::CertThumbprint(d) => value::serialize_tagged(559, d, s),
            InstanceIdChoice::PkixAsn1DerCert(b) => value::serialize_tagged_bytes(562, b, s),
        }
    }
}

impl<'de> Deserialize<'de> for InstanceIdChoice {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Tag(550, inner) => Ok(InstanceIdChoice::Ueid(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 550 must wrap bytes"))?)),
            Value::Tag(37, inner) => {
                let b = inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 37 must wrap bytes"))?;
                Ok(InstanceIdChoice::Uuid(b.try_into().map_err(|_| serde::de::Error::custom("UUID must be 16 bytes"))?))
            }
            Value::Tag(554, inner) => match *inner { Value::Text(t) => Ok(InstanceIdChoice::PkixBase64Key(t)), _ => Err(serde::de::Error::custom("tag 554 must wrap text")) },
            Value::Tag(555, inner) => match *inner { Value::Text(t) => Ok(InstanceIdChoice::PkixBase64Cert(t)), _ => Err(serde::de::Error::custom("tag 555 must wrap text")) },
            Value::Tag(558, inner) => Ok(InstanceIdChoice::CoseKey(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 558 must wrap bytes"))?)),
            Value::Tag(557, inner) => {
                let arr = inner.into_array().ok_or_else(|| serde::de::Error::custom("tag 557 must wrap array"))?;
                Ok(InstanceIdChoice::KeyThumbprint(digest_from_value_array(arr)?))
            }
            Value::Tag(559, inner) => {
                let arr = inner.into_array().ok_or_else(|| serde::de::Error::custom("tag 559 must wrap array"))?;
                Ok(InstanceIdChoice::CertThumbprint(digest_from_value_array(arr)?))
            }
            Value::Tag(562, inner) => Ok(InstanceIdChoice::PkixAsn1DerCert(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 562 must wrap bytes"))?)),
            Value::Tag(560, inner) => Ok(InstanceIdChoice::Bytes(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 560 must wrap bytes"))?)),
            _ => Err(serde::de::Error::custom("expected tagged UEID, UUID, bytes, or crypto key")),
        }
    }
}

/// `$group-id-type-choice` — UUID or bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum GroupIdChoice {
    /// UUID (CBOR tag 37).
    Uuid([u8; 16]),
    /// Generic tagged bytes (CBOR tag 560).
    Bytes(Vec<u8>),
}

impl Serialize for GroupIdChoice {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GroupIdChoice::Uuid(u) => value::serialize_tagged_bytes(37, u, s),
            GroupIdChoice::Bytes(b) => value::serialize_tagged_bytes(560, b, s),
        }
    }
}

impl<'de> Deserialize<'de> for GroupIdChoice {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Tag(37, inner) => {
                let b = inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 37 must wrap bytes"))?;
                Ok(GroupIdChoice::Uuid(b.try_into().map_err(|_| serde::de::Error::custom("UUID must be 16 bytes"))?))
            }
            Value::Tag(560, inner) => Ok(GroupIdChoice::Bytes(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 560 must wrap bytes"))?)),
            _ => Err(serde::de::Error::custom("expected tagged UUID or bytes")),
        }
    }
}

/// `$measured-element-type-choice` — OID, UUID, uint, or text.
#[derive(Clone, Debug, PartialEq)]
pub enum MeasuredElement {
    /// OID (CBOR tag 111).
    Oid(Vec<u8>),
    /// UUID (CBOR tag 37).
    Uuid([u8; 16]),
    /// Unsigned integer.
    Uint(u64),
    /// Text string.
    Text(String),
}

impl Serialize for MeasuredElement {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            MeasuredElement::Oid(b) => value::serialize_tagged_bytes(111, b, s),
            MeasuredElement::Uuid(u) => value::serialize_tagged_bytes(37, u, s),
            MeasuredElement::Uint(n) => s.serialize_u64(*n),
            MeasuredElement::Text(t) => s.serialize_str(t),
        }
    }
}

impl<'de> Deserialize<'de> for MeasuredElement {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Tag(111, inner) => Ok(MeasuredElement::Oid(inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 111 must wrap bytes"))?)),
            Value::Tag(37, inner) => {
                let b = inner.into_bytes().ok_or_else(|| serde::de::Error::custom("tag 37 must wrap bytes"))?;
                Ok(MeasuredElement::Uuid(b.try_into().map_err(|_| serde::de::Error::custom("UUID must be 16 bytes"))?))
            }
            Value::Integer(n) => Ok(MeasuredElement::Uint(n.try_into().map_err(|_| serde::de::Error::custom("expected unsigned integer"))?)),
            Value::Text(t) => Ok(MeasuredElement::Text(t)),
            _ => Err(serde::de::Error::custom("expected OID, UUID, uint, or text")),
        }
    }
}

/// `$crypto-key-type-choice` — covers CBOR tags 554–562.
#[derive(Clone, Debug, PartialEq)]
pub enum CryptoKey {
    /// PEM SubjectPublicKeyInfo (CBOR tag 554).
    PkixBase64Key(String),
    /// PEM X.509 certificate (CBOR tag 555).
    PkixBase64Cert(String),
    /// PEM X.509 certificate chain (CBOR tag 556).
    PkixBase64CertPath(String),
    /// Key thumbprint `[alg, val]` (CBOR tag 557).
    KeyThumbprint(Digest),
    /// CBOR-encoded COSE_Key (CBOR tag 558).
    CoseKey(Vec<u8>),
    /// Certificate thumbprint (CBOR tag 559).
    CertThumbprint(Digest),
    /// Certification path thumbprint (CBOR tag 561).
    CertPathThumbprint(Digest),
    /// ASN.1 DER X.509 certificate (CBOR tag 562).
    PkixAsn1DerCert(Vec<u8>),
    /// Opaque key identifier (CBOR tag 560).
    Bytes(Vec<u8>),
}

impl Serialize for CryptoKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            CryptoKey::PkixBase64Key(v) => value::serialize_tagged(554, v, s),
            CryptoKey::PkixBase64Cert(v) => value::serialize_tagged(555, v, s),
            CryptoKey::PkixBase64CertPath(v) => value::serialize_tagged(556, v, s),
            CryptoKey::KeyThumbprint(v) => value::serialize_tagged(557, v, s),
            CryptoKey::CoseKey(v) => value::serialize_tagged_bytes(558, v, s),
            CryptoKey::CertThumbprint(v) => value::serialize_tagged(559, v, s),
            CryptoKey::CertPathThumbprint(v) => value::serialize_tagged(561, v, s),
            CryptoKey::PkixAsn1DerCert(v) => value::serialize_tagged_bytes(562, v, s),
            CryptoKey::Bytes(v) => value::serialize_tagged_bytes(560, v, s),
        }
    }
}

impl<'de> Deserialize<'de> for CryptoKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let val = Value::deserialize(d)?;
        match val {
            Value::Tag(554, inner) => match *inner { Value::Text(t) => Ok(CryptoKey::PkixBase64Key(t)), _ => Err(serde::de::Error::custom("tag 554 must wrap text")) },
            Value::Tag(555, inner) => match *inner { Value::Text(t) => Ok(CryptoKey::PkixBase64Cert(t)), _ => Err(serde::de::Error::custom("tag 555 must wrap text")) },
            Value::Tag(556, inner) => match *inner { Value::Text(t) => Ok(CryptoKey::PkixBase64CertPath(t)), _ => Err(serde::de::Error::custom("tag 556 must wrap text")) },
            Value::Tag(557, inner) => {
                let arr = inner.into_array().ok_or_else(|| serde::de::Error::custom("tag 557 must wrap array"))?;
                Ok(CryptoKey::KeyThumbprint(digest_from_value_array(arr)?))
            }
            Value::Tag(558, inner) => match *inner { Value::Bytes(b) => Ok(CryptoKey::CoseKey(b)), _ => Err(serde::de::Error::custom("tag 558 must wrap bytes")) },
            Value::Tag(559, inner) => {
                let arr = inner.into_array().ok_or_else(|| serde::de::Error::custom("tag 559 must wrap array"))?;
                Ok(CryptoKey::CertThumbprint(digest_from_value_array(arr)?))
            }
            Value::Tag(561, inner) => {
                let arr = inner.into_array().ok_or_else(|| serde::de::Error::custom("tag 561 must wrap array"))?;
                Ok(CryptoKey::CertPathThumbprint(digest_from_value_array(arr)?))
            }
            Value::Tag(562, inner) => match *inner { Value::Bytes(b) => Ok(CryptoKey::PkixAsn1DerCert(b)), _ => Err(serde::de::Error::custom("tag 562 must wrap bytes")) },
            Value::Tag(560, inner) => match *inner { Value::Bytes(b) => Ok(CryptoKey::Bytes(b)), _ => Err(serde::de::Error::custom("tag 560 must wrap bytes")) },
            _ => Err(serde::de::Error::custom("expected a tagged crypto key")),
        }
    }
}

/// `linked-tag-map` — references another tag.
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct LinkedTagMap {
    /// `linked-tag-id` (key 0).
    #[cbor(key = 0)]
    pub linked_tag_id: TagIdChoice,
    /// `tag-rel` (key 1): supplements(0) or replaces(1).
    #[cbor(key = 1)]
    pub tag_rel: i64,
}

// ---------------------------------------------------------------------------
// Digest helper
// ---------------------------------------------------------------------------

/// Deserialize a `[alg, val]` array of [`Value`]s into a [`Digest`].
fn digest_from_value_array<E: serde::de::Error>(arr: Vec<Value>) -> Result<Digest, E> {
    if arr.len() != 2 {
        return Err(E::custom("digest must be [alg, val]"));
    }
    let mut it = arr.into_iter();
    let alg = match it.next().unwrap() {
        Value::Integer(n) => n as i64,
        _ => return Err(E::custom("digest alg must be int")),
    };
    let val = match it.next().unwrap() {
        Value::Bytes(b) => b,
        _ => return Err(E::custom("digest val must be bytes")),
    };
    Ok(Digest::new(alg, val))
}

// ---------------------------------------------------------------------------
// Role constants
// ---------------------------------------------------------------------------

/// `tag-creator` CoMID role (0).
pub const COMID_ROLE_TAG_CREATOR: i64 = 0;
/// `creator` CoMID role (1).
pub const COMID_ROLE_CREATOR: i64 = 1;
/// `maintainer` CoMID role (2).
pub const COMID_ROLE_MAINTAINER: i64 = 2;

/// `manifest-creator` CoRIM role (1).
pub const CORIM_ROLE_MANIFEST_CREATOR: i64 = 1;
/// `manifest-signer` CoRIM role (2).
pub const CORIM_ROLE_MANIFEST_SIGNER: i64 = 2;

/// `supplements` tag relation (0).
pub const TAG_REL_SUPPLEMENTS: i64 = 0;
/// `replaces` tag relation (1).
pub const TAG_REL_REPLACES: i64 = 1;
