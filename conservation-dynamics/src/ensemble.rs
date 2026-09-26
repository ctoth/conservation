//! Independent dense trajectories using the existing compiled settlement owner.
use crate::{DenseState, DenseTolerance, FlowTopology, StockFlowError};
use conservation_core::Kind;
use std::{collections::TryReserveError, fmt, sync::Arc};

/// A floating-point balance diagnostic, never an exact conservation witness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DenseBalance {
    pub residual: f64,
    pub satisfied: bool,
}

/// Owned lane-major output, with axes `(lane, sample, stock)`. Samples include
/// the initial state. Stock and kind order come from the shared topology.
#[derive(Clone, Debug)]
pub struct DenseTrajectory<K> {
    pub topology: Arc<FlowTopology<K>>,
    pub shape: [usize; 3],
    pub amounts: Vec<f64>,
    /// Lane-major `(lane, sample, kind)` diagnostics at every sample.
    pub balances: Vec<DenseBalance>,
    pub tolerance: DenseTolerance,
}

/// Batch errors keep the initial lane or transition and the original refusal.
#[derive(Debug)]
pub enum EnsembleError<K, E> {
    Shape {
        expected: usize,
        actual: usize,
    },
    ShapeOverflow {
        lanes: usize,
        steps: usize,
        stocks: usize,
    },
    Allocation(TryReserveError),
    Tolerance(DenseTolerance),
    Settlement {
        lane: usize,
        step: Option<usize>,
        source: Box<StockFlowError<K>>,
    },
    Proposal {
        lane: usize,
        step: usize,
        source: Box<E>,
    },
}
impl<K: Kind, E: fmt::Display> fmt::Display for EnsembleError<K, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape { expected, actual } => write!(
                f,
                "expected {expected} initial stock coordinates, got {actual}"
            ),
            Self::ShapeOverflow {
                lanes,
                steps,
                stocks,
            } => write!(
                f,
                "ensemble shape overflows: {lanes} lanes, {steps} steps, {stocks} stocks"
            ),
            Self::Allocation(source) => source.fmt(f),
            Self::Tolerance(tolerance) => write!(f, "invalid dense tolerance {tolerance:?}"),
            Self::Settlement { lane, step, source } => {
                write!(f, "lane {lane}, step {step:?}: {source}")
            }
            Self::Proposal { lane, step, source } => {
                write!(f, "lane {lane}, step {step}: {source}")
            }
        }
    }
}
impl<K: Kind, E: std::error::Error + 'static> std::error::Error for EnsembleError<K, E> {}

impl<K: Kind> DenseState<K> {
    /// Execute independent lanes through compiled dense settlement. `initial`
    /// is `(lanes, stocks)` in topology order and is copied before any callback.
    /// A native proposal callback fills nonnegative flow magnitudes for one
    /// lane/step from its current state; requests start at zero each time.
    /// All initial lanes validate before proposals run. Any refusal returns no
    /// partial trajectory. Empty batches and zero steps are supported.
    pub fn ensemble<E>(
        topology: Arc<FlowTopology<K>>,
        initial: &[f64],
        lanes: usize,
        steps: usize,
        tolerance: DenseTolerance,
        mut propose: impl FnMut(usize, usize, &Self, &mut [f64]) -> Result<(), E>,
    ) -> Result<DenseTrajectory<K>, EnsembleError<K, E>> {
        let stocks = topology.stocks().len();
        let overflow = || EnsembleError::ShapeOverflow {
            lanes,
            steps,
            stocks,
        };
        let samples = steps.checked_add(1).ok_or_else(overflow)?;
        let expected = lanes.checked_mul(stocks).ok_or_else(overflow)?;
        if initial.len() != expected {
            return Err(EnsembleError::Shape {
                expected,
                actual: initial.len(),
            });
        }
        if !tolerance.is_valid() {
            return Err(EnsembleError::Tolerance(tolerance));
        }
        let elements = expected.checked_mul(samples).ok_or_else(overflow)?;
        let diagnostics = lanes
            .checked_mul(samples)
            .and_then(|n| n.checked_mul(topology.kinds().len()))
            .ok_or_else(overflow)?;
        let mut amounts = Vec::new();
        amounts
            .try_reserve_exact(elements)
            .map_err(EnsembleError::Allocation)?;
        let mut balances = Vec::new();
        balances
            .try_reserve_exact(diagnostics)
            .map_err(EnsembleError::Allocation)?;
        let mut states = Vec::new();
        states
            .try_reserve_exact(lanes)
            .map_err(EnsembleError::Allocation)?;
        for lane in 0..lanes {
            states.push(
                Self::new(
                    topology.clone(),
                    initial[lane * stocks..(lane + 1) * stocks].to_vec(),
                )
                .map_err(|source| EnsembleError::Settlement {
                    lane,
                    step: None,
                    source: Box::new(source),
                })?,
            );
        }
        let mut requested = vec![0.0; topology.flows().len()];
        for (lane, mut state) in states.into_iter().enumerate() {
            for sample in 0..samples {
                amounts.extend_from_slice(state.amounts());
                balances.extend(topology.kinds().iter().map(|&kind| DenseBalance {
                    residual: state.balance_residual(kind),
                    satisfied: state.balance_within(kind, tolerance),
                }));
                if sample == steps {
                    break;
                }
                requested.fill(0.0);
                propose(lane, sample, &state, &mut requested).map_err(|source| {
                    EnsembleError::Proposal {
                        lane,
                        step: sample,
                        source: Box::new(source),
                    }
                })?;
                state
                    .settle_discard(&requested)
                    .map_err(|source| EnsembleError::Settlement {
                        lane,
                        step: Some(sample),
                        source: Box::new(source),
                    })?;
            }
        }
        Ok(DenseTrajectory {
            topology,
            shape: [lanes, samples, stocks],
            amounts,
            balances,
            tolerance,
        })
    }
}
