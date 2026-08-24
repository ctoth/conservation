# conservation

Exact conservation-law derivation and trace evidence for typed quantitative systems.

This Cargo workspace contains four publishable crates:

- `conservation-core` defines validated typed axis and kind identifiers, exact
  rational balance laws, and origin metadata tags. Origin metadata records how
  a law is asserted to have arisen; it is not a correctness certificate.
- `conservation-linear` derives deterministic left-nullspace bases from integer
  or rational transition matrices with arbitrary-precision exact arithmetic.
  Its primitive-integer vectors form a rational vector-space basis, not an
  integer-lattice basis.
- `conservation-trace` checks finite-state traces exactly and returns typed
  satisfied/violated verdicts, structurally separate from malformed-trace
  errors. Trace witnesses do not inherit derivation-origin metadata.
- `conservation-institution` is the downstream executable-institution bridge.
  It connects validated conservation signatures and bijective axis and kind
  symbol renamings to exact balance-law sentences and trace models. For local
  development it uses a path-plus-version dependency on the sibling
  `institution` crate; a published package resolves that version from the
  registry.

The bridge package payload can be inspected locally with `cargo package
--allow-dirty --list`. Full `cargo package` verification is release-order
blocked until its registry dependencies (`institution`, `conservation-core`,
and `conservation-trace`) have been published at their declared versions.

## Boundary

The core, linear, and trace foundation is deliberately independent of any
institution model and of Bridgman. It contains no Python bindings,
floating-point tolerances, or Noether derivation. `Provenance::Noether` is a
tag reserved for laws produced elsewhere.

The bridge remains downstream in `conservation-institution`: `institution`
does not depend on Conservation, and the core, linear, and trace crates remain
usable without the bridge.

Licensed under the MIT License.
