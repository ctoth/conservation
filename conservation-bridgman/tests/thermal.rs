use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bridgman_core::{DerivationError, Grade, Op, ProductOp, QuantityError, Registry};
use conservation_bridgman::*;
use conservation_core::{Affine, DimensionAlgebra, Kind, KindRegistry};
use conservation_dynamics::{
    ExactState, FlowSpec, FlowTopology, ProcessDefinition, ProcessId, Rationing, StockDefinition,
    StockFlowError, StockId,
};
use conservation_exchange::{KindContext, Law};
use num_rational::BigRational;

type Quantity = conservation_exchange::Quantity<BridgmanKind>;
type Stock = conservation_exchange::Stock<BridgmanKind>;
type Exchange = conservation_exchange::Exchange<BridgmanKind>;
type Engine = conservation_exchange::Engine<BridgmanKind>;
type Error = conservation_exchange::Error<BridgmanKind>;
type Expr = conservation_exchange::Expr<BridgmanKind>;
type Constraint = conservation_exchange::Constraint<BridgmanKind>;
type Model = conservation_exchange::Model<BridgmanKind>;

fn thermal() -> BridgmanKinds {
    BridgmanKinds::thermal().unwrap()
}

fn kind(id: &str) -> BridgmanKind {
    thermal().resolve(id).unwrap()
}

fn rational(v: i64) -> BigRational {
    BigRational::from_integer(v.into())
}

/// A law whose one slot `stock` and one boundary `supply` both have `kind`,
/// and whose constraint is that the stock's delta equals the supply.
fn supply_law(id: &str, kind: BridgmanKind) -> Law<BridgmanKind> {
    Law {
        id: id.into(),
        slots: BTreeMap::from([("stock".into(), kind)]),
        boundaries: BTreeMap::from([("supply".into(), kind)]),
        facts: BTreeMap::new(),
        participants: BTreeSet::new(),
        constraints: vec![Constraint::equal(
            id,
            Expr::delta("stock"),
            Expr::boundary("supply"),
        )],
    }
}

fn model(laws: Vec<Law<BridgmanKind>>) -> Model {
    Model {
        owners: BTreeSet::from(["owner".into()]),
        capacities: BTreeMap::new(),
        laws,
    }
}

fn stock(id: &str, kind: BridgmanKind) -> Stock {
    Stock {
        id: id.into(),
        owner: "owner".into(),
        kind,
        capacities: BTreeMap::new(),
    }
}

/// An exchange under `law` that creates `stock`, binds it to the slot, and
/// moves `amount` as both its delta and the supply.
fn withdrawal(id: &str, stock: Stock, law: &str, amount: Quantity) -> Exchange {
    let mut proposal = Exchange::new(id, law);
    proposal.bindings.insert("stock".into(), stock.id.clone());
    proposal.creates.push(stock);
    proposal.deltas.insert("stock".into(), amount.clone());
    proposal.boundaries.insert("supply".into(), amount);
    proposal
}

#[test]
fn enthalpy_stock_runs_through_dynamics() {
    let (enthalpy, energy) = (kind("enthalpy"), kind("energy"));
    // Bridgman declares power the rate of energy: power over a duration is
    // energy, which is the difference of enthalpy.
    assert_eq!(kind("power").bridgman().rate_of(), Some(energy.bridgman()));
    let moved = kind("power")
        .bridgman()
        .combine(Op::Mul, kind("duration").bridgman())
        .unwrap();
    assert_eq!(thermal().of(moved), Ok(enthalpy.difference()));
    assert_eq!(enthalpy.floor(), None);

    let kettle = StockId::new("kettle").unwrap();
    let room = StockId::new("room").unwrap();
    let conduction = ProcessId::new("conduction").unwrap();
    let conducting = |kind| {
        FlowTopology::new(
            [
                StockDefinition {
                    id: kettle.clone(),
                    kind: enthalpy,
                },
                StockDefinition {
                    id: room.clone(),
                    kind: enthalpy,
                },
            ],
            [FlowSpec {
                process: conduction.clone(),
                kind,
                source: Some(kettle.clone()),
                target: Some(room.clone()),
            }],
            [ProcessDefinition {
                id: conduction.clone(),
                rationing: Rationing::Refuse,
            }],
        )
    };
    let topology = Arc::new(conducting(energy).unwrap());
    let mut state = ExactState::new(topology.clone(), vec![rational(500), rational(100)]).unwrap();
    let report = state.settle(&[rational(600)]).unwrap();
    // Enthalpy declares no minimum, so the withdrawal is not limited.
    assert_eq!(
        topology
            .materialize_exact_report(&report)
            .unwrap()
            .applied_by(&conduction),
        rational(600)
    );
    assert_eq!(state.amount(&kettle), Some(&rational(-100)));
    assert_eq!(state.amount(&room), Some(&rational(700)));
    // Enthalpy is a point kind: its values do not sum, through Bridgman's role.
    let refused = StockFlowError::PointSum { kind: enthalpy };
    assert_eq!(state.total(enthalpy), Err(refused.clone()));
    assert_eq!(state.balance_residual(enthalpy), Err(refused));
    assert!(matches!(
        conducting(enthalpy),
        Err(StockFlowError::KindMismatch { stock_kind, flow_kind, .. })
            if stock_kind == enthalpy && flow_kind == enthalpy
    ));
}

#[test]
fn exchange_leg_of_another_kind_names_both_bridgman_kinds() {
    let (energy, torque, mass) = (kind("energy"), kind("torque"), kind("mass"));
    assert_eq!(
        energy.dimensions().dimensions,
        torque.dimensions().dimensions
    );
    let engine = Engine::new(model(vec![
        supply_law("energy-supply", energy),
        supply_law("mass-supply", mass),
    ]))
    .unwrap();

    let spanner = withdrawal(
        "spanner-supply",
        stock("spanner", torque),
        "energy-supply",
        Quantity::new(rational(0), energy),
    );
    let error = engine.prepare(spanner, vec![]).err().unwrap();
    assert_eq!(
        error,
        Error::Kinds {
            context: KindContext::Slot {
                slot: "stock".into(),
                stock: "spanner".into(),
            },
            expected: energy,
            found: torque,
        }
    );
    let message = error.to_string();
    assert!(message.contains("energy") && message.contains("torque"));
    assert!(engine.stock("spanner").is_err());

    assert_ne!(mass.dimensions(), energy.dimensions());
    let battery = withdrawal(
        "battery-supply",
        stock("battery", energy),
        "mass-supply",
        Quantity::new(rational(0), mass),
    );
    assert!(matches!(
        engine.prepare(battery, vec![]),
        Err(Error::Kinds { expected, found, .. }) if expected == mass && found == energy
    ));
}

#[test]
fn thermal_kinds_report_role_floor_and_difference() {
    let (enthalpy, energy) = (kind("enthalpy"), kind("energy"));
    let (temperature, temperature_delta) = (kind("temperature"), kind("temperature_delta"));
    assert_eq!(enthalpy.affine(), Affine::Point { difference: energy });
    assert_eq!(
        temperature.affine(),
        Affine::Point {
            difference: temperature_delta
        }
    );
    assert_eq!(temperature_delta.affine(), Affine::Linear);
    assert_eq!(energy.affine(), Affine::Linear);
    assert_eq!(kind("mass").floor(), Some(rational(0)));
    assert_eq!(temperature.floor(), Some(rational(0)));
    assert_eq!(enthalpy.floor(), None);
    assert_eq!(energy.floor(), None);
    assert_eq!(enthalpy.difference(), energy);
}

#[test]
fn dimensions_carry_grade_and_points() {
    let (energy, torque, mass) = (kind("energy"), kind("torque"), kind("mass"));
    let (specific_energy, velocity) = (kind("specific_energy"), kind("velocity"));
    assert_ne!(energy.dimensions(), torque.dimensions());
    assert_eq!(energy.dimensions().grade, Grade::Scalar);
    assert_eq!(torque.dimensions().grade, Grade::Bivector);
    assert_eq!(
        BridgmanKind::product(&mass.dimensions(), &specific_energy.dimensions()),
        Ok(energy.dimensions())
    );
    assert_eq!(
        BridgmanKind::quotient(&energy.dimensions(), &mass.dimensions()),
        Ok(specific_energy.dimensions())
    );
    assert_eq!(
        BridgmanKind::product(&velocity.dimensions(), &velocity.dimensions()),
        Err(BridgmanAlgebraError::Ungraded {
            op: ProductOp::Mul,
            left: Grade::Vector,
            right: Grade::Vector,
        })
    );
    assert!(matches!(
        BridgmanKind::product(&kind("enthalpy").dimensions(), &mass.dimensions()),
        Err(BridgmanAlgebraError::Point {
            op: ProductOp::Mul,
            ..
        })
    ));
}

#[test]
fn registry_resolves_every_name_it_displays() {
    for k in thermal().kinds() {
        assert_eq!(thermal().resolve(&k.to_string()), Some(k));
    }
    assert_eq!(thermal().resolve("Energy"), None);
    assert_eq!(thermal().kinds().len(), 25);
}

#[test]
fn typed_kinds_cross_only_their_own_registry() {
    assert_eq!(
        thermal().of(bridgman_core::thermal().kind("energy").unwrap()),
        Ok(kind("energy"))
    );
    let leaked = BridgmanKinds::leak(bridgman_core::thermal().clone()).unwrap();
    assert_ne!(leaked.resolve("energy").unwrap(), kind("energy"));
    assert!(matches!(
        thermal().of(leaked.resolve("energy").unwrap().bridgman()),
        Err(BridgmanKindError::ForeignRegistry { .. })
    ));
}

#[test]
fn unresolved_dimensions_are_refused_at_admission() {
    let registry = Registry::from_yaml("schema: 4\nkinds:\n  - {id: vague}\nunits: []\n");
    assert!(registry.is_ok());
    let error = BridgmanKinds::leak(registry.unwrap()).err().unwrap();
    let BridgmanKindError::Dimensions { kind: vague, .. } = &error else {
        panic!("expected unresolved dimensions, got {error:?}");
    };
    assert_eq!(vague.id(), "vague");
    let unresolved =
        QuantityError::Derivation(DerivationError::UnresolvedDimensions { kind: *vague });
    assert!(matches!(
        &error,
        BridgmanKindError::Dimensions { source, .. } if **source == unresolved
    ));
    assert_eq!(
        std::error::Error::source(&error).and_then(|source| source.downcast_ref::<QuantityError>()),
        Some(&unresolved)
    );
}

#[test]
fn exchange_floor_comes_from_bridgman_minimum() {
    let mass = kind("mass");
    let engine = Engine::new(model(vec![supply_law("mass-supply", mass)])).unwrap();
    let sack = withdrawal(
        "sack-supply",
        stock("sack", mass),
        "mass-supply",
        Quantity::new(rational(-1), mass),
    );
    assert_eq!(
        engine.prepare(sack, vec![]).err(),
        Some(Error::BelowFloor {
            stock: "sack".into(),
            kind: mass,
            amount: rational(-1),
        })
    );
}

#[test]
fn exchange_snapshot_restores_bridgman_kinds_by_name() {
    let mass = kind("mass");
    let mut engine = Engine::new(model(vec![supply_law("mass-supply", mass)])).unwrap();
    let sack = withdrawal(
        "sack-supply",
        stock("sack", mass),
        "mass-supply",
        Quantity::new(rational(5), mass),
    );
    let prepared = engine.prepare(sack, vec![]).unwrap();
    engine.publish(prepared).unwrap();
    let bytes = engine.snapshot().unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("\"kind\":\"mass\""));
    assert_eq!(
        Engine::restore(&bytes, &thermal())
            .unwrap()
            .snapshot()
            .unwrap(),
        bytes
    );
}
