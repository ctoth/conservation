use conservation_core::{AxisId, KindId, Provenance};
use conservation_linear::{MatrixError, NullspaceSource, TransitionMatrix, derive_left_nullspace};
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

fn closed_flow_matrix(axes: [AxisId; 3]) -> TransitionMatrix {
    let rows = axes
        .iter()
        .map(|axis| match axis.as_str() {
            "A" => vec![q(-1), q(0), q(1)],
            "B" => vec![q(1), q(-1), q(0)],
            "C" => vec![q(0), q(1), q(-1)],
            _ => unreachable!(),
        })
        .collect();
    TransitionMatrix::new(axes, rows).unwrap()
}

#[test]
fn closed_flow_network_has_total_amount_conservation_law() {
    let axes = [axis("A"), axis("B"), axis("C")];
    let matrix = closed_flow_matrix(axes.clone());
    let laws = derive_left_nullspace(&matrix, kind("amount"), NullspaceSource::Incidence).unwrap();

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].provenance(), &Provenance::IncidenceNullspace);
    for axis in axes {
        assert_eq!(laws[0].coefficient(&axis), &q(1));
    }
}

#[test]
fn jointly_permuted_axes_and_rows_produce_identical_named_laws() {
    let canonical = closed_flow_matrix([axis("A"), axis("B"), axis("C")]);
    let permuted = closed_flow_matrix([axis("C"), axis("A"), axis("B")]);

    assert_eq!(canonical.axes(), permuted.axes());
    assert_eq!(
        derive_left_nullspace(&canonical, kind("amount"), NullspaceSource::Incidence).unwrap(),
        derive_left_nullspace(&permuted, kind("amount"), NullspaceSource::Incidence).unwrap()
    );
}

#[test]
fn rational_matrix_is_reduced_to_a_primitive_integer_vector_space_basis() {
    let x = axis("x");
    let y = axis("y");
    let matrix = TransitionMatrix::new(
        [x.clone(), y.clone()],
        vec![
            vec![BigRational::new(BigInt::from(-1), BigInt::from(2))],
            vec![BigRational::new(BigInt::from(1), BigInt::from(3))],
        ],
    )
    .unwrap();

    let laws =
        derive_left_nullspace(&matrix, kind("charge"), NullspaceSource::Stoichiometric).unwrap();

    assert_eq!(laws.len(), 1);
    assert_eq!(laws[0].coefficient(&x), &q(2));
    assert_eq!(laws[0].coefficient(&y), &q(3));
}

#[test]
fn zero_matrix_has_one_canonical_law_per_axis() {
    let a = axis("A");
    let b = axis("B");
    let matrix =
        TransitionMatrix::new([a.clone(), b.clone()], vec![vec![q(0)], vec![q(0)]]).unwrap();
    let laws = derive_left_nullspace(&matrix, kind("amount"), NullspaceSource::Incidence).unwrap();

    assert_eq!(laws.len(), 2);
    assert_eq!(laws[0].coefficient(&a), &q(1));
    assert_eq!(laws[0].coefficient(&b), &q(0));
    assert_eq!(laws[1].coefficient(&a), &q(0));
    assert_eq!(laws[1].coefficient(&b), &q(1));
}

#[test]
fn full_rank_matrix_has_no_conservation_law() {
    let matrix = TransitionMatrix::new(
        [axis("A"), axis("B")],
        vec![vec![q(1), q(0)], vec![q(0), q(1)]],
    )
    .unwrap();

    assert!(
        derive_left_nullspace(&matrix, kind("amount"), NullspaceSource::Incidence)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn dependent_rows_have_a_deterministic_multivector_basis() {
    let a = axis("A");
    let b = axis("B");
    let c = axis("C");
    let matrix = TransitionMatrix::new(
        [a.clone(), b.clone(), c.clone()],
        vec![vec![q(1), q(2)], vec![q(2), q(4)], vec![q(3), q(6)]],
    )
    .unwrap();
    let laws = derive_left_nullspace(&matrix, kind("amount"), NullspaceSource::Incidence).unwrap();

    assert_eq!(laws.len(), 2);
    assert_eq!(laws[0].coefficient(&a), &q(2));
    assert_eq!(laws[0].coefficient(&b), &q(-1));
    assert_eq!(laws[0].coefficient(&c), &q(0));
    assert_eq!(laws[1].coefficient(&a), &q(3));
    assert_eq!(laws[1].coefficient(&b), &q(0));
    assert_eq!(laws[1].coefficient(&c), &q(-1));
}

#[test]
fn malformed_matrices_are_rejected() {
    assert_eq!(TransitionMatrix::new([], vec![]), Err(MatrixError::NoAxes));
    assert_eq!(
        TransitionMatrix::new([axis("A")], vec![vec![]]),
        Err(MatrixError::NoTransitions)
    );
    assert_eq!(
        TransitionMatrix::new([axis("A")], vec![]),
        Err(MatrixError::AxisRowCount { axes: 1, rows: 0 })
    );
    assert_eq!(
        TransitionMatrix::new([axis("A"), axis("B")], vec![vec![q(1)], vec![q(2), q(3)]],),
        Err(MatrixError::RaggedRows {
            row: 1,
            expected: 1,
            actual: 2,
        })
    );
    assert_eq!(
        TransitionMatrix::new([axis("A"), axis("A")], vec![vec![q(1)], vec![q(-1)]],),
        Err(MatrixError::DuplicateAxis(axis("A")))
    );
}

proptest! {
    #[test]
    fn scaling_a_closed_flow_network_preserves_the_same_canonical_law(scale in 1_i64..10_000) {
        let axes = [axis("A"), axis("B"), axis("C")];
        let matrix = TransitionMatrix::new(
            axes.clone(),
            vec![
                vec![q(-scale), q(0), q(scale)],
                vec![q(scale), q(-scale), q(0)],
                vec![q(0), q(scale), q(-scale)],
            ],
        ).unwrap();

        let laws = derive_left_nullspace(
            &matrix,
            kind("amount"),
            NullspaceSource::Stoichiometric,
        ).unwrap();

        prop_assert_eq!(laws.len(), 1);
        prop_assert_eq!(laws[0].provenance(), &Provenance::StoichiometricNullspace);
        for axis in axes {
            prop_assert_eq!(laws[0].coefficient(&axis), &q(1));
        }
    }
}
