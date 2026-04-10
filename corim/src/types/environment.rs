// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! `environment-map` and `class-map` types.

use corim_derive::{CborDeserialize, CborSerialize};

use super::common::{ClassIdChoice, GroupIdChoice, InstanceIdChoice};

// ---------------------------------------------------------------------------
// class-map  { class-id: 0, vendor: 1, model: 2, layer: 3, index: 4 }
// ---------------------------------------------------------------------------

/// `class-map` — identifies a component class.
///
/// CDDL: `non-empty<{ ?0 => class-id, ?1 => vendor, ?2 => model, ?3 => layer, ?4 => index }>`
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
#[cbor(non_empty)]
pub struct ClassMap {
    /// `class-id` (key 0): optional platform-specific identifier.
    #[cbor(key = 0, optional)]
    pub class_id: Option<ClassIdChoice>,

    /// `vendor` (key 1): e.g. "Intel", "AMD", "Microsoft".
    #[cbor(key = 1, optional)]
    pub vendor: Option<String>,

    /// `model` (key 2): e.g. "TDX", "SEV-SNP", "VBS-CVM".
    #[cbor(key = 2, optional)]
    pub model: Option<String>,

    /// `layer` (key 3): optional layer number.
    #[cbor(key = 3, optional)]
    pub layer: Option<u64>,

    /// `index` (key 4): optional index number.
    #[cbor(key = 4, optional)]
    pub index: Option<u64>,
}

// ---------------------------------------------------------------------------
// environment-map  { class: 0, instance: 1, group: 2 }
// ---------------------------------------------------------------------------

/// `environment-map` — identifies a Target Environment.
///
/// CDDL: `non-empty<{ ?0 => class-map, ?1 => instance-id, ?2 => group-id }>`
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
#[cbor(non_empty)]
pub struct EnvironmentMap {
    /// `class` (key 0): component class.
    #[cbor(key = 0, optional)]
    pub class: Option<ClassMap>,

    /// `instance` (key 1): specific instance identifier.
    #[cbor(key = 1, optional)]
    pub instance: Option<InstanceIdChoice>,

    /// `group` (key 2): group identifier.
    #[cbor(key = 2, optional)]
    pub group: Option<GroupIdChoice>,
}
