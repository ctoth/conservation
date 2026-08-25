#![forbid(unsafe_code)]

//! Exact and finite-validated dense settlement for typed stock-flow processes.
//!
//! [`FlowTopology`] compiles identifiers and endpoints into immutable indices
//! shared by [`ExactState`] and [`DenseState`]. A batch is evaluated against one
//! pre-settlement state. When proposed withdrawals exceed a stock, all
//! withdrawals from that stock receive the same proportional scale. Boundary
//! inputs become available only after the batch. These rules make settlement
//! independent of proposal order (exactly for rational state and under an
//! explicit tolerance for binary64 state). [`StockFlowSystem`] retains the
//! original dynamic exact-flow interface as an independent compatibility path.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use conservation_core::{IdentifierError, KindId};
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

mod compiled;
mod topology;

pub use compiled::{CompiledSettlementReport, DenseState, DenseTolerance, ExactState};
pub use topology::{CompiledFlow, FlowSpec, FlowTopology, StockDefinition};

/// Identifies one stored quantity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StockId(String);

impl StockId {
    /// Creates a nonblank stock identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(IdentifierError::Blank);
        }
        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Identifies the process proposing a flow.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProcessId(String);

impl ProcessId {
    /// Creates a nonblank process identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(IdentifierError::Blank);
        }
        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Declares one stock and its conserved kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockSpec {
    /// Stable identifier used by flows and queries.
    pub id: StockId,
    /// Quantity kind stored here.
    pub kind: KindId,
    /// Exact initial amount.
    pub initial: BigRational,
}

/// A flow request before resource limitation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposedFlow {
    /// Process responsible for the request.
    pub process: ProcessId,
    /// Conserved kind being moved.
    pub kind: KindId,
    /// Source stock, or `None` for a boundary input.
    pub source: Option<StockId>,
    /// Target stock, or `None` for a boundary output.
    pub target: Option<StockId>,
    /// Nonnegative exact requested amount.
    pub amount: BigRational,
}

/// The boundary role of an accepted flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlowRole {
    /// A flow from outside the modeled boundary into one stock.
    Input,
    /// A flow between two stocks within the boundary.
    Transfer,
    /// A flow from one stock out of the modeled boundary.
    Output,
}

/// An exact flow after simultaneous resource limitation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedFlow {
    /// Process responsible for the flow.
    pub process: ProcessId,
    /// Conserved kind moved by the flow.
    pub kind: KindId,
    /// Source stock, absent for an input.
    pub source: Option<StockId>,
    /// Target stock, absent for an output.
    pub target: Option<StockId>,
    /// Requested amount before limitation.
    pub requested: BigRational,
    /// Amount actually settled.
    pub applied: BigRational,
    /// Boundary role of the flow.
    pub role: FlowRole,
}

/// Result of one atomic settlement batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementReport {
    applied: Vec<AppliedFlow>,
}

impl SettlementReport {
    /// Returns settled flows in proposal order.
    pub fn flows(&self) -> &[AppliedFlow] {
        &self.applied
    }

    /// Sums the applied amount attributed to a process.
    pub fn applied_by(&self, process: &ProcessId) -> BigRational {
        self.applied
            .iter()
            .filter(|flow| &flow.process == process)
            .map(|flow| flow.applied.clone())
            .sum()
    }
}

/// Invalid stock declarations or flow batches.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StockFlowError {
    /// A system must contain at least one stock.
    NoStocks,
    /// The same stock identifier was declared twice.
    DuplicateStock(StockId),
    /// An initial stock or proposed flow amount was negative.
    NegativeAmount,
    /// A flow has neither source nor target.
    DisconnectedFlow,
    /// A transfer names the same source and target.
    SameStock(StockId),
    /// A flow references an undeclared stock.
    UnknownStock(StockId),
    /// A flow kind differs from a referenced stock kind.
    KindMismatch {
        /// Referenced stock.
        stock: StockId,
        /// Kind declared by the stock.
        stock_kind: KindId,
        /// Kind declared by the flow.
        flow_kind: KindId,
    },
    /// A compiled state's amount vector has the wrong length.
    AmountCount {
        /// Number of stock or flow slots required by the topology.
        expected: usize,
        /// Number of supplied values.
        actual: usize,
    },
    /// A compiled report did not originate from the supplied topology.
    TopologyMismatch,
    /// A dense initial or proposed amount was NaN or infinite.
    NonFiniteAmount,
    /// Finite inputs produced a non-finite dense intermediate or account.
    ArithmeticOverflow,
}

impl fmt::Display for StockFlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStocks => formatter.write_str("a stock-flow system needs at least one stock"),
            Self::DuplicateStock(stock) => write!(formatter, "duplicate stock {stock}"),
            Self::NegativeAmount => {
                formatter.write_str("stock and flow amounts must be nonnegative")
            }
            Self::DisconnectedFlow => formatter.write_str("a flow needs a source or target"),
            Self::SameStock(stock) => write!(formatter, "flow source and target are both {stock}"),
            Self::UnknownStock(stock) => write!(formatter, "unknown stock {stock}"),
            Self::KindMismatch {
                stock,
                stock_kind,
                flow_kind,
            } => write!(
                formatter,
                "stock {stock} contains kind {stock_kind}, not flow kind {flow_kind}"
            ),
            Self::AmountCount { expected, actual } => write!(
                formatter,
                "topology requires {expected} amounts, but {actual} were supplied"
            ),
            Self::TopologyMismatch => {
                formatter.write_str("compiled report does not match the supplied topology")
            }
            Self::NonFiniteAmount => formatter.write_str("dense amounts must be finite"),
            Self::ArithmeticOverflow => {
                formatter.write_str("dense settlement produced a non-finite value")
            }
        }
    }
}

impl Error for StockFlowError {}

/// A data-oriented stock state with exact boundary accounts and event history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockFlowSystem {
    stocks: Vec<StockId>,
    kinds: Vec<KindId>,
    indices: BTreeMap<StockId, usize>,
    amounts: Vec<BigRational>,
    initial: BTreeMap<KindId, BigRational>,
    inputs: BTreeMap<KindId, BigRational>,
    outputs: BTreeMap<KindId, BigRational>,
    history: Vec<AppliedFlow>,
}

impl StockFlowSystem {
    /// Constructs a system from typed, nonnegative stock declarations.
    pub fn new(specs: impl IntoIterator<Item = StockSpec>) -> Result<Self, StockFlowError> {
        let specs: Vec<_> = specs.into_iter().collect();
        if specs.is_empty() {
            return Err(StockFlowError::NoStocks);
        }

        let mut seen = BTreeSet::new();
        let mut stocks = Vec::with_capacity(specs.len());
        let mut kinds = Vec::with_capacity(specs.len());
        let mut amounts = Vec::with_capacity(specs.len());
        let mut initial = BTreeMap::<KindId, BigRational>::new();
        for spec in specs {
            if spec.initial.is_negative() {
                return Err(StockFlowError::NegativeAmount);
            }
            if !seen.insert(spec.id.clone()) {
                return Err(StockFlowError::DuplicateStock(spec.id));
            }
            *initial.entry(spec.kind.clone()).or_default() += &spec.initial;
            stocks.push(spec.id);
            kinds.push(spec.kind);
            amounts.push(spec.initial);
        }
        let indices = stocks
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, stock)| (stock, index))
            .collect();

        Ok(Self {
            stocks,
            kinds,
            indices,
            amounts,
            initial,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            history: Vec::new(),
        })
    }

    /// Returns the exact amount in a declared stock.
    pub fn amount(&self, stock: &StockId) -> Option<&BigRational> {
        self.indices.get(stock).map(|index| &self.amounts[*index])
    }

    /// Returns the current total of a conserved kind.
    pub fn total(&self, kind: &KindId) -> BigRational {
        self.amounts
            .iter()
            .zip(&self.kinds)
            .filter(|(_, stock_kind)| *stock_kind == kind)
            .map(|(amount, _)| amount.clone())
            .sum()
    }

    /// Returns cumulative boundary input for a kind.
    pub fn inputs(&self, kind: &KindId) -> BigRational {
        self.inputs.get(kind).cloned().unwrap_or_default()
    }

    /// Returns cumulative boundary output for a kind.
    pub fn outputs(&self, kind: &KindId) -> BigRational {
        self.outputs.get(kind).cloned().unwrap_or_default()
    }

    /// Returns `initial + inputs - outputs - current` exactly.
    pub fn balance_residual(&self, kind: &KindId) -> BigRational {
        self.initial.get(kind).cloned().unwrap_or_default() + self.inputs(kind)
            - self.outputs(kind)
            - self.total(kind)
    }

    /// Returns all accepted flows across batches.
    pub fn history(&self) -> &[AppliedFlow] {
        &self.history
    }

    /// Validates and atomically settles one simultaneous batch.
    pub fn settle(
        &mut self,
        proposals: &[ProposedFlow],
    ) -> Result<SettlementReport, StockFlowError> {
        struct Resolved<'a> {
            proposal: &'a ProposedFlow,
            source: Option<usize>,
            target: Option<usize>,
            role: FlowRole,
        }

        let mut resolved = Vec::with_capacity(proposals.len());
        let mut requested = vec![BigRational::zero(); self.amounts.len()];
        for proposal in proposals {
            if proposal.amount.is_negative() {
                return Err(StockFlowError::NegativeAmount);
            }
            let source = proposal
                .source
                .as_ref()
                .map(|stock| self.resolve(stock, &proposal.kind))
                .transpose()?;
            let target = proposal
                .target
                .as_ref()
                .map(|stock| self.resolve(stock, &proposal.kind))
                .transpose()?;
            let role = match (source, target) {
                (None, None) => return Err(StockFlowError::DisconnectedFlow),
                (Some(source), Some(target)) if source == target => {
                    return Err(StockFlowError::SameStock(self.stocks[source].clone()));
                }
                (None, Some(_)) => FlowRole::Input,
                (Some(_), None) => FlowRole::Output,
                (Some(_), Some(_)) => FlowRole::Transfer,
            };
            if let Some(source) = source {
                requested[source] += &proposal.amount;
            }
            resolved.push(Resolved {
                proposal,
                source,
                target,
                role,
            });
        }

        let scales: Vec<_> = requested
            .iter()
            .zip(&self.amounts)
            .map(|(requested, available)| {
                if requested.is_zero() || requested <= available {
                    BigRational::one()
                } else {
                    available / requested
                }
            })
            .collect();
        let mut deltas = vec![BigRational::zero(); self.amounts.len()];
        let mut applied = Vec::with_capacity(resolved.len());
        let mut batch_inputs = BTreeMap::<KindId, BigRational>::new();
        let mut batch_outputs = BTreeMap::<KindId, BigRational>::new();

        for flow in resolved {
            let amount = flow.source.map_or_else(
                || flow.proposal.amount.clone(),
                |source| &flow.proposal.amount * &scales[source],
            );
            if let Some(source) = flow.source {
                deltas[source] -= &amount;
            } else {
                *batch_inputs.entry(flow.proposal.kind.clone()).or_default() += &amount;
            }
            if let Some(target) = flow.target {
                deltas[target] += &amount;
            } else {
                *batch_outputs.entry(flow.proposal.kind.clone()).or_default() += &amount;
            }
            applied.push(AppliedFlow {
                process: flow.proposal.process.clone(),
                kind: flow.proposal.kind.clone(),
                source: flow.proposal.source.clone(),
                target: flow.proposal.target.clone(),
                requested: flow.proposal.amount.clone(),
                applied: amount,
                role: flow.role,
            });
        }

        for (amount, delta) in self.amounts.iter_mut().zip(deltas) {
            *amount += delta;
            debug_assert!(!amount.is_negative());
        }
        for (kind, amount) in batch_inputs {
            *self.inputs.entry(kind).or_default() += amount;
        }
        for (kind, amount) in batch_outputs {
            *self.outputs.entry(kind).or_default() += amount;
        }
        self.history.extend(applied.iter().cloned());

        Ok(SettlementReport { applied })
    }

    fn resolve(&self, stock: &StockId, flow_kind: &KindId) -> Result<usize, StockFlowError> {
        let index = self
            .indices
            .get(stock)
            .copied()
            .ok_or_else(|| StockFlowError::UnknownStock(stock.clone()))?;
        if &self.kinds[index] != flow_kind {
            return Err(StockFlowError::KindMismatch {
                stock: stock.clone(),
                stock_kind: self.kinds[index].clone(),
                flow_kind: flow_kind.clone(),
            });
        }
        Ok(index)
    }
}
