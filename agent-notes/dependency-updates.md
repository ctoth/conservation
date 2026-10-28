# Dependency and security updates

## Goal

Resolve Conservation's reported dependency vulnerabilities and refresh its dependencies and CI tooling. Keep the existing Rust 1.85 minimum and Python >=3.10 API contract.

## State

Q authorized the complete plan with "make it so!" on 2026-10-28. Implementation is on branch `dependency-updates`: manifests, CI pins/Python versions, README and lockfiles updated. Publication and hosted CI are next. No local CI commands have been run.

The checkout is on master and was clean at inspection. The remote master matched it. Open Dependabot PR #3, branch `dependabot/cargo/pyo3-0.29.0`, changes only the Python binding manifest and Cargo lockfile. Its existing three CI jobs succeeded; those results are not verification of the newer updates planned here.

GitHub's four open Dependabot alerts are duplicate manifest/lockfile reports for two PyO3 advisories: GHSA-36hh-v3qg-5jq4 (iterator out-of-bounds reads) and GHSA-chgr-c6px-7xpp (closure Sync bound). Both are patched in >=0.29.0. An OSV batch query of 133 registry package/version entries in Cargo.lock and uv.lock also returned these advisories, plus the informational unmaintained warning RUSTSEC-2024-0449 for unsafe-libyaml. The scan excluded Git and workspace packages; it is not a source security audit.

## Decisions and why

Smallest implementation: roughly 10-25 hand-edited configuration/documentation lines plus generated lockfile changes and any binding changes actually required by the upgrade. No new feature, compatibility wrapper, security suppression, test suite or CI gate.

1. Upgrade `conservation-python/Cargo.toml:23` from PyO3 0.27 to 0.30.1, currently the latest stable release. Keep abi3-py310. Remove generate-import-lib, which upstream now documents as a no-op with raw-dylib Windows linking. Inspect compiler findings through the existing PR CI and fix only necessary binding migration changes.
2. Upgrade `conservation-dynamics/Cargo.toml:18` from Criterion 0.5 to 0.7, the newest compatible release for Rust 1.85. The benchmark uses standard Criterion APIs and already uses std::hint::black_box. Criterion 0.8 requires Rust 1.86. Keep proptest at a Rust-1.85-compatible resolution: 1.11 is compatible, whereas 1.12 requires Rust 1.88.
3. Refresh compatible registry resolutions in Cargo.lock and uv.lock. Known available updates include serde_json 1.0.151 to 1.0.154 and pytest 9.1.1 to 9.2.0. Maturin 1.15.0 and Pyright 1.1.414 are already current. Preserve the Bridgman Git pin. Preserve num-bigint 0.4 because num-rational 0.4.2 requires ^0.4.0 and shares BigInt types with this workspace; moving only the direct dependency to 0.5 would split those types.
4. Update the existing action pins in `.github/workflows/ci.yml:30-42` to verified release revisions for checkout v7.0.1, setup-python v7.1.0 and setup-uv v10.3.2, and uv 0.8.8 to 0.13.6. Refresh the rust-toolchain action pin. Keep all existing Rust, Python, wheel, typing and dependency-boundary checks. Change the two current-Python jobs from 3.13 to 3.15, retaining the minimum Rust/Python job and existing Linux/Windows coverage. Update `conservation-exchange/README.md:235` to match. Python.org lists 3.15 as the current stable series and 3.10 as end-of-life; this task does not drop declared Python 3.10 compatibility.
5. After authorization, work on branch `dependency-updates`, commit and push, and open one maintenance PR. Use its CI for the existing checks; do not run CI steps locally to gate the push. Re-query OSV for the final lockfiles. Review the actual final code changes and current repository merge requirements. Under approval of the complete delivery plan, normally merge after successful checks and review, verify the four alerts close, close superseded Dependabot PR #3, and remove this task's branch/worktree. No package release or downstream repository change is included.

Primary sources read: workspace and every member Cargo manifest, pyproject.toml, both lockfiles, workflow, binding source, benchmark source and exchange verification documentation. Upstream PyO3 v0.30.1 Cargo.toml, CHANGELOG.md and guide/src/migration.md confirm Rust 1.85, Python 3.15 support, no-op generate-import-lib and the changed FromPyObject defaults. The binding's pyclasses do not derive Clone, so that particular migration does not apply to the current code. Actual compatibility remains for CI to establish.

Version checks used crates.io and PyPI registry APIs and GitHub release APIs on 2026-10-27. Advisory sources:

- https://rustsec.org/advisories/RUSTSEC-2026-0176.html
- https://rustsec.org/advisories/RUSTSEC-2026-0177.html
- https://rustsec.org/advisories/RUSTSEC-2024-0449.html
- https://raw.githubusercontent.com/PyO3/pyo3/v0.30.1/CHANGELOG.md
- https://www.python.org/downloads/

## Surprises

`Cargo.lock:54-65` shows bridgman-core brings in serde_yaml; `Cargo.lock:929-938` shows serde_yaml brings in unsafe-libyaml 0.2.11. RustSec marks it informational/unmaintained, with no patched versions. The pinned and current remote `crates/bridgman-core/Cargo.toml` both declare serde_yaml 0.9.34. A lockfile update in Conservation cannot remove that dependency. Replacing Bridgman's YAML parser is outside this repository's update plan and remains explicitly unresolved, with no suppression proposed.

An initial action-version PowerShell query was malformed; the corrected query succeeded. The CPython latest-release GitHub endpoint returns 404 because it is not a release inventory; Python.org supplied the version evidence instead.

The local uv resolver applies a global 14-day release delay. Preserve that selection policy: Python updates selected iniconfig 2.3.1, nodeenv 1.11.0 and tomli 2.5.0; pytest remains 9.1.1. Do not disable the release delay just to select pytest 9.2.0. Removed the machine-specific exclude-newer options emitted into uv.lock, keeping the selected versions; the repository and hosted CI have no such configuration. Cargo's resolver selected Rust-1.85-compatible versions and retained the Bridgman pin, proptest 1.11 and num-bigint 0.4.8.

A config probe passed PowerShell's -ErrorAction option to rg, causing an option parsing error; corrected the command by filtering existing paths in PowerShell first.

## Next step

Publish the maintenance PR and use its CI to verify compatibility, then scan final registry dependencies and complete the review/merge. Authorization includes implementation, PR publication, CI repairs within scope, successful normal merge and closure of superseded PR #3.
