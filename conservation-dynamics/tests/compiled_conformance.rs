use std::collections::BTreeSet;
use std::sync::Arc;

use conservation_dynamics::{
    DenseState, DenseTolerance, ExactState, FlowRole, FlowSpec, FlowTopology, ProcessDefinition,
    ProcessId, ProposedFlow, Rationing, StockDefinition, StockFlowError, StockFlowSystem, StockId,
    StockSpec,
};
use conservation_test_kinds::TestKind;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use proptest::prelude::*;

fn stock(value: &str) -> StockId {
    StockId::new(value).unwrap()
}

fn process(value: &str) -> ProcessId {
    ProcessId::new(value).unwrap()
}

fn integer(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn definition(id: &str, kind: TestKind) -> StockDefinition<TestKind> {
    StockDefinition {
        id: stock(id),
        kind,
    }
}

fn flow(
    process_name: &str,
    kind: TestKind,
    source: Option<&str>,
    target: Option<&str>,
) -> FlowSpec<TestKind> {
    FlowSpec {
        process: process(process_name),
        kind,
        source: source.map(stock),
        target: target.map(stock),
    }
}

fn declare(process_name: &str, rationing: Rationing) -> ProcessDefinition {
    ProcessDefinition {
        id: process(process_name),
        rationing,
    }
}

fn ration_all(flows: &[FlowSpec<TestKind>]) -> Vec<ProcessDefinition> {
    flows
        .iter()
        .map(|flow| flow.process.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|id| ProcessDefinition {
            id,
            rationing: Rationing::Ration,
        })
        .collect()
}

fn material_topology() -> Arc<FlowTopology<TestKind>> {
    Arc::new(
        FlowTopology::new(
            [
                definition("a", TestKind::Material),
                definition("b", TestKind::Material),
            ],
            [
                flow("input", TestKind::Material, None, Some("a")),
                flow("move", TestKind::Material, Some("a"), Some("b")),
                flow("export", TestKind::Material, Some("a"), None),
            ],
            [
                declare("move", Rationing::Ration),
                declare("export", Rationing::Ration),
            ],
        )
        .unwrap(),
    )
}

fn proposal(
    process_name: &str,
    source: Option<&str>,
    target: Option<&str>,
    amount: BigRational,
) -> ProposedFlow<TestKind> {
    ProposedFlow {
        process: process(process_name),
        kind: TestKind::Material,
        source: source.map(stock),
        target: target.map(stock),
        amount,
    }
}

#[test]
fn compilation_assigns_stable_stock_kind_process_and_endpoint_indices() {
    let topology = material_topology();

    assert_eq!(topology.stock_index(&stock("a")), Some(0));
    assert_eq!(topology.stock_index(&stock("b")), Some(1));
    assert_eq!(topology.kind_index(TestKind::Material), Some(0));
    assert_eq!(topology.process_index(&process("input")), Some(0));
    assert_eq!(topology.process_index(&process("move")), Some(1));
    assert_eq!(topology.flows()[0].source(), None);
    assert_eq!(topology.flows()[0].target(), Some(0));
    assert_eq!(topology.flows()[0].role(), FlowRole::Input);
    assert_eq!(topology.flows()[1].source(), Some(0));
    assert_eq!(topology.flows()[1].target(), Some(1));
    assert_eq!(topology.flows()[1].role(), FlowRole::Transfer);
    assert_eq!(topology.flows()[2].role(), FlowRole::Output);
}

#[test]
fn topology_compilation_rejects_every_malformed_shape() {
    assert_eq!(
        FlowTopology::new([], std::iter::empty::<FlowSpec<TestKind>>(), []),
        Err(StockFlowError::NoStocks)
    );
    assert!(matches!(
        FlowTopology::new(
            [
                definition("a", TestKind::Material),
                definition("a", TestKind::Material)
            ],
            [],
            []
        ),
        Err(StockFlowError::DuplicateStock(_))
    ));
    assert_eq!(
        FlowTopology::new(
            [definition("a", TestKind::Material)],
            [flow("lost", TestKind::Material, None, None)],
            []
        ),
        Err(StockFlowError::DisconnectedFlow)
    );
    assert!(matches!(
        FlowTopology::new(
            [definition("a", TestKind::Material)],
            [flow("loop", TestKind::Material, Some("a"), Some("a"))],
            []
        ),
        Err(StockFlowError::SameStock(_))
    ));
    assert!(matches!(
        FlowTopology::new(
            [definition("a", TestKind::Material)],
            [flow(
                "missing",
                TestKind::Material,
                Some("a"),
                Some("missing")
            )],
            []
        ),
        Err(StockFlowError::UnknownStock(_))
    ));
    assert!(matches!(
        FlowTopology::new(
            [
                definition("a", TestKind::Material),
                definition("b", TestKind::Energy)
            ],
            [flow("wrong-kind", TestKind::Material, Some("a"), Some("b"))],
            []
        ),
        Err(StockFlowError::KindMismatch { .. })
    ));
}

#[test]
fn flows_carry_the_difference_kind_of_a_point_stock() {
    assert!(
        FlowTopology::new(
            [definition("body", TestKind::Temperature)],
            [flow("warm", TestKind::TemperatureDelta, None, Some("body"))],
            []
        )
        .is_ok()
    );
    assert_eq!(
        FlowTopology::new(
            [definition("body", TestKind::Temperature)],
            [flow("warm", TestKind::Temperature, None, Some("body"))],
            []
        ),
        Err(StockFlowError::KindMismatch {
            stock: stock("body"),
            stock_kind: TestKind::Temperature,
            flow_kind: TestKind::Temperature,
        })
    );
}

#[test]
fn a_transfer_between_balances_that_share_a_difference_is_rejected() {
    assert_eq!(
        FlowTopology::new(
            [
                definition("region", TestKind::Enthalpy),
                definition("sink", TestKind::Heat),
            ],
            [flow("leak", TestKind::Heat, Some("region"), Some("sink"))],
            []
        ),
        Err(StockFlowError::TransferKinds {
            source: stock("region"),
            source_kind: TestKind::Enthalpy,
            target: stock("sink"),
            target_kind: TestKind::Heat,
        })
    );
}

#[test]
fn both_backends_observe_pre_step_state_and_defer_inputs_until_the_next_batch() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(0), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![0.0, 0.0]).unwrap();

    let exact_first = exact
        .settle(&[integer(10), integer(10), integer(0)])
        .unwrap();
    let dense_first = dense.settle(&[10.0, 10.0, 0.0]).unwrap();
    assert_eq!(
        exact_first.applied(),
        &[integer(10), integer(0), integer(0)]
    );
    assert_eq!(dense_first.applied(), &[10.0, 0.0, 0.0]);
    assert_eq!(exact.amounts(), &[integer(10), integer(0)]);
    assert_eq!(dense.amounts(), &[10.0, 0.0]);

    exact
        .settle(&[integer(0), integer(10), integer(0)])
        .unwrap();
    dense.settle(&[0.0, 10.0, 0.0]).unwrap();
    assert_eq!(exact.amounts(), &[integer(0), integer(10)]);
    assert_eq!(dense.amounts(), &[0.0, 10.0]);
}

#[test]
fn both_backends_limit_competing_withdrawals_proportionally() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![10.0, 0.0]).unwrap();

    let exact_report = exact
        .settle(&[integer(0), integer(8), integer(12)])
        .unwrap();
    let dense_report = dense.settle(&[0.0, 8.0, 12.0]).unwrap();

    assert_eq!(
        exact_report.applied(),
        &[integer(0), integer(4), integer(6)]
    );
    assert_eq!(dense_report.applied(), &[0.0, 4.0, 6.0]);
    assert_eq!(exact.amounts(), &[integer(0), integer(4)]);
    assert_eq!(dense.amounts(), &[0.0, 4.0]);
}

#[test]
fn dense_source_limitation_exhausts_the_source_without_a_roundoff_residue() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(789_091), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![789_091.0, 0.0]).unwrap();
    let requested_exact = [integer(0), integer(719_946), integer(302_488)];
    let requested_dense = [0.0, 719_946.0, 302_488.0];

    exact.settle(&requested_exact).unwrap();
    dense.settle(&requested_dense).unwrap();

    assert!(exact.amounts()[0].is_zero());
    assert_eq!(dense.amounts()[0], 0.0);
    assert!(
        DenseTolerance::default()
            .contains(exact.amounts()[1].to_f64().unwrap(), dense.amounts()[1])
    );
}

#[test]
fn invalid_compiled_batches_are_atomic_in_both_backends() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![10.0, 0.0]).unwrap();
    let exact_before = exact.clone();
    let dense_before = dense.clone();

    assert_eq!(
        exact.settle(&[integer(0), integer(-1), integer(0)]),
        Err(StockFlowError::NegativeFlow {
            index: 1,
            process: process("move"),
            amount: Box::new(integer(-1)),
        })
    );
    assert_eq!(
        dense.settle(&[0.0, f64::NAN, 0.0]),
        Err(StockFlowError::NonFiniteAmount)
    );
    assert_eq!(
        dense.settle(&[0.0, f64::MAX, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(exact, exact_before);
    assert_eq!(dense, dense_before);
}

#[test]
fn dense_reusable_workspace_recovers_after_an_overflowing_batch() {
    let topology = material_topology();
    let mut interrupted = DenseState::new(topology.clone(), vec![10.0, 0.0]).unwrap();
    let mut uninterrupted = DenseState::new(topology, vec![10.0, 0.0]).unwrap();
    let first = [3.0, 7.0, 4.0];
    let second = [2.0, 1.0, 1.0];

    interrupted.settle(&first).unwrap();
    uninterrupted.settle(&first).unwrap();
    let before_failure = interrupted.clone();
    assert_eq!(
        interrupted.settle(&[0.0, f64::MAX, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(interrupted, before_failure);

    let interrupted_report = interrupted.settle(&second).unwrap();
    let uninterrupted_report = uninterrupted.settle(&second).unwrap();
    assert_eq!(interrupted_report, uninterrupted_report);
    assert_eq!(interrupted, uninterrupted);
}

#[test]
fn dense_reusable_workspace_recovers_from_late_failure_stages() {
    let topology = material_topology();

    // The batch aggregates are finite, but applying the boundary input would
    // overflow a stock amount.
    let mut amount_failure = DenseState::new(topology.clone(), vec![f64::MAX, 0.0]).unwrap();
    let mut amount_control = amount_failure.clone();
    let amount_before = amount_failure.clone();
    assert_eq!(
        amount_failure.settle(&[f64::MAX, 0.0, 0.0]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(amount_failure, amount_before);
    assert_eq!(
        amount_failure.settle(&[0.0, 0.0, f64::MAX]),
        amount_control.settle(&[0.0, 0.0, f64::MAX])
    );
    assert_eq!(amount_failure, amount_control);

    // A successful first input fills the cumulative account. The next batch
    // computes finite stock amounts and batch totals but overflows that account.
    let mut account_failure = DenseState::new(topology, vec![0.0, 0.0]).unwrap();
    account_failure.settle(&[f64::MAX, 0.0, 0.0]).unwrap();
    let mut account_control = account_failure.clone();
    let account_before = account_failure.clone();
    assert_eq!(
        account_failure.settle(&[f64::MAX, 0.0, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(account_failure, account_before);
    assert_eq!(
        account_failure.settle(&[0.0, 0.0, f64::MAX]),
        account_control.settle(&[0.0, 0.0, f64::MAX])
    );
    assert_eq!(account_failure, account_control);

    // Two individually finite inputs overflow their shared kind's batch
    // account after all flow and next-amount buffers have been populated.
    let input_flows = [
        flow("input-a", TestKind::Material, None, Some("a")),
        flow("input-b", TestKind::Material, None, Some("b")),
    ];
    let input_topology = Arc::new(
        FlowTopology::new(
            [
                definition("a", TestKind::Material),
                definition("b", TestKind::Material),
            ],
            input_flows.clone(),
            ration_all(&input_flows),
        )
        .unwrap(),
    );
    let mut batch_failure = DenseState::new(input_topology, vec![0.0, 0.0]).unwrap();
    let mut batch_control = batch_failure.clone();
    let batch_before = batch_failure.clone();
    assert_eq!(
        batch_failure.settle(&[f64::MAX, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(batch_failure, batch_before);
    assert_eq!(
        batch_failure.settle(&[1.0, 2.0]),
        batch_control.settle(&[1.0, 2.0])
    );
    assert_eq!(batch_failure, batch_control);
}

#[test]
fn dense_construction_and_settlement_reject_aggregate_overflow() {
    let topology = material_topology();
    assert_eq!(
        DenseState::new(topology.clone(), vec![-1.0, 0.0]),
        Err(StockFlowError::InitialBelowFloor {
            stock: stock("a"),
            floor: Box::new(integer(0)),
            amount: Box::new(integer(-1)),
        })
    );
    assert_eq!(
        DenseState::new(topology.clone(), vec![f64::INFINITY, 0.0]),
        Err(StockFlowError::NonFiniteAmount)
    );
    assert_eq!(
        DenseState::new(topology.clone(), vec![f64::MAX, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );

    let mut state = DenseState::new(topology, vec![f64::MAX, 0.0]).unwrap();
    let before = state.clone();
    assert_eq!(
        state.settle(&[0.0, f64::MAX, f64::MAX]),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(state, before);
}

#[test]
fn dense_tracks_the_exact_reference_across_repeated_batches() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![10.0, 0.0]).unwrap();
    let exact_request = [integer(3), integer(7), integer(4)];
    let dense_request = [3.0, 7.0, 4.0];

    for _ in 0..100 {
        exact.settle(&exact_request).unwrap();
        dense.settle(&dense_request).unwrap();
    }

    let tolerance = DenseTolerance::default();
    for (exact_amount, dense_amount) in exact.amounts().iter().zip(dense.amounts()) {
        assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
    }
    assert!(tolerance.contains(
        exact.inputs(TestKind::Material).to_f64().unwrap(),
        dense.inputs(TestKind::Material)
    ));
    assert!(tolerance.contains(
        exact.outputs(TestKind::Material).to_f64().unwrap(),
        dense.outputs(TestKind::Material)
    ));
    assert!(dense.balance_within(TestKind::Material, tolerance));
}

#[test]
fn dense_discard_path_matches_owned_reports_and_remains_atomic() {
    let topology = material_topology();
    let mut reported = DenseState::new(topology.clone(), vec![10.0, 0.0]).unwrap();
    let mut discarded = DenseState::new(topology, vec![10.0, 0.0]).unwrap();
    let requests = [3.0, 8.0, 12.0];

    let report = reported.settle(&requests).unwrap();
    discarded.settle_discard(&requests).unwrap();
    assert_eq!(report.requested(), &requests);
    assert_eq!(reported, discarded);

    let before = discarded.clone();
    assert_eq!(
        discarded.settle_discard(&[0.0, f64::INFINITY, 0.0]),
        Err(StockFlowError::NonFiniteAmount)
    );
    assert_eq!(discarded, before);
}

#[test]
fn compiled_exact_state_agrees_with_the_independent_legacy_settlement_path() {
    let topology = material_topology();
    let mut compiled = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut legacy = StockFlowSystem::new(
        [
            StockSpec {
                id: stock("a"),
                kind: TestKind::Material,
                initial: integer(10),
            },
            StockSpec {
                id: stock("b"),
                kind: TestKind::Material,
                initial: integer(0),
            },
        ],
        [
            declare("input", Rationing::Ration),
            declare("move", Rationing::Ration),
            declare("export", Rationing::Ration),
        ],
    )
    .unwrap();

    let compiled_report = compiled
        .settle(&[integer(3), integer(8), integer(12)])
        .unwrap();
    let legacy_report = legacy
        .settle(&[
            proposal("input", None, Some("a"), integer(3)),
            proposal("move", Some("a"), Some("b"), integer(8)),
            proposal("export", Some("a"), None, integer(12)),
        ])
        .unwrap();
    let materialized = topology.materialize_exact_report(&compiled_report).unwrap();

    assert_eq!(materialized, legacy_report);
    assert_eq!(compiled.amount(&stock("a")), legacy.amount(&stock("a")));
    assert_eq!(compiled.amount(&stock("b")), legacy.amount(&stock("b")));
    assert_eq!(
        compiled.inputs(TestKind::Material),
        legacy.inputs(TestKind::Material)
    );
    assert_eq!(
        compiled.outputs(TestKind::Material),
        legacy.outputs(TestKind::Material)
    );
}

#[test]
fn exact_report_materialization_rejects_a_different_equal_length_topology() {
    let topology = material_topology();
    let mut state = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let report = state
        .settle(&[integer(3), integer(8), integer(12)])
        .unwrap();
    let structurally_equal = material_topology();
    assert!(structurally_equal.materialize_exact_report(&report).is_ok());

    let different_flows = [
        flow("other-input", TestKind::Material, None, Some("a")),
        flow("other-move", TestKind::Material, Some("b"), Some("a")),
        flow("other-export", TestKind::Material, Some("b"), None),
    ];
    let different = FlowTopology::new(
        [
            definition("a", TestKind::Material),
            definition("b", TestKind::Material),
        ],
        different_flows.clone(),
        ration_all(&different_flows),
    )
    .unwrap();
    assert_eq!(
        different.materialize_exact_report(&report),
        Err(StockFlowError::TopologyMismatch)
    );
}

#[test]
fn each_conserved_kind_balances_independently() {
    let flows = [
        flow(
            "matter_move",
            TestKind::Matter,
            Some("matter_a"),
            Some("matter_b"),
        ),
        flow("matter_input", TestKind::Matter, None, Some("matter_a")),
        flow(
            "energy_move",
            TestKind::Energy,
            Some("energy_a"),
            Some("energy_b"),
        ),
        flow("energy_output", TestKind::Energy, Some("energy_a"), None),
    ];
    let topology = Arc::new(
        FlowTopology::new(
            [
                definition("matter_a", TestKind::Matter),
                definition("matter_b", TestKind::Matter),
                definition("energy_a", TestKind::Energy),
                definition("energy_b", TestKind::Energy),
            ],
            flows.clone(),
            ration_all(&flows),
        )
        .unwrap(),
    );
    let mut exact = ExactState::new(
        topology.clone(),
        vec![integer(10), integer(0), integer(20), integer(0)],
    )
    .unwrap();
    let mut dense = DenseState::new(topology, vec![10.0, 0.0, 20.0, 0.0]).unwrap();

    exact
        .settle(&[integer(7), integer(3), integer(4), integer(6)])
        .unwrap();
    dense.settle(&[7.0, 3.0, 4.0, 6.0]).unwrap();

    for conserved_kind in [TestKind::Matter, TestKind::Energy] {
        assert!(exact.balance_residual(conserved_kind).is_zero());
        assert!(DenseTolerance::default().contains(dense.balance_residual(conserved_kind), 0.0));
    }
}

#[test]
fn flow_declaration_permutation_does_not_change_observable_accounts() {
    let stocks = [
        definition("a", TestKind::Material),
        definition("b", TestKind::Material),
    ];
    let forward_flows = [
        flow("move", TestKind::Material, Some("a"), Some("b")),
        flow("export", TestKind::Material, Some("a"), None),
        flow("input", TestKind::Material, None, Some("b")),
    ];
    let reverse_flows = [
        flow("input", TestKind::Material, None, Some("b")),
        flow("export", TestKind::Material, Some("a"), None),
        flow("move", TestKind::Material, Some("a"), Some("b")),
    ];
    let forward = Arc::new(
        FlowTopology::new(
            stocks.clone(),
            forward_flows.clone(),
            ration_all(&forward_flows),
        )
        .unwrap(),
    );
    let reverse = Arc::new(
        FlowTopology::new(stocks, reverse_flows.clone(), ration_all(&reverse_flows)).unwrap(),
    );
    let mut exact_forward =
        ExactState::new(forward.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut exact_reverse =
        ExactState::new(reverse.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut dense_forward = DenseState::new(forward, vec![10.0, 0.0]).unwrap();
    let mut dense_reverse = DenseState::new(reverse, vec![10.0, 0.0]).unwrap();

    exact_forward
        .settle(&[integer(8), integer(12), integer(3)])
        .unwrap();
    exact_reverse
        .settle(&[integer(3), integer(12), integer(8)])
        .unwrap();
    dense_forward.settle(&[8.0, 12.0, 3.0]).unwrap();
    dense_reverse.settle(&[3.0, 12.0, 8.0]).unwrap();

    assert_eq!(exact_forward.amounts(), exact_reverse.amounts());
    assert_eq!(
        exact_forward.inputs(TestKind::Material),
        exact_reverse.inputs(TestKind::Material)
    );
    assert_eq!(
        exact_forward.outputs(TestKind::Material),
        exact_reverse.outputs(TestKind::Material)
    );
    for (left, right) in dense_forward.amounts().iter().zip(dense_reverse.amounts()) {
        assert!(DenseTolerance::default().contains(*left, *right));
    }
    assert!(DenseTolerance::default().contains(
        dense_forward.inputs(TestKind::Material),
        dense_reverse.inputs(TestKind::Material)
    ));
    assert!(DenseTolerance::default().contains(
        dense_forward.outputs(TestKind::Material),
        dense_reverse.outputs(TestKind::Material)
    ));
}

#[test]
fn dense_permutation_is_stable_across_adversarial_dynamic_range() {
    let stocks = [
        definition("source", TestKind::Material),
        definition("big", TestKind::Material),
        definition("small", TestKind::Material),
    ];
    let mut flows = vec![flow("big", TestKind::Material, Some("source"), Some("big"))];
    flows.extend(
        (0..2_000).map(|_| flow("small", TestKind::Material, Some("source"), Some("small"))),
    );
    let mut reversed_flows = flows.clone();
    reversed_flows.reverse();
    let forward_processes = ration_all(&flows);
    let reverse_processes = ration_all(&reversed_flows);
    let forward = Arc::new(FlowTopology::new(stocks.clone(), flows, forward_processes).unwrap());
    let reverse = Arc::new(FlowTopology::new(stocks, reversed_flows, reverse_processes).unwrap());
    let mut forward_state = DenseState::new(forward, vec![1e16, 0.0, 0.0]).unwrap();
    let mut reverse_state = DenseState::new(reverse, vec![1e16, 0.0, 0.0]).unwrap();
    let mut forward_requests = vec![1.0; 2_001];
    forward_requests[0] = 1e16;
    let mut reverse_requests = forward_requests.clone();
    reverse_requests.reverse();

    forward_state.settle(&forward_requests).unwrap();
    reverse_state.settle(&reverse_requests).unwrap();

    for stock_name in ["source", "big", "small"] {
        assert_eq!(
            forward_state.amount(&stock(stock_name)),
            reverse_state.amount(&stock(stock_name))
        );
    }
}

#[test]
fn dense_settles_a_valid_eleven_thousand_way_proportional_batch() {
    const FANOUT: usize = 11_000;
    let stocks = [
        definition("source", TestKind::Material),
        definition("target", TestKind::Material),
    ];
    let flows: Vec<_> = (0..FANOUT)
        .map(|index| {
            flow(
                &format!("branch-{index}"),
                TestKind::Material,
                Some("source"),
                Some("target"),
            )
        })
        .collect();
    let processes = ration_all(&flows);
    let topology = Arc::new(FlowTopology::new(stocks, flows, processes).unwrap());
    let mut state = DenseState::new(topology, vec![1.0, 0.0]).unwrap();

    let report = state.settle(&vec![1.0; FANOUT]).unwrap();

    let expected_branch = 1.0 / FANOUT as f64;
    assert!(
        report
            .applied()
            .iter()
            .all(|amount| DenseTolerance::default().contains(*amount, expected_branch))
    );
    assert!(DenseTolerance::default().contains(state.amount(&stock("source")).unwrap(), 0.0));
    assert!(DenseTolerance::default().contains(state.amount(&stock("target")).unwrap(), 1.0));
    assert!(state.balance_within(TestKind::Material, DenseTolerance::default()));
}

#[test]
fn dense_source_overflow_rejection_is_independent_of_flow_order() {
    let stocks = [
        definition("source", TestKind::Material),
        definition("target", TestKind::Material),
    ];
    let mut flows = vec![flow(
        "large",
        TestKind::Material,
        Some("source"),
        Some("target"),
    )];
    flows.extend((0..3).map(|_| flow("small", TestKind::Material, Some("source"), Some("target"))));
    let mut reversed_flows = flows.clone();
    reversed_flows.reverse();
    let forward_processes = ration_all(&flows);
    let reverse_processes = ration_all(&reversed_flows);
    let forward = Arc::new(FlowTopology::new(stocks.clone(), flows, forward_processes).unwrap());
    let reverse = Arc::new(FlowTopology::new(stocks, reversed_flows, reverse_processes).unwrap());
    let mut forward_state = DenseState::new(forward, vec![f64::MAX, 0.0]).unwrap();
    let mut reverse_state = DenseState::new(reverse, vec![f64::MAX, 0.0]).unwrap();
    let small = 2.0_f64.powi(969);
    let forward_requests = [f64::MAX, small, small, small];
    let mut reverse_requests = forward_requests;
    reverse_requests.reverse();
    let forward_before = forward_state.clone();
    let reverse_before = reverse_state.clone();

    assert_eq!(
        forward_state.settle(&forward_requests),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(
        reverse_state.settle(&reverse_requests),
        Err(StockFlowError::ArithmeticOverflow)
    );
    assert_eq!(forward_state, forward_before);
    assert_eq!(reverse_state, reverse_before);
}

#[test]
fn dense_balance_diagnostics_do_not_overflow_on_cancelling_maxima() {
    let flows = [
        flow("input", TestKind::Material, None, Some("stock")),
        flow("output", TestKind::Material, Some("stock"), None),
    ];
    let topology = Arc::new(
        FlowTopology::new(
            [definition("stock", TestKind::Material)],
            flows.clone(),
            ration_all(&flows),
        )
        .unwrap(),
    );
    let mut state = DenseState::new(topology, vec![f64::MAX]).unwrap();
    state.settle(&[f64::MAX, f64::MAX]).unwrap();

    assert_eq!(state.balance_residual(TestKind::Material), 0.0);
    assert!(state.balance_within(TestKind::Material, DenseTolerance::default()));
    assert!(!state.balance_within(TestKind::Charge, DenseTolerance::default()));
    assert!(!state.balance_within(
        TestKind::Material,
        DenseTolerance {
            absolute: f64::NAN,
            relative: 0.0,
        }
    ));
}

#[test]
fn compiled_amount_count_and_nonfinite_failures_are_atomic() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(1), integer(2)]).unwrap();
    let mut dense = DenseState::new(topology, vec![1.0, 2.0]).unwrap();
    let exact_before = exact.clone();
    let dense_before = dense.clone();

    assert!(matches!(
        exact.settle(&[integer(1)]),
        Err(StockFlowError::AmountCount { .. })
    ));
    assert_eq!(
        dense.settle(&[0.0, f64::INFINITY, 0.0]),
        Err(StockFlowError::NonFiniteAmount)
    );
    assert_eq!(exact, exact_before);
    assert_eq!(dense, dense_before);
}

fn exact_and_dense(
    stocks: &[(&str, TestKind)],
    flows: &[FlowSpec<TestKind>],
    processes: Vec<ProcessDefinition>,
    initial: &[i64],
) -> (ExactState<TestKind>, DenseState<TestKind>) {
    let topology = Arc::new(
        FlowTopology::new(
            stocks.iter().map(|(id, kind)| definition(id, *kind)),
            flows.to_vec(),
            processes,
        )
        .unwrap(),
    );
    let exact = ExactState::new(
        topology.clone(),
        initial.iter().copied().map(integer).collect(),
    )
    .unwrap();
    let dense = DenseState::new(
        topology,
        initial.iter().map(|value| *value as f64).collect(),
    )
    .unwrap();
    (exact, dense)
}

#[test]
fn floorless_stock_starts_negative_and_settles_below_zero_in_both_backends() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("a", TestKind::MaterialBalance)],
        &[flow("export", TestKind::MaterialBalance, Some("a"), None)],
        Vec::new(),
        &[-2],
    );

    exact.settle(&[integer(5)]).unwrap();
    dense.settle(&[5.0]).unwrap();

    assert_eq!(exact.amounts(), &[integer(-7)]);
    assert_eq!(dense.amounts(), &[-7.0]);
    assert!(exact.balance_residual(TestKind::MaterialBalance).is_zero());
    assert_eq!(dense.balance_residual(TestKind::MaterialBalance), 0.0);
    assert!(dense.balance_within(TestKind::MaterialBalance, DenseTolerance::default()));
}

#[test]
fn cooling_trace_does_not_depend_on_the_enthalpy_reference() {
    let topology = Arc::new(
        FlowTopology::new(
            [definition("region", TestKind::Enthalpy)],
            [
                flow("bath_exchange", TestKind::Heat, None, Some("region")),
                flow("bath_exchange", TestKind::Heat, Some("region"), None),
            ],
            [],
        )
        .unwrap(),
    );
    let capacity = integer(1000);
    // T₇ = 300 + 100·(9/10)⁷ = 347.82969.
    let expected = integer(300) + BigRational::new(BigInt::from(4_782_969), BigInt::from(100_000));

    for reference in [300_i64, 350] {
        let reference_exact = integer(reference);
        let mut exact = ExactState::new(
            topology.clone(),
            vec![(integer(400) - &reference_exact) * &capacity],
        )
        .unwrap();
        let mut dense =
            DenseState::new(topology.clone(), vec![(400.0 - reference as f64) * 1000.0]).unwrap();

        for _ in 0..7 {
            let temperature = &reference_exact + &exact.amounts()[0] / &capacity;
            let heat = integer(100) * (integer(300) - temperature);
            let slots = if heat.is_negative() {
                [integer(0), -heat]
            } else {
                [heat, integer(0)]
            };
            exact.settle(&slots).unwrap();

            let dense_temperature = reference as f64 + dense.amounts()[0] / 1000.0;
            let dense_heat = 100.0 * (300.0 - dense_temperature);
            let dense_slots = if dense_heat < 0.0 {
                [0.0, -dense_heat]
            } else {
                [dense_heat, 0.0]
            };
            dense.settle(&dense_slots).unwrap();
        }

        assert_eq!(&reference_exact + &exact.amounts()[0] / &capacity, expected);
        assert!(DenseTolerance::default().contains(
            reference as f64 + dense.amounts()[0] / 1000.0,
            expected.to_f64().unwrap()
        ));
    }

    let below_reference = integer(1000) * (integer(340) - integer(350));
    assert!(ExactState::new(topology.clone(), vec![below_reference]).is_ok());
    assert!(DenseState::new(topology, vec![1000.0 * (340.0 - 350.0)]).is_ok());
}

#[test]
fn signed_stock_settles_from_plus_two_to_minus_three_through_one_flow() {
    let (mut exact, mut dense) = exact_and_dense(
        &[
            ("p", TestKind::MaterialBalance),
            ("q", TestKind::MaterialBalance),
        ],
        &[flow(
            "push",
            TestKind::MaterialBalance,
            Some("p"),
            Some("q"),
        )],
        Vec::new(),
        &[2, 0],
    );

    exact.settle(&[integer(5)]).unwrap();
    dense.settle(&[5.0]).unwrap();

    assert_eq!(exact.amounts(), &[integer(-3), integer(5)]);
    assert_eq!(dense.amounts(), &[-3.0, 5.0]);
    assert!(exact.balance_residual(TestKind::MaterialBalance).is_zero());
    assert_eq!(dense.balance_residual(TestKind::MaterialBalance), 0.0);
}

fn below_floor(
    stock_name: &str,
    floor: i64,
    process_name: &str,
    available: i64,
    withdrawal: i64,
) -> StockFlowError<TestKind> {
    StockFlowError::BelowFloor {
        stock: stock(stock_name),
        floor: Box::new(integer(floor)),
        process: process(process_name),
        available: Box::new(integer(available)),
        withdrawal: Box::new(integer(withdrawal)),
    }
}

#[test]
fn withdrawal_below_floor_is_refused_naming_stock_floor_and_process() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("vault", TestKind::Reserve)],
        &[flow("spend", TestKind::Reserve, Some("vault"), None)],
        Vec::new(),
        &[12],
    );
    let exact_before = exact.clone();
    let dense_before = dense.clone();
    let expected = below_floor("vault", 10, "spend", 12, 5);

    assert_eq!(exact.settle(&[integer(5)]), Err(expected.clone()));
    assert_eq!(dense.settle(&[5.0]), Err(expected.clone()));
    assert_eq!(exact, exact_before);
    assert_eq!(dense, dense_before);

    let mut system = StockFlowSystem::new(
        [StockSpec {
            id: stock("vault"),
            kind: TestKind::Reserve,
            initial: integer(12),
        }],
        [],
    )
    .unwrap();
    let system_before = system.clone();
    assert_eq!(
        system.settle(&[ProposedFlow {
            process: process("spend"),
            kind: TestKind::Reserve,
            source: Some(stock("vault")),
            target: None,
            amount: integer(5),
        }]),
        Err(expected.clone())
    );
    assert_eq!(system, system_before);

    let message = expected.to_string();
    for part in ["vault", "10", "spend"] {
        assert!(message.contains(part), "{message}");
    }
}

#[test]
fn material_withdrawal_beyond_the_stock_is_refused_by_default() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("a", TestKind::Material)],
        &[flow("export", TestKind::Material, Some("a"), None)],
        Vec::new(),
        &[3],
    );
    let expected = below_floor("a", 0, "export", 3, 5);

    assert_eq!(exact.settle(&[integer(5)]), Err(expected.clone()));
    assert_eq!(dense.settle(&[5.0]), Err(expected));
}

#[test]
fn a_refusing_process_sharing_a_breached_stock_refuses_the_batch() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("vault", TestKind::Reserve)],
        &[
            flow("share", TestKind::Reserve, Some("vault"), None),
            flow("spend", TestKind::Reserve, Some("vault"), None),
        ],
        vec![declare("share", Rationing::Ration)],
        &[12],
    );
    let expected = below_floor("vault", 10, "spend", 12, 3);

    assert_eq!(
        exact.settle(&[integer(2), integer(1)]),
        Err(expected.clone())
    );
    assert_eq!(dense.settle(&[2.0, 1.0]), Err(expected));
}

#[test]
fn rationed_withdrawals_share_the_headroom_above_a_nonzero_floor() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("vault", TestKind::Reserve)],
        &[
            flow("a", TestKind::Reserve, Some("vault"), None),
            flow("b", TestKind::Reserve, Some("vault"), None),
        ],
        vec![
            declare("a", Rationing::Ration),
            declare("b", Rationing::Ration),
        ],
        &[12],
    );

    let exact_report = exact.settle(&[integer(3), integer(1)]).unwrap();
    let dense_report = dense.settle(&[3.0, 1.0]).unwrap();

    assert_eq!(
        exact_report.applied(),
        &[
            BigRational::new(BigInt::from(3), BigInt::from(2)),
            BigRational::new(BigInt::from(1), BigInt::from(2)),
        ]
    );
    assert_eq!(exact.amounts(), &[integer(10)]);
    assert_eq!(dense_report.applied(), &[1.5, 0.5]);
    assert_eq!(dense.amounts(), &[10.0]);
}

#[test]
fn floorless_stock_is_never_rationed() {
    let (mut exact, mut dense) = exact_and_dense(
        &[("m", TestKind::MaterialBalance)],
        &[flow("draw", TestKind::MaterialBalance, Some("m"), None)],
        vec![declare("draw", Rationing::Ration)],
        &[1],
    );

    let exact_report = exact.settle(&[integer(5)]).unwrap();
    let dense_report = dense.settle(&[5.0]).unwrap();

    assert_eq!(exact_report.applied(), &[integer(5)]);
    assert_eq!(exact.amounts(), &[integer(-4)]);
    assert_eq!(dense_report.applied(), &[5.0]);
    assert_eq!(dense.amounts(), &[-4.0]);
}

#[test]
fn process_declarations_reject_unknown_and_duplicate_processes() {
    let stocks = [
        definition("a", TestKind::Material),
        definition("b", TestKind::Material),
    ];
    let flows = [flow("move", TestKind::Material, Some("a"), Some("b"))];
    assert_eq!(
        FlowTopology::new(
            stocks.clone(),
            flows.clone(),
            [declare("ghost", Rationing::Ration)]
        ),
        Err(StockFlowError::UnknownProcess(process("ghost")))
    );
    assert_eq!(
        FlowTopology::new(
            stocks,
            flows,
            [
                declare("move", Rationing::Ration),
                declare("move", Rationing::Refuse)
            ]
        ),
        Err(StockFlowError::DuplicateProcess(process("move")))
    );
    assert_eq!(
        StockFlowSystem::new(
            [StockSpec {
                id: stock("a"),
                kind: TestKind::Material,
                initial: integer(0),
            }],
            [
                declare("move", Rationing::Ration),
                declare("move", Rationing::Ration)
            ]
        ),
        Err(StockFlowError::DuplicateProcess(process("move")))
    );
}

#[test]
fn topology_reports_declared_rationing() {
    let topology = material_topology();
    assert_eq!(
        topology.rationing(&process("move")),
        Some(Rationing::Ration)
    );
    assert_eq!(
        topology.rationing(&process("input")),
        Some(Rationing::Refuse)
    );
    assert_eq!(topology.rationing(&process("ghost")), None);
}

#[test]
fn initial_amount_below_the_floor_is_rejected_naming_stock_and_floor() {
    let topology = Arc::new(
        FlowTopology::new(
            [definition("vault", TestKind::Reserve)],
            [flow("spend", TestKind::Reserve, Some("vault"), None)],
            [],
        )
        .unwrap(),
    );
    let expected = StockFlowError::InitialBelowFloor {
        stock: stock("vault"),
        floor: Box::new(integer(10)),
        amount: Box::new(integer(9)),
    };

    assert_eq!(
        ExactState::new(topology.clone(), vec![integer(9)]),
        Err(expected.clone())
    );
    assert_eq!(DenseState::new(topology, vec![9.0]), Err(expected.clone()));
    assert_eq!(
        StockFlowSystem::new(
            [StockSpec {
                id: stock("vault"),
                kind: TestKind::Reserve,
                initial: integer(9),
            }],
            [],
        ),
        Err(expected)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn exact_and_dense_agree_on_floor_refusal_and_rationing(
        vault in 10_i64..1_000,
        safe in 10_i64..1_000,
        spend in 0_i64..1_000,
        share in 0_i64..1_000,
        drain in 0_i64..1_000,
    ) {
        let (mut exact, mut dense) = exact_and_dense(
            &[("vault", TestKind::Reserve), ("safe", TestKind::Reserve)],
            &[
                flow("spend", TestKind::Reserve, Some("vault"), None),
                flow("share", TestKind::Reserve, Some("vault"), Some("safe")),
                flow("drain", TestKind::Reserve, Some("safe"), None),
            ],
            vec![declare("share", Rationing::Ration), declare("drain", Rationing::Ration)],
            &[vault, safe],
        );
        let exact_result = exact.settle(&[integer(spend), integer(share), integer(drain)]);
        let dense_result = dense.settle(&[spend as f64, share as f64, drain as f64]);
        let tolerance = DenseTolerance::default();

        match (exact_result, dense_result) {
            (Err(exact_error), Err(dense_error)) => prop_assert_eq!(exact_error, dense_error),
            (Ok(exact_report), Ok(dense_report)) => {
                for (exact_amount, dense_amount) in exact.amounts().iter().zip(dense.amounts()) {
                    prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
                    prop_assert!(*exact_amount >= integer(10));
                    prop_assert!(*dense_amount >= 10.0);
                }
                for (exact_amount, dense_amount) in exact_report.applied().iter().zip(dense_report.applied()) {
                    prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
                }
                prop_assert!(exact.balance_residual(TestKind::Reserve).is_zero());
                prop_assert!(dense.balance_within(TestKind::Reserve, tolerance));
            }
            (exact_result, dense_result) => {
                prop_assert!(false, "exact {exact_result:?} but dense {dense_result:?}");
            }
        }
    }

    #[test]
    fn dense_is_finite_nonnegative_balanced_and_agrees_with_exact(
        a in 0_i64..1_000_000,
        b in 0_i64..1_000_000,
        input in 0_i64..1_000_000,
        transfer in 0_i64..1_000_000,
        output in 0_i64..1_000_000,
    ) {
        let topology = material_topology();
        let mut exact = ExactState::new(topology.clone(), vec![integer(a), integer(b)]).unwrap();
        let mut dense = DenseState::new(topology, vec![a as f64, b as f64]).unwrap();
        let exact_report = exact
            .settle(&[integer(input), integer(transfer), integer(output)])
            .unwrap();
        let dense_report = dense
            .settle(&[input as f64, transfer as f64, output as f64])
            .unwrap();
        let tolerance = DenseTolerance::default();

        for amount in dense.amounts() {
            prop_assert!(amount.is_finite());
            prop_assert!(*amount >= 0.0);
        }
        prop_assert!(dense.balance_within(TestKind::Material, tolerance));
        for (exact_amount, dense_amount) in exact.amounts().iter().zip(dense.amounts()) {
            prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
        }
        for (exact_amount, dense_amount) in exact_report.applied().iter().zip(dense_report.applied()) {
            prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
        }
        prop_assert!(exact.amounts().iter().all(|amount| !amount.is_negative()));
        prop_assert!(exact.balance_residual(TestKind::Material).is_zero());
    }
}
