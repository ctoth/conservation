"""Runtime topology and laws publish in the same native commit as quantities."""

import pytest

import conservation_exchange as ce


REGISTRY = ce.KindRegistry({
    "mass": ce.KindDeclaration({"mass": 1}, floor="0"),
    "ratio": ce.KindDeclaration(),
})
MASS = REGISTRY.kind("mass")
RATIO = REGISTRY.kind("ratio")


def q(value: int) -> ce.Quantity:
    return ce.Quantity(str(value), MASS)


def request(amount: int = 3, *, maximum: int = 5) -> ce.Exchange:
    law = ce.Law(
        "new-supply", {"stock": MASS},
        [ce.Constraint("balance", ce.Expr.delta("stock"), ce.Expr.boundary("input"))],
        boundaries={"input": MASS}, participants={"new-owner"},
    )
    return ce.Exchange(
        "create-hold", "new-supply", {"stock": "ore"},
        deltas={"stock": q(amount)}, boundaries={"input": q(amount)},
        creates=[ce.Stock("ore", "new-owner", MASS, capacities={"hold": ce.Quantity("1", RATIO)})],
        declarations=ce.Model({"new-owner"}, [law], {"hold": ce.Capacity(q(maximum))}),
    )


def prepared(target: ce.Engine, proposal: ce.Exchange) -> ce.Prepared:
    return target.prepare(proposal, [target.participate(
        "new-owner", proposal, [ce.RecordWrite("created", b"ore")],
    )])


def test_runtime_definitions_publish_with_stock_and_evidence_once() -> None:
    target = ce.Engine({"initial"}, [])
    proposal = request()
    before = target.snapshot()
    staged = prepared(target, proposal)
    assert target.snapshot() == before
    receipt = target.publish(staged)
    assert receipt.revision == 1
    assert target.amount("ore") == q(3)
    assert target.record("new-owner", "created") == b"ore"
    committed = target.snapshot()
    assert target.publish(prepared(target, proposal)) == receipt
    assert target.snapshot() == committed
    restored = ce.Engine.restore(committed, REGISTRY)
    assert restored.snapshot() == committed
    assert restored.publish(prepared(restored, proposal)) == receipt
    with pytest.raises(ce.ForeignPreparationError):
        restored.publish(staged)


def test_rejected_definitions_and_quantities_leave_no_partial_model() -> None:
    target = ce.Engine({"initial"}, [])
    before = target.snapshot()
    with pytest.raises(ce.CapacityError):
        prepared(target, request(6))
    assert target.snapshot() == before
    # A different definition succeeds because the rejected one never existed.
    target.publish(prepared(target, request(6, maximum=8)))
    assert target.amount("ore") == q(6)


def test_competing_model_publications_invalidate_preparation() -> None:
    target = ce.Engine({"initial"}, [])
    stale = prepared(target, request())
    target.publish(prepared(target, request(4)))
    before = target.snapshot()
    with pytest.raises(ce.DuplicateExchangeError):
        target.publish(stale)
    assert target.snapshot() == before


def test_existing_definitions_cannot_be_replaced() -> None:
    target = ce.Engine({"initial"}, [])
    target.publish(prepared(target, request()))
    before = target.snapshot()
    with pytest.raises(ce.InvalidExchange, match="capacity.*hold"):
        prepared(target, request(maximum=10))
    assert target.snapshot() == before


def test_new_law_arity_spends_existing_stock_without_rebuilding_engine() -> None:
    target = ce.Engine({"initial"}, [])
    target.publish(prepared(target, request()))
    law = ce.Law(
        "split", {slot: MASS for slot in ("source", "a", "b")},
        [ce.Constraint("mass", ce.Expr.sum([
            ce.Expr.delta(slot) for slot in ("source", "a", "b")
        ]), ce.Expr.constant(q(0)))],
    )
    proposal = ce.Exchange(
        "split", "split", {"source": "ore", "a": "ore-a", "b": "ore-b"},
        deltas={"source": q(-3), "a": q(1), "b": q(2)},
        creates=[ce.Stock(name, "new-owner", MASS) for name in ("ore-a", "ore-b")],
        declarations=ce.Model(set(), [law]),
    )
    target.publish(target.prepare(proposal))
    assert target.amount("ore") == q(0)
    assert target.amount("ore-a") == q(1)
    assert target.amount("ore-b") == q(2)
    restored = ce.Engine.restore(target.snapshot(), REGISTRY)
    assert restored.snapshot() == target.snapshot()


def test_model_commit_stales_other_quantities_and_participation() -> None:
    target = ce.Engine({"initial"}, [])
    stale = prepared(target, request())
    proposal = ce.Exchange(
        "declare-other", "empty", {},
        declarations=ce.Model(set(), [ce.Law("empty", {}, [])]),
    )
    target.publish(target.prepare(proposal))
    before = target.snapshot()
    with pytest.raises(ce.StalePreparationError):
        target.publish(stale)
    assert target.snapshot() == before
    target.publish(prepared(target, request()))
    assert target.amount("ore") == q(3)


def test_invalid_runtime_law_leaves_existing_engine_usable() -> None:
    target = ce.Engine({"initial"}, [])
    bad = ce.Law("bad", {"x": MASS}, [])
    proposal = ce.Exchange(
        "bad", "bad", {"x": "stock"}, declarations=ce.Model(set(), [bad]),
    )
    before = target.snapshot()
    with pytest.raises(ce.InvalidExchange):
        target.prepare(proposal)
    assert target.snapshot() == before
    target.publish(prepared(target, request()))


def test_runtime_capacity_change_checks_complete_result() -> None:
    target = ce.Engine({"initial"}, [])
    target.publish(prepared(target, request()))
    law = ce.Law("move", {"stock": MASS}, [
        ce.Constraint("mass", ce.Expr.delta("stock"), ce.Expr.constant(q(0))),
    ])

    def move(maximum: int) -> ce.Exchange:
        return ce.Exchange(
            "resize", "move", {"stock": "ore"},
            moves={"ore": ce.Placement("new-owner", {"replacement": ce.Quantity("1", RATIO)})},
            declarations=ce.Model(set(), [law], {"replacement": ce.Capacity(q(maximum))}),
        )

    before = target.snapshot()
    with pytest.raises(ce.CapacityError):
        target.prepare(move(2))
    assert target.snapshot() == before
    target.publish(target.prepare(move(3)))
    assert target.amount("ore") == q(3)
    assert ce.Engine.restore(target.snapshot(), REGISTRY).snapshot() == target.snapshot()


def test_runtime_law_order_is_canonical_and_conflicts_fail() -> None:
    target = ce.Engine({"initial"}, [])
    laws = [ce.Law("a", {}, []), ce.Law("b", {}, [])]
    first = ce.Exchange("register", "a", {}, declarations=ce.Model(set(), laws))
    receipt = target.publish(target.prepare(first))
    reverse = ce.Exchange("register", "a", {}, declarations=ce.Model(set(), list(reversed(laws))))
    assert target.publish(target.prepare(reverse)) == receipt
    before = target.snapshot()
    different = ce.Law("a", {}, [], participants={"initial"})
    bad = ce.Exchange("conflict", "a", {}, declarations=ce.Model(set(), [different]))
    with pytest.raises(ce.InvalidExchange, match="replace law a"):
        target.prepare(bad)
    assert target.snapshot() == before
