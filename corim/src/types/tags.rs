// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! CBOR tag number constants for CoRIM/CoMID types.
//!
//! These correspond to the `tagged-*` productions in the CDDL schema.

/// `tagged-unsigned-corim-map` = `#6.501(unsigned-corim-map)`.
pub const CORIM_TAG: u64 = 501;

/// `tagged-concise-mid-tag` = `#6.506(bytes .cbor concise-mid-tag)`.
pub const COMID_TAG: u64 = 506;

/// `tagged-concise-swid-tag` = `#6.505(...)`.
pub const COSWID_TAG: u64 = 505;

/// `tagged-concise-tl-tag` = `#6.508(...)`.
pub const COTL_TAG: u64 = 508;

/// `signed-corim` = `#6.18(COSE-Sign1-corim)`.
pub const SIGNED_CORIM_TAG: u64 = 18;

/// `tagged-uuid-type` = `#6.37(bytes .size 16)`.
pub const UUID_TAG: u64 = 37;

/// `tagged-oid-type` = `#6.111(bytes)`.
pub const OID_TAG: u64 = 111;

/// `tagged-ueid-type` = `#6.550(bytes .size (7..33))`.
pub const UEID_TAG: u64 = 550;

/// `tagged-svn` = `#6.552(uint)` — exact SVN.
pub const SVN_TAG: u64 = 552;

/// `tagged-min-svn` = `#6.553(uint)` — minimum SVN.
pub const MIN_SVN_TAG: u64 = 553;

/// `tagged-bytes` = `#6.560(bytes)`.
pub const TAGGED_BYTES_TAG: u64 = 560;

/// `tagged-masked-raw-value` = `#6.563([value, mask])`.
pub const MASKED_RAW_VALUE_TAG: u64 = 563;

/// `tagged-int-range` = `#6.564(int-range)`.
pub const INT_RANGE_TAG: u64 = 564;

// Crypto key tags

/// `tagged-pkix-base64-key-type` = `#6.554(tstr)`.
pub const PKIX_BASE64_KEY_TAG: u64 = 554;
/// `tagged-pkix-base64-cert-type` = `#6.555(tstr)`.
pub const PKIX_BASE64_CERT_TAG: u64 = 555;
/// `tagged-pkix-base64-cert-path-type` = `#6.556(tstr)`.
pub const PKIX_BASE64_CERT_PATH_TAG: u64 = 556;
/// `tagged-key-thumbprint-type` = `#6.557(eatmc.digest)`.
pub const KEY_THUMBPRINT_TAG: u64 = 557;
/// `tagged-cose-key-type` = `#6.558(COSE_Key)`.
pub const COSE_KEY_TAG: u64 = 558;
/// `tagged-cert-thumbprint-type` = `#6.559(eatmc.digest)`.
pub const CERT_THUMBPRINT_TAG: u64 = 559;
/// `tagged-cert-path-thumbprint-type` = `#6.561(eatmc.digest)`.
pub const CERT_PATH_THUMBPRINT_TAG: u64 = 561;
/// `tagged-pkix-asn1der-cert-type` = `#6.562(bstr)`.
pub const PKIX_ASN1_DER_CERT_TAG: u64 = 562;
