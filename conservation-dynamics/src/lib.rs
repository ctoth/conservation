#![forbid(unsafe_code)]

//! Exact and finite-validated dense settlement for typed stock-flow processes.
//!
//! [`FlowTopology`] compiles identifiers and endpoints into immutable indices
//! shared by [`ExactState`] and [`DenseState`]. A batch is evaluated against one
//! pre-settlement state. A stock holds a point of its kind and flows move the
//! kind's difference. A batch that would take a floored stock below its kind's
//! floor is refused unless every process withdrawing from it is declared
//! `Rationing::Ration`; those withdrawals then share the scale
//! (available − floor) / withdrawal. Floorless stocks are never limited. Boundary
//! inputs become available only after the batch. These rules make settlement
//! independent of proposal order (exactly for rational state and under an
//! explicit tolerance for binary64 state). The compiled topology is the one
//! settlement engine: endpoints are resolved once, when it is compiled.

use std::error::Error;
use std::fmt;

use conservation_core::{Kind, identifier};
use num_rational::BigRational;

mod compiled;
mod topology;

pub use compiled::{CompiledSettlementReport, DenseState, DenseTolerance, ExactState};
pub use topology::{
    CompiledFlow, FlowSpec, FlowTopology, ProcessDefinition, Rationing, StockDefinition,
};

identifier!(StockId, "Identifies one stored quantity.");
identifier!(ProcessId, "Identifies the process proposing a flow.");

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
pub struct AppliedFlow<K> {
    /// Process responsible for the flow.
    pub process: ProcessId,
    /// The kind of the amount moved: the endpoint stocks' `kind.difference()`.
    pub kind: K,
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
pub struct SettlementReport<K> {
    applied: Vec<AppliedFlow<K>>,
}

impl<K: Kind> SettlementReport<K> {
    /// Returns settled flows in proposal order.
    pub fn flows(&self) -> &[AppliedFlow<K>] {
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
pub enum StockFlowError<K> {
    /// A system must contain at least one stock.
    NoStocks,
    /// The same stock identifier was declared twice.
    DuplicateStock(StockId),
    /// A flow has neither source nor target.
    DisconnectedFlow,
    /// A transfer names the same source and target.
    SameStock(StockId),
    /// A flow references an undeclared stock.
    UnknownStock(StockId),
    /// A flow kind differs from the difference kind of a referenced stock.
    KindMismatch {
        /// Referenced stock.
        stock: StockId,
        /// Kind declared by the stock.
        stock_kind: K,
        /// Kind declared by the flow.
        flow_kind: K,
    },
    /// A requested flow amount was negative. Flows are magnitudes moved from source to target.
    NegativeFlow {
        /// Position of the flow in the supplied batch.
        index: usize,
        /// Process responsible for the flow.
        process: ProcessId,
        /// The negative amount requested.
        amount: Box<BigRational>,
    },
    /// A stock's initial coordinate lies below its kind's floor.
    InitialBelowFloor {
        /// The stock.
        stock: StockId,
        /// The floor of the stock's kind.
        floor: Box<BigRational>,
        /// The initial coordinate supplied.
        amount: Box<BigRational>,
    },
    /// A process not declared `Ration` would take a floored stock below its floor.
    BelowFloor {
        /// The stock that would be breached.
        stock: StockId,
        /// The floor of the stock's kind.
        floor: Box<BigRational>,
        /// The least refusing process withdrawing from the stock.
        process: ProcessId,
        /// The stock's coordinate before the batch.
        available: Box<BigRational>,
        /// The summed withdrawal from the stock in the batch.
        withdrawal: Box<BigRational>,
    },
    /// A transfer joins stocks of different kinds.
    TransferKinds {
        /// Source stock.
        source: StockId,
        /// Kind declared by the source stock.
        source_kind: K,
        /// Target stock.
        target: StockId,
        /// Kind declared by the target stock.
        target_kind: K,
    },
    /// A process was declared twice.
    DuplicateProcess(ProcessId),
    /// A declared process has no flow in the topology.
    UnknownProcess(ProcessId),
    /// A kind's exact floor has no finite binary64 value (dense state only).
    FloorNotRepresentable {
        /// The kind whose floor was converted.
        kind: K,
        /// The exact floor.
        floor: Box<BigRational>,
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

impl<K: Kind> fmt::Display for StockFlowError<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStocks => formatter.write_str("a stock-flow system needs at least one stock"),
            Self::DuplicateStock(stock) => write!(formatter, "duplicate stock {stock}"),
            Self::DisconnectedFlow => formatter.write_str("a flow needs a source or target"),
            Self::SameStock(stock) => write!(formatter, "flow source and target are both {stock}"),
            Self::UnknownStock(stock) => write!(formatter, "unknown stock {stock}"),
            Self::KindMismatch {
                stock,
                stock_kind,
                flow_kind,
            } => write!(
                formatter,
                "stock {stock} of kind {stock_kind} moves differences of kind {}, not flow kind {flow_kind}",
                stock_kind.difference()
            ),
            Self::NegativeFlow {
                index,
                process,
                amount,
            } => write!(
                formatter,
                "flow {index} of process {process} requests {amount}; flow amounts are nonnegative magnitudes"
            ),
            Self::InitialBelowFloor {
                stock,
                floor,
                amount,
            } => write!(
                formatter,
                "stock {stock} starts at {amount}, below its floor {floor}"
            ),
            Self::BelowFloor {
                stock,
                floor,
                process,
                available,
                withdrawal,
            } => write!(
                formatter,
                "process {process} withdraws {withdrawal} from stock {stock}, which holds {available}; that would take it below its floor {floor}"
            ),
            Self::TransferKinds {
                source,
                source_kind,
                target,
                target_kind,
            } => write!(
                formatter,
                "transfer from stock {source} of kind {source_kind} to stock {target} of kind {target_kind} crosses balances"
            ),
            Self::DuplicateProcess(process) => {
                write!(formatter, "duplicate process declaration {process}")
            }
            Self::UnknownProcess(process) => {
                write!(formatter, "process {process} is declared but has no flow")
            }
            Self::FloorNotRepresentable { kind, floor } => write!(
                formatter,
                "floor {floor} of kind {kind} has no finite binary64 value"
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

impl<K: Kind> Error for StockFlowError<K> {}
