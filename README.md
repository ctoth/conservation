# conservation

Exact conservation-law derivation and trace evidence for typed quantitative systems.

This Cargo workspace contains four publishable crates:

- `conservation-core` defines validated typed axis and kind identifiers, exact
  rational balance laws, and origin metadata tags. Origin metadata records how
  a law is asserted to have arisen; it is not a correctness certificate.
- `conservation-linear` derives deterministic left-nullspace bases from integer
  or rational transition matrices with arbitrary-precision exact arithmetic.
  Its primitive-integer vectors form a rational vector-space basis, not an
  integer-lattice basis. These are structural linear conservation laws; the
  crate does not claim to enumerate nonlinear or kinetic-parameter-specific
  invariants.
- `conservation-dynamics` compiles typed stocks and processes into one immutable
  indexed topology shared by exact-rational and finite-validated binary64
  states. Each batch observes one pre-settlement state; competing withdrawals
  receive a common proportional limit, while boundary inputs are unavailable
  until the next batch. The exact state is an exact-arithmetic reference; the
  contiguous binary64 state is a candidate execution path with an explicit
  comparison tolerance and atomic overflow rejection. Performance claims wait
  for domain-scale benchmarks.
- `conservation-trace` checks finite-state traces exactly and returns typed
  satisfied/violated verdicts, structurally separate from malformed-trace
  errors. Trace witnesses do not inherit derivation-origin metadata.
## Boundary

The core, dynamics, linear, and trace foundation is deliberately independent
of any institution model and of Bridgman. It contains no Python bindings,
BLAS integration, domain-specific ecosystem equations, or Noether derivation.
`Provenance::Noether` is a tag reserved for laws produced elsewhere.

The executable institution adapter lives downstream as
`institution-conservation` in the `institution` repository. This repository
does not depend on institution theory.

Licensed under the MIT License.
