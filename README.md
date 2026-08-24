# conservation

Exact conservation-law derivation and trace evidence for typed quantitative systems.

This Cargo workspace contains three publishable crates:

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
## Boundary

The core, linear, and trace foundation is deliberately independent of any
institution model and of Bridgman. It contains no Python bindings,
floating-point tolerances, or Noether derivation. `Provenance::Noether` is a
tag reserved for laws produced elsewhere.

The executable institution adapter lives downstream as
`institution-conservation` in the `institution` repository. This repository
does not depend on institution theory.

Licensed under the MIT License.
