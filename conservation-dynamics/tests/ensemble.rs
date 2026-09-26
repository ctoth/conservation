use conservation_dynamics::{
    DenseState, DenseTolerance, EnsembleError, FlowSpec, FlowTopology, ProcessId, StockDefinition,
    StockId,
};
use conservation_test_kinds::TestKind;
use std::sync::Arc;

fn topology() -> Arc<FlowTopology<TestKind>> {
    Arc::new(
        FlowTopology::new(
            ["a", "b"].map(|id| StockDefinition {
                id: StockId::new(id).unwrap(),
                kind: TestKind::Material,
            }),
            [FlowSpec {
                process: ProcessId::new("move").unwrap(),
                kind: TestKind::Material,
                source: Some(StockId::new("a").unwrap()),
                target: Some(StockId::new("b").unwrap()),
            }],
            [],
        )
        .unwrap(),
    )
}

#[test]
fn independent_lanes_match_dense_scalar_execution_and_keep_owned_outputs() {
    let initial = [10.0, 0.0, 20.0, 1.0];
    let trajectory = DenseState::ensemble(
        topology(),
        &initial,
        2,
        3,
        DenseTolerance::default(),
        |lane, _step, state, requested| -> Result<(), &'static str> {
            requested[0] = state.amounts()[0] * (lane + 1) as f64 / 10.0;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(trajectory.shape, [2, 4, 2]);
    for lane in 0..2 {
        let mut scalar =
            DenseState::new(topology(), initial[lane * 2..lane * 2 + 2].to_vec()).unwrap();
        for sample in 0..4 {
            assert_eq!(
                &trajectory.amounts[(lane * 4 + sample) * 2..(lane * 4 + sample + 1) * 2],
                scalar.amounts()
            );
            assert!(trajectory.balances[lane * 4 + sample].satisfied);
            if sample < 3 {
                scalar
                    .settle_discard(&[scalar.amounts()[0] * (lane + 1) as f64 / 10.0])
                    .unwrap();
            }
        }
    }
    assert_eq!(initial, [10.0, 0.0, 20.0, 1.0]);
}

#[test]
fn validation_precedes_callbacks_and_errors_keep_lane_step_and_source() {
    let mut calls = 0;
    let failed = DenseState::ensemble(
        topology(),
        &[10.0, 0.0, f64::NAN, 0.0],
        2,
        2,
        DenseTolerance::default(),
        |_, _, _, _| -> Result<(), &'static str> {
            calls += 1;
            Ok(())
        },
    );
    assert!(matches!(
        failed,
        Err(EnsembleError::Settlement {
            lane: 1,
            step: None,
            ..
        })
    ));
    assert_eq!(calls, 0);
    let failed = DenseState::ensemble(
        topology(),
        &[10.0, 0.0],
        1,
        2,
        DenseTolerance::default(),
        |_, step, _, _| {
            if step == 1 {
                Err("rate refusal")
            } else {
                Ok(())
            }
        },
    );
    assert!(
        matches!(failed, Err(EnsembleError::Proposal { lane: 0, step: 1, source }) if *source == "rate refusal")
    );
    let failed = DenseState::ensemble(
        topology(),
        &[],
        1,
        2,
        DenseTolerance::default(),
        |_, _, _, _| -> Result<(), ()> { unreachable!() },
    );
    assert!(matches!(
        failed,
        Err(EnsembleError::Shape {
            expected: 2,
            actual: 0
        })
    ));
    let empty = DenseState::ensemble(
        topology(),
        &[],
        0,
        0,
        DenseTolerance::default(),
        |_, _, _, _| -> Result<(), ()> { unreachable!() },
    )
    .unwrap();
    assert_eq!(empty.shape, [0, 1, 2]);
    let refused = DenseState::ensemble(
        topology(),
        &[10.0, 0.0],
        1,
        1,
        DenseTolerance::default(),
        |_, _, _, requested| -> Result<(), ()> {
            requested[0] = 11.0;
            Ok(())
        },
    );
    assert!(matches!(
        refused,
        Err(EnsembleError::Settlement {
            lane: 0,
            step: Some(0),
            ..
        })
    ));
    let overflow = DenseState::ensemble(
        topology(),
        &[],
        0,
        usize::MAX,
        DenseTolerance::default(),
        |_, _, _, _| -> Result<(), ()> { unreachable!() },
    );
    assert!(matches!(overflow, Err(EnsembleError::ShapeOverflow { .. })));
    let invalid = DenseState::ensemble(
        topology(),
        &[],
        0,
        0,
        DenseTolerance {
            absolute: f64::NAN,
            relative: 0.0,
        },
        |_, _, _, _| -> Result<(), ()> { unreachable!() },
    );
    assert!(matches!(invalid, Err(EnsembleError::Tolerance(_))));
}
