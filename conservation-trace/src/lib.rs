#![forbid(unsafe_code)]

//! Exact evidence for finite-state conservation traces.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use conservation_core::{AxisId, BalanceLaw, KindId};
use num_rational::BigRational;
use num_traits::Zero;

/// One exact state in a finite trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceState {
    values: BTreeMap<AxisId, BigRational>,
}

/// An error constructing a trace state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceStateError {
    /// A state contained no axes.
    Empty,
    /// An axis occurred more than once in the state input.
    DuplicateAxis(AxisId),
}

impl fmt::Display for TraceStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("trace state must contain at least one axis"),
            Self::DuplicateAxis(axis) => write!(formatter, "duplicate trace-state axis {axis}"),
        }
    }
}

impl Error for TraceStateError {}

impl TraceState {
    /// Validates and constructs a state from exact axis values.
    pub fn new(
        values: impl IntoIterator<Item = (AxisId, BigRational)>,
    ) -> Result<Self, TraceStateError> {
        let mut canonical = BTreeMap::new();
        for (axis, value) in values {
            if canonical.insert(axis.clone(), value).is_some() {
                return Err(TraceStateError::DuplicateAxis(axis));
            }
        }
        if canonical.is_empty() {
            return Err(TraceStateError::Empty);
        }
        Ok(Self { values: canonical })
    }

    /// Returns an exact value when the axis is present.
    pub fn value(&self, axis: &AxisId) -> Option<&BigRational> {
        self.values.get(axis)
    }

    /// Iterates through the state's axes in deterministic order.
    pub fn axes(&self) -> impl ExactSizeIterator<Item = &AxisId> {
        self.values.keys()
    }
}

/// Positive evidence that every checked state has the same exact balance.
///
/// This evidence concerns only the supplied trace. It deliberately carries no
/// derivation-origin metadata from the law.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceWitness {
    /// Number of states checked, always at least two.
    pub states_checked: usize,
    /// Exact balance shared by every checked state.
    pub conserved_value: BigRational,
    /// Kind checked by the law.
    pub kind: KindId,
}

/// Semantic evidence that a structurally valid trace violates a balance law.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViolatedBalance {
    /// Index of the first state whose balance differs from the initial state.
    pub state_index: usize,
    /// Exact balance established by the initial state.
    pub expected: BigRational,
    /// Exact balance observed at `state_index`.
    pub observed: BigRational,
}

/// The semantic outcome of checking a structurally valid trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceVerdict {
    /// Every state has the same exact balance.
    Satisfied(TraceWitness),
    /// A state has a different exact balance.
    Violated(ViolatedBalance),
}

/// A structural error that prevents semantic trace checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceError {
    /// At least two states are required to establish and compare a balance.
    TooShort { states: usize },
    /// A state omitted an axis required by the law.
    MissingAxis { state_index: usize, axis: AxisId },
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { states } => {
                write!(
                    formatter,
                    "trace has {states} states; at least two are required"
                )
            }
            Self::MissingAxis { state_index, axis } => {
                write!(
                    formatter,
                    "trace state {state_index} is missing axis {axis}"
                )
            }
        }
    }
}

impl Error for TraceError {}

/// Checks a finite trace against a law using exact rational arithmetic.
pub fn check_trace(law: &BalanceLaw, states: &[TraceState]) -> Result<TraceVerdict, TraceError> {
    if states.len() < 2 {
        return Err(TraceError::TooShort {
            states: states.len(),
        });
    }

    let expected = evaluate(law, &states[0], 0)?;
    for (state_index, state) in states.iter().enumerate().skip(1) {
        let observed = evaluate(law, state, state_index)?;
        if observed != expected {
            return Ok(TraceVerdict::Violated(ViolatedBalance {
                state_index,
                expected,
                observed,
            }));
        }
    }

    Ok(TraceVerdict::Satisfied(TraceWitness {
        states_checked: states.len(),
        conserved_value: expected,
        kind: law.kind().clone(),
    }))
}

fn evaluate(
    law: &BalanceLaw,
    state: &TraceState,
    state_index: usize,
) -> Result<BigRational, TraceError> {
    let mut balance = BigRational::zero();
    for (axis, coefficient) in law.coefficients() {
        let value = state.value(axis).ok_or_else(|| TraceError::MissingAxis {
            state_index,
            axis: axis.clone(),
        })?;
        balance += coefficient * value;
    }
    Ok(balance)
}
