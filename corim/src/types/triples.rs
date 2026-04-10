// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Triple types from `triples-map`.
//!
//! All triple record types are CBOR arrays (not maps), so they use standard
//! serde tuple serialization.

use corim_derive::{CborDeserialize, CborSerialize};
use serde::{Deserialize, Serialize};

use super::common::{CryptoKey, MeasuredElement, TagIdChoice};
use super::environment::EnvironmentMap;
use super::measurement::MeasurementMap;

// ---------------------------------------------------------------------------
// triples-map
// ---------------------------------------------------------------------------

/// `triples-map` — the core payload of a CoMID.
///
/// At least one triple type must be present (`non-empty`).
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
#[cbor(non_empty)]
pub struct TriplesMap {
    /// `reference-triples` (key 0).
    #[cbor(key = 0, optional)]
    pub reference_triples: Option<Vec<ReferenceTriple>>,
    /// `endorsed-triples` (key 1).
    #[cbor(key = 1, optional)]
    pub endorsed_triples: Option<Vec<EndorsedTriple>>,
    /// `identity-triples` (key 2).
    #[cbor(key = 2, optional)]
    pub identity_triples: Option<Vec<IdentityTriple>>,
    /// `attest-key-triples` (key 3).
    #[cbor(key = 3, optional)]
    pub attest_key_triples: Option<Vec<AttestKeyTriple>>,
    /// `dependency-triples` (key 4).
    #[cbor(key = 4, optional)]
    pub dependency_triples: Option<Vec<DomainDependencyTriple>>,
    /// `membership-triples` (key 5).
    #[cbor(key = 5, optional)]
    pub membership_triples: Option<Vec<DomainMembershipTriple>>,
    /// `coswid-triples` (key 6).
    #[cbor(key = 6, optional)]
    pub coswid_triples: Option<Vec<CoswidTriple>>,
    /// `conditional-endorsement-series-triples` (key 8).
    #[cbor(key = 8, optional)]
    pub conditional_endorsement_series: Option<Vec<ConditionalEndorsementSeriesTriple>>,
    /// `conditional-endorsement-triples` (key 10).
    #[cbor(key = 10, optional)]
    pub conditional_endorsement: Option<Vec<ConditionalEndorsementTriple>>,
}

// ---------------------------------------------------------------------------
// reference-triple-record = [ environment-map, [+ measurement-map] ]
// ---------------------------------------------------------------------------

/// `reference-triple-record` — reference values for a target environment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceTriple(pub EnvironmentMap, pub Vec<MeasurementMap>);

impl ReferenceTriple {
    /// Create a new reference triple.
    pub fn new(environment: EnvironmentMap, measurements: Vec<MeasurementMap>) -> Self {
        Self(environment, measurements)
    }
    /// Get the target environment.
    pub fn environment(&self) -> &EnvironmentMap { &self.0 }
    /// Get the reference measurements.
    pub fn measurements(&self) -> &[MeasurementMap] { &self.1 }
}

// ---------------------------------------------------------------------------
// endorsed-triple-record = [ environment-map, [+ measurement-map] ]
// ---------------------------------------------------------------------------

/// `endorsed-triple-record` — endorsed values for a target environment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EndorsedTriple(pub EnvironmentMap, pub Vec<MeasurementMap>);

impl EndorsedTriple {
    /// Create a new endorsed triple.
    pub fn new(condition: EnvironmentMap, endorsement: Vec<MeasurementMap>) -> Self {
        Self(condition, endorsement)
    }
    /// Get the condition environment.
    pub fn condition(&self) -> &EnvironmentMap { &self.0 }
    /// Get the endorsement measurements.
    pub fn endorsement(&self) -> &[MeasurementMap] { &self.1 }
}

// ---------------------------------------------------------------------------
// Key triple conditions (shared by identity and attest-key triples)
// ---------------------------------------------------------------------------

/// Conditions map for identity/attest-key triples.
///
/// CDDL: `non-empty<{ ?mkey: 0, ?authorized-by: 1 }>`
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
#[cbor(non_empty)]
pub struct KeyTripleConditions {
    /// `mkey` (key 0): optional measured element key.
    #[cbor(key = 0, optional)]
    pub mkey: Option<MeasuredElement>,
    /// `authorized-by` (key 1): optional authority keys.
    #[cbor(key = 1, optional)]
    pub authorized_by: Option<Vec<CryptoKey>>,
}

// ---------------------------------------------------------------------------
// identity-triple-record
// ---------------------------------------------------------------------------

/// `identity-triple-record` — device identity keys.
///
/// CDDL:
/// ```text
/// identity-triple-record = [
///   environment: environment-map,
///   key-list: [+ $crypto-key-type-choice],
///   ? conditions: non-empty<{ ?mkey, ?authorized-by }>,
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentityTriple(
    pub EnvironmentMap,
    pub Vec<CryptoKey>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub Option<KeyTripleConditions>,
);

impl IdentityTriple {
    /// Create a new identity triple.
    pub fn new(
        environment: EnvironmentMap,
        keys: Vec<CryptoKey>,
        conditions: Option<KeyTripleConditions>,
    ) -> Self {
        Self(environment, keys, conditions)
    }
    /// Get the environment.
    pub fn environment(&self) -> &EnvironmentMap { &self.0 }
    /// Get the key list.
    pub fn keys(&self) -> &[CryptoKey] { &self.1 }
    /// Get optional conditions.
    pub fn conditions(&self) -> Option<&KeyTripleConditions> { self.2.as_ref() }
}

// ---------------------------------------------------------------------------
// attest-key-triple-record
// ---------------------------------------------------------------------------

/// `attest-key-triple-record` — attestation key endorsement.
///
/// Same structure as identity-triple-record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttestKeyTriple(
    pub EnvironmentMap,
    pub Vec<CryptoKey>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub Option<KeyTripleConditions>,
);

impl AttestKeyTriple {
    /// Create a new attest-key triple.
    pub fn new(
        environment: EnvironmentMap,
        keys: Vec<CryptoKey>,
        conditions: Option<KeyTripleConditions>,
    ) -> Self {
        Self(environment, keys, conditions)
    }
    /// Get the environment.
    pub fn environment(&self) -> &EnvironmentMap { &self.0 }
    /// Get the key list.
    pub fn keys(&self) -> &[CryptoKey] { &self.1 }
    /// Get optional conditions.
    pub fn conditions(&self) -> Option<&KeyTripleConditions> { self.2.as_ref() }
}

// ---------------------------------------------------------------------------
// domain-dependency-triple-record
// ---------------------------------------------------------------------------

/// `domain-dependency-triple-record` — trust dependencies between domains.
///
/// CDDL: `[domain-id: domain-type, trustees: [+ domain-type]]`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DomainDependencyTriple(pub EnvironmentMap, pub Vec<EnvironmentMap>);

impl DomainDependencyTriple {
    /// Create a new domain dependency triple.
    pub fn new(domain_id: EnvironmentMap, trustees: Vec<EnvironmentMap>) -> Self {
        Self(domain_id, trustees)
    }
    /// Get the domain identifier.
    pub fn domain_id(&self) -> &EnvironmentMap { &self.0 }
    /// Get the trustee domains.
    pub fn trustees(&self) -> &[EnvironmentMap] { &self.1 }
}

// ---------------------------------------------------------------------------
// domain-membership-triple-record
// ---------------------------------------------------------------------------

/// `domain-membership-triple-record` — domain composition.
///
/// CDDL: `[domain-id: domain-type, members: [+ domain-type]]`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DomainMembershipTriple(pub EnvironmentMap, pub Vec<EnvironmentMap>);

impl DomainMembershipTriple {
    /// Create a new domain membership triple.
    pub fn new(domain_id: EnvironmentMap, members: Vec<EnvironmentMap>) -> Self {
        Self(domain_id, members)
    }
    /// Get the domain identifier.
    pub fn domain_id(&self) -> &EnvironmentMap { &self.0 }
    /// Get the member environments.
    pub fn members(&self) -> &[EnvironmentMap] { &self.1 }
}

// ---------------------------------------------------------------------------
// coswid-triple-record = [ environment-map, [+ coswid.tag-id] ]
// ---------------------------------------------------------------------------

/// `coswid-triple-record` — links an environment to CoSWID tags.
///
/// The tag-ids are `text / bstr .size 16` (string or UUID).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoswidTriple(pub EnvironmentMap, pub Vec<TagIdChoice>);

impl CoswidTriple {
    /// Create a new CoSWID triple.
    pub fn new(environment: EnvironmentMap, tag_ids: Vec<TagIdChoice>) -> Self {
        Self(environment, tag_ids)
    }
    /// Get the environment.
    pub fn environment(&self) -> &EnvironmentMap { &self.0 }
    /// Get the CoSWID tag identifiers.
    pub fn tag_ids(&self) -> &[TagIdChoice] { &self.1 }
}

// ---------------------------------------------------------------------------
// conditional-endorsement-series
// ---------------------------------------------------------------------------

/// Condition block for conditional-endorsement-series triples.
///
/// CDDL:
/// ```text
/// condition: [
///   environment: environment-map,
///   claims-list: [* measurement-map],
///   ? authorized-by: [+ $crypto-key-type-choice],
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CesCondition {
    /// The target environment.
    pub environment: EnvironmentMap,
    /// Measurement conditions (may be empty).
    pub claims_list: Vec<MeasurementMap>,
    /// Optional authority condition.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub authorized_by: Option<Vec<CryptoKey>>,
}

/// `conditional-endorsement-series-triple-record`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionalEndorsementSeriesTriple(pub CesCondition, pub Vec<ConditionalSeriesRecord>);

impl ConditionalEndorsementSeriesTriple {
    /// Create a new CES triple.
    pub fn new(condition: CesCondition, series: Vec<ConditionalSeriesRecord>) -> Self {
        Self(condition, series)
    }
    /// Get the condition.
    pub fn condition(&self) -> &CesCondition { &self.0 }
    /// Get the series records.
    pub fn series(&self) -> &[ConditionalSeriesRecord] { &self.1 }
}

/// `conditional-series-record = [selection: [+ measurement-map], addition: [+ measurement-map]]`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionalSeriesRecord(pub Vec<MeasurementMap>, pub Vec<MeasurementMap>);

impl ConditionalSeriesRecord {
    /// Create a new conditional series record.
    pub fn new(selection: Vec<MeasurementMap>, addition: Vec<MeasurementMap>) -> Self {
        Self(selection, addition)
    }
    /// Get the selection criteria.
    pub fn selection(&self) -> &[MeasurementMap] { &self.0 }
    /// Get the addition values.
    pub fn addition(&self) -> &[MeasurementMap] { &self.1 }
}

// ---------------------------------------------------------------------------
// conditional-endorsement-triple-record
// ---------------------------------------------------------------------------

/// `stateful-environment-record = [environment-map, [+ measurement-map]]`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatefulEnvironmentRecord(pub EnvironmentMap, pub Vec<MeasurementMap>);

/// `conditional-endorsement-triple-record`.
///
/// CDDL:
/// ```text
/// conditional-endorsement-triple-record = [
///   conditions: [+ stateful-environment-record],
///   endorsements: [+ endorsed-triple-record],
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionalEndorsementTriple(
    pub Vec<StatefulEnvironmentRecord>,
    pub Vec<EndorsedTriple>,
);
