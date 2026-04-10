# Microsoft Open Source Compliance Checklist

Tracking compliance with the [Microsoft Open Source release requirements](https://github.com/microsoft/repo-templates)
using the standard **"mit"** template projection.

## Required Files

- [x] **LICENSE** — MIT License, `Copyright (c) Microsoft Corporation.`
- [x] **SECURITY.md** — Standard Microsoft security reporting policy (`SECURITY.MD V1.0.0 BLOCK`)
- [x] **CODE_OF_CONDUCT.md** — Microsoft Open Source Code of Conduct
- [x] **CONTRIBUTING.md** — CLA reference, Code of Conduct reference, contribution guidelines
- [x] **SUPPORT.md** — How to file issues and get help, Microsoft support policy
- [x] **README.md** — Project description with Contributing, Trademarks, and Code of Conduct sections
- [x] **.gitignore** — Standard Rust gitignore

## Required README Sections

- [x] Project description
- [x] **Contributing** section — CLA reference + Code of Conduct link
- [x] **Trademarks** section — Microsoft Trademark & Brand Guidelines link
- [x] **License** section — link to LICENSE file

## Source File Headers

- [x] All `.rs` source files include the Microsoft copyright header:
  ```
  // Copyright (c) Microsoft Corporation.
  // Licensed under the MIT License.
  ```

## Cargo.toml Metadata (for crates.io publishing)

- [x] `license = "MIT"` in both `corim/Cargo.toml` and `corim_derive/Cargo.toml`
- [x] `license-file = "../LICENSE"` pointing to root LICENSE
- [x] `repository` pointing to `https://github.com/microsoft/corim`
- [x] `description` present
- [x] `authors = ["Microsoft"]`
- [x] `readme` pointing to README.md

## GitHub Community Files

- [x] `.github/workflows/ci.yml` — CI pipeline (build, test, fmt, clippy, docs)
- [x] `.github/ISSUE_TEMPLATE/bug_report.md` — Bug report template
- [x] `.github/ISSUE_TEMPLATE/feature_request.md` — Feature request template
- [x] `.github/PULL_REQUEST_TEMPLATE.md` — PR template with checklist

## Recommended (post-publish)

- [ ] **CODEOWNERS** — assign default reviewers (fill in once team is established)
- [ ] **Branch protection** — require PR reviews, CI passing on `main`
- [ ] **CLA bot** — ensure `microsoft/cla` GitHub App is installed on the repo
- [ ] **Dependabot** — enable dependency update alerts (`.github/dependabot.yml`)
- [ ] **NOTICE** file — if third-party components require attribution beyond Cargo.toml
- [ ] **Signed commits** — enforce GPG-signed commits policy
- [ ] **Release workflow** — automated crates.io publishing via GitHub Actions
