use conservation_core::{
    AxisId, BalanceLaw, BalanceLawError, Grade, GradedLaw, IdentifierError, KindId, Provenance,
};
use num_bigint::BigInt;
use num_rational::BigRational;

fn axis(value: &str) -> AxisId {
    AxisId::new(value).unwrap()
}

fn kind(value: &str) -> KindId {
    KindId::new(value).unwrap()
}

fn q(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

#[test]
fn identifiers_reject_empty_and_whitespace_only_text() {
    assert_eq!(AxisId::new(""), Err(IdentifierError::Blank));
    assert_eq!(AxisId::new(" \t\n"), Err(IdentifierError::Blank));
    assert_eq!(KindId::new(""), Err(IdentifierError::Blank));
    assert_eq!(KindId::new("  "), Err(IdentifierError::Blank));
}

#[test]
fn balance_law_keeps_exact_coefficients_and_origin_metadata() {
    let amount = kind("amount");
    let source = axis("source");
    let sink = axis("sink");
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));

    let law = BalanceLaw::new(
        amount.clone(),
        [(source.clone(), half.clone()), (sink.clone(), half.clone())],
        Provenance::Declared,
    )
    .unwrap();

    assert_eq!(law.kind(), &amount);
    assert_eq!(law.coefficient(&source), &half);
    assert_eq!(law.coefficient(&sink), &half);
    assert_eq!(law.provenance(), &Provenance::Declared);
}

#[test]
fn balance_law_rejects_empty_and_fully_cancelled_coefficients() {
    assert_eq!(
        BalanceLaw::new(kind("amount"), [], Provenance::Declared),
        Err(BalanceLawError::Empty)
    );

    let x = axis("x");
    assert_eq!(
        BalanceLaw::new(kind("amount"), [(x.clone(), q(0))], Provenance::Declared,),
        Err(BalanceLawError::Empty)
    );
    assert_eq!(
        BalanceLaw::new(
            kind("amount"),
            [(x.clone(), q(7)), (x, q(-7))],
            Provenance::Declared,
        ),
        Err(BalanceLawError::Empty)
    );
}

#[test]
fn graded_law_keeps_its_form_and_grade() {
    let form = BalanceLaw::new(
        kind("energy"),
        [(axis("reservoir"), q(1)), (axis("dissipated"), q(1))],
        Provenance::Declared,
    )
    .unwrap();

    let sentence = GradedLaw::new(form.clone(), Grade::Nondecreasing);
    assert_eq!(sentence.form(), &form);
    assert_eq!(sentence.grade(), Grade::Nondecreasing);

    let lifted = GradedLaw::from(form.clone());
    assert_eq!(lifted.form(), &form);
    assert_eq!(lifted.grade(), Grade::Invariant);
    assert_ne!(lifted, sentence);
}

#[test]
fn grades_are_distinct_and_named() {
    assert_ne!(Grade::Invariant, Grade::Nonnegative);
    assert_ne!(Grade::Nonnegative, Grade::Nondecreasing);
    assert_eq!(Grade::Invariant.to_string(), "invariant");
    assert_eq!(Grade::Nonnegative.to_string(), "nonnegative");
    assert_eq!(Grade::Nondecreasing.to_string(), "nondecreasing");
}

#[test]
fn provenance_sources_remain_distinct_origin_metadata() {
    assert_ne!(
        Provenance::IncidenceNullspace,
        Provenance::StoichiometricNullspace
    );
    assert_ne!(Provenance::ForcedBalance, Provenance::Noether);
}
