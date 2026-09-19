# conservation-exchange

Exact, indivisible exchanges over canonical quantities and domain-owned records.
This crate implements issue [#1](https://github.com/ctoth/conservation/issues/1).
The Python package calls this implementation through `conservation-python`;
there is no Python settlement implementation or numeric fallback.

## Model and authority

`Model` registers owners, dimensional capacity constraints and `Law`s. An engine
starts empty. Creating and funding a stock is an ordinary recorded exchange
against a declared boundary-input law, not an unrestricted initial-balance setter.

A `Stock` has a permanent identity, an owner, a dimension, a nonnegative or signed
domain, and capacity weights. A weight converts stock units to a capacity unit:
for example, a volume constraint can use a domain-supplied reciprocal density.
Capacity checks sum **resulting** contents, so consumption can release space for
products in the same exchange. Capacity weights must be nonnegative; signed
coordinates do not participate in these storage constraints.

A law declares stock roles, signed boundary inputs, required participants,
evaluated facts and constraints. An `Exchange` binds each role to a distinct
stock and supplies exact deltas. Unspecified declared deltas and boundary inputs
are zero; undeclared ones are errors. Constraints cover every quantity role and
boundary with an equality. Stock and record creation order is canonicalized;
stock-role aliasing and duplicate record keys are rejected.

The model author is trusted to specify the right physics, dimensions, boundary
permissions and tolerances. Merely declaring an equality does not prove it is a
physical conservation law. Owner names identify domain responsibility, not a
security principal: authorization belongs to the application before preparation.

## Preparation and publication

1. Each required domain owner validates its policy and prepares `RecordWrite`s
   and/or declared evaluated facts using `Engine::participate`. These contributions
   are tied to the exact canonical proposal and the observed immutable state root.
2. `Engine::prepare` validates all quantities, constraints, capacities, topology
   changes and contributions and builds a private candidate root. It neither
   reserves resources nor publishes evidence. Any error leaves the current root
   unchanged, including records staged earlier in that rejected request.
3. `Engine::publish` verifies lineage, duplicate identity and current root, then
   swaps the complete root. No user code, callback or IO runs during publication.
4. The caller may notify observers after receiving the receipt. Observers have
   no participation callback that could undo or execute the transition again.

Two competing preparations cannot both spend the same revision. The second is
stale even when its changes are disjoint: this first implementation deliberately
uses whole-engine revision validation, not fine-grained locking. It does not choose
winners or promise order-independent selection among competing indivisible requests.
Consumers schedule proposals and may recompute a smaller feasible complete request.
The engine never independently rescales its legs.

The candidate is fully allocated and validated before the root swap. Rust's
exclusive `&mut Engine` and the binding's native borrow guard prevent concurrent
publication on an engine. This is **in-memory logical atomicity**, not a durable
distributed transaction or a promise to recover from process termination mid-call.

## Domain participation is real ownership

Records are canonical immutable byte strings keyed by `(owner, key)` **inside
the same state root**. Domain owners define and validate their schemas before
staging them. Relationships, claims and required journal evidence can therefore
publish with quantities without exposing partial state. A journal may use one
record per exchange; it need not replace a growing serialized journal on every call.

The engine does not synchronize external dictionaries, call `apply/rollback`,
clone another owner's private state, or make an external database atomic. A
consumer retaining an independently authoritative external ledger cannot use this
API to claim atomic joint publication. It must first put the participating state
under this owner contract or supply a separately proven publication mechanism.
Domain-only record exchanges may use a law with no stock roles and required
participants. Omitting a required participant rejects the request.

Facts have declared owners and dimensions. `Participation` captures their values
with the full proposal and source root, preventing substitution into another
request or use after an intervening commit. The kernel checks those associations
and the declared equations; it does not independently recompute an external domain
law. Put reproducible arithmetic in the expression language when possible.

## Quantities, expressions and tolerances

All execution uses arbitrary-precision `BigRational`. Python accepts integer or
fraction strings (for example `"17"` and `"1/1000"`), never implicit floats,
decimal rounding, NaN or infinity. Python `Quantity.fraction` returns the exact
canonical value. Dimensions use integer exponents over application-defined base
dimensions; product and quotient check exponent overflow. Display-unit conversion
is the application's responsibility before supplying canonical quantities.

Expressions support exact constants; before/after/delta stock-role values;
declared boundary inputs and owner facts; sums, products and quotients. Addition
and comparisons require matching dimensions. Products and quotients derive their
dimensions, including dimensioned conversion coefficients. Division by zero is an
error. Nesting above 64 and unknown expression variants are rejected.

This covers linear transforms and evaluated nonlinear algebra such as `p*p/(2*m)`.
It does not implement square roots, transcendental functions, differential-equation
solvers, frame transformations or gravity theory. Those remain domain calculations
whose version/frame/error metadata can be carried in owner records. A mechanical
law should constrain energy derived from its owning motion state; it must not
introduce an independently spendable duplicate kinetic-energy stock.

Each equality or upper-bound constraint has exact rational absolute and relative
tolerances. Absolute tolerance is expressed in the comparison's derived canonical
unit. Both default to zero; relative tolerance must be in `[0, 1)`. The threshold
is `absolute + relative * max(abs(left), abs(right))`. Domain and capacity bounds
are strict, with no clamping. Errors identify the failed exchange/constraint or
stock/capacity and exact residual. Model authors remain responsible for selecting
tolerances small enough to resolve the signal; the tests include a `9/100` energy
release from a `1/10^18` material change beside a `10^30` material stock.

There is no dense backend in this release. Existing `conservation-dynamics`
continues to provide proportional flow settlement for its own consumers. A Pyspace
quantity must not be mirrored into both engines as independently mutable state.

## Identity, topology and snapshots

An identical committed request returns its original receipt, without another
mutation. Reusing its ID with different canonical content, including participant
record bytes or facts, fails. Rejected requests are not committed. Callers supply
IDs; the kernel does not invent timestamps, random IDs or scheduling order.

Creation, zero-balance removal and ownership/capacity reassignment are part of an
exchange. Removal requires an accounted final zero; retired stock IDs cannot be
reused. Reassignment preserves identity and quantity and rechecks all capacities.

Snapshot format 1 contains the model and ordered canonical committed requests.
Restore replays and revalidates them, rebuilding quantities, revisions, retired
IDs, records, receipts and boundary evidence once. It never accepts separately
serialized balances. The input is trusted model/history, not an authenticated or
tamper-proof artifact. Unsupported versions, unknown fields and invalid transitions
are rejected; no legacy reader is provided.

Preparations are not serialized. Restoring creates a fresh process-local lineage,
so a pre-restore preparation cannot publish even at an equal revision. Prepare it
again from the restored state. Identical already-committed requests still deduplicate.

The initial implementation copies its **own** canonical root when preparing and
replays the full history on restore. It makes no large-world throughput or bounded
history claim. This is distinct from snapshotting foreign objects for rollback.

## Python example

```python
from conservation_exchange import Constraint, Dimension, Engine, Exchange, Expr, Law, Quantity, Stock

kg = Dimension("mass")
supply = Law(
    "supply", {"stock": kg},
    [Constraint("balance", Expr.delta("stock"), Expr.boundary("source"))],
    boundaries={"source": kg},
)
engine = Engine({"hold"}, [supply])
request = Exchange(
    "initial-load", "supply", {"stock": "fuel"},
    creates=[Stock("fuel", "hold", kg)],
    deltas={"stock": Quantity("3/2", kg)},
    boundaries={"source": Quantity("3/2", kg)},
)
receipt = engine.publish(engine.prepare(request))
assert engine.amount("fuel").fraction == "3/2"
assert engine.publish(engine.prepare(request)) == receipt
restored = Engine.restore(engine.snapshot())
assert restored.amount("fuel") == engine.amount("fuel")
```

The package exports native typed exceptions, immutable model/request values,
opaque preparation handles and PEP 561 declarations. Installation requires the
native extension. `__version__` matches the distribution version; consumers pin
the git revision and check their supported API/package version explicitly.

## Build and verify

Use an installed 64-bit CPython matching the Rust target. `uv sync --locked`
creates the isolated environment and builds the extension. Run binding-aware
Cargo commands through `uv run --no-sync` so PyO3 uses that interpreter rather
than an unrelated system Python. If `PYO3_PYTHON` is set externally, it must point
to that same interpreter.

```text
uv sync --locked
uv run --no-sync cargo fmt --all -- --check
uv run --no-sync cargo test --workspace --locked
uv run --no-sync cargo clippy --workspace --all-targets --locked -- -D warnings
uv run --no-sync pytest python/tests
uv run --no-sync pyright
uv run --no-sync maturin build --locked --out target/contract-wheels
```

Keep the wheel-test output separate from maturin's editable-build artifacts and
require exactly one wheel there. After building, install it with `uv pip install --reinstall
--no-deps <wheel>` and rerun pytest with `--no-sync` to exercise the installed
artifact. The package test checks the actual native module, version and typing
files. CI tests Rust 1.85 compatibility and uses stable Rust for formatting/lint
policy; it covers Python 3.10/3.13, Linux and Windows.
