use std::sync::Arc;

use conservation_dynamics::{
    ExactState, FlowSpec, FlowTopology, ProcessDefinition, ProcessId, Rationing, StockDefinition,
    StockId,
};
use conservation_test_kinds::TestKind;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use proptest::prelude::*;

fn integer(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn id(value: &str) -> StockId {
    StockId::new(value).unwrap()
}

fn process(value: &str) -> ProcessId {
    ProcessId::new(value).unwrap()
}

fn flow(name: &str, source: Option<&str>, target: Option<&str>) -> FlowSpec<TestKind> {
    FlowSpec {
        process: process(name),
        kind: TestKind::Material,
        source: source.map(id),
        target: target.map(id),
    }
}

/// Two material stocks `a` and `b` whose every process rations.
fn system(flows: Vec<FlowSpec<TestKind>>, a: i64, b: i64) -> ExactState<TestKind> {
    let mut names: Vec<_> = flows.iter().map(|flow| flow.process.clone()).collect();
    names.sort();
    names.dedup();
    let topology = FlowTopology::new(
        [
            StockDefinition {
                id: id("a"),
                kind: TestKind::Material,
            },
            StockDefinition {
                id: id("b"),
                kind: TestKind::Material,
            },
        ],
        flows,
        names.into_iter().map(|id| ProcessDefinition {
            id,
            rationing: Rationing::Ration,
        }),
    )
    .unwrap();
    ExactState::new(Arc::new(topology), vec![integer(a), integer(b)]).unwrap()
}

#[test]
fn competing_withdrawals_are_limited_proportionally() {
    let mut state = system(
        vec![
            flow("first", Some("a"), Some("b")),
            flow("export", Some("a"), None),
        ],
        10,
        0,
    );
    let report = state.settle(&[integer(8), integer(12)]).unwrap();

    assert_eq!(report.applied(), &[integer(4), integer(6)]);
    assert_eq!(state.amount(&id("a")), Some(&integer(0)));
    assert_eq!(state.amount(&id("b")), Some(&integer(4)));
    assert!(state.balance_residual(TestKind::Material).is_zero());
}

proptest! {
    #[test]
    fn arbitrary_batches_preserve_nonnegativity_and_exact_balance(
        a in 0_i64..1_000_000,
        b in 0_i64..1_000_000,
        ab in 0_i64..1_000_000,
        ba in 0_i64..1_000_000,
        output_a in 0_i64..1_000_000,
        input_b in 0_i64..1_000_000,
    ) {
        let mut state = system(
            vec![
                flow("ab", Some("a"), Some("b")),
                flow("ba", Some("b"), Some("a")),
                flow("output", Some("a"), None),
                flow("input", None, Some("b")),
            ],
            a,
            b,
        );
        state
            .settle(&[integer(ab), integer(ba), integer(output_a), integer(input_b)])
            .unwrap();

        prop_assert!(!state.amount(&id("a")).unwrap().is_negative());
        prop_assert!(!state.amount(&id("b")).unwrap().is_negative());
        prop_assert!(state.balance_residual(TestKind::Material).is_zero());
    }

    #[test]
    fn flow_permutation_does_not_change_stock_or_boundary_accounts(
        initial in 0_i64..1_000_000,
        first in 0_i64..1_000_000,
        second in 0_i64..1_000_000,
        output in 0_i64..1_000_000,
    ) {
        let flows = vec![
            flow("first", Some("a"), Some("b")),
            flow("second", Some("a"), Some("b")),
            flow("output", Some("a"), None),
        ];
        let requests = [integer(first), integer(second), integer(output)];
        let mut forward = system(flows.clone(), initial, 0);
        forward.settle(&requests).unwrap();
        let mut reverse = system(flows.into_iter().rev().collect(), initial, 0);
        reverse
            .settle(&requests.into_iter().rev().collect::<Vec<_>>())
            .unwrap();

        prop_assert_eq!(forward.amount(&id("a")), reverse.amount(&id("a")));
        prop_assert_eq!(forward.amount(&id("b")), reverse.amount(&id("b")));
        prop_assert_eq!(forward.inputs(TestKind::Material), reverse.inputs(TestKind::Material));
        prop_assert_eq!(forward.outputs(TestKind::Material), reverse.outputs(TestKind::Material));
    }

    #[test]
    fn splitting_a_same_role_flow_preserves_observable_state(
        initial in 0_i64..1_000_000,
        first in 0_i64..1_000_000,
        second in 0_i64..1_000_000,
    ) {
        let mut merged = system(vec![flow("move", Some("a"), Some("b"))], initial, 0);
        merged.settle(&[integer(first + second)]).unwrap();
        let mut split = system(
            vec![
                flow("move", Some("a"), Some("b")),
                flow("move", Some("a"), Some("b")),
            ],
            initial,
            0,
        );
        split.settle(&[integer(first), integer(second)]).unwrap();

        prop_assert_eq!(merged.amount(&id("a")), split.amount(&id("a")));
        prop_assert_eq!(merged.amount(&id("b")), split.amount(&id("b")));
        prop_assert_eq!(
            merged.balance_residual(TestKind::Material),
            split.balance_residual(TestKind::Material)
        );
    }
}
