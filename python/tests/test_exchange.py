"""Contract tests import the installed native package, never a Python stand-in."""

import json
from importlib.metadata import version
from importlib.util import find_spec
from pathlib import Path

import pytest

import conservation_exchange as ce

REGISTRY = ce.KindRegistry({
    "material": ce.KindDeclaration({"material": 1}, floor="0"),
    "material_balance": ce.KindDeclaration({"material": 1}),
    "energy": ce.KindDeclaration({"energy": 1}, floor="0"),
    "torque": ce.KindDeclaration({"energy": 1}),
    "ratio": ce.KindDeclaration(),
    "temperature": ce.KindDeclaration({"temperature": 1}, floor="0", difference="temperature_delta"),
    "temperature_delta": ce.KindDeclaration({"temperature": 1}),
})
MATERIAL = REGISTRY.kind("material")
MATERIAL_BALANCE = REGISTRY.kind("material_balance")
ENERGY = REGISTRY.kind("energy")
TORQUE = REGISTRY.kind("torque")
RATIO = REGISTRY.kind("ratio")


def q(amount: int, kind: ce.Kind = MATERIAL) -> ce.Quantity:
    return ce.Quantity(str(amount), kind)


def supply_law(id: str, kind: ce.Kind) -> ce.Law:
    return ce.Law(
        id, {"stock": kind},
        [ce.Constraint("supply", ce.Expr.delta("stock"), ce.Expr.boundary("supply"))],
        boundaries={"supply": kind},
    )


def engine(*, capacity: int = 100) -> ce.Engine:
    reaction = ce.Law(
        "combine", {key: MATERIAL for key in ("a", "b", "out")},
        [
            ce.Constraint("stoichiometry", ce.Expr.delta("a"), ce.Expr.delta("b")),
            ce.Constraint("balance", ce.Expr.sum([ce.Expr.delta(key) for key in ("a", "b", "out")]), ce.Expr.constant(q(0))),
        ],
        participants={"journal"},
    )
    laws = [supply_law("supply", MATERIAL), supply_law("signed-supply", MATERIAL_BALANCE), reaction]
    return ce.Engine({"owner", "journal"}, laws, {"hold": ce.Capacity(q(capacity))})


def supply(target: ce.Engine, stock: str, amount: int, *, signed: bool = False) -> ce.Receipt:
    kind = MATERIAL_BALANCE if signed else MATERIAL
    proposal = ce.Exchange(
        f"supply-{stock}", "signed-supply" if signed else "supply", {"stock": stock},
        deltas={"stock": q(amount, kind)}, boundaries={"supply": q(amount, kind)},
        creates=[ce.Stock(stock, "owner", kind,
                          capacities=None if signed else {"hold": ce.Quantity("1", RATIO)})],
    )
    return target.publish(target.prepare(proposal))


def combine(id: str, amount: int) -> ce.Exchange:
    return ce.Exchange(id, "combine", {key: key for key in ("a", "b", "out")},
                       deltas={"a": q(-amount), "b": q(-amount), "out": q(2 * amount)})


def prepare(target: ce.Engine, proposal: ce.Exchange) -> ce.Prepared:
    return target.prepare(proposal, [target.participate(
        "journal", proposal, [ce.RecordWrite(proposal.id, b"accepted")]
    )])


def test_installed_package_is_native_and_versioned() -> None:
    native = find_spec("conservation_exchange._native")
    assert native is not None and native.origin is not None
    assert Path(native.origin).suffix in {".pyd", ".so"}
    assert ce.Engine.__module__ == "conservation_exchange._native"
    assert version("conservation-exchange") == ce.__version__ == "0.3.0"
    assert Path(ce.__file__).with_name("py.typed").is_file()
    assert Path(ce.__file__).with_name("_native.pyi").is_file()


def test_coupled_publication_and_duplicate_are_atomic() -> None:
    target = engine(capacity=20)
    for stock, amount in (("a", 10), ("b", 10), ("out", 0)):
        supply(target, stock, amount)
    proposal = combine("one", 4)
    prepared = prepare(target, proposal)
    assert target.amount("a") == q(10)
    assert target.record("journal", "one") is None
    receipt = target.publish(prepared)
    assert target.amount("a") == q(6)
    assert target.amount("out") == q(8)
    assert target.record("journal", "one") == b"accepted"
    assert target.publish(prepared) == receipt
    assert receipt.changes["a"] == (q(10), q(6))
    with pytest.raises(ce.DuplicateExchangeError):
        prepare(target, combine("one", 2))


def test_shortage_and_participant_failure_leave_no_partial_state() -> None:
    target = engine()
    for stock, amount in (("a", 10), ("b", 0), ("out", 0)):
        supply(target, stock, amount)
    before = target.snapshot()
    with pytest.raises(ce.DomainError, match="b"):
        prepare(target, combine("shortage", 2))
    with pytest.raises(ce.ParticipationError):
        target.prepare(combine("no-journal", 0))
    proposal = combine("bad-journal", 0)
    part = target.participate("journal", proposal, [ce.RecordWrite("missing", None)])
    with pytest.raises(ce.ParticipationError):
        target.prepare(proposal, [part])
    assert target.snapshot() == before


def test_stale_preparations_and_restored_lineages_cannot_publish() -> None:
    target = engine()
    for stock, amount in (("a", 10), ("b", 10), ("out", 0)):
        supply(target, stock, amount)
    first = prepare(target, combine("first", 6))
    second = prepare(target, combine("second", 6))
    target.publish(first)
    with pytest.raises(ce.StalePreparationError):
        target.publish(second)
    restored = ce.Engine.restore(target.snapshot(), REGISTRY)
    assert restored.snapshot() == target.snapshot()
    assert restored.amount("out") == q(12)
    with pytest.raises(ce.ForeignPreparationError):
        restored.publish(first)
    assert restored.publish(prepare(restored, combine("first", 6))) == target.receipt("first")


def test_capacity_signed_values_and_boundary_validation() -> None:
    target = engine(capacity=3)
    supply(target, "signed", -10, signed=True)
    assert target.amount("signed") == q(-10, MATERIAL_BALANCE)
    assert target.stock("signed").kind.floor is None
    supply(target, "a", 3)
    before = target.snapshot()
    with pytest.raises(ce.CapacityError, match="hold"):
        supply(target, "b", 1)
    unknown = ce.Exchange("magic", "supply", {"stock": "a"}, boundaries={"magic": q(1)})
    with pytest.raises(ce.InvalidExchange, match="undeclared boundary"):
        target.prepare(unknown)
    assert target.snapshot() == before


@pytest.mark.parametrize("value", ["NaN", "inf", "1/0", "0.1", ""])
def test_quantities_reject_nonexact_or_nonfinite_input(value: str) -> None:
    with pytest.raises(ce.InvalidExchange):
        ce.Quantity(value, RATIO)


def test_dimensions_and_native_definitions_are_immutable_snapshots() -> None:
    assert ENERGY.dimensions == {"energy": 1}
    assert RATIO.dimensions == {}
    assert ce.Quantity("2/4", ENERGY).fraction == "1/2"
    slots = {"stock": MATERIAL}
    law = ce.Law("supply", slots, [ce.Constraint("supply", ce.Expr.delta("stock"), ce.Expr.boundary("supply"))], boundaries={"supply": MATERIAL})
    slots["stock"] = ENERGY
    target = ce.Engine({"owner", "journal"}, [law])
    request = ce.Exchange("seed", "supply", {"stock": "x"}, deltas={"stock": q(1)}, boundaries={"supply": q(1)}, creates=[ce.Stock("x", "owner", MATERIAL)])
    target.publish(target.prepare(request))
    assert target.amount("x") == q(1)
    with pytest.raises(AttributeError):
        setattr(request, "id", "mutated")


def test_evaluated_fact_and_topology_change_use_same_native_commit() -> None:
    seed = ce.Law("seed", {"stock": MATERIAL},
                  [ce.Constraint("source", ce.Expr.delta("stock"), ce.Expr.boundary("source"))],
                  boundaries={"source": MATERIAL})
    move = ce.Law("move", {"stock": MATERIAL},
                  [ce.Constraint("preserve", ce.Expr.delta("stock"), ce.Expr.constant(q(0))),
                   ce.Constraint("limit", ce.Expr.after("stock"), ce.Expr.fact("limit"), less_or_equal=True)],
                  participants={"owner"}, facts={"limit": ce.Fact("owner", MATERIAL)})
    target = ce.Engine({"owner", "destination"}, [seed, move])
    request = ce.Exchange("seed", "seed", {"stock": "x"}, deltas={"stock": q(3)},
                          boundaries={"source": q(3)}, creates=[ce.Stock("x", "owner", MATERIAL)])
    target.publish(target.prepare(request))
    request = ce.Exchange("move", "move", {"stock": "x"}, moves={"x": ce.Placement("destination")})
    insufficient = target.participate("owner", request, facts={"limit": q(2)})
    with pytest.raises(ce.ConstraintError, match="limit"):
        target.prepare(request, [insufficient])
    assert target.stock("x").owner == "owner"
    sufficient = target.participate("owner", request, [ce.RecordWrite("relation", b"destination")], {"limit": q(3)})
    target.publish(target.prepare(request, [sufficient]))
    assert target.stock("x").owner == "destination"
    assert target.amount("x") == q(3)
    restored = ce.Engine.restore(target.snapshot(), REGISTRY)
    assert restored.record("owner", "relation") == b"destination"
    assert restored.stock("x").owner == "destination"


def test_required_journal_commits_before_observer_and_is_not_repeated() -> None:
    target = engine()
    for stock, amount in (("a", 3), ("b", 3), ("out", 0)):
        supply(target, stock, amount)
    prepared = prepare(target, combine("observe", 1))

    def observer(receipt: ce.Receipt) -> None:
        assert receipt.id == "observe"
        assert target.record("journal", "observe") == b"accepted"
        raise RuntimeError("observer unavailable")

    with pytest.raises(RuntimeError, match="observer unavailable"):
        observer(target.publish(prepared))
    assert target.amount("out") == q(2)
    assert target.publish(prepared) == target.receipt("observe")
    assert target.amount("out") == q(2)


def test_invalid_model_and_snapshot_fail_loudly() -> None:
    invalid = ce.Law("wrong-dimension", {"stock": MATERIAL},
                     [ce.Constraint("bad", ce.Expr.delta("stock"), ce.Expr.constant(ce.Quantity("0", ENERGY)))])
    with pytest.raises(ce.DimensionError, match="bad"):
        ce.Engine({"owner"}, [invalid])
    with pytest.raises(ce.SnapshotError):
        ce.Engine.restore(b'{"version": 999}', REGISTRY)


def test_registry_resolves_names_and_rejects_unknown_by_name() -> None:
    assert REGISTRY.kind("material").name == "material"
    assert REGISTRY.kind("material") == MATERIAL
    assert hash(REGISTRY.kind("material")) == hash(MATERIAL)
    assert MATERIAL != MATERIAL_BALANCE
    with pytest.raises(ce.UnknownKindError, match="materiel"):
        REGISTRY.kind("materiel")


def test_kinds_report_floor_and_difference() -> None:
    temperature = REGISTRY.kind("temperature")
    delta = temperature.difference
    assert delta is not None and delta.name == "temperature_delta"
    assert delta.difference is None
    assert temperature.floor == "0"
    assert MATERIAL.floor == "0"
    assert MATERIAL_BALANCE.floor is None
    assert MATERIAL.difference is None


def test_kind_declarations_are_validated() -> None:
    material = ce.KindDeclaration({"material": 1})
    invalid: list[dict[str, ce.KindDeclaration]] = [
        {" ": material},
        {"material": ce.KindDeclaration({" ": 1})},
        {"material": ce.KindDeclaration({"material": 0})},
        {"point": ce.KindDeclaration({"material": 1}, difference="missing")},
        {"point": ce.KindDeclaration({"material": 1}, difference="other"),
         "other": ce.KindDeclaration({"material": 1}, difference="material"),
         "material": material},
        {"point": ce.KindDeclaration({"material": 1}, difference="energy"),
         "energy": ce.KindDeclaration({"energy": 1})},
    ]
    for declarations in invalid:
        with pytest.raises(ce.KindDeclarationError):
            ce.KindRegistry(declarations)


def test_mismatched_leg_names_both_kinds() -> None:
    target = engine()
    proposal = ce.Exchange("battery", "supply", {"stock": "battery"},
                           creates=[ce.Stock("battery", "owner", ENERGY)])
    with pytest.raises(ce.DimensionError) as error:
        target.prepare(proposal)
    assert "material" in str(error.value) and "energy" in str(error.value)


def test_equal_dimension_kinds_cannot_share_a_slot() -> None:
    assert ENERGY.dimensions == TORQUE.dimensions
    target = ce.Engine({"owner"}, [supply_law("energy-supply", ENERGY)])
    proposal = ce.Exchange("spanner", "energy-supply", {"stock": "spanner"},
                           creates=[ce.Stock("spanner", "owner", TORQUE)])
    with pytest.raises(ce.DimensionError) as error:
        target.prepare(proposal)
    assert "energy" in str(error.value) and "torque" in str(error.value)


def test_restore_resolves_kinds_against_the_supplied_registry() -> None:
    target = engine()
    supply(target, "a", 3)
    without_material = ce.KindRegistry({
        "material_balance": ce.KindDeclaration({"material": 1}),
        "ratio": ce.KindDeclaration(),
    })
    with pytest.raises(ce.UnknownKindError, match="material"):
        ce.Engine.restore(target.snapshot(), without_material)
    assert ce.Engine.restore(target.snapshot(), REGISTRY).snapshot() == target.snapshot()


def test_restore_rejects_version_2() -> None:
    target = engine()
    supply(target, "a", 3)
    snapshot = json.loads(target.snapshot())
    assert snapshot["version"] == 3
    snapshot["version"] = 2
    with pytest.raises(ce.SnapshotError):
        ce.Engine.restore(json.dumps(snapshot).encode(), REGISTRY)
