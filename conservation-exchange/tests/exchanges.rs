use std::collections::{BTreeMap, BTreeSet};

use conservation_exchange::*;
use num_rational::BigRational;

fn q(value: i64) -> Quantity {
    Quantity::new(
        BigRational::from_integer(value.into()),
        Dimension::base("material").unwrap(),
    )
}

fn spec(id: &str) -> Stock {
    Stock {
        id: id.into(),
        owner: "owner".into(),
        dimension: q(0).dimension,
        domain: Domain::Nonnegative,
        capacities: BTreeMap::new(),
    }
}

fn equation(id: &str, left: Expr, right: Expr) -> Constraint {
    Constraint::equal(id, left, right)
}

fn model() -> Model {
    Model {
        owners: BTreeSet::from(["owner".into(), "journal".into()]),
        capacities: BTreeMap::new(),
        laws: vec![
            Law {
                id: "supply".into(),
                slots: BTreeMap::from([("stock".into(), q(0).dimension)]),
                boundaries: BTreeMap::from([("supply".into(), q(0).dimension)]),
                facts: BTreeMap::new(),
                participants: BTreeSet::new(),
                constraints: vec![equation(
                    "supply",
                    Expr::delta("stock"),
                    Expr::boundary("supply"),
                )],
            },
            Law {
                id: "combine".into(),
                slots: ["a", "b", "out"]
                    .into_iter()
                    .map(|id| (id.into(), q(0).dimension))
                    .collect(),
                boundaries: BTreeMap::new(),
                facts: BTreeMap::new(),
                participants: BTreeSet::from(["journal".into()]),
                constraints: vec![
                    equation("stoichiometry", Expr::delta("a"), Expr::delta("b")),
                    equation(
                        "balance",
                        Expr::sum([Expr::delta("a"), Expr::delta("b"), Expr::delta("out")]),
                        Expr::constant(q(0)),
                    ),
                ],
            },
        ],
    }
}

fn supply(engine: &mut Engine, id: &str, amount: i64) {
    let mut proposal = Exchange::new(format!("supply-{id}"), "supply");
    proposal.bindings.insert("stock".into(), id.into());
    proposal.deltas.insert("stock".into(), q(amount));
    proposal.boundaries.insert("supply".into(), q(amount));
    proposal.creates.push(spec(id));
    let prepared = engine.prepare(proposal, vec![]).unwrap();
    engine.publish(prepared).unwrap();
}

fn combine(id: &str, amount: i64) -> Exchange {
    let mut proposal = Exchange::new(id, "combine");
    proposal.bindings = ["a", "b", "out"]
        .into_iter()
        .map(|id| (id.into(), id.into()))
        .collect();
    proposal.deltas = BTreeMap::from([
        ("a".into(), q(-amount)),
        ("b".into(), q(-amount)),
        ("out".into(), q(2 * amount)),
    ]);
    proposal
}

fn journal(engine: &Engine, proposal: &Exchange) -> Participation {
    engine
        .participate(
            "journal",
            proposal,
            vec![RecordWrite {
                key: proposal.id.clone(),
                value: Some(b"accepted".to_vec()),
            }],
            BTreeMap::new(),
        )
        .unwrap()
}

#[test]
fn missing_second_input_preserves_every_stock_and_record() {
    let mut engine = Engine::new(model()).unwrap();
    supply(&mut engine, "a", 10);
    supply(&mut engine, "b", 0);
    supply(&mut engine, "out", 0);
    let before = engine.snapshot().unwrap();
    let proposal = combine("reaction", 2);
    let evidence = journal(&engine, &proposal);
    assert!(matches!(
        engine.prepare(proposal, vec![evidence]),
        Err(Error::Domain { .. })
    ));
    assert_eq!(engine.snapshot().unwrap(), before);
    assert_eq!(engine.record("journal", "reaction"), None);
}

#[test]
fn publication_is_one_root_and_stale_competitor_cannot_spend() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let first = combine("first", 7);
    let second = combine("second", 7);
    let prepared_first = engine
        .prepare(first.clone(), vec![journal(&engine, &first)])
        .unwrap();
    let prepared_second = engine
        .prepare(second.clone(), vec![journal(&engine, &second)])
        .unwrap();
    assert_eq!(engine.amount("a").unwrap(), &q(10).amount);
    assert_eq!(engine.record("journal", "first"), None);
    engine.publish(prepared_first).unwrap();
    assert_eq!(engine.amount("a").unwrap(), &q(3).amount);
    assert_eq!(
        engine.record("journal", "first"),
        Some(b"accepted".as_slice())
    );
    let before = engine.snapshot().unwrap();
    assert!(matches!(engine.publish(prepared_second), Err(Error::Stale)));
    assert_eq!(engine.snapshot().unwrap(), before);
}

#[test]
fn duplicate_restore_and_conflicting_reuse_are_distinct() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let proposal = combine("first", 2);
    let prepared = engine
        .prepare(proposal.clone(), vec![journal(&engine, &proposal)])
        .unwrap();
    let first = engine.publish(prepared.clone()).unwrap();
    assert_eq!(engine.publish(prepared.clone()).unwrap(), first);
    let bytes = engine.snapshot().unwrap();
    let mut restored = Engine::restore(&bytes).unwrap();
    assert_eq!(restored.snapshot().unwrap(), bytes);
    assert!(matches!(
        restored.publish(prepared),
        Err(Error::ForeignPreparation)
    ));
    let duplicate = restored
        .prepare(proposal.clone(), vec![journal(&restored, &proposal)])
        .unwrap();
    assert_eq!(restored.publish(duplicate).unwrap(), first);
    let conflict = combine("first", 3);
    assert!(matches!(
        restored.prepare(conflict.clone(), vec![journal(&restored, &conflict)]),
        Err(Error::Duplicate { .. })
    ));
}

#[test]
fn missing_required_participant_and_changed_binding_fail_before_publication() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let proposal = combine("first", 2);
    assert!(matches!(
        engine.prepare(proposal.clone(), vec![]),
        Err(Error::Participant { .. })
    ));
    let evidence = journal(&engine, &proposal);
    let changed = combine("first", 3);
    assert!(matches!(
        engine.prepare(changed, vec![evidence]),
        Err(Error::Participant { .. })
    ));
}

fn supply_stock(engine: &mut Engine, stock: Stock, amount: Quantity, law: &str) {
    let mut proposal = Exchange::new(format!("supply-{}", stock.id), law);
    proposal.bindings.insert("stock".into(), stock.id.clone());
    proposal.deltas.insert("stock".into(), amount.clone());
    proposal.boundaries.insert("supply".into(), amount);
    proposal.creates.push(stock);
    let prepared = engine.prepare(proposal, vec![]).unwrap();
    engine.publish(prepared).unwrap();
}

fn one() -> Quantity {
    Quantity::new(
        BigRational::from_integer(1.into()),
        Dimension::dimensionless(),
    )
}

#[test]
fn capacity_uses_whole_result_and_rejects_without_losing_inputs() {
    let mut declaration = model();
    declaration
        .capacities
        .insert("hold".into(), Capacity { maximum: q(20) });
    declaration
        .capacities
        .insert("product-bin".into(), Capacity { maximum: q(3) });
    let mut engine = Engine::new(declaration).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        let mut stock = spec(id);
        stock.capacities.insert("hold".into(), one());
        if id == "out" {
            stock.capacities.insert("product-bin".into(), one());
        }
        supply_stock(&mut engine, stock, q(amount), "supply");
    }
    let proposal = combine("fits", 1);
    // The hold starts full: products fit only when consumed input capacity is released.
    let prepared = engine
        .prepare(proposal.clone(), vec![journal(&engine, &proposal)])
        .unwrap();
    engine.publish(prepared).unwrap();
    let before = engine.snapshot().unwrap();
    let rejected = combine("overflow", 1);
    assert!(
        matches!(engine.prepare(rejected.clone(), vec![journal(&engine, &rejected)]), Err(Error::Capacity { id, .. }) if id == "product-bin")
    );
    assert_eq!(engine.snapshot().unwrap(), before);
}

#[test]
fn signed_coordinate_is_distinct_from_nonnegative_stock() {
    let mut engine = Engine::new(model()).unwrap();
    let mut signed = spec("signed");
    signed.domain = Domain::Signed;
    supply_stock(&mut engine, signed, q(-4), "supply");
    assert_eq!(engine.amount("signed").unwrap(), &q(-4).amount);
    let mut proposal = Exchange::new("negative-material", "supply");
    proposal.bindings.insert("stock".into(), "material".into());
    proposal.creates.push(spec("material"));
    proposal.deltas.insert("stock".into(), q(-1));
    proposal.boundaries.insert("supply".into(), q(-1));
    assert!(matches!(
        engine.prepare(proposal, vec![]),
        Err(Error::Domain { .. })
    ));
    assert!(engine.stock("material").is_err());
}

#[test]
fn coupled_conversion_resolves_tiny_release_beside_huge_rest_stock() {
    let energy = Dimension::base("energy").unwrap();
    let material = q(0).dimension;
    let mut declaration = model();
    let mut energy_supply = declaration.laws[0].clone();
    energy_supply.id = "energy-supply".into();
    energy_supply.slots.insert("stock".into(), energy.clone());
    energy_supply
        .boundaries
        .insert("supply".into(), energy.clone());
    declaration.laws.push(energy_supply);
    let conversion = Quantity::new(
        BigRational::from_integer(90_000_000_000_000_000_i64.into()),
        energy.quotient(&material).unwrap(),
    );
    declaration.laws.push(Law {
        id: "conversion".into(),
        slots: BTreeMap::from([
            ("mass".into(), material.clone()),
            ("energy".into(), energy.clone()),
        ]),
        boundaries: BTreeMap::new(),
        facts: BTreeMap::new(),
        participants: BTreeSet::new(),
        constraints: vec![equation(
            "mass-energy",
            Expr::sum([
                Expr::delta("mass").product(Expr::constant(conversion)),
                Expr::delta("energy"),
            ]),
            Expr::constant(Quantity::new(q(0).amount, energy.clone())),
        )],
    });
    let mut engine = Engine::new(declaration).unwrap();
    let huge = BigRational::from_integer(num_bigint::BigInt::from(10).pow(30));
    supply_stock(
        &mut engine,
        spec("fuel"),
        Quantity::new(huge.clone(), material.clone()),
        "supply",
    );
    let mut electric = spec("electric");
    electric.dimension = energy.clone();
    supply_stock(
        &mut engine,
        electric,
        Quantity::new(q(0).amount, energy.clone()),
        "energy-supply",
    );
    let mut proposal = Exchange::new("convert", "conversion");
    proposal.bindings = BTreeMap::from([
        ("mass".into(), "fuel".into()),
        ("energy".into(), "electric".into()),
    ]);
    let consumed = BigRational::new(1.into(), 1_000_000_000_000_000_000_i64.into());
    proposal
        .deltas
        .insert("mass".into(), Quantity::new(-consumed.clone(), material));
    let before = engine.snapshot().unwrap();
    assert!(
        matches!(engine.prepare(proposal.clone(), vec![]), Err(Error::Constraint { id, .. }) if id == "mass-energy")
    );
    assert_eq!(engine.snapshot().unwrap(), before);
    proposal.deltas.insert(
        "energy".into(),
        Quantity::new(BigRational::new(9.into(), 100.into()), energy),
    );
    let prepared = engine.prepare(proposal, vec![]).unwrap();
    engine.publish(prepared).unwrap();
    assert_eq!(engine.amount("fuel").unwrap(), &(huge - consumed));
    assert_eq!(
        engine.amount("electric").unwrap(),
        &BigRational::new(9.into(), 100.into())
    );
}

#[test]
fn law_dimension_unknown_boundary_and_bad_coefficients_are_rejected() {
    let mut invalid_model = model();
    invalid_model.laws[1]
        .slots
        .insert("a".into(), Dimension::base("energy").unwrap());
    assert!(matches!(
        Engine::new(invalid_model),
        Err(Error::Dimension { .. })
    ));
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let mut proposal = combine("unequal-inputs", 2);
    proposal.deltas.insert("a".into(), q(-1));
    assert!(
        matches!(engine.prepare(proposal.clone(), vec![journal(&engine, &proposal)]), Err(Error::Constraint { id, .. }) if id == "stoichiometry")
    );
    proposal.boundaries.insert("magic".into(), q(1));
    assert!(
        matches!(engine.prepare(proposal, vec![]), Err(Error::Invalid(message)) if message.contains("undeclared boundary"))
    );
}

#[test]
fn evaluated_facts_are_owned_and_bound_to_exact_proposal_and_revision() {
    let mut declaration = model();
    declaration.laws[1].facts.insert(
        "limit".into(),
        Fact {
            owner: "journal".into(),
            dimension: q(0).dimension,
        },
    );
    let mut limit = equation("owner-limit", Expr::delta("out"), Expr::fact("limit"));
    limit.relation = Relation::LessOrEqual;
    declaration.laws[1].constraints.push(limit);
    let mut engine = Engine::new(declaration).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let proposal = combine("first", 2);
    let facts = BTreeMap::from([("limit".into(), q(4))]);
    let wrong_owner = engine
        .participate("owner", &proposal, vec![], facts.clone())
        .unwrap();
    assert!(matches!(
        engine.prepare(
            proposal.clone(),
            vec![wrong_owner, journal(&engine, &proposal)]
        ),
        Err(Error::Participant { .. })
    ));
    let approved = engine
        .participate("journal", &proposal, vec![], facts)
        .unwrap();
    supply(&mut engine, "other", 1);
    assert!(matches!(
        engine.prepare(proposal, vec![approved]),
        Err(Error::Stale)
    ));
}

#[test]
fn topology_preserves_quantity_identity_and_retires_only_accounted_empty_stocks() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let mut bad = combine("bad-remove", 0);
    bad.removes.insert("a".into());
    assert!(
        matches!(engine.prepare(bad.clone(), vec![journal(&engine, &bad)]), Err(Error::Invalid(message)) if message.contains("nonempty"))
    );
    let mut valid = combine("consume-and-move", 10);
    valid.removes = BTreeSet::from(["a".into(), "b".into()]);
    valid.moves.insert(
        "out".into(),
        Placement {
            owner: "journal".into(),
            capacities: BTreeMap::new(),
        },
    );
    let prepared = engine
        .prepare(valid.clone(), vec![journal(&engine, &valid)])
        .unwrap();
    engine.publish(prepared).unwrap();
    assert_eq!(engine.stock("out").unwrap().owner, "journal");
    assert_eq!(engine.amount("out").unwrap(), &q(20).amount);
    assert!(engine.stock("a").is_err());
    let mut reuse = Exchange::new("reuse", "supply");
    reuse.bindings.insert("stock".into(), "a".into());
    reuse.creates.push(spec("a"));
    assert!(
        matches!(engine.prepare(reuse, vec![]), Err(Error::Invalid(message)) if message.contains("already used"))
    );
    let restored = Engine::restore(&engine.snapshot().unwrap()).unwrap();
    assert_eq!(restored.stock("out").unwrap().owner, "journal");
}

#[test]
fn rejected_record_change_does_not_publish_quantities_or_earlier_record_writes() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let proposal = combine("rejected-journal", 2);
    let writes = vec![
        RecordWrite {
            key: "a-valid".into(),
            value: Some(b"new".to_vec()),
        },
        RecordWrite {
            key: "z-missing".into(),
            value: None,
        },
    ];
    let part = engine
        .participate("journal", &proposal, writes, BTreeMap::new())
        .unwrap();
    let before = engine.snapshot().unwrap();
    assert!(matches!(
        engine.prepare(proposal, vec![part]),
        Err(Error::Participant { .. })
    ));
    assert_eq!(engine.snapshot().unwrap(), before);
}

#[test]
fn restore_revalidates_history_and_does_not_accept_unaccounted_balances() {
    let mut engine = Engine::new(model()).unwrap();
    supply(&mut engine, "a", 10);
    let bytes = engine.snapshot().unwrap();
    let mut snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    snapshot["version"] = serde_json::json!(2);
    assert!(matches!(
        Engine::restore(&serde_json::to_vec(&snapshot).unwrap()),
        Err(Error::Snapshot(_))
    ));
    snapshot["version"] = serde_json::json!(1);
    snapshot["requests"][0]["exchange"]["boundaries"] = serde_json::json!({});
    assert!(matches!(
        Engine::restore(&serde_json::to_vec(&snapshot).unwrap()),
        Err(Error::Constraint { .. })
    ));
    let mut snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    snapshot["balances"] = serde_json::json!({"a": 1000});
    assert!(matches!(
        Engine::restore(&serde_json::to_vec(&snapshot).unwrap()),
        Err(Error::Snapshot(_))
    ));
}

#[test]
fn nonlinear_derived_constraint_checks_motion_without_a_second_energy_stock() {
    let mut declaration = model();
    let mut nonlinear = declaration.laws[0].clone();
    nonlinear.id = "bounded-square".into();
    let squared = q(0).dimension.product(&q(0).dimension).unwrap();
    let mut constraint = equation(
        "square-budget",
        Expr::after("stock").product(Expr::after("stock")),
        Expr::constant(Quantity::new(q(25).amount, squared)),
    );
    constraint.relation = Relation::LessOrEqual;
    nonlinear.constraints.push(constraint);
    declaration.laws.push(nonlinear);
    let mut engine = Engine::new(declaration).unwrap();
    supply(&mut engine, "x", 4);
    let mut proposal = Exchange::new("overshoot", "bounded-square");
    proposal.bindings.insert("stock".into(), "x".into());
    proposal.deltas.insert("stock".into(), q(2));
    proposal.boundaries.insert("supply".into(), q(2));
    assert!(
        matches!(engine.prepare(proposal, vec![]), Err(Error::Constraint { id, residual, .. }) if id == "square-budget" && residual == q(11).amount)
    );
    assert_eq!(engine.amount("x").unwrap(), &q(4).amount);
}

#[test]
fn foreign_preparation_and_participation_do_not_cross_engine_boundaries() {
    let mut first = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut first, id, amount);
    }
    let mut second = Engine::restore(&first.snapshot().unwrap()).unwrap();
    let proposal = combine("foreign", 2);
    let participation = journal(&first, &proposal);
    assert!(matches!(
        second.prepare(proposal.clone(), vec![participation.clone()]),
        Err(Error::ForeignPreparation)
    ));
    let prepared = first.prepare(proposal, vec![participation]).unwrap();
    assert!(matches!(
        second.publish(prepared),
        Err(Error::ForeignPreparation)
    ));
}

#[test]
fn missing_sink_and_slot_alias_cannot_turn_a_transform_into_a_partial_flow() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let mut proposal = combine("missing-sink", 2);
    proposal.deltas.remove("out");
    assert!(
        matches!(engine.prepare(proposal.clone(), vec![journal(&engine, &proposal)]), Err(Error::Constraint { id, .. }) if id == "balance")
    );
    proposal.bindings.insert("b".into(), "a".into());
    assert!(
        matches!(engine.prepare(proposal, vec![]), Err(Error::Invalid(message)) if message.contains("alias"))
    );
}

#[test]
fn creation_and_record_order_have_one_canonical_request_identity() {
    let mut engine = Engine::new(model()).unwrap();
    let mut proposal = combine("empty", 0);
    proposal.creates = vec![spec("out"), spec("b"), spec("a")];
    let writes = vec![
        RecordWrite {
            key: "z".into(),
            value: Some(vec![2]),
        },
        RecordWrite {
            key: "a".into(),
            value: Some(vec![1]),
        },
    ];
    let part = engine
        .participate("journal", &proposal, writes.clone(), BTreeMap::new())
        .unwrap();
    let prepared = engine.prepare(proposal.clone(), vec![part]).unwrap();
    let committed = engine.publish(prepared).unwrap();
    proposal.creates.reverse();
    let part = engine
        .participate(
            "journal",
            &proposal,
            writes.into_iter().rev().collect(),
            BTreeMap::new(),
        )
        .unwrap();
    let repeated = engine.prepare(proposal, vec![part]).unwrap();
    assert_eq!(engine.publish(repeated).unwrap(), committed);
}

#[test]
fn record_updates_are_canonical_state_and_restore_preserves_them() {
    let mut engine = Engine::new(model()).unwrap();
    for (id, amount) in [("a", 10), ("b", 10), ("out", 0)] {
        supply(&mut engine, id, amount);
    }
    let proposal = combine("committed", 1);
    let prepared = engine
        .prepare(proposal.clone(), vec![journal(&engine, &proposal)])
        .unwrap();
    let receipt = engine.publish(prepared.clone()).unwrap();
    assert_eq!(engine.receipt("committed"), Some(&receipt));
    assert_eq!(engine.publish(prepared).unwrap(), receipt);
    let update = combine("update-record", 0);
    let part = engine
        .participate(
            "journal",
            &update,
            vec![RecordWrite {
                key: "committed".into(),
                value: Some(b"reconciled".to_vec()),
            }],
            BTreeMap::new(),
        )
        .unwrap();
    let prepared = engine.prepare(update, vec![part]).unwrap();
    engine.publish(prepared).unwrap();
    let restored = Engine::restore(&engine.snapshot().unwrap()).unwrap();
    assert_eq!(
        restored.record("journal", "committed"),
        Some(b"reconciled".as_slice())
    );
}

proptest::proptest! {
    #[test]
    fn whole_exchange_is_exact_and_replay_preserves_the_committed_cut(a in 0i64..1000, b in 0i64..1000, draw in 0i64..1500) {
        let mut engine = Engine::new(model()).unwrap();
        for (id, amount) in [("a", a), ("b", b), ("out", 0)] { supply(&mut engine, id, amount); }
        let before = engine.snapshot().unwrap();
        let proposal = combine("property", draw);
        let attempt = engine.prepare(proposal.clone(), vec![journal(&engine, &proposal)]);
        if draw <= a && draw <= b {
            engine.publish(attempt.unwrap()).unwrap();
            proptest::prop_assert_eq!(engine.amount("a").unwrap() + engine.amount("b").unwrap() + engine.amount("out").unwrap(), q(a + b).amount);
            let restored = Engine::restore(&engine.snapshot().unwrap()).unwrap();
            proptest::prop_assert_eq!(restored.snapshot().unwrap(), engine.snapshot().unwrap());
        } else {
            proptest::prop_assert!(matches!(attempt, Err(Error::Domain { .. })), "shortage must reject the entire exchange");
            proptest::prop_assert_eq!(engine.snapshot().unwrap(), before);
        }
    }
}
