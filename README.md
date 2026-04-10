# corim

**Concise Reference Integrity Manifest (CoRIM)** — Rust implementation of
[draft-ietf-rats-corim-10](https://www.ietf.org/archive/id/draft-ietf-rats-corim-10.html).

This crate provides CBOR-native Rust types for the CoRIM / CoMID CDDL schema,
a builder API, and validation/appraisal logic for Remote Attestation (RATS)
Endorsements and Reference Values.

## Features

- **Full CDDL coverage** — types for `corim-map`, `concise-mid-tag` (CoMID),
  `concise-tl-tag` (CoTL), all triple types (reference, endorsed, identity,
  attest-key, domain dependency/membership, CoSWID, conditional endorsement,
  conditional endorsement series), `measurement-values-map` with all fields
  (digests, SVN, flags, raw-value, MAC/IP addresses, integrity registers,
  int-range, crypto keys, etc.).

- **Integer-keyed CBOR maps** — derive macros (`CborSerialize` /
  `CborDeserialize`) emit deterministic CBOR with integer keys per RFC 8949
  §4.2.1.

- **Swappable CBOR backend** — the `cbor` module defines a `CborCodec` trait.
  The default implementation uses [ciborium](https://docs.rs/ciborium) behind
  the `cbor-ciborium` feature gate. To plug in a different backend, disable
  the default feature and implement the trait.

- **Builder API** — fluent `ComidBuilder` and `CorimBuilder` for constructing
  tagged CoRIM payloads with auto-derived UUIDv5 tag identifiers.

- **Validation & Appraisal** — reference value matching (Phase 3) and
  conditional endorsement series application (Phase 4) per §9 of the spec.

## Quick start

```rust
use corim::builder::{ComidBuilder, CorimBuilder};
use corim::types::corim::CorimId;
use corim::types::measurement::{Digest, SvnChoice};

// Build a CoMID with reference values
let comid = ComidBuilder::for_platform("AMD", "SEV-SNP")
    .add_reference_value("MEASUREMENT", vec![Digest::new(7, vec![0xAA; 48])])
    .add_conditional_endorsement_series(
        "MEASUREMENT",
        Digest::new(7, vec![0xAA; 48]),
        SvnChoice::ExactValue(1),
    );

// Wrap in a CoRIM and encode to CBOR
let bytes = CorimBuilder::new(CorimId::Text("my-corim".into()))
    .add_comid(comid).unwrap()
    .build_bytes().unwrap();

// Decode and validate
let (corim, comids) = corim::validate::decode_and_validate(&bytes).unwrap();
```

## Crate structure

| Crate | Description |
|-------|-------------|
| `corim` | Main library — types, builder, validation, CBOR abstraction |
| `corim_derive` | Proc-macro derives for integer-keyed CBOR map serde |

## Feature flags

| Flag | Default | Description |
|------|---------|-------------|
| `cbor-ciborium` | ✅ | Use ciborium as the CBOR codec |

## Contributing

This project welcomes contributions and suggestions. Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit [Contributor License Agreements](https://cla.opensource.microsoft.com).

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.

## License

[MIT](LICENSE)
