// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Builder API for CoRIM and CoMID generation.
//!
//! Provides a fluent interface for constructing platform-agnostic launch
//! endorsements per draft-ietf-rats-corim-10.

use uuid::Uuid;

use crate::cbor;
use crate::error::BuilderError;
use crate::types::comid::ComidTag;
use crate::types::common::{EntityMap, TagIdChoice, TagIdentity, ValidityMap};
use crate::types::corim::{ConciseTagChoice, CorimId, CorimMap, ProfileChoice};
use crate::types::environment::{ClassMap, EnvironmentMap};
use crate::types::measurement::{Digest, MeasurementMap, MeasurementValuesMap, SvnChoice};
use crate::types::triples::{
    CesCondition, ConditionalEndorsementSeriesTriple, ConditionalSeriesRecord, ReferenceTriple,
    TriplesMap,
};

/// Fixed namespace UUID for UUIDv5 tag-id derivation.
const TAG_ID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x85, 0xf3, 0xf1, 0xc2, 0x22, 0xa8, 0x44, 0x1e, 0xa1, 0xb9, 0xbc, 0xcf, 0xb6, 0x3e, 0xd5,
    0xf7,
]);

/// Derive a deterministic tag-id UUID from vendor and model strings.
///
/// Uses UUIDv5 with a fixed namespace and input `"{vendor}/{model}"`.
pub fn derive_tag_id(vendor: &str, model: &str) -> Uuid {
    let input = format!("{}/{}", vendor, model);
    Uuid::new_v5(&TAG_ID_NAMESPACE, input.as_bytes())
}

// ---------------------------------------------------------------------------
// ComidBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing a `ComidTag` (CoMID).
pub struct ComidBuilder {
    vendor: String,
    model: String,
    tag_version: Option<u64>,
    language: Option<String>,
    entities: Option<Vec<EntityMap>>,
    reference_measurements: Vec<MeasurementMap>,
    ces_entries: Vec<ConditionalSeriesRecord>,
}

impl ComidBuilder {
    /// Create a builder for the given platform (vendor + model).
    ///
    /// The `tag-id` is auto-derived via UUIDv5.
    pub fn for_platform(vendor: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            vendor: vendor.into(),
            model: model.into(),
            tag_version: None,
            language: None,
            entities: None,
            reference_measurements: Vec::new(),
            ces_entries: Vec::new(),
        }
    }

    /// Set the tag version (for evolution workflow). Defaults to 0 if not set.
    pub fn set_tag_version(mut self, version: u64) -> Self {
        self.tag_version = Some(version);
        self
    }

    /// Set the optional language tag (BCP 47).
    pub fn set_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Add entities to the CoMID.
    pub fn add_entity(mut self, entity: EntityMap) -> Self {
        self.entities.get_or_insert_with(Vec::new).push(entity);
        self
    }

    /// Add a reference value (digest) for the given measurement key.
    pub fn add_reference_value(
        mut self,
        mkey: impl Into<String>,
        digests: Vec<Digest>,
    ) -> Self {
        let meas = MeasurementMap {
            mkey: Some(crate::types::common::MeasuredElement::Text(mkey.into())),
            mval: MeasurementValuesMap {
                digests: Some(digests),
                ..MeasurementValuesMap::default()
            },
            authorized_by: None,
        };
        self.reference_measurements.push(meas);
        self
    }

    /// Add a conditional-endorsement-series entry mapping a digest to an SVN.
    pub fn add_conditional_endorsement_series(
        mut self,
        mkey: impl Into<String>,
        digest: Digest,
        svn: SvnChoice,
    ) -> Self {
        let mkey_str = mkey.into();

        let selection_meas = MeasurementMap {
            mkey: Some(crate::types::common::MeasuredElement::Text(mkey_str.clone())),
            mval: MeasurementValuesMap {
                digests: Some(vec![digest]),
                ..MeasurementValuesMap::default()
            },
            authorized_by: None,
        };

        let addition_meas = MeasurementMap {
            mkey: None,
            mval: MeasurementValuesMap {
                svn: Some(svn),
                ..MeasurementValuesMap::default()
            },
            authorized_by: None,
        };

        self.ces_entries.push(ConditionalSeriesRecord::new(
            vec![selection_meas],
            vec![addition_meas],
        ));
        self
    }

    /// Build the `ComidTag`.
    pub fn build(self) -> Result<ComidTag, BuilderError> {
        // Derive tag-id
        let tag_uuid = derive_tag_id(&self.vendor, &self.model);
        let tag_id = TagIdChoice::Text(tag_uuid.to_string().to_uppercase());

        let tag_identity = TagIdentity {
            tag_id,
            tag_version: self.tag_version,
        };

        // Build environment
        let environment = EnvironmentMap {
            class: Some(ClassMap {
                class_id: None,
                vendor: Some(self.vendor.clone()),
                model: Some(self.model.clone()),
                layer: None,
                index: None,
            }),
            instance: None,
            group: None,
        };

        // Build triples
        let reference_triples = if self.reference_measurements.is_empty() {
            None
        } else {
            Some(vec![ReferenceTriple::new(
                environment.clone(),
                self.reference_measurements,
            )])
        };

        let conditional_endorsement_series = if self.ces_entries.is_empty() {
            None
        } else {
            let condition = CesCondition {
                environment: environment.clone(),
                claims_list: Vec::new(),
                authorized_by: None,
            };
            Some(vec![ConditionalEndorsementSeriesTriple::new(
                condition,
                self.ces_entries,
            )])
        };

        // Must have at least one triple type
        if reference_triples.is_none() && conditional_endorsement_series.is_none() {
            return Err(BuilderError::EmptyTriples);
        }

        let triples = TriplesMap {
            reference_triples,
            endorsed_triples: None,
            identity_triples: None,
            attest_key_triples: None,
            dependency_triples: None,
            membership_triples: None,
            coswid_triples: None,
            conditional_endorsement_series,
            conditional_endorsement: None,
        };

        Ok(ComidTag {
            language: self.language,
            tag_identity,
            entities: self.entities,
            linked_tags: None,
            triples,
        })
    }
}

// ---------------------------------------------------------------------------
// CorimBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing a `CorimMap` (top-level CoRIM).
pub struct CorimBuilder {
    id: CorimId,
    profile: Option<ProfileChoice>,
    rim_validity: Option<ValidityMap>,
    entities: Option<Vec<EntityMap>>,
    comid_tags: Vec<ComidTag>,
}

impl CorimBuilder {
    /// Create a new CoRIM builder with the given identifier.
    pub fn new(id: CorimId) -> Self {
        Self {
            id,
            profile: None,
            rim_validity: None,
            entities: None,
            comid_tags: Vec::new(),
        }
    }

    /// Set the optional profile.
    pub fn set_profile(mut self, profile: ProfileChoice) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Set the optional validity window.
    pub fn set_validity(mut self, not_before: Option<i64>, not_after: i64) -> Self {
        self.rim_validity = Some(ValidityMap {
            not_before,
            not_after,
        });
        self
    }

    /// Add an entity.
    pub fn add_entity(mut self, entity: EntityMap) -> Self {
        self.entities.get_or_insert_with(Vec::new).push(entity);
        self
    }

    /// Add a CoMID tag (built from a `ComidBuilder`).
    pub fn add_comid(mut self, comid_builder: ComidBuilder) -> Result<Self, BuilderError> {
        let comid = comid_builder.build()?;
        self.comid_tags.push(comid);
        Ok(self)
    }

    /// Add a pre-built CoMID tag.
    pub fn add_comid_tag(mut self, comid: ComidTag) -> Self {
        self.comid_tags.push(comid);
        self
    }

    /// Build the `CorimMap`.
    pub fn build(self) -> Result<CorimMap, BuilderError> {
        if self.comid_tags.is_empty() {
            return Err(BuilderError::NoTags);
        }

        // Encode each CoMID as CBOR bytes and wrap as ConciseTagChoice::Comid
        let mut tags = Vec::with_capacity(self.comid_tags.len());
        for comid in &self.comid_tags {
            let comid_bytes = cbor::encode(comid)?;
            tags.push(ConciseTagChoice::Comid(comid_bytes));
        }

        Ok(CorimMap {
            id: self.id,
            tags,
            dependent_rims: None,
            profile: self.profile,
            rim_validity: self.rim_validity,
            entities: self.entities,
        })
    }

    /// Build and encode as deterministic CBOR bytes with tag 501 wrapper.
    pub fn build_bytes(self) -> Result<Vec<u8>, BuilderError> {
        let corim = self.build()?;

        // Wrap in CBOR tag 501
        let tagged = cbor::value::Tagged::new(501, corim);
        let bytes = cbor::encode(&tagged)?;
        Ok(bytes)
    }
}
