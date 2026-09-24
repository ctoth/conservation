use conservation_dynamics::{
    ProcessDefinition, ProcessId, ProposedFlow, Rationing, StockFlowError, StockFlowSystem,
    StockId, StockSpec,
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

fn system(a: i64, b: i64) -> StockFlowSystem<TestKind> {
    StockFlowSystem::new(
        [
            StockSpec {
                id: id("a"),
                kind: TestKind::Material,
                initial: integer(a),
            },
            StockSpec {
                id: id("b"),
                kind: TestKind::Material,
                initial: integer(b),
            },
        ],
        [
            "ab", "ba", "export", "first", "input", "invalid", "move", "output", "second", "valid",
        ]
        .map(|name| ProcessDefinition {
            id: process(name),
            rationing: Rationing::Ration,
        }),
    )
    .unwrap()
}

fn transfer(name: &str, source: &str, target: &str, amount: i64) -> ProposedFlow<TestKind> {
    ProposedFlow {
        process: process(name),
        kind: TestKind::Material,
        source: Some(id(source)),
        target: Some(id(target)),
        amount: integer(amount),
    }
}

#[test]
fn competing_withdrawals_are_limited_proportionally() {
    let mut state = system(10, 0);
    let report = state
        .settle(&[
            transfer("first", "a", "b", 8),
            ProposedFlow {
                process: process("export"),
                kind: TestKind::Material,
                source: Some(id("a")),
                target: None,
                amount: integer(12),
            },
        ])
        .unwrap();

    assert_eq!(report.flows()[0].applied, integer(4));
    assert_eq!(report.flows()[1].applied, integer(6));
    assert_eq!(state.amount(&id("a")), Some(&integer(0)));
    assert_eq!(state.amount(&id("b")), Some(&integer(4)));
    assert!(state.balance_residual(TestKind::Material).is_zero());
}

#[test]
fn a_malformed_batch_is_atomic() {
    let mut state = system(10, 0);
    let before = state.clone();
    let error = state
        .settle(&[
            transfer("valid", "a", "b", 2),
            transfer("invalid", "missing", "b", 1),
        ])
        .unwrap_err();
    assert!(matches!(error, StockFlowError::UnknownStock(_)));
    assert_eq!(state, before);
}

fn signed_system(stocks: &[(&str, i64)]) -> StockFlowSystem<TestKind> {
    StockFlowSystem::new(
        stocks.iter().map(|(name, initial)| StockSpec {
            id: id(name),
            kind: TestKind::MaterialBalance,
            initial: integer(*initial),
        }),
        [],
    )
    .unwrap()
}

#[test]
fn floorless_stock_accepts_negative_initial_amount_and_settles_below_zero() {
    let mut state = signed_system(&[("a", -2)]);
    state
        .settle(&[ProposedFlow {
            process: process("export"),
            kind: TestKind::MaterialBalance,
            source: Some(id("a")),
            target: None,
            amount: integer(5),
        }])
        .unwrap();

    assert_eq!(state.amount(&id("a")), Some(&integer(-7)));
    assert!(state.balance_residual(TestKind::MaterialBalance).is_zero());
}

#[test]
fn signed_stock_settles_from_plus_two_to_minus_three_through_one_flow() {
    let mut state = signed_system(&[("p", 2), ("q", 0)]);
    state
        .settle(&[ProposedFlow {
            process: process("push"),
            kind: TestKind::MaterialBalance,
            source: Some(id("p")),
            target: Some(id("q")),
            amount: integer(5),
        }])
        .unwrap();

    assert_eq!(state.amount(&id("p")), Some(&integer(-3)));
    assert_eq!(state.amount(&id("q")), Some(&integer(5)));
    assert!(state.balance_residual(TestKind::MaterialBalance).is_zero());
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
        let mut state = system(a, b);
        state.settle(&[
            transfer("ab", "a", "b", ab),
            transfer("ba", "b", "a", ba),
            ProposedFlow { process: process("output"), kind: TestKind::Material, source: Some(id("a")), target: None, amount: integer(output_a) },
            ProposedFlow { process: process("input"), kind: TestKind::Material, source: None, target: Some(id("b")), amount: integer(input_b) },
        ]).unwrap();

        prop_assert!(!state.amount(&id("a")).unwrap().is_negative());
        prop_assert!(!state.amount(&id("b")).unwrap().is_negative());
        prop_assert!(state.balance_residual(TestKind::Material).is_zero());
    }

    #[test]
    fn proposal_permutation_does_not_change_stock_or_boundary_accounts(
        initial in 0_i64..1_000_000,
        first in 0_i64..1_000_000,
        second in 0_i64..1_000_000,
        output in 0_i64..1_000_000,
    ) {
        let proposals = [
            transfer("first", "a", "b", first),
            transfer("second", "a", "b", second),
            ProposedFlow { process: process("output"), kind: TestKind::Material, source: Some(id("a")), target: None, amount: integer(output) },
        ];
        let mut forward = system(initial, 0);
        forward.settle(&proposals).unwrap();
        let mut reverse = system(initial, 0);
        reverse.settle(&proposals.into_iter().rev().collect::<Vec<_>>()).unwrap();

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
        let mut merged = system(initial, 0);
        merged.settle(&[transfer("move", "a", "b", first + second)]).unwrap();
        let mut split = system(initial, 0);
        split.settle(&[
            transfer("move", "a", "b", first),
            transfer("move", "a", "b", second),
        ]).unwrap();

        prop_assert_eq!(merged.amount(&id("a")), split.amount(&id("a")));
        prop_assert_eq!(merged.amount(&id("b")), split.amount(&id("b")));
        prop_assert_eq!(merged.balance_residual(TestKind::Material), split.balance_residual(TestKind::Material));
    }
}
