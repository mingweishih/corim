// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! CDDL-derived Rust types for CoRIM / CoMID.
//!
//! The type hierarchy mirrors the CDDL productions in `corim.cddl`.
//! Map types use `CborSerialize`/`CborDeserialize` derives for integer-keyed
//! CBOR encoding. Array types (triple records) use standard serde derives.

pub mod common;
pub mod comid;
pub mod corim;
pub mod environment;
pub mod measurement;
pub mod tags;
pub mod triples;

pub use self::common::*;
pub use self::comid::*;
pub use self::corim::*;
pub use self::environment::*;
pub use self::measurement::*;
pub use self::tags::*;
pub use self::triples::*;
