use std::sync::Arc;

use conservation_core::KindId;
use conservation_dynamics::{
    DenseState, DenseTolerance, ExactState, FlowRole, FlowSpec, FlowTopology, ProcessId,
    ProposedFlow, StockDefinition, StockFlowError, StockFlowSystem, StockId, StockSpec,
};
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

fn kind(value: &str) -> KindId {
    KindId::new(value).unwrap()
}

fn integer(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn definition(id: &str, kind_name: &str) -> StockDefinition {
    StockDefinition {
        id: stock(id),
        kind: kind(kind_name),
    }
}

fn flow(
    process_name: &str,
    kind_name: &str,
    source: Option<&str>,
    target: Option<&str>,
) -> FlowSpec {
    FlowSpec {
        process: process(process_name),
        kind: kind(kind_name),
        source: source.map(stock),
        target: target.map(stock),
    }
}

fn material_topology() -> Arc<FlowTopology> {
    Arc::new(
        FlowTopology::new(
            [definition("a", "material"), definition("b", "material")],
            [
                flow("input", "material", None, Some("a")),
                flow("move", "material", Some("a"), Some("b")),
                flow("export", "material", Some("a"), None),
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
) -> ProposedFlow {
    ProposedFlow {
        process: process(process_name),
        kind: kind("material"),
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
    assert_eq!(topology.kind_index(&kind("material")), Some(0));
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
        FlowTopology::new([], std::iter::empty::<FlowSpec>()),
        Err(StockFlowError::NoStocks)
    );
    assert!(matches!(
        FlowTopology::new(
            [definition("a", "material"), definition("a", "material")],
            []
        ),
        Err(StockFlowError::DuplicateStock(_))
    ));
    assert_eq!(
        FlowTopology::new(
            [definition("a", "material")],
            [flow("lost", "material", None, None)]
        ),
        Err(StockFlowError::DisconnectedFlow)
    );
    assert!(matches!(
        FlowTopology::new(
            [definition("a", "material")],
            [flow("loop", "material", Some("a"), Some("a"))]
        ),
        Err(StockFlowError::SameStock(_))
    ));
    assert!(matches!(
        FlowTopology::new(
            [definition("a", "material")],
            [flow("missing", "material", Some("a"), Some("missing"))]
        ),
        Err(StockFlowError::UnknownStock(_))
    ));
    assert!(matches!(
        FlowTopology::new(
            [definition("a", "material"), definition("b", "energy")],
            [flow("wrong-kind", "material", Some("a"), Some("b"))]
        ),
        Err(StockFlowError::KindMismatch { .. })
    ));
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
fn invalid_compiled_batches_are_atomic_in_both_backends() {
    let topology = material_topology();
    let mut exact = ExactState::new(topology.clone(), vec![integer(10), integer(0)]).unwrap();
    let mut dense = DenseState::new(topology, vec![10.0, 0.0]).unwrap();
    let exact_before = exact.clone();
    let dense_before = dense.clone();

    assert_eq!(
        exact.settle(&[integer(0), integer(-1), integer(0)]),
        Err(StockFlowError::NegativeAmount)
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
    let input_topology = Arc::new(
        FlowTopology::new(
            [definition("a", "material"), definition("b", "material")],
            [
                flow("input-a", "material", None, Some("a")),
                flow("input-b", "material", None, Some("b")),
            ],
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
        Err(StockFlowError::NegativeAmount)
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
        exact.inputs(&kind("material")).to_f64().unwrap(),
        dense.inputs(&kind("material"))
    ));
    assert!(tolerance.contains(
        exact.outputs(&kind("material")).to_f64().unwrap(),
        dense.outputs(&kind("material"))
    ));
    assert!(dense.balance_within(&kind("material"), tolerance));
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
    let mut legacy = StockFlowSystem::new([
        StockSpec {
            id: stock("a"),
            kind: kind("material"),
            initial: integer(10),
        },
        StockSpec {
            id: stock("b"),
            kind: kind("material"),
            initial: integer(0),
        },
    ])
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
        compiled.inputs(&kind("material")),
        legacy.inputs(&kind("material"))
    );
    assert_eq!(
        compiled.outputs(&kind("material")),
        legacy.outputs(&kind("material"))
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

    let different = FlowTopology::new(
        [definition("a", "material"), definition("b", "material")],
        [
            flow("other-input", "material", None, Some("a")),
            flow("other-move", "material", Some("b"), Some("a")),
            flow("other-export", "material", Some("b"), None),
        ],
    )
    .unwrap();
    assert_eq!(
        different.materialize_exact_report(&report),
        Err(StockFlowError::TopologyMismatch)
    );
}

#[test]
fn each_conserved_kind_balances_independently() {
    let topology = Arc::new(
        FlowTopology::new(
            [
                definition("matter_a", "matter"),
                definition("matter_b", "matter"),
                definition("energy_a", "energy"),
                definition("energy_b", "energy"),
            ],
            [
                flow("matter_move", "matter", Some("matter_a"), Some("matter_b")),
                flow("matter_input", "matter", None, Some("matter_a")),
                flow("energy_move", "energy", Some("energy_a"), Some("energy_b")),
                flow("energy_output", "energy", Some("energy_a"), None),
            ],
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

    for conserved_kind in [kind("matter"), kind("energy")] {
        assert!(exact.balance_residual(&conserved_kind).is_zero());
        assert!(DenseTolerance::default().contains(dense.balance_residual(&conserved_kind), 0.0));
    }
}

#[test]
fn flow_declaration_permutation_does_not_change_observable_accounts() {
    let stocks = [definition("a", "material"), definition("b", "material")];
    let forward = Arc::new(
        FlowTopology::new(
            stocks.clone(),
            [
                flow("move", "material", Some("a"), Some("b")),
                flow("export", "material", Some("a"), None),
                flow("input", "material", None, Some("b")),
            ],
        )
        .unwrap(),
    );
    let reverse = Arc::new(
        FlowTopology::new(
            stocks,
            [
                flow("input", "material", None, Some("b")),
                flow("export", "material", Some("a"), None),
                flow("move", "material", Some("a"), Some("b")),
            ],
        )
        .unwrap(),
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
        exact_forward.inputs(&kind("material")),
        exact_reverse.inputs(&kind("material"))
    );
    assert_eq!(
        exact_forward.outputs(&kind("material")),
        exact_reverse.outputs(&kind("material"))
    );
    for (left, right) in dense_forward.amounts().iter().zip(dense_reverse.amounts()) {
        assert!(DenseTolerance::default().contains(*left, *right));
    }
    assert!(DenseTolerance::default().contains(
        dense_forward.inputs(&kind("material")),
        dense_reverse.inputs(&kind("material"))
    ));
    assert!(DenseTolerance::default().contains(
        dense_forward.outputs(&kind("material")),
        dense_reverse.outputs(&kind("material"))
    ));
}

#[test]
fn dense_permutation_is_stable_across_adversarial_dynamic_range() {
    let stocks = [
        definition("source", "material"),
        definition("big", "material"),
        definition("small", "material"),
    ];
    let mut flows = vec![flow("big", "material", Some("source"), Some("big"))];
    flows.extend((0..2_000).map(|_| flow("small", "material", Some("source"), Some("small"))));
    let mut reversed_flows = flows.clone();
    reversed_flows.reverse();
    let forward = Arc::new(FlowTopology::new(stocks.clone(), flows).unwrap());
    let reverse = Arc::new(FlowTopology::new(stocks, reversed_flows).unwrap());
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
        definition("source", "material"),
        definition("target", "material"),
    ];
    let flows = (0..FANOUT).map(|index| {
        flow(
            &format!("branch-{index}"),
            "material",
            Some("source"),
            Some("target"),
        )
    });
    let topology = Arc::new(FlowTopology::new(stocks, flows).unwrap());
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
    assert!(state.balance_within(&kind("material"), DenseTolerance::default()));
}

#[test]
fn dense_source_overflow_rejection_is_independent_of_flow_order() {
    let stocks = [
        definition("source", "material"),
        definition("target", "material"),
    ];
    let mut flows = vec![flow("large", "material", Some("source"), Some("target"))];
    flows.extend((0..3).map(|_| flow("small", "material", Some("source"), Some("target"))));
    let mut reversed_flows = flows.clone();
    reversed_flows.reverse();
    let forward = Arc::new(FlowTopology::new(stocks.clone(), flows).unwrap());
    let reverse = Arc::new(FlowTopology::new(stocks, reversed_flows).unwrap());
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
    let topology = Arc::new(
        FlowTopology::new(
            [definition("stock", "material")],
            [
                flow("input", "material", None, Some("stock")),
                flow("output", "material", Some("stock"), None),
            ],
        )
        .unwrap(),
    );
    let mut state = DenseState::new(topology, vec![f64::MAX]).unwrap();
    state.settle(&[f64::MAX, f64::MAX]).unwrap();

    assert_eq!(state.balance_residual(&kind("material")), 0.0);
    assert!(state.balance_within(&kind("material"), DenseTolerance::default()));
    assert!(!state.balance_within(&kind("misspelled"), DenseTolerance::default()));
    assert!(!state.balance_within(
        &kind("material"),
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

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
        prop_assert!(dense.balance_within(&kind("material"), tolerance));
        for (exact_amount, dense_amount) in exact.amounts().iter().zip(dense.amounts()) {
            prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
        }
        for (exact_amount, dense_amount) in exact_report.applied().iter().zip(dense_report.applied()) {
            prop_assert!(tolerance.contains(exact_amount.to_f64().unwrap(), *dense_amount));
        }
        prop_assert!(exact.amounts().iter().all(|amount| !amount.is_negative()));
        prop_assert!(exact.balance_residual(&kind("material")).is_zero());
    }
}
