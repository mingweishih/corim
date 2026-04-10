// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Validation and appraisal logic per draft-ietf-rats-corim-10 §9.
//!
//! Covers reference value matching (Phase 3) and conditional-endorsement-series
//! application (Phase 4).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::cbor;
use crate::error::ValidationError;
use crate::types::comid::ComidTag;
use crate::types::corim::{ConciseTagChoice, CorimMap};
use crate::types::environment::EnvironmentMap;
use crate::types::measurement::{Digest, MeasurementMap, SvnChoice};
use crate::types::triples::{
    ConditionalEndorsementSeriesTriple, ConditionalSeriesRecord, ReferenceTriple,
};

// ---------------------------------------------------------------------------
// Phase 1: Input validation (§9.2)
// ---------------------------------------------------------------------------

/// Decode CBOR bytes as a CoRIM and validate structural requirements.
///
/// Expects the bytes to be a CBOR tag-501-wrapped `corim-map`. Validates:
/// - `rim-validity` is not expired (if present)
/// - Each CoMID tag decodes correctly and has non-empty triples
pub fn decode_and_validate(bytes: &[u8]) -> Result<(CorimMap, Vec<ComidTag>), ValidationError> {
    // Decode the tag-501 wrapped CoRIM
    let tagged: cbor::value::Tagged<CorimMap> =
        cbor::decode(bytes).map_err(ValidationError::Decode)?;
    if tagged.tag != 501 {
        return Err(ValidationError::Decode(
            crate::error::DecodeError::UnexpectedTag {
                expected: 501,
                found: tagged.tag,
            },
        ));
    }
    let corim = tagged.value;

    // Check rim-validity
    if let Some(ref validity) = corim.rim_validity {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if validity.not_after < now {
            return Err(ValidationError::Expired);
        }
    }

    // Extract and validate each CoMID
    let mut comids = Vec::new();
    for tag in &corim.tags {
        match tag {
            ConciseTagChoice::Comid(comid_bytes) => {
                let comid: ComidTag =
                    cbor::decode(comid_bytes).map_err(ValidationError::Decode)?;
                validate_comid(&comid)?;
                comids.push(comid);
            }
            // Skip non-CoMID tags (CoSWID, CoTL, Unknown)
            _ => {}
        }
    }

    if comids.is_empty() {
        return Err(ValidationError::EmptyTriples);
    }

    Ok((corim, comids))
}

/// Validate a single CoMID tag.
fn validate_comid(comid: &ComidTag) -> Result<(), ValidationError> {
    let t = &comid.triples;
    let has_triples = t.reference_triples.is_some()
        || t.endorsed_triples.is_some()
        || t.identity_triples.is_some()
        || t.attest_key_triples.is_some()
        || t.dependency_triples.is_some()
        || t.membership_triples.is_some()
        || t.coswid_triples.is_some()
        || t.conditional_endorsement_series.is_some()
        || t.conditional_endorsement.is_some();

    if !has_triples {
        return Err(ValidationError::EmptyTriples);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 3: Reference value matching (§9.3.3)
// ---------------------------------------------------------------------------

/// A claim that has been corroborated by reference value matching.
#[derive(Clone, Debug)]
pub struct CorroboratedClaim {
    /// The environment that was matched.
    pub environment: EnvironmentMap,
    /// The measurement(s) that matched.
    pub measurements: Vec<MeasurementMap>,
}

/// Match reference values against evidence digests.
///
/// For each `ReferenceTriple`, compares the environment and digests per
/// §9.4.2 (environment) and §9.4.6.1.3 (digests):
/// - Absent condition field = wildcard
/// - All common algorithms must agree
pub fn match_reference_values(
    ref_triples: &[ReferenceTriple],
    evidence: &[EvidenceClaim],
) -> Vec<CorroboratedClaim> {
    let mut corroborated = Vec::new();

    for triple in ref_triples {
        for ev in evidence {
            if !environment_matches(triple.environment(), &ev.environment) {
                continue;
            }

            let mut matched_measurements = Vec::new();
            for ref_meas in triple.measurements() {
                if measurement_matches(ref_meas, &ev.measurements) {
                    matched_measurements.push(ref_meas.clone());
                }
            }

            if !matched_measurements.is_empty() {
                corroborated.push(CorroboratedClaim {
                    environment: triple.environment().clone(),
                    measurements: matched_measurements,
                });
            }
        }
    }

    corroborated
}

/// An evidence claim (from the attestation report).
#[derive(Clone, Debug)]
pub struct EvidenceClaim {
    /// The environment this evidence belongs to.
    pub environment: EnvironmentMap,
    /// The measurements reported in evidence.
    pub measurements: Vec<MeasurementMap>,
}

// ---------------------------------------------------------------------------
// Phase 4: Conditional endorsement series (§9.3.4.3)
// ---------------------------------------------------------------------------

/// An endorsed claim produced by conditional endorsement series matching.
#[derive(Clone, Debug)]
pub struct EndorsedClaim {
    /// The environment that was matched.
    pub environment: EnvironmentMap,
    /// The endorsement values that were applied.
    pub endorsements: Vec<MeasurementMap>,
}

/// Apply conditional-endorsement-series triples to produce endorsed claims.
///
/// Per §9.3.4.3:
/// 1. Match condition environment against provided evidence
/// 2. Iterate series in order — first `selection` match wins
/// 3. Apply the corresponding `addition` as endorsed values
pub fn apply_endorsement_series(
    ces_triples: &[ConditionalEndorsementSeriesTriple],
    evidence: &[EvidenceClaim],
) -> Result<Vec<EndorsedClaim>, ValidationError> {
    let mut endorsed = Vec::new();

    for triple in ces_triples {
        let condition = triple.condition();

        let matching_evidence: Vec<_> = evidence
            .iter()
            .filter(|ev| environment_matches(&condition.environment, &ev.environment))
            .collect();

        if matching_evidence.is_empty() {
            continue;
        }

        validate_series_mkeys(triple.series())?;

        for ev in &matching_evidence {
            if let Some(addition) = find_matching_series(triple.series(), &ev.measurements) {
                endorsed.push(EndorsedClaim {
                    environment: condition.environment.clone(),
                    endorsements: addition,
                });
            }
        }
    }

    Ok(endorsed)
}

/// Validate that all series entries use the same `mkey`s (§5.1.8.1.1).
fn validate_series_mkeys(series: &[ConditionalSeriesRecord]) -> Result<(), ValidationError> {
    if series.len() <= 1 {
        return Ok(());
    }

    let first_mkeys: Vec<_> = series[0].selection().iter().map(|m| &m.mkey).collect();

    for record in &series[1..] {
        let mkeys: Vec<_> = record.selection().iter().map(|m| &m.mkey).collect();
        if mkeys != first_mkeys {
            return Err(ValidationError::InconsistentMkeys);
        }
    }

    Ok(())
}

/// Find the first matching series entry and return its addition.
fn find_matching_series(
    series: &[ConditionalSeriesRecord],
    evidence_measurements: &[MeasurementMap],
) -> Option<Vec<MeasurementMap>> {
    for record in series {
        let all_match = record
            .selection()
            .iter()
            .all(|sel| measurement_matches(sel, evidence_measurements));

        if all_match {
            return Some(record.addition().to_vec());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// SVN comparison (§9.4.6.1.2)
// ---------------------------------------------------------------------------

/// Compare an SVN value against evidence.
///
/// - `ExactValue(n)`: evidence SVN must equal `n`
/// - `MinValue(n)`: evidence SVN must be `>= n`
pub fn svn_matches(reference: &SvnChoice, evidence_svn: u64) -> bool {
    match reference {
        SvnChoice::ExactValue(n) => evidence_svn == *n,
        SvnChoice::MinValue(n) => evidence_svn >= *n,
    }
}

// ---------------------------------------------------------------------------
// Comparison helpers (§9.4)
// ---------------------------------------------------------------------------

/// Compare two environments per §9.4.2.
///
/// Absent fields in the condition are wildcards.
fn environment_matches(condition: &EnvironmentMap, target: &EnvironmentMap) -> bool {
    if let Some(ref cond_class) = condition.class {
        match &target.class {
            None => return false,
            Some(tgt_class) => {
                if !class_matches(cond_class, tgt_class) {
                    return false;
                }
            }
        }
    }

    if condition.instance.is_some() && condition.instance != target.instance {
        return false;
    }

    if condition.group.is_some() && condition.group != target.group {
        return false;
    }

    true
}

/// Compare two class-maps. Absent condition fields are wildcards.
fn class_matches(
    condition: &crate::types::environment::ClassMap,
    target: &crate::types::environment::ClassMap,
) -> bool {
    if condition.class_id.is_some() && condition.class_id != target.class_id {
        return false;
    }
    if condition.vendor.is_some() && condition.vendor != target.vendor {
        return false;
    }
    if condition.model.is_some() && condition.model != target.model {
        return false;
    }
    if condition.layer.is_some() && condition.layer != target.layer {
        return false;
    }
    if condition.index.is_some() && condition.index != target.index {
        return false;
    }
    true
}

/// Check if a reference measurement matches any evidence measurement.
fn measurement_matches(reference: &MeasurementMap, evidence: &[MeasurementMap]) -> bool {
    for ev_meas in evidence {
        if let Some(ref ref_mkey) = reference.mkey {
            match &ev_meas.mkey {
                Some(ev_mkey) if ev_mkey == ref_mkey => {}
                _ => continue,
            }
        }

        if let Some(ref ref_digests) = reference.mval.digests {
            if let Some(ref ev_digests) = ev_meas.mval.digests {
                if digests_match(ref_digests, ev_digests) {
                    return true;
                }
            }
            continue;
        }

        return true;
    }
    false
}

/// Compare digest lists per §9.4.6.1.3.
fn digests_match(reference: &[Digest], evidence: &[Digest]) -> bool {
    let mut has_common = false;

    for ref_d in reference {
        for ev_d in evidence {
            if ref_d.alg() == ev_d.alg() {
                has_common = true;
                if ref_d.value() != ev_d.value() {
                    return false;
                }
            }
        }
    }

    has_common
}

// ---------------------------------------------------------------------------
// Appraisal context
// ---------------------------------------------------------------------------

/// The type of a claim in the Appraisal Context Set.
#[derive(Clone, Debug, PartialEq)]
pub enum ClaimType {
    /// Claim from attestation evidence.
    Evidence,
    /// Claim corroborated by reference values (Phase 3).
    ReferenceValues,
    /// Claim endorsed by endorsement triples (Phase 4).
    Endorsement,
}

/// An entry in the Appraisal Context Set (ACS).
#[derive(Clone, Debug)]
pub struct EnvironmentClaimTuple {
    /// The environment this claim is about.
    pub environment: EnvironmentMap,
    /// The measurements/endorsements.
    pub measurements: Vec<MeasurementMap>,
    /// The claim type.
    pub claim_type: ClaimType,
}

/// The Appraisal Context Set — accumulates claims across appraisal phases.
#[derive(Clone, Debug, Default)]
pub struct AppraisalContext {
    /// All claim entries.
    pub entries: Vec<EnvironmentClaimTuple>,
}

impl AppraisalContext {
    /// Create a new empty appraisal context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize with evidence claims (Phase 2).
    pub fn add_evidence(&mut self, claims: Vec<EvidenceClaim>) {
        for claim in claims {
            self.entries.push(EnvironmentClaimTuple {
                environment: claim.environment,
                measurements: claim.measurements,
                claim_type: ClaimType::Evidence,
            });
        }
    }

    /// Apply reference value matching (Phase 3).
    pub fn apply_reference_values(
        &mut self,
        ref_triples: &[ReferenceTriple],
    ) -> Vec<CorroboratedClaim> {
        let evidence: Vec<EvidenceClaim> = self
            .entries
            .iter()
            .filter(|e| e.claim_type == ClaimType::Evidence)
            .map(|e| EvidenceClaim {
                environment: e.environment.clone(),
                measurements: e.measurements.clone(),
            })
            .collect();

        let corroborated = match_reference_values(ref_triples, &evidence);

        for claim in &corroborated {
            self.entries.push(EnvironmentClaimTuple {
                environment: claim.environment.clone(),
                measurements: claim.measurements.clone(),
                claim_type: ClaimType::ReferenceValues,
            });
        }

        corroborated
    }

    /// Apply conditional endorsement series (Phase 4).
    pub fn apply_conditional_endorsements(
        &mut self,
        ces_triples: &[ConditionalEndorsementSeriesTriple],
    ) -> Result<Vec<EndorsedClaim>, ValidationError> {
        let evidence: Vec<EvidenceClaim> = self
            .entries
            .iter()
            .map(|e| EvidenceClaim {
                environment: e.environment.clone(),
                measurements: e.measurements.clone(),
            })
            .collect();

        let endorsed = apply_endorsement_series(ces_triples, &evidence)?;

        for claim in &endorsed {
            self.entries.push(EnvironmentClaimTuple {
                environment: claim.environment.clone(),
                measurements: claim.endorsements.clone(),
                claim_type: ClaimType::Endorsement,
            });
        }

        Ok(endorsed)
    }
}
