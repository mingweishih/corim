// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! `concise-mid-tag` (CoMID) type.

use corim_derive::{CborDeserialize, CborSerialize};

use super::common::{EntityMap, LinkedTagMap, TagIdentity};
use super::triples::TriplesMap;

// ---------------------------------------------------------------------------
// concise-mid-tag  { language: 0, tag-identity: 1, entities: 2,
//                    linked-tags: 3, triples: 4 }
// ---------------------------------------------------------------------------

/// `concise-mid-tag` — a CoMID tag containing triples.
#[derive(Clone, Debug, PartialEq, CborSerialize, CborDeserialize)]
pub struct ComidTag {
    /// `language` (key 0): optional BCP 47 language tag.
    #[cbor(key = 0, optional)]
    pub language: Option<String>,

    /// `tag-identity` (key 1): identifies this CoMID.
    #[cbor(key = 1)]
    pub tag_identity: TagIdentity,

    /// `entities` (key 2): optional list of entities.
    #[cbor(key = 2, optional)]
    pub entities: Option<Vec<EntityMap>>,

    /// `linked-tags` (key 3): optional references to other tags.
    #[cbor(key = 3, optional)]
    pub linked_tags: Option<Vec<LinkedTagMap>>,

    /// `triples` (key 4): the measurement triples.
    #[cbor(key = 4)]
    pub triples: TriplesMap,
}
