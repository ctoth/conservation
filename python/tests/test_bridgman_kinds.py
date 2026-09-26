"""Physical kinds are handles from Bridgman's bundled catalog."""

import pytest

import conservation_exchange as ce


def test_catalog_identity_dimensions_and_floors() -> None:
    kinds = ce.bridgman_kinds()
    assert kinds.kind("mass") == ce.bridgman_kinds().kind("mass")
    assert kinds.kind("volume").dimensions == {"L": "3"}
    assert kinds.kind("specific_volume").dimensions == {"M": "-1", "L": "3"}
    assert kinds.kind("count").floor == "0"
    assert kinds.kind("ratio").floor is None
    assert kinds.kind("count") != kinds.kind("ratio")
    with pytest.raises(ce.UnknownKindError, match="unitless"):
        kinds.kind("unitless")


def test_stock_floor_and_delta_keep_the_declared_kind() -> None:
    count = ce.bridgman_kinds().kind("count")
    law = ce.Law(
        "occupancy", {"item": count},
        [ce.Constraint("source", ce.Expr.delta("item"), ce.Expr.boundary("source"))],
        boundaries={"source": count},
    )
    engine = ce.Engine({"hold"}, [law])
    negative = ce.Exchange(
        "negative", "occupancy", {"item": "seat"},
        creates=[ce.Stock("seat", "hold", count)],
        deltas={"item": ce.Quantity("-1", count)},
        boundaries={"source": ce.Quantity("-1", count)},
    )
    before = engine.snapshot()
    with pytest.raises(ce.DomainError):
        engine.prepare(negative)
    assert engine.snapshot() == before
    ratio = ce.bridgman_kinds().kind("ratio")
    wrong = ce.Exchange(
        "wrong", "occupancy", {"item": "seat"},
        creates=[ce.Stock("seat", "hold", count)],
        deltas={"item": ce.Quantity("1", ratio)},
        boundaries={"source": ce.Quantity("1", count)},
    )
    with pytest.raises(ce.DimensionError, match="count.*ratio|ratio.*count"):
        engine.prepare(wrong)
    assert engine.snapshot() == before
