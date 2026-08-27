use std::sync::Arc;

use conservation_core::{AxisId, BalanceLaw, Grade, GradedLaw, KindId, Provenance};
use conservation_dynamics::{
    ExactState, FlowSpec, FlowTopology, ProcessId, StockDefinition, StockId,
};
use conservation_stock_flow::{
    BoundaryCorrespondence, BoundaryId, BoundaryVerdict, ChannelId, ExactAmounts,
    FlowConstraintVerdict, FlowId, GradedStateLaw, LedgerDefinition, LedgerId,
    LinearFlowConstraint, OpenBalanceVerdict, SentenceId, StockAxisDefinition, StockFlowCarrier,
    StockFlowError, StockFlowLawSuite, SuiteVerdict, TransitionEquation, TransitionRecord,
    TransitionRecordData, TransitionTrace, TransitionVerdict, certify_nullspace,
    check_boundary_correspondence, check_graded_state_law, check_linear_flow_constraint,
    check_open_balance, check_transition_equation, derive_nullspace_basis,
};
use conservation_trace::{LawVerdict, LawViolation};
use num_bigint::BigInt;
use num_rational::BigRational;
use proptest::prelude::*;

fn q(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn kind() -> KindId {
    KindId::new("material").unwrap()
}

fn stock(value: &str) -> StockId {
    StockId::new(value).unwrap()
}

fn axis(value: &str) -> AxisId {
    AxisId::new(value).unwrap()
}

fn flow(value: &str) -> FlowId {
    FlowId::new(value).unwrap()
}

fn boundary(value: &str) -> BoundaryId {
    BoundaryId::new(value).unwrap()
}

fn ledger(value: &str) -> LedgerId {
    LedgerId::new(value).unwrap()
}

fn sentence(value: &str) -> SentenceId {
    SentenceId::new(value).unwrap()
}

fn topology(reordered: bool) -> Arc<FlowTopology> {
    let material = kind();
    let a = StockDefinition {
        id: stock("A"),
        kind: material.clone(),
    };
    let b = StockDefinition {
        id: stock("B"),
        kind: material.clone(),
    };
    let f1 = FlowSpec {
        process: ProcessId::new("transfer-1").unwrap(),
        kind: material.clone(),
        source: Some(stock("A")),
        target: Some(stock("B")),
    };
    let f2 = FlowSpec {
        process: ProcessId::new("transfer-2").unwrap(),
        kind: material.clone(),
        source: Some(stock("A")),
        target: Some(stock("B")),
    };
    let input = FlowSpec {
        process: ProcessId::new("input").unwrap(),
        kind: material.clone(),
        source: None,
        target: Some(stock("A")),
    };
    let output = FlowSpec {
        process: ProcessId::new("output").unwrap(),
        kind: material,
        source: Some(stock("B")),
        target: None,
    };
    let (stocks, flows) = if reordered {
        (vec![b, a], vec![output, input, f2, f1])
    } else {
        (vec![a, b], vec![f1, f2, input, output])
    };
    Arc::new(FlowTopology::new(stocks, flows).unwrap())
}

fn carrier(reordered: bool) -> Arc<StockFlowCarrier> {
    let channels = if reordered {
        vec![
            ChannelId::Boundary(boundary("out")),
            ChannelId::Boundary(boundary("in")),
            ChannelId::Internal(flow("f2")),
            ChannelId::Internal(flow("f1")),
        ]
    } else {
        vec![
            ChannelId::Internal(flow("f1")),
            ChannelId::Internal(flow("f2")),
            ChannelId::Boundary(boundary("in")),
            ChannelId::Boundary(boundary("out")),
        ]
    };
    Arc::new(
        StockFlowCarrier::new(
            topology(reordered),
            [
                StockAxisDefinition {
                    stock: stock("B"),
                    axis: axis("B"),
                },
                StockAxisDefinition {
                    stock: stock("A"),
                    axis: axis("A"),
                },
            ],
            channels,
            [
                LedgerDefinition {
                    id: ledger("input-ledger"),
                    axis: axis("cumulative-input"),
                    kind: kind(),
                    boundaries: vec![boundary("in")],
                },
                LedgerDefinition {
                    id: ledger("output-ledger"),
                    axis: axis("cumulative-output"),
                    kind: kind(),
                    boundaries: vec![boundary("out")],
                },
            ],
        )
        .unwrap(),
    )
}

fn stocks(a: i64, b: i64) -> ExactAmounts<AxisId> {
    ExactAmounts::new([(axis("A"), kind(), q(a)), (axis("B"), kind(), q(b))]).unwrap()
}

fn internals(f1: i64, f2: i64) -> ExactAmounts<FlowId> {
    ExactAmounts::new([(flow("f1"), kind(), q(f1)), (flow("f2"), kind(), q(f2))]).unwrap()
}

fn boundaries(input: i64, output: i64) -> ExactAmounts<BoundaryId> {
    ExactAmounts::new([
        (boundary("in"), kind(), q(input)),
        (boundary("out"), kind(), q(output)),
    ])
    .unwrap()
}

fn ledgers(input: i64, output: i64) -> ExactAmounts<LedgerId> {
    ExactAmounts::new([
        (ledger("input-ledger"), kind(), q(input)),
        (ledger("output-ledger"), kind(), q(output)),
    ])
    .unwrap()
}

fn record_data(before: [i64; 2], after: [i64; 2]) -> TransitionRecordData {
    TransitionRecordData {
        before: stocks(before[0], before[1]),
        after: stocks(after[0], after[1]),
        requested_internal: internals(2, 1),
        settled_internal: internals(2, 1),
        requested_boundary: boundaries(3, 0),
        settled_boundary: boundaries(3, 0),
        ledger_before: ledgers(0, 0),
        ledger_after: ledgers(3, 0),
    }
}

fn trace_with(after: [i64; 2]) -> TransitionTrace {
    let carrier = carrier(false);
    let record = TransitionRecord::new(&carrier, record_data([10, 0], after)).unwrap();
    TransitionTrace::new(carrier, vec![record]).unwrap()
}

#[test]
fn carrier_compiles_total_exact_matrices_in_canonical_named_order() {
    let system = carrier(false);
    let internal = system.internal_effects();
    assert_eq!(
        internal.axes().cloned().collect::<Vec<_>>(),
        vec![axis("A"), axis("B")]
    );
    assert_eq!(
        internal.columns().cloned().collect::<Vec<_>>(),
        vec![flow("f1"), flow("f2")]
    );
    assert_eq!(internal.coefficient(&axis("A"), &flow("f1")), Some(&q(-1)));
    assert_eq!(internal.coefficient(&axis("B"), &flow("f1")), Some(&q(1)));
    assert_eq!(
        system
            .boundary_effects()
            .coefficient(&axis("A"), &boundary("in")),
        Some(&q(1))
    );
    assert_eq!(
        system
            .boundary_effects()
            .coefficient(&axis("B"), &boundary("out")),
        Some(&q(-1))
    );
    assert_eq!(system.identity(), carrier(true).identity());
}

#[test]
fn carrier_rejects_malformed_symbol_shapes_before_semantics() {
    let topology = topology(false);
    let stock_axes = [
        StockAxisDefinition {
            stock: stock("A"),
            axis: axis("A"),
        },
        StockAxisDefinition {
            stock: stock("B"),
            axis: axis("B"),
        },
    ];
    assert!(matches!(
        StockFlowCarrier::new(topology.clone(), stock_axes.clone(), [], []),
        Err(StockFlowError::ChannelCount {
            expected: 4,
            actual: 0
        })
    ));
    let wrong_roles = [
        ChannelId::Boundary(boundary("bad")),
        ChannelId::Internal(flow("f2")),
        ChannelId::Boundary(boundary("in")),
        ChannelId::Boundary(boundary("out")),
    ];
    assert!(matches!(
        StockFlowCarrier::new(topology.clone(), stock_axes.clone(), wrong_roles, []),
        Err(StockFlowError::ChannelRole { .. })
    ));
    let duplicate = [
        ChannelId::Internal(flow("same")),
        ChannelId::Internal(flow("same")),
        ChannelId::Boundary(boundary("in")),
        ChannelId::Boundary(boundary("out")),
    ];
    assert!(matches!(
        StockFlowCarrier::new(topology, stock_axes, duplicate, []),
        Err(StockFlowError::DuplicateFlow(id)) if id == flow("same")
    ));
}

#[test]
fn signed_observations_are_models_but_negative_flows_are_structural_errors() {
    let carrier = carrier(false);
    let mut signed = record_data([-1, 0], [-1, 3]);
    signed.requested_internal = internals(0, 0);
    signed.settled_internal = internals(0, 0);
    signed.requested_boundary = boundaries(0, 0);
    signed.settled_boundary = boundaries(0, 0);
    signed.ledger_after = ledgers(0, 0);
    assert!(TransitionRecord::new(&carrier, signed).is_ok());

    let mut negative = record_data([10, 0], [10, 3]);
    negative.requested_internal =
        ExactAmounts::new([(flow("f1"), kind(), q(-1)), (flow("f2"), kind(), q(1))]).unwrap();
    assert_eq!(
        TransitionRecord::new(&carrier, negative),
        Err(StockFlowError::NegativeAmount(
            conservation_stock_flow::SymbolId::Flow(flow("f1"))
        ))
    );
}

#[test]
fn records_reject_missing_extra_wrong_kind_and_over_settled_values() {
    let carrier = carrier(false);
    let mut missing = record_data([10, 0], [10, 3]);
    missing.before = ExactAmounts::new([(axis("A"), kind(), q(10))]).unwrap();
    assert!(matches!(
        TransitionRecord::new(&carrier, missing),
        Err(StockFlowError::MissingValue { .. })
    ));

    let mut extra = record_data([10, 0], [10, 3]);
    extra.before = ExactAmounts::new([
        (axis("A"), kind(), q(10)),
        (axis("B"), kind(), q(0)),
        (axis("C"), kind(), q(0)),
    ])
    .unwrap();
    assert!(matches!(
        TransitionRecord::new(&carrier, extra),
        Err(StockFlowError::ExtraValue { .. })
    ));

    let mut wrong_kind = record_data([10, 0], [10, 3]);
    wrong_kind.before = ExactAmounts::new([
        (axis("A"), KindId::new("energy").unwrap(), q(10)),
        (axis("B"), kind(), q(0)),
    ])
    .unwrap();
    assert!(matches!(
        TransitionRecord::new(&carrier, wrong_kind),
        Err(StockFlowError::KindMismatch { .. })
    ));

    let mut over = record_data([10, 0], [10, 3]);
    over.requested_internal = internals(1, 1);
    assert!(matches!(
        TransitionRecord::new(&carrier, over),
        Err(StockFlowError::SettledExceedsRequested { .. })
    ));
}

#[test]
fn traces_own_independent_records_and_reject_discontinuity() {
    let carrier = carrier(false);
    let first = TransitionRecord::new(&carrier, record_data([10, 0], [10, 3])).unwrap();
    let mut second_data = record_data([9, 3], [9, 6]);
    second_data.ledger_before = ledgers(3, 0);
    second_data.ledger_after = ledgers(6, 0);
    let second = TransitionRecord::new(&carrier, second_data).unwrap();
    assert!(matches!(
        TransitionTrace::new(carrier.clone(), vec![first, second]),
        Err(StockFlowError::DiscontinuousState {
            transition: 1,
            axis: got_axis,
        }) if got_axis == axis("A")
    ));

    let original = trace_with([10, 3]);
    let cloned = original.clone();
    assert_eq!(original.records(), cloned.records());
    assert!(!std::ptr::eq(
        original.records().as_ptr(),
        cloned.records().as_ptr()
    ));
}

#[test]
fn settlement_report_builds_a_witness_without_reimplementing_settlement() {
    let carrier = carrier(false);
    let mut state = ExactState::new(carrier.topology().clone(), vec![q(10), q(0)]).unwrap();
    let before = state.amounts().to_vec();
    let report = state.settle(&[q(2), q(1), q(3), q(0)]).unwrap();
    let record = carrier
        .record_from_settlement(
            &before,
            state.amounts(),
            &report,
            ledgers(0, 0),
            ledgers(3, 0),
        )
        .unwrap();
    let trace = TransitionTrace::new(carrier, vec![record]).unwrap();
    assert!(
        check_transition_equation(&TransitionEquation::new(sentence("transition")), &trace)
            .unwrap()
            .is_satisfied()
    );
}

#[test]
fn empty_trace_is_structural_and_misrouting_reports_first_canonical_axis() {
    let carrier = carrier(false);
    let empty = TransitionTrace::new(carrier, vec![]).unwrap();
    assert_eq!(
        check_transition_equation(&TransitionEquation::new(sentence("transition")), &empty),
        Err(StockFlowError::TooShort { transitions: 0 })
    );

    let trace = trace_with([9, 4]);
    let TransitionVerdict::Violated(violation) =
        check_transition_equation(&TransitionEquation::new(sentence("transition")), &trace)
            .unwrap()
    else {
        panic!("expected misrouted transition to fail");
    };
    assert_eq!(violation.transition, 0);
    assert_eq!(violation.axis, axis("A"));
    assert_eq!(violation.observed_delta, q(-1));
    assert_eq!(violation.accounted_delta, q(0));
}

#[test]
fn flow_and_boundary_checkers_return_typed_first_offense_evidence() {
    let good = trace_with([10, 3]);
    let ratio = LinearFlowConstraint::new(
        good.carrier(),
        sentence("partition"),
        kind(),
        [(flow("f1"), q(1)), (flow("f2"), q(-2))],
        q(0),
    )
    .unwrap();
    assert!(
        check_linear_flow_constraint(&ratio, &good)
            .unwrap()
            .is_satisfied()
    );
    let correspondence =
        BoundaryCorrespondence::new(sentence("input-ledger"), ledger("input-ledger"));
    assert!(
        check_boundary_correspondence(&correspondence, &good)
            .unwrap()
            .is_satisfied()
    );

    let carrier = carrier(false);
    let mut bad_ratio = record_data([10, 0], [10, 3]);
    bad_ratio.requested_internal = internals(1, 2);
    bad_ratio.settled_internal = internals(1, 2);
    let trace = TransitionTrace::new(
        carrier.clone(),
        vec![TransitionRecord::new(&carrier, bad_ratio).unwrap()],
    )
    .unwrap();
    let FlowConstraintVerdict::Violated(violation) =
        check_linear_flow_constraint(&ratio, &trace).unwrap()
    else {
        panic!("expected wrong ratio to fail");
    };
    assert_eq!(violation.transition, 0);
    assert_eq!(violation.observed, q(-3));

    let mut bad_ledger = record_data([10, 0], [10, 3]);
    bad_ledger.ledger_after = ledgers(0, 0);
    let trace = TransitionTrace::new(
        carrier.clone(),
        vec![TransitionRecord::new(&carrier, bad_ledger).unwrap()],
    )
    .unwrap();
    let BoundaryVerdict::Violated(violation) =
        check_boundary_correspondence(&correspondence, &trace).unwrap()
    else {
        panic!("expected dishonest ledger to fail");
    };
    assert_eq!(violation.ledger, ledger("input-ledger"));
    assert_eq!(violation.boundaries, vec![boundary("in")]);
    assert_eq!(violation.observed_increment, q(0));
    assert_eq!(violation.settled_total, q(3));
}

#[test]
fn perturbing_settled_process_or_boundary_amount_is_reproducible() {
    let carrier = carrier(false);
    let equation = TransitionEquation::new(sentence("transition"));

    let mut process = record_data([10, 0], [10, 3]);
    process.requested_internal = internals(3, 1);
    process.settled_internal = internals(3, 1);
    let process_trace = TransitionTrace::new(
        carrier.clone(),
        vec![TransitionRecord::new(&carrier, process).unwrap()],
    )
    .unwrap();
    let TransitionVerdict::Violated(process_violation) =
        check_transition_equation(&equation, &process_trace).unwrap()
    else {
        panic!("expected perturbed settled process to fail");
    };
    assert_eq!(process_violation.axis, axis("A"));
    assert_eq!(process_violation.residual, q(1));

    let mut port = record_data([10, 0], [10, 3]);
    port.requested_boundary = boundaries(4, 0);
    port.settled_boundary = boundaries(4, 0);
    let port_trace = TransitionTrace::new(
        carrier.clone(),
        vec![TransitionRecord::new(&carrier, port).unwrap()],
    )
    .unwrap();
    let TransitionVerdict::Violated(port_violation) =
        check_transition_equation(&equation, &port_trace).unwrap()
    else {
        panic!("expected perturbed settled boundary to fail");
    };
    assert_eq!(port_violation.axis, axis("A"));
    assert_eq!(port_violation.residual, q(-1));
}

#[test]
fn signed_projection_reuses_existing_graded_false_semantics() {
    let carrier = carrier(false);
    let mut data = record_data([-1, 0], [-1, 0]);
    data.requested_internal = internals(0, 0);
    data.settled_internal = internals(0, 0);
    data.requested_boundary = boundaries(0, 0);
    data.settled_boundary = boundaries(0, 0);
    data.ledger_after = ledgers(0, 0);
    let record = TransitionRecord::new(&carrier, data).unwrap();
    let trace = TransitionTrace::new(carrier, vec![record]).unwrap();
    let law = GradedLaw::new(
        BalanceLaw::new(kind(), [(axis("A"), q(1))], Provenance::Declared).unwrap(),
        Grade::Nonnegative,
    );
    let verdict =
        check_graded_state_law(&GradedStateLaw::new(sentence("A>=0"), law), &trace).unwrap();
    assert!(matches!(
        verdict,
        LawVerdict::Violated(LawViolation::Negative { state_index: 0, .. })
    ));
}

#[test]
fn checked_certificates_recompute_nullspace_and_seal_incidence_provenance() {
    let carrier = carrier(false);
    assert!(matches!(
        certify_nullspace(&carrier, kind(), [(axis("A"), q(1))],),
        Err(StockFlowError::NonNullCertificate { .. })
    ));
    assert!(matches!(
        certify_nullspace(&carrier, kind(), [(axis("A"), q(1)), (axis("A"), q(-1))],),
        Err(StockFlowError::BalanceLaw(_))
    ));

    let incidence =
        certify_nullspace(&carrier, kind(), [(axis("A"), q(1)), (axis("B"), q(1))]).unwrap();
    assert_eq!(
        incidence.law().provenance(),
        &Provenance::IncidenceNullspace
    );
    assert!(
        incidence
            .annihilation()
            .values()
            .all(num_traits::Zero::is_zero)
    );
}

#[test]
fn basis_has_rows_minus_rank_members_and_direct_open_balance_is_semantic() {
    let carrier = carrier(false);
    let basis = derive_nullspace_basis(&carrier, kind()).unwrap();
    assert_eq!(basis.len(), 1); // two rows minus rank one
    assert_eq!(basis[0].law().coefficient(&axis("A")), &q(1));
    assert_eq!(basis[0].law().coefficient(&axis("B")), &q(1));
    let open = basis[0].open_balance(sentence("open-material"));
    assert!(
        check_open_balance(&open, &trace_with([10, 3]))
            .unwrap()
            .is_satisfied()
    );

    let OpenBalanceVerdict::Violated(violation) =
        check_open_balance(&open, &trace_with([9, 3])).unwrap()
    else {
        panic!("expected corrupted model to falsify direct open balance");
    };
    assert_eq!(violation.transition, 0);
    assert_eq!(violation.observed_delta, q(2));
    assert_eq!(violation.boundary_delta, q(3));
    assert_eq!(violation.residual, q(-1));
}

#[test]
fn checked_open_balance_projects_to_existing_graded_invariant() {
    let carrier = carrier(false);
    let certificate =
        certify_nullspace(&carrier, kind(), [(axis("A"), q(1)), (axis("B"), q(1))]).unwrap();
    let projected = certificate
        .graded_invariant(&carrier, [ledger("input-ledger"), ledger("output-ledger")])
        .unwrap();
    assert_eq!(projected.form().coefficient(&axis("A")), &q(1));
    assert_eq!(projected.form().coefficient(&axis("B")), &q(1));
    assert_eq!(
        projected.form().coefficient(&axis("cumulative-input")),
        &q(-1)
    );
    assert_eq!(
        projected.form().coefficient(&axis("cumulative-output")),
        &q(1)
    );
    assert!(matches!(
        check_graded_state_law(
            &GradedStateLaw::new(sentence("projected-open"), projected),
            &trace_with([10, 3]),
        )
        .unwrap(),
        LawVerdict::Satisfied(_)
    ));
}

#[test]
fn suite_retains_every_named_typed_outcome_in_canonical_order() {
    let carrier = carrier(false);
    let certificate =
        certify_nullspace(&carrier, kind(), [(axis("A"), q(1)), (axis("B"), q(1))]).unwrap();
    let suite = StockFlowLawSuite::new(
        Some(TransitionEquation::new(sentence("d-transition"))),
        [LinearFlowConstraint::new(
            &carrier,
            sentence("a-partition"),
            kind(),
            [(flow("f1"), q(1)), (flow("f2"), q(-2))],
            q(0),
        )
        .unwrap()],
        [BoundaryCorrespondence::new(
            sentence("b-input-ledger"),
            ledger("input-ledger"),
        )],
        [certificate.open_balance(sentence("c-open"))],
        [],
    )
    .unwrap();
    let verdicts = suite.check(&trace_with([10, 3])).unwrap();
    assert_eq!(verdicts.len(), 4);
    assert_eq!(
        verdicts.iter().map(SuiteVerdict::id).collect::<Vec<_>>(),
        vec![
            &sentence("a-partition"),
            &sentence("b-input-ledger"),
            &sentence("c-open"),
            &sentence("d-transition"),
        ]
    );
    assert!(verdicts.iter().all(SuiteVerdict::is_satisfied));
}

proptest! {
    #[test]
    fn arbitrary_exact_settlements_satisfy_transition_and_every_derived_open_balance(
        a in 0_i64..1_000,
        b in 0_i64..1_000,
        r1 in 0_i64..1_000,
        r2 in 0_i64..1_000,
        input in 0_i64..1_000,
        output in 0_i64..1_000,
    ) {
        let carrier = carrier(false);
        let mut state = ExactState::new(carrier.topology().clone(), vec![q(a), q(b)]).unwrap();
        let before = state.amounts().to_vec();
        let report = state.settle(&[q(r1), q(r2), q(input), q(output)]).unwrap();
        let input_applied = report.applied()[2].clone();
        let output_applied = report.applied()[3].clone();
        let record = carrier.record_from_settlement(
            &before,
            state.amounts(),
            &report,
            ledgers(0, 0),
            ExactAmounts::new([
                (ledger("input-ledger"), kind(), input_applied),
                (ledger("output-ledger"), kind(), output_applied),
            ]).unwrap(),
        ).unwrap();
        let trace = TransitionTrace::new(carrier.clone(), vec![record]).unwrap();
        prop_assert!(check_transition_equation(
            &TransitionEquation::new(sentence("transition")),
            &trace,
        ).unwrap().is_satisfied());
        for certificate in derive_nullspace_basis(&carrier, kind()).unwrap() {
            prop_assert!(check_open_balance(
                &certificate.open_balance(sentence("derived")),
                &trace,
            ).unwrap().is_satisfied());
        }
    }
}
