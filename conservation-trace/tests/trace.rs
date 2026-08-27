use conservation_core::{AxisId, BalanceLaw, Grade, GradedLaw, KindId, Provenance};
use conservation_trace::{
    LawVerdict, LawViolation, LawWitness, NondecreasingWitness, NonnegativeWitness, TraceError,
    TraceState, TraceStateError, TraceVerdict, ViolatedBalance, check_law, check_trace,
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

fn states(values: &[(i64, i64)]) -> Vec<TraceState> {
    let a = axis("A");
    let b = axis("B");
    values
        .iter()
        .map(|(left, right)| {
            TraceState::new([(a.clone(), q(*left)), (b.clone(), q(*right))]).unwrap()
        })
        .collect()
}

#[test]
fn invariant_grade_agrees_with_check_trace_in_both_outcomes() {
    let a = axis("A");
    let b = axis("B");
    let law = GradedLaw::from(total_law(&a, &b));

    let conserved = states(&[(3, 2), (2, 3), (1, 4)]);
    let LawVerdict::Satisfied(LawWitness::Invariant(witness)) =
        check_law(&law, &conserved).unwrap()
    else {
        panic!("expected an invariant witness");
    };
    assert_eq!(witness.states_checked, 3);
    assert_eq!(witness.conserved_value, q(5));

    let corrupted = states(&[(3, 2), (2, 3), (1, 3)]);
    assert_eq!(
        check_law(&law, &corrupted),
        Ok(LawVerdict::Violated(LawViolation::Invariant(
            ViolatedBalance {
                state_index: 2,
                expected: q(5),
                observed: q(4),
            }
        )))
    );
}

#[test]
fn nonnegative_grade_admits_zero_and_reports_the_first_negative_state() {
    let a = axis("A");
    let law = GradedLaw::new(
        BalanceLaw::new(kind("amount"), [(a.clone(), q(1))], Provenance::Declared).unwrap(),
        Grade::Nonnegative,
    );

    let touching_zero = states(&[(2, 0), (0, 0), (5, 0)]);
    let LawVerdict::Satisfied(LawWitness::Nonnegative(witness)) =
        check_law(&law, &touching_zero).unwrap()
    else {
        panic!("expected a nonnegative witness");
    };
    assert_eq!(
        witness,
        NonnegativeWitness {
            states_checked: 3,
            minimum: q(0),
            kind: kind("amount"),
        }
    );

    let dipping = states(&[(2, 0), (-1, 0), (-3, 0)]);
    assert_eq!(
        check_law(&law, &dipping),
        Ok(LawVerdict::Violated(LawViolation::Negative {
            state_index: 1,
            observed: q(-1),
        }))
    );

    let starting_negative = states(&[(-4, 0), (1, 0)]);
    assert_eq!(
        check_law(&law, &starting_negative),
        Ok(LawVerdict::Violated(LawViolation::Negative {
            state_index: 0,
            observed: q(-4),
        }))
    );
}

#[test]
fn nondecreasing_grade_admits_plateaus_and_reports_the_first_decrease() {
    let a = axis("A");
    let law = GradedLaw::new(
        BalanceLaw::new(kind("entropy"), [(a.clone(), q(1))], Provenance::Declared).unwrap(),
        Grade::Nondecreasing,
    );

    let plateau = states(&[(1, 0), (1, 0), (4, 0)]);
    let LawVerdict::Satisfied(LawWitness::Nondecreasing(witness)) =
        check_law(&law, &plateau).unwrap()
    else {
        panic!("expected a nondecreasing witness");
    };
    assert_eq!(
        witness,
        NondecreasingWitness {
            states_checked: 3,
            initial: q(1),
            last: q(4),
            kind: kind("entropy"),
        }
    );

    let dipping = states(&[(1, 0), (5, 0), (3, 0), (0, 0)]);
    assert_eq!(
        check_law(&law, &dipping),
        Ok(LawVerdict::Violated(LawViolation::Decrease {
            state_index: 2,
            previous: q(5),
            observed: q(3),
        }))
    );
}

#[test]
fn graded_checking_shares_the_structural_trace_contract() {
    let a = axis("A");
    let b = axis("B");
    let form = total_law(&a, &b);

    for grade in [Grade::Invariant, Grade::Nonnegative, Grade::Nondecreasing] {
        let law = GradedLaw::new(form.clone(), grade);
        assert_eq!(
            check_law(&law, &[]),
            Err(TraceError::TooShort { states: 0 })
        );
        assert_eq!(
            check_law(&law, &states(&[(1, 1)])),
            Err(TraceError::TooShort { states: 1 })
        );

        let missing = vec![
            TraceState::new([(a.clone(), q(1)), (b.clone(), q(1))]).unwrap(),
            TraceState::new([(a.clone(), q(2))]).unwrap(),
        ];
        assert_eq!(
            check_law(&law, &missing),
            Err(TraceError::MissingAxis {
                state_index: 1,
                axis: b.clone(),
            })
        );
    }
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
