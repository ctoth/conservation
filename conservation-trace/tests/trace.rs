use conservation_core::{AxisId, BalanceLaw, KindId, Provenance};
use conservation_trace::{
    TraceError, TraceState, TraceStateError, TraceVerdict, ViolatedBalance, check_trace,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use proptest::prelude::*;

fn axis(value: &str) -> AxisId {
    AxisId::new(value).unwrap()
}

fn kind(value: &str) -> KindId {
    KindId::new(value).unwrap()
}

fn q(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn total_law(a: &AxisId, b: &AxisId) -> BalanceLaw {
    BalanceLaw::new(
        kind("amount"),
        [(a.clone(), q(1)), (b.clone(), q(1))],
        Provenance::Declared,
    )
    .unwrap()
}

#[test]
fn trace_state_rejects_empty_and_duplicate_axes() {
    assert_eq!(TraceState::new([]), Err(TraceStateError::Empty));
    let a = axis("A");
    assert_eq!(
        TraceState::new([(a.clone(), q(1)), (a.clone(), q(2))]),
        Err(TraceStateError::DuplicateAxis(a))
    );
}

#[test]
fn exact_violated_trace_returns_a_semantic_verdict() {
    let a = axis("A");
    let b = axis("B");
    let law = total_law(&a, &b);
    let states = vec![
        TraceState::new([(a.clone(), q(3)), (b.clone(), q(2))]).unwrap(),
        TraceState::new([(a.clone(), q(2)), (b.clone(), q(3))]).unwrap(),
        TraceState::new([(a, q(1)), (b, q(3))]).unwrap(),
    ];

    assert_eq!(
        check_trace(&law, &states),
        Ok(TraceVerdict::Violated(ViolatedBalance {
            state_index: 2,
            expected: q(5),
            observed: q(4),
        }))
    );
}

#[test]
fn fractional_conserved_trace_returns_a_witness_without_origin_metadata() {
    let a = axis("A");
    let b = axis("B");
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let law = BalanceLaw::new(
        kind("amount"),
        [(a.clone(), half.clone()), (b.clone(), half)],
        Provenance::ForcedBalance,
    )
    .unwrap();
    let states = vec![
        TraceState::new([(a.clone(), q(1)), (b.clone(), q(2))]).unwrap(),
        TraceState::new([
            (a, BigRational::new(BigInt::from(3), BigInt::from(2))),
            (b, BigRational::new(BigInt::from(3), BigInt::from(2))),
        ])
        .unwrap(),
    ];

    let TraceVerdict::Satisfied(witness) = check_trace(&law, &states).unwrap() else {
        panic!("expected satisfied trace");
    };
    assert_eq!(witness.states_checked, 2);
    assert_eq!(
        witness.conserved_value,
        BigRational::new(BigInt::from(3), BigInt::from(2))
    );
    assert_eq!(witness.kind, kind("amount"));
}

#[test]
fn empty_and_single_state_traces_are_structural_errors() {
    let a = axis("A");
    let law = BalanceLaw::new(kind("amount"), [(a.clone(), q(1))], Provenance::Declared).unwrap();

    assert_eq!(
        check_trace(&law, &[]),
        Err(TraceError::TooShort { states: 0 })
    );
    assert_eq!(
        check_trace(&law, &[TraceState::new([(a, q(0))]).unwrap()]),
        Err(TraceError::TooShort { states: 1 })
    );
}

#[test]
fn a_missing_law_axis_is_a_structural_error() {
    let a = axis("A");
    let b = axis("B");
    let law = total_law(&a, &b);
    let states = vec![
        TraceState::new([(a.clone(), q(1)), (b, q(1))]).unwrap(),
        TraceState::new([(a, q(2))]).unwrap(),
    ];

    assert_eq!(
        check_trace(&law, &states),
        Err(TraceError::MissingAxis {
            state_index: 1,
            axis: axis("B"),
        })
    );
}

proptest! {
    #[test]
    fn arbitrary_exact_partitions_of_one_total_produce_a_witness(
        total in -1_000_000_i64..=1_000_000,
        left_values in prop::collection::vec(-1_000_000_i64..=1_000_000, 2..64),
    ) {
        let a = axis("A");
        let b = axis("B");
        let law = total_law(&a, &b);
        let states = left_values
            .into_iter()
            .map(|left| {
                TraceState::new([
                    (a.clone(), q(left)),
                    (b.clone(), q(total - left)),
                ])
                .unwrap()
            })
            .collect::<Vec<_>>();

        let TraceVerdict::Satisfied(witness) = check_trace(&law, &states).unwrap() else {
            prop_assert!(false, "exactly conserved trace was rejected");
            return Ok(());
        };
        prop_assert_eq!(witness.states_checked, states.len());
        prop_assert_eq!(witness.conserved_value, q(total));
    }

    #[test]
    fn first_corrupted_state_is_reported_exactly(
        total in -1_000_000_i64..=1_000_000,
        left_values in prop::collection::vec(-1_000_000_i64..=1_000_000, 2..32),
        delta in prop_oneof![-1_000_i64..0, 1_i64..=1_000],
    ) {
        let a = axis("A");
        let b = axis("B");
        let law = total_law(&a, &b);
        let final_index = left_values.len();
        let mut states = left_values
            .into_iter()
            .map(|left| {
                TraceState::new([
                    (a.clone(), q(left)),
                    (b.clone(), q(total - left)),
                ])
                .unwrap()
            })
            .collect::<Vec<_>>();
        states.push(
            TraceState::new([(a, q(0)), (b, q(total + delta))]).unwrap(),
        );

        prop_assert_eq!(
            check_trace(&law, &states),
            Ok(TraceVerdict::Violated(ViolatedBalance {
                state_index: final_index,
                expected: q(total),
                observed: q(total + delta),
            }))
        );
    }
}
