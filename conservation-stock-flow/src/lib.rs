#![forbid(unsafe_code)]

//! Exact, domain-neutral stock-flow carriers and semantic evidence.
//!
//! This crate compiles identifier-rich exact matrices from
//! [`conservation_dynamics::FlowTopology`], records accepted settlements
//! without reimplementing settlement, and checks the resulting finite traces.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use conservation_core::{
    AxisId, BalanceLaw, BalanceLawError, GradedLaw, IdentifierError, Kind, nonblank,
};
use conservation_dynamics::{
    CompiledSettlementReport, FlowRole, FlowTopology, StockFlowError as DynamicsError, StockId,
};
use conservation_linear::{MatrixError, NullspaceSource, TransitionMatrix, derive_left_nullspace};
use conservation_trace::{LawVerdict, TraceError, TraceState, TraceStateError, check_law};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

macro_rules! identifier {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a nonblank identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                nonblank(&value)?;
                Ok(Self(value))
            }

            /// Returns the identifier text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

identifier!(FlowId, "Identifies one scalar internal-flow channel.");
identifier!(BoundaryId, "Identifies one scalar boundary-flow channel.");
identifier!(LedgerId, "Identifies one cumulative boundary ledger.");
identifier!(SentenceId, "Identifies one named stock-flow sentence.");

/// A stable identifier from any carrier symbol class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolId {
    /// A stock or projected ledger axis.
    Axis(AxisId),
    /// An internal-flow channel.
    Flow(FlowId),
    /// A boundary-flow channel.
    Boundary(BoundaryId),
    /// A cumulative ledger.
    Ledger(LedgerId),
}

impl fmt::Display for SymbolId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Axis(id) => write!(formatter, "axis {id}"),
            Self::Flow(id) => write!(formatter, "flow {id}"),
            Self::Boundary(id) => write!(formatter, "boundary {id}"),
            Self::Ledger(id) => write!(formatter, "ledger {id}"),
        }
    }
}

/// Converts a typed carrier identifier into diagnostic identity.
pub trait Symbol: Clone + Ord {
    /// Returns a type-preserving diagnostic identifier.
    fn symbol_id(&self) -> SymbolId;
}

impl Symbol for AxisId {
    fn symbol_id(&self) -> SymbolId {
        SymbolId::Axis(self.clone())
    }
}

impl Symbol for FlowId {
    fn symbol_id(&self) -> SymbolId {
        SymbolId::Flow(self.clone())
    }
}

impl Symbol for BoundaryId {
    fn symbol_id(&self) -> SymbolId {
        SymbolId::Boundary(self.clone())
    }
}

impl Symbol for LedgerId {
    fn symbol_id(&self) -> SymbolId {
        SymbolId::Ledger(self.clone())
    }
}

/// Assigns a domain-neutral axis name to one settlement stock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockAxisDefinition {
    /// Existing settlement stock.
    pub stock: StockId,
    /// Domain-neutral matrix axis.
    pub axis: AxisId,
}

/// Names one compiled flow slot according to its structural role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelId {
    /// A transfer with one internal source and target.
    Internal(FlowId),
    /// An input or output crossing the modeled boundary.
    Boundary(BoundaryId),
}

/// Declares one cumulative ledger and its projected trace axis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerDefinition<K> {
    /// Stable ledger identifier used by correspondence sentences.
    pub id: LedgerId,
    /// Axis used by graded state-trace projections.
    pub axis: AxisId,
    /// Quantity kind accumulated by the ledger.
    pub kind: K,
    /// Nonempty same-kind input or output ports accumulated by the ledger.
    pub boundaries: Vec<BoundaryId>,
}

/// Canonical identity of one cumulative boundary ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerIdentity<K> {
    axis: AxisId,
    kind: K,
    boundaries: BTreeSet<BoundaryId>,
}

impl<K: Kind> LedgerIdentity<K> {
    /// Axis used by graded state-trace projections.
    pub fn axis(&self) -> &AxisId {
        &self.axis
    }

    /// Quantity kind accumulated by the ledger.
    pub fn kind(&self) -> K {
        self.kind
    }

    /// Canonical mapped boundary ports.
    pub fn boundaries(&self) -> &BTreeSet<BoundaryId> {
        &self.boundaries
    }
}

/// A canonical exact matrix over named axes and typed columns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactEffectMatrix<C, K> {
    rows: BTreeMap<AxisId, K>,
    columns: BTreeMap<C, K>,
    entries: BTreeMap<(AxisId, C), BigRational>,
}

impl<C, K: Kind> ExactEffectMatrix<C, K>
where
    C: Clone + Ord,
{
    fn new(
        rows: BTreeMap<AxisId, K>,
        columns: BTreeMap<C, K>,
        entry: impl Fn(&AxisId, &C) -> BigRational,
    ) -> Self {
        let entries = rows
            .keys()
            .flat_map(|axis| {
                columns
                    .keys()
                    .map(|column| ((axis.clone(), column.clone()), entry(axis, column)))
            })
            .collect();
        Self {
            rows,
            columns,
            entries,
        }
    }

    /// Iterates through canonical row identifiers.
    pub fn axes(&self) -> impl ExactSizeIterator<Item = &AxisId> {
        self.rows.keys()
    }

    /// Iterates through canonical column identifiers.
    pub fn columns(&self) -> impl ExactSizeIterator<Item = &C> {
        self.columns.keys()
    }

    /// Returns the kind of a row axis.
    pub fn axis_kind(&self, axis: &AxisId) -> Option<K> {
        self.rows.get(axis).copied()
    }

    /// Returns the kind of a matrix column.
    pub fn column_kind(&self, column: &C) -> Option<K> {
        self.columns.get(column).copied()
    }

    /// Returns an exact matrix coefficient, or `None` outside the total shape.
    pub fn coefficient(&self, axis: &AxisId, column: &C) -> Option<&BigRational> {
        self.entries.get(&(axis.clone(), column.clone()))
    }
}

/// Canonical carrier identity retained by records and witnesses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierIdentity<K> {
    internal: ExactEffectMatrix<FlowId, K>,
    boundary: ExactEffectMatrix<BoundaryId, K>,
    ledgers: BTreeMap<LedgerId, LedgerIdentity<K>>,
    boundary_roles: BTreeMap<BoundaryId, FlowRole>,
}

impl<K: Kind> CarrierIdentity<K> {
    /// Exact internal incidence matrix.
    pub fn internal_effects(&self) -> &ExactEffectMatrix<FlowId, K> {
        &self.internal
    }

    /// Exact boundary incidence matrix.
    pub fn boundary_effects(&self) -> &ExactEffectMatrix<BoundaryId, K> {
        &self.boundary
    }

    /// Canonical cumulative-ledger definitions.
    pub fn ledgers(&self) -> &BTreeMap<LedgerId, LedgerIdentity<K>> {
        &self.ledgers
    }

    /// Returns whether a boundary is an input or output.
    pub fn boundary_role(&self, boundary: &BoundaryId) -> Option<FlowRole> {
        self.boundary_roles.get(boundary).copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InternalColumn<K> {
    kind: K,
    source: AxisId,
    target: AxisId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryColumn<K> {
    kind: K,
    stock: AxisId,
    role: FlowRole,
}

/// Immutable exact stock-flow carrier compiled over the settlement topology.
#[derive(Clone, Debug)]
pub struct StockFlowCarrier<K> {
    topology: Arc<FlowTopology<K>>,
    identity: CarrierIdentity<K>,
    axes_by_stock: BTreeMap<StockId, AxisId>,
    internal: BTreeMap<FlowId, InternalColumn<K>>,
    boundaries: BTreeMap<BoundaryId, BoundaryColumn<K>>,
    slot_channels: Vec<ChannelId>,
}

/// Structural failure constructing or using an exact stock-flow carrier.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StockFlowError<K> {
    /// Stock-axis declarations do not cover the topology exactly.
    StockAxisCount { expected: usize, actual: usize },
    /// A stock was assigned more than one axis.
    DuplicateStock(StockId),
    /// An axis was assigned to more than one stock or ledger.
    DuplicateAxis(AxisId),
    /// An axis declaration references an unknown stock.
    UnknownStock(StockId),
    /// Flow-channel declarations do not cover topology slots exactly.
    ChannelCount { expected: usize, actual: usize },
    /// An internal-flow identifier occurred more than once.
    DuplicateFlow(FlowId),
    /// A boundary identifier occurred more than once.
    DuplicateBoundary(BoundaryId),
    /// A channel identifier was assigned to the wrong topology role.
    ChannelRole {
        channel: ChannelId,
        actual: FlowRole,
    },
    /// A ledger identifier occurred more than once.
    DuplicateLedger(LedgerId),
    /// A ledger declares a kind absent from the carrier.
    UnknownKind(K),
    /// An exact vector repeats one symbol.
    DuplicateValue(SymbolId),
    /// An exact amount was negative.
    NegativeAmount(SymbolId),
    /// A vector omitted a required symbol.
    MissingValue { role: ValueRole, symbol: SymbolId },
    /// A vector included a symbol outside its carrier role.
    ExtraValue { role: ValueRole, symbol: SymbolId },
    /// A vector supplied the wrong kind for a symbol.
    KindMismatch {
        role: ValueRole,
        symbol: SymbolId,
        expected: K,
        actual: K,
    },
    /// A settled amount exceeded its requested amount.
    SettledExceedsRequested {
        symbol: SymbolId,
        requested: Box<BigRational>,
        settled: Box<BigRational>,
    },
    /// A record or report belongs to a different exact carrier.
    CarrierMismatch,
    /// A before/after amount vector has the wrong topology length.
    AmountCount { expected: usize, actual: usize },
    /// Adjacent stock states are not continuous.
    DiscontinuousState { transition: usize, axis: AxisId },
    /// Adjacent cumulative-ledger states are not continuous.
    DiscontinuousLedger { transition: usize, ledger: LedgerId },
    /// Existing settlement validation rejected a report.
    Settlement(DynamicsError<K>),
    /// Projection into the existing exact trace carrier failed.
    TraceState(TraceStateError),
    /// A semantic checker cannot witness an empty transition trace.
    TooShort { transitions: usize },
    /// A sentence references an unknown internal-flow channel.
    UnknownFlow(FlowId),
    /// A sentence references an unknown stock or projected ledger axis.
    UnknownAxis(AxisId),
    /// A sentence references an unknown boundary-flow channel.
    UnknownBoundary(BoundaryId),
    /// A sentence references an unknown cumulative ledger.
    UnknownLedger(LedgerId),
    /// A linear constraint has no effective term.
    EmptyConstraint,
    /// A boundary correspondence has no mapped port.
    EmptyBoundaryMapping,
    /// A boundary correspondence repeats one port.
    DuplicateBoundaryMapping(BoundaryId),
    /// A boundary correspondence mixes input and output ports.
    MixedBoundaryRoles,
    /// A sentence combines symbols of different kinds.
    SentenceKindMismatch {
        sentence: SentenceId,
        symbol: SymbolId,
        expected: K,
        actual: K,
    },
    /// A ledger mapping combines symbols of different kinds.
    LedgerBoundaryKindMismatch {
        ledger: LedgerId,
        boundary: BoundaryId,
        ledger_kind: K,
        boundary_kind: K,
    },
    /// Existing graded trace checking rejected the projected model.
    Trace(TraceError),
    /// Exact matrix construction failed.
    Matrix(MatrixError),
    /// Canonical balance-law construction failed.
    BalanceLaw(BalanceLawError),
    /// A proposed certificate does not annihilate one internal-flow column.
    NonNullCertificate {
        flow: FlowId,
        residual: Box<BigRational>,
    },
    /// A projected invariant does not account for a nonzero boundary term.
    UncoveredBoundary(BoundaryId),
    /// Selected ledger mappings account for one boundary more than once.
    DuplicateBoundaryCoverage(BoundaryId),
    /// One selected ledger groups boundaries with different law coefficients.
    IncompatibleLedgerCoefficient(LedgerId),
    /// A law suite repeats one sentence identifier.
    DuplicateSentence(SentenceId),
}

impl<K: Kind> fmt::Display for StockFlowError<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StockAxisCount { expected, actual } => write!(
                formatter,
                "carrier requires {expected} stock-axis declarations, got {actual}"
            ),
            Self::DuplicateStock(stock) => write!(formatter, "duplicate stock mapping {stock}"),
            Self::DuplicateAxis(axis) => write!(formatter, "duplicate carrier axis {axis}"),
            Self::UnknownStock(stock) => write!(formatter, "unknown carrier stock {stock}"),
            Self::ChannelCount { expected, actual } => write!(
                formatter,
                "carrier requires {expected} channel identifiers, got {actual}"
            ),
            Self::DuplicateFlow(flow) => write!(formatter, "duplicate flow {flow}"),
            Self::DuplicateBoundary(boundary) => {
                write!(formatter, "duplicate boundary {boundary}")
            }
            Self::ChannelRole { channel, actual } => {
                write!(formatter, "channel {channel:?} does not match {actual:?}")
            }
            Self::DuplicateLedger(ledger) => write!(formatter, "duplicate ledger {ledger}"),
            Self::UnknownKind(kind) => write!(formatter, "unknown carrier kind {kind}"),
            Self::DuplicateValue(symbol) => write!(formatter, "duplicate value for {symbol}"),
            Self::NegativeAmount(symbol) => write!(formatter, "negative amount for {symbol}"),
            Self::MissingValue { role, symbol } => {
                write!(formatter, "{role} is missing {symbol}")
            }
            Self::ExtraValue { role, symbol } => {
                write!(formatter, "{role} contains unexpected {symbol}")
            }
            Self::KindMismatch {
                role,
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "{role} gives {symbol} kind {actual}; expected {expected}"
            ),
            Self::SettledExceedsRequested {
                symbol,
                requested,
                settled,
            } => write!(
                formatter,
                "settled amount {settled} exceeds request {requested} for {symbol}"
            ),
            Self::CarrierMismatch => formatter.write_str("record and carrier identities differ"),
            Self::AmountCount { expected, actual } => {
                write!(formatter, "expected {expected} amounts, got {actual}")
            }
            Self::DiscontinuousState { transition, axis } => write!(
                formatter,
                "transition {transition} does not continue stock axis {axis}"
            ),
            Self::DiscontinuousLedger { transition, ledger } => write!(
                formatter,
                "transition {transition} does not continue ledger {ledger}"
            ),
            Self::Settlement(error) => error.fmt(formatter),
            Self::TraceState(error) => error.fmt(formatter),
            Self::TooShort { transitions } => write!(
                formatter,
                "trace has {transitions} transitions; at least one is required"
            ),
            Self::UnknownFlow(flow) => write!(formatter, "unknown flow {flow}"),
            Self::UnknownAxis(axis) => write!(formatter, "unknown axis {axis}"),
            Self::UnknownBoundary(boundary) => write!(formatter, "unknown boundary {boundary}"),
            Self::UnknownLedger(ledger) => write!(formatter, "unknown ledger {ledger}"),
            Self::EmptyConstraint => {
                formatter.write_str("linear flow constraint needs a nonzero term")
            }
            Self::EmptyBoundaryMapping => {
                formatter.write_str("boundary correspondence needs at least one port")
            }
            Self::DuplicateBoundaryMapping(boundary) => {
                write!(formatter, "duplicate mapped boundary {boundary}")
            }
            Self::MixedBoundaryRoles => {
                formatter.write_str("boundary correspondence mixes inputs and outputs")
            }
            Self::SentenceKindMismatch {
                sentence,
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "sentence {sentence} gives {symbol} kind {actual}; expected {expected}"
            ),
            Self::LedgerBoundaryKindMismatch {
                ledger,
                boundary,
                ledger_kind,
                boundary_kind,
            } => write!(
                formatter,
                "ledger {ledger} has kind {ledger_kind}, but boundary {boundary} has kind {boundary_kind}"
            ),
            Self::Trace(error) => error.fmt(formatter),
            Self::Matrix(error) => error.fmt(formatter),
            Self::BalanceLaw(error) => error.fmt(formatter),
            Self::NonNullCertificate { flow, residual } => write!(
                formatter,
                "coefficient vector does not annihilate flow {flow}: {residual}"
            ),
            Self::UncoveredBoundary(boundary) => {
                write!(formatter, "no selected ledger covers boundary {boundary}")
            }
            Self::DuplicateBoundaryCoverage(boundary) => write!(
                formatter,
                "selected ledgers cover boundary {boundary} more than once"
            ),
            Self::IncompatibleLedgerCoefficient(ledger) => write!(
                formatter,
                "ledger {ledger} groups unequal open-balance coefficients"
            ),
            Self::DuplicateSentence(sentence) => {
                write!(formatter, "duplicate suite sentence {sentence}")
            }
        }
    }
}

impl<K: Kind> Error for StockFlowError<K> {}

impl<K: Kind> From<DynamicsError<K>> for StockFlowError<K> {
    fn from(error: DynamicsError<K>) -> Self {
        Self::Settlement(error)
    }
}

impl<K: Kind> From<TraceStateError> for StockFlowError<K> {
    fn from(error: TraceStateError) -> Self {
        Self::TraceState(error)
    }
}

impl<K: Kind> From<TraceError> for StockFlowError<K> {
    fn from(error: TraceError) -> Self {
        Self::Trace(error)
    }
}

impl<K: Kind> From<MatrixError> for StockFlowError<K> {
    fn from(error: MatrixError) -> Self {
        Self::Matrix(error)
    }
}

impl<K: Kind> From<BalanceLawError> for StockFlowError<K> {
    fn from(error: BalanceLawError) -> Self {
        Self::BalanceLaw(error)
    }
}

/// Identifies one exact vector within a transition record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueRole {
    /// Stock state before a transition.
    Before,
    /// Stock state after a transition.
    After,
    /// Requested internal-flow amounts.
    RequestedInternal,
    /// Settled internal-flow amounts.
    SettledInternal,
    /// Requested boundary-flow amounts.
    RequestedBoundary,
    /// Settled boundary-flow amounts.
    SettledBoundary,
    /// Cumulative ledgers before a transition.
    LedgerBefore,
    /// Cumulative ledgers after a transition.
    LedgerAfter,
}

impl fmt::Display for ValueRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl<K: Kind> StockFlowCarrier<K> {
    /// Compiles exact incidence matrices and canonical symbol tables.
    ///
    /// `channels` follows the topology's stable flow-slot order; every other
    /// public carrier order is canonical by identifier.
    pub fn new(
        topology: Arc<FlowTopology<K>>,
        stock_axes: impl IntoIterator<Item = StockAxisDefinition>,
        channels: impl IntoIterator<Item = ChannelId>,
        ledgers: impl IntoIterator<Item = LedgerDefinition<K>>,
    ) -> Result<Self, StockFlowError<K>> {
        let stock_axes = stock_axes.into_iter().collect::<Vec<_>>();
        if stock_axes.len() != topology.stocks().len() {
            return Err(StockFlowError::StockAxisCount {
                expected: topology.stocks().len(),
                actual: stock_axes.len(),
            });
        }

        let known_stocks = topology.stocks().iter().cloned().collect::<BTreeSet<_>>();
        let mut axes_by_stock = BTreeMap::new();
        let mut rows = BTreeMap::new();
        for definition in stock_axes {
            if !known_stocks.contains(&definition.stock) {
                return Err(StockFlowError::UnknownStock(definition.stock));
            }
            if axes_by_stock
                .insert(definition.stock.clone(), definition.axis.clone())
                .is_some()
            {
                return Err(StockFlowError::DuplicateStock(definition.stock));
            }
            let kind = topology
                .stock_kind_for(&definition.stock)
                .expect("known topology stock has a kind");
            if rows.insert(definition.axis.clone(), kind).is_some() {
                return Err(StockFlowError::DuplicateAxis(definition.axis));
            }
        }
        if axes_by_stock.len() != topology.stocks().len() {
            return Err(StockFlowError::StockAxisCount {
                expected: topology.stocks().len(),
                actual: axes_by_stock.len(),
            });
        }

        let channels = channels.into_iter().collect::<Vec<_>>();
        if channels.len() != topology.flows().len() {
            return Err(StockFlowError::ChannelCount {
                expected: topology.flows().len(),
                actual: channels.len(),
            });
        }

        let mut internal = BTreeMap::new();
        let mut boundaries = BTreeMap::new();
        let mut boundary_roles = BTreeMap::new();
        for (flow, channel) in topology.flows().iter().zip(&channels) {
            let kind = topology.kinds()[flow.kind()];
            match (flow.role(), channel) {
                (FlowRole::Transfer, ChannelId::Internal(id)) => {
                    let column = InternalColumn {
                        kind,
                        source: axes_by_stock[&topology.stocks()[flow.source().unwrap()]].clone(),
                        target: axes_by_stock[&topology.stocks()[flow.target().unwrap()]].clone(),
                    };
                    if internal.insert(id.clone(), column).is_some() {
                        return Err(StockFlowError::DuplicateFlow(id.clone()));
                    }
                }
                (FlowRole::Input | FlowRole::Output, ChannelId::Boundary(id)) => {
                    let stock = flow.source().or(flow.target()).unwrap();
                    let column = BoundaryColumn {
                        kind,
                        stock: axes_by_stock[&topology.stocks()[stock]].clone(),
                        role: flow.role(),
                    };
                    if boundaries.insert(id.clone(), column).is_some() {
                        return Err(StockFlowError::DuplicateBoundary(id.clone()));
                    }
                    boundary_roles.insert(id.clone(), flow.role());
                }
                (actual, channel) => {
                    return Err(StockFlowError::ChannelRole {
                        channel: channel.clone(),
                        actual,
                    });
                }
            }
        }

        let internal_kinds = internal
            .iter()
            .map(|(id, column)| (id.clone(), column.kind))
            .collect();
        let boundary_kinds = boundaries
            .iter()
            .map(|(id, column)| (id.clone(), column.kind))
            .collect();
        let internal_effects =
            ExactEffectMatrix::new(rows.clone(), internal_kinds, |axis, flow| {
                match &internal[flow] {
                    column if axis == &column.source => -BigRational::from_integer(1.into()),
                    column if axis == &column.target => BigRational::from_integer(1.into()),
                    _ => BigRational::zero(),
                }
            });
        let boundary_effects = ExactEffectMatrix::new(rows, boundary_kinds, |axis, boundary| {
            match &boundaries[boundary] {
                column if axis == &column.stock && column.role == FlowRole::Input => {
                    BigRational::from_integer(1.into())
                }
                column if axis == &column.stock && column.role == FlowRole::Output => {
                    -BigRational::from_integer(1.into())
                }
                _ => BigRational::zero(),
            }
        });

        let known_kinds = topology.kinds().iter().copied().collect::<BTreeSet<_>>();
        let mut ledger_map = BTreeMap::new();
        let mut used_axes = internal_effects.axes().cloned().collect::<BTreeSet<_>>();
        for ledger in ledgers {
            if !known_kinds.contains(&ledger.kind) {
                return Err(StockFlowError::UnknownKind(ledger.kind));
            }
            if ledger.boundaries.is_empty() {
                return Err(StockFlowError::EmptyBoundaryMapping);
            }
            let mut mapped = BTreeSet::new();
            let mut role = None;
            for boundary in ledger.boundaries {
                if !mapped.insert(boundary.clone()) {
                    return Err(StockFlowError::DuplicateBoundaryMapping(boundary));
                }
                let Some(column) = boundaries.get(&boundary) else {
                    return Err(StockFlowError::UnknownBoundary(boundary));
                };
                if column.kind != ledger.kind {
                    return Err(StockFlowError::LedgerBoundaryKindMismatch {
                        ledger: ledger.id,
                        boundary,
                        ledger_kind: ledger.kind,
                        boundary_kind: column.kind,
                    });
                }
                if role
                    .replace(column.role)
                    .is_some_and(|prior| prior != column.role)
                {
                    return Err(StockFlowError::MixedBoundaryRoles);
                }
            }
            if !used_axes.insert(ledger.axis.clone()) {
                return Err(StockFlowError::DuplicateAxis(ledger.axis));
            }
            if ledger_map
                .insert(
                    ledger.id.clone(),
                    LedgerIdentity {
                        axis: ledger.axis,
                        kind: ledger.kind,
                        boundaries: mapped,
                    },
                )
                .is_some()
            {
                return Err(StockFlowError::DuplicateLedger(ledger.id));
            }
        }

        Ok(Self {
            topology,
            identity: CarrierIdentity {
                internal: internal_effects,
                boundary: boundary_effects,
                ledgers: ledger_map,
                boundary_roles,
            },
            axes_by_stock,
            internal,
            boundaries,
            slot_channels: channels,
        })
    }

    /// Underlying immutable settlement topology.
    pub fn topology(&self) -> &Arc<FlowTopology<K>> {
        &self.topology
    }

    /// Canonical exact identity retained by records and evidence.
    pub fn identity(&self) -> &CarrierIdentity<K> {
        &self.identity
    }

    /// Exact internal incidence matrix.
    pub fn internal_effects(&self) -> &ExactEffectMatrix<FlowId, K> {
        &self.identity.internal
    }

    /// Exact boundary incidence matrix.
    pub fn boundary_effects(&self) -> &ExactEffectMatrix<BoundaryId, K> {
        &self.identity.boundary
    }

    /// Returns the axis assigned to one settlement stock.
    pub fn axis_for_stock(&self, stock: &StockId) -> Option<&AxisId> {
        self.axes_by_stock.get(stock)
    }

    /// Returns one ledger's projected axis and kind.
    pub fn ledger(&self, ledger: &LedgerId) -> Option<&LedgerIdentity<K>> {
        self.identity.ledgers.get(ledger)
    }

    /// Flow-slot names in settlement-topology order.
    pub fn slot_channels(&self) -> &[ChannelId] {
        &self.slot_channels
    }

    /// Builds a validated record from one report emitted by exact settlement.
    ///
    /// `before` and `after` follow the topology's stock order. Requested and
    /// settled channel amounts are taken directly from `report`.
    pub fn record_from_settlement(
        &self,
        before: &[BigRational],
        after: &[BigRational],
        report: &CompiledSettlementReport<BigRational, K>,
        ledger_before: ExactAmounts<LedgerId, K>,
        ledger_after: ExactAmounts<LedgerId, K>,
    ) -> Result<TransitionRecord<K>, StockFlowError<K>> {
        self.topology.materialize_exact_report(report)?;
        for values in [before, after] {
            if values.len() != self.topology.stocks().len() {
                return Err(StockFlowError::AmountCount {
                    expected: self.topology.stocks().len(),
                    actual: values.len(),
                });
            }
        }

        let stock_values =
            |values: &[BigRational]| {
                ExactAmounts::new(self.topology.stocks().iter().zip(values).map(
                    |(stock, value)| {
                        let axis = self.axes_by_stock[stock].clone();
                        let kind = self
                            .topology
                            .stock_kind_for(stock)
                            .expect("topology stock has a kind");
                        (axis, kind, value.clone())
                    },
                ))
            };

        let mut requested_internal = Vec::new();
        let mut settled_internal = Vec::new();
        let mut requested_boundary = Vec::new();
        let mut settled_boundary = Vec::new();
        for (slot, (requested, settled)) in self
            .slot_channels
            .iter()
            .zip(report.requested().iter().zip(report.applied()))
        {
            match slot {
                ChannelId::Internal(flow) => {
                    let kind = self.internal[flow].kind;
                    requested_internal.push((flow.clone(), kind, requested.clone()));
                    settled_internal.push((flow.clone(), kind, settled.clone()));
                }
                ChannelId::Boundary(boundary) => {
                    let kind = self.boundaries[boundary].kind;
                    requested_boundary.push((boundary.clone(), kind, requested.clone()));
                    settled_boundary.push((boundary.clone(), kind, settled.clone()));
                }
            }
        }

        TransitionRecord::new(
            self,
            TransitionRecordData {
                before: stock_values(before)?,
                after: stock_values(after)?,
                requested_internal: ExactAmounts::new(requested_internal)?,
                settled_internal: ExactAmounts::new(settled_internal)?,
                requested_boundary: ExactAmounts::new(requested_boundary)?,
                settled_boundary: ExactAmounts::new(settled_boundary)?,
                ledger_before,
                ledger_after,
            },
        )
    }
}

/// A complete, canonical vector of exact typed amounts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactAmounts<I, K> {
    values: BTreeMap<I, (K, BigRational)>,
}

impl<I, K: Kind> ExactAmounts<I, K>
where
    I: Symbol,
{
    /// Constructs an exact vector, rejecting duplicate entries.
    pub fn new(
        values: impl IntoIterator<Item = (I, K, BigRational)>,
    ) -> Result<Self, StockFlowError<K>> {
        let mut canonical = BTreeMap::new();
        for (id, kind, amount) in values {
            if canonical.insert(id.clone(), (kind, amount)).is_some() {
                return Err(StockFlowError::DuplicateValue(id.symbol_id()));
            }
        }
        Ok(Self { values: canonical })
    }

    /// Returns one exact amount when its symbol is present.
    pub fn amount(&self, id: &I) -> Option<&BigRational> {
        self.values.get(id).map(|(_, amount)| amount)
    }

    /// Returns one supplied kind when its symbol is present.
    pub fn kind(&self, id: &I) -> Option<K> {
        self.values.get(id).map(|(kind, _)| *kind)
    }

    /// Iterates through values in canonical identifier order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&I, K, &BigRational)> {
        self.values
            .iter()
            .map(|(id, (kind, amount))| (id, *kind, amount))
    }

    /// Returns the number of named amounts.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this vector has no named amounts.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Unvalidated input data for one exact accepted transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionRecordData<K> {
    /// Stock state before settlement.
    pub before: ExactAmounts<AxisId, K>,
    /// Stock state after settlement.
    pub after: ExactAmounts<AxisId, K>,
    /// Requested internal-flow amounts.
    pub requested_internal: ExactAmounts<FlowId, K>,
    /// Settled internal-flow amounts.
    pub settled_internal: ExactAmounts<FlowId, K>,
    /// Requested boundary-flow amounts.
    pub requested_boundary: ExactAmounts<BoundaryId, K>,
    /// Settled boundary-flow amounts.
    pub settled_boundary: ExactAmounts<BoundaryId, K>,
    /// Cumulative ledgers before settlement.
    pub ledger_before: ExactAmounts<LedgerId, K>,
    /// Cumulative ledgers after settlement.
    pub ledger_after: ExactAmounts<LedgerId, K>,
}

/// One immutable, structurally valid exact accepted transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionRecord<K> {
    carrier: CarrierIdentity<K>,
    data: TransitionRecordData<K>,
}

impl<K: Kind> TransitionRecord<K> {
    /// Validates a complete record against one exact carrier.
    pub fn new(
        carrier: &StockFlowCarrier<K>,
        data: TransitionRecordData<K>,
    ) -> Result<Self, StockFlowError<K>> {
        validate_values(
            &data.before,
            &carrier.identity.internal.rows,
            ValueRole::Before,
        )?;
        validate_values(
            &data.after,
            &carrier.identity.internal.rows,
            ValueRole::After,
        )?;
        validate_values(
            &data.requested_internal,
            &carrier.identity.internal.columns,
            ValueRole::RequestedInternal,
        )?;
        validate_values(
            &data.settled_internal,
            &carrier.identity.internal.columns,
            ValueRole::SettledInternal,
        )?;
        validate_values(
            &data.requested_boundary,
            &carrier.identity.boundary.columns,
            ValueRole::RequestedBoundary,
        )?;
        validate_values(
            &data.settled_boundary,
            &carrier.identity.boundary.columns,
            ValueRole::SettledBoundary,
        )?;
        let ledger_kinds = carrier
            .identity
            .ledgers
            .iter()
            .map(|(id, ledger)| (id.clone(), ledger.kind))
            .collect();
        validate_values(&data.ledger_before, &ledger_kinds, ValueRole::LedgerBefore)?;
        validate_values(&data.ledger_after, &ledger_kinds, ValueRole::LedgerAfter)?;
        validate_nonnegative(&data.requested_internal)?;
        validate_nonnegative(&data.settled_internal)?;
        validate_nonnegative(&data.requested_boundary)?;
        validate_nonnegative(&data.settled_boundary)?;
        validate_settlement(&data.requested_internal, &data.settled_internal)?;
        validate_settlement(&data.requested_boundary, &data.settled_boundary)?;

        Ok(Self {
            carrier: carrier.identity.clone(),
            data,
        })
    }

    /// Exact carrier identity under which this record was validated.
    pub fn carrier_identity(&self) -> &CarrierIdentity<K> {
        &self.carrier
    }

    /// Stock state before settlement.
    pub fn before(&self) -> &ExactAmounts<AxisId, K> {
        &self.data.before
    }

    /// Stock state after settlement.
    pub fn after(&self) -> &ExactAmounts<AxisId, K> {
        &self.data.after
    }

    /// Requested internal-flow amounts.
    pub fn requested_internal(&self) -> &ExactAmounts<FlowId, K> {
        &self.data.requested_internal
    }

    /// Settled internal-flow amounts.
    pub fn settled_internal(&self) -> &ExactAmounts<FlowId, K> {
        &self.data.settled_internal
    }

    /// Requested boundary-flow amounts.
    pub fn requested_boundary(&self) -> &ExactAmounts<BoundaryId, K> {
        &self.data.requested_boundary
    }

    /// Settled boundary-flow amounts.
    pub fn settled_boundary(&self) -> &ExactAmounts<BoundaryId, K> {
        &self.data.settled_boundary
    }

    /// Cumulative ledgers before settlement.
    pub fn ledger_before(&self) -> &ExactAmounts<LedgerId, K> {
        &self.data.ledger_before
    }

    /// Cumulative ledgers after settlement.
    pub fn ledger_after(&self) -> &ExactAmounts<LedgerId, K> {
        &self.data.ledger_after
    }

    /// Decomposes the record into rebuildable exact data.
    pub fn into_data(self) -> TransitionRecordData<K> {
        self.data
    }
}

/// Immutable, structurally continuous finite exact transition trace.
#[derive(Clone, Debug)]
pub struct TransitionTrace<K> {
    carrier: Arc<StockFlowCarrier<K>>,
    records: Vec<TransitionRecord<K>>,
}

impl<K: Kind> TransitionTrace<K> {
    /// Validates record identity and exact before/after continuity.
    ///
    /// Empty traces are representable but semantic checkers reject them with a
    /// structural `TooShort` result rather than creating vacuous witnesses.
    pub fn new(
        carrier: Arc<StockFlowCarrier<K>>,
        records: Vec<TransitionRecord<K>>,
    ) -> Result<Self, StockFlowError<K>> {
        for record in &records {
            if record.carrier_identity() != carrier.identity() {
                return Err(StockFlowError::CarrierMismatch);
            }
        }
        for (index, pair) in records.windows(2).enumerate() {
            for axis in carrier.identity.internal.axes() {
                if pair[0].after().amount(axis) != pair[1].before().amount(axis) {
                    return Err(StockFlowError::DiscontinuousState {
                        transition: index + 1,
                        axis: axis.clone(),
                    });
                }
            }
            for ledger in carrier.identity.ledgers.keys() {
                if pair[0].ledger_after().amount(ledger) != pair[1].ledger_before().amount(ledger) {
                    return Err(StockFlowError::DiscontinuousLedger {
                        transition: index + 1,
                        ledger: ledger.clone(),
                    });
                }
            }
        }
        Ok(Self { carrier, records })
    }

    /// Exact carrier shared by all records.
    pub fn carrier(&self) -> &Arc<StockFlowCarrier<K>> {
        &self.carrier
    }

    /// Accepted records in trace order.
    pub fn records(&self) -> &[TransitionRecord<K>] {
        &self.records
    }

    /// Projects stock and ledger states into the existing graded trace carrier.
    pub fn graded_states(&self) -> Result<Vec<TraceState>, StockFlowError<K>> {
        let Some(first) = self.records.first() else {
            return Ok(Vec::new());
        };
        let mut states = Vec::with_capacity(self.records.len() + 1);
        states.push(project_state(
            &self.carrier,
            first.before(),
            first.ledger_before(),
        )?);
        for record in &self.records {
            states.push(project_state(
                &self.carrier,
                record.after(),
                record.ledger_after(),
            )?);
        }
        Ok(states)
    }
}

fn validate_values<I, K: Kind>(
    actual: &ExactAmounts<I, K>,
    expected: &BTreeMap<I, K>,
    role: ValueRole,
) -> Result<(), StockFlowError<K>>
where
    I: Symbol,
{
    if let Some(id) = actual.values.keys().find(|id| !expected.contains_key(*id)) {
        return Err(StockFlowError::ExtraValue {
            role,
            symbol: id.symbol_id(),
        });
    }
    for (id, expected_kind) in expected {
        let Some((actual_kind, _)) = actual.values.get(id) else {
            return Err(StockFlowError::MissingValue {
                role,
                symbol: id.symbol_id(),
            });
        };
        if actual_kind != expected_kind {
            return Err(StockFlowError::KindMismatch {
                role,
                symbol: id.symbol_id(),
                expected: *expected_kind,
                actual: *actual_kind,
            });
        }
    }
    Ok(())
}

fn validate_settlement<I, K: Kind>(
    requested: &ExactAmounts<I, K>,
    settled: &ExactAmounts<I, K>,
) -> Result<(), StockFlowError<K>>
where
    I: Symbol,
{
    for (id, _, amount) in settled.iter() {
        let requested_amount = requested
            .amount(id)
            .expect("record shape was validated before settlement bounds");
        if amount > requested_amount {
            return Err(StockFlowError::SettledExceedsRequested {
                symbol: id.symbol_id(),
                requested: Box::new(requested_amount.clone()),
                settled: Box::new(amount.clone()),
            });
        }
    }
    Ok(())
}

fn project_state<K: Kind>(
    carrier: &StockFlowCarrier<K>,
    stocks: &ExactAmounts<AxisId, K>,
    ledgers: &ExactAmounts<LedgerId, K>,
) -> Result<TraceState, StockFlowError<K>> {
    let values = stocks
        .iter()
        .map(|(axis, _, amount)| (axis.clone(), amount.clone()))
        .chain(ledgers.iter().map(|(ledger, _, amount)| {
            (
                carrier.identity.ledgers[ledger].axis.clone(),
                amount.clone(),
            )
        }));
    Ok(TraceState::new(values)?)
}

fn validate_nonnegative<I, K: Kind>(amounts: &ExactAmounts<I, K>) -> Result<(), StockFlowError<K>>
where
    I: Symbol,
{
    if let Some((id, _, _)) = amounts.iter().find(|(_, _, amount)| amount.is_negative()) {
        return Err(StockFlowError::NegativeAmount(id.symbol_id()));
    }
    Ok(())
}

/// Named transition-equation sentence over the carrier's exact matrices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionEquation {
    id: SentenceId,
}

impl TransitionEquation {
    /// Constructs a named transition-equation sentence.
    pub fn new(id: SentenceId) -> Self {
        Self { id }
    }

    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }
}

/// Positive exact evidence for a transition equation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionWitness<K> {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// Number of accepted transitions checked.
    pub transitions_checked: usize,
    /// Number of stock axes checked per transition.
    pub axes_checked: usize,
    /// Exact residual shared by all checked coordinates.
    pub residual: BigRational,
    /// Exact carrier identity against which the witness was produced.
    pub carrier: CarrierIdentity<K>,
}

/// First exact transition-equation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionViolation {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// First offending transition.
    pub transition: usize,
    /// First offending canonical stock axis.
    pub axis: AxisId,
    /// Exact observed state delta.
    pub observed_delta: BigRational,
    /// Exact delta accounted for by settled flows.
    pub accounted_delta: BigRational,
    /// `observed_delta - accounted_delta`.
    pub residual: BigRational,
}

/// Typed semantic outcome for a transition-equation sentence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransitionVerdict<K> {
    /// Every transition and axis satisfied the equation.
    Satisfied(TransitionWitness<K>),
    /// The first transition/axis mismatch.
    Violated(TransitionViolation),
}

impl<K: Kind> TransitionVerdict<K> {
    /// Returns whether this verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }
}

/// Checks `after - before = S f + B b` exactly and axis-wise.
pub fn check_transition_equation<K: Kind>(
    sentence: &TransitionEquation,
    trace: &TransitionTrace<K>,
) -> Result<TransitionVerdict<K>, StockFlowError<K>> {
    require_transitions(trace)?;
    let carrier = trace.carrier();
    for (transition, record) in trace.records.iter().enumerate() {
        for axis in carrier.internal_effects().axes() {
            let observed_delta =
                record.after().amount(axis).unwrap() - record.before().amount(axis).unwrap();
            let internal = carrier
                .internal_effects()
                .columns()
                .map(|flow| {
                    carrier.internal_effects().coefficient(axis, flow).unwrap()
                        * record.settled_internal().amount(flow).unwrap()
                })
                .sum::<BigRational>();
            let boundary = carrier
                .boundary_effects()
                .columns()
                .map(|port| {
                    carrier.boundary_effects().coefficient(axis, port).unwrap()
                        * record.settled_boundary().amount(port).unwrap()
                })
                .sum::<BigRational>();
            let accounted_delta = internal + boundary;
            if observed_delta != accounted_delta {
                let residual = &observed_delta - &accounted_delta;
                return Ok(TransitionVerdict::Violated(TransitionViolation {
                    sentence: sentence.id.clone(),
                    transition,
                    axis: axis.clone(),
                    observed_delta,
                    accounted_delta,
                    residual,
                }));
            }
        }
    }
    Ok(TransitionVerdict::Satisfied(TransitionWitness {
        sentence: sentence.id.clone(),
        transitions_checked: trace.records.len(),
        axes_checked: carrier.internal_effects().axes().len(),
        residual: BigRational::zero(),
        carrier: carrier.identity.clone(),
    }))
}

/// Named exact linear equation over settled internal-flow channels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearFlowConstraint<K> {
    id: SentenceId,
    kind: K,
    coefficients: BTreeMap<FlowId, BigRational>,
    expected: BigRational,
}

impl<K: Kind> LinearFlowConstraint<K> {
    /// Canonicalizes terms and validates every referenced channel against a carrier.
    pub fn new(
        carrier: &StockFlowCarrier<K>,
        id: SentenceId,
        kind: K,
        coefficients: impl IntoIterator<Item = (FlowId, BigRational)>,
        expected: BigRational,
    ) -> Result<Self, StockFlowError<K>> {
        let mut canonical = BTreeMap::<FlowId, BigRational>::new();
        for (flow, coefficient) in coefficients {
            *canonical.entry(flow).or_default() += coefficient;
        }
        canonical.retain(|_, coefficient| !coefficient.is_zero());
        if canonical.is_empty() {
            return Err(StockFlowError::EmptyConstraint);
        }
        let sentence = Self {
            id,
            kind,
            coefficients: canonical,
            expected,
        };
        sentence.validate(carrier)?;
        Ok(sentence)
    }

    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }

    /// Quantity kind shared by every term.
    pub fn kind(&self) -> K {
        self.kind
    }

    /// Canonical exact nonzero coefficients.
    pub fn coefficients(&self) -> &BTreeMap<FlowId, BigRational> {
        &self.coefficients
    }

    /// Exact expected value.
    pub fn expected(&self) -> &BigRational {
        &self.expected
    }

    /// Validates every referenced channel against a carrier.
    pub fn validate(&self, carrier: &StockFlowCarrier<K>) -> Result<(), StockFlowError<K>> {
        for flow in self.coefficients.keys() {
            let Some(actual) = carrier.internal_effects().column_kind(flow) else {
                return Err(StockFlowError::UnknownFlow(flow.clone()));
            };
            if actual != self.kind {
                return Err(StockFlowError::SentenceKindMismatch {
                    sentence: self.id.clone(),
                    symbol: flow.symbol_id(),
                    expected: self.kind,
                    actual,
                });
            }
        }
        Ok(())
    }
}

/// Positive exact evidence for a linear flow constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowConstraintWitness {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// Number of transitions checked.
    pub transitions_checked: usize,
    /// Exact expected value matched at every transition.
    pub expected: BigRational,
}

/// First exact linear flow-constraint failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowConstraintViolation {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// First offending transition.
    pub transition: usize,
    /// Exact linear combination observed.
    pub observed: BigRational,
    /// Exact expected value.
    pub expected: BigRational,
}

/// Typed semantic outcome for a linear flow constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowConstraintVerdict {
    /// Every transition satisfied the constraint.
    Satisfied(FlowConstraintWitness),
    /// The first mismatching transition.
    Violated(FlowConstraintViolation),
}

impl FlowConstraintVerdict {
    /// Returns whether this verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }
}

/// Checks one exact linear flow constraint over every accepted transition.
pub fn check_linear_flow_constraint<K: Kind>(
    sentence: &LinearFlowConstraint<K>,
    trace: &TransitionTrace<K>,
) -> Result<FlowConstraintVerdict, StockFlowError<K>> {
    sentence.validate(trace.carrier())?;
    require_transitions(trace)?;
    for (transition, record) in trace.records.iter().enumerate() {
        let observed = sentence
            .coefficients
            .iter()
            .map(|(flow, coefficient)| {
                coefficient * record.settled_internal().amount(flow).unwrap()
            })
            .sum::<BigRational>();
        if observed != sentence.expected {
            return Ok(FlowConstraintVerdict::Violated(FlowConstraintViolation {
                sentence: sentence.id.clone(),
                transition,
                observed,
                expected: sentence.expected.clone(),
            }));
        }
    }
    Ok(FlowConstraintVerdict::Satisfied(FlowConstraintWitness {
        sentence: sentence.id.clone(),
        transitions_checked: trace.records.len(),
        expected: sentence.expected.clone(),
    }))
}

/// Named equality between one cumulative ledger and its declared ports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryCorrespondence {
    id: SentenceId,
    ledger: LedgerId,
}

impl BoundaryCorrespondence {
    /// Constructs a sentence using the carrier-authoritative ledger mapping.
    pub fn new(id: SentenceId, ledger: LedgerId) -> Self {
        Self { id, ledger }
    }

    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }

    /// Cumulative ledger checked by this sentence.
    pub fn ledger(&self) -> &LedgerId {
        &self.ledger
    }

    /// Validates that the carrier declares the ledger and its mapped ports.
    pub fn validate<'a, K: Kind>(
        &self,
        carrier: &'a StockFlowCarrier<K>,
    ) -> Result<&'a LedgerIdentity<K>, StockFlowError<K>> {
        carrier
            .ledger(&self.ledger)
            .ok_or_else(|| StockFlowError::UnknownLedger(self.ledger.clone()))
    }
}

/// Positive exact evidence for one boundary-ledger correspondence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryWitness {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// Ledger checked.
    pub ledger: LedgerId,
    /// Carrier-authoritative ledger axis checked.
    pub ledger_axis: AxisId,
    /// Canonical mapped ports.
    pub boundaries: Vec<BoundaryId>,
    /// Number of transitions checked.
    pub transitions_checked: usize,
}

/// First exact boundary-correspondence failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryViolation {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// First offending transition.
    pub transition: usize,
    /// Ledger whose increment was checked.
    pub ledger: LedgerId,
    /// Carrier-authoritative ledger axis checked.
    pub ledger_axis: AxisId,
    /// Canonical mapped ports.
    pub boundaries: Vec<BoundaryId>,
    /// Exact observed ledger increment.
    pub observed_increment: BigRational,
    /// Exact total settled across mapped ports.
    pub settled_total: BigRational,
}

/// Typed semantic outcome for a boundary correspondence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundaryVerdict {
    /// Every ledger increment matched settled ports.
    Satisfied(BoundaryWitness),
    /// The first mismatching transition.
    Violated(BoundaryViolation),
}

impl BoundaryVerdict {
    /// Returns whether this verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }
}

/// Checks one carrier-authoritative ledger/port correspondence exactly.
pub fn check_boundary_correspondence<K: Kind>(
    sentence: &BoundaryCorrespondence,
    trace: &TransitionTrace<K>,
) -> Result<BoundaryVerdict, StockFlowError<K>> {
    let ledger = sentence.validate(trace.carrier())?;
    require_transitions(trace)?;
    let boundaries = ledger.boundaries.iter().cloned().collect::<Vec<_>>();
    for (transition, record) in trace.records.iter().enumerate() {
        let observed_increment = record.ledger_after().amount(&sentence.ledger).unwrap()
            - record.ledger_before().amount(&sentence.ledger).unwrap();
        let settled_total = ledger
            .boundaries
            .iter()
            .map(|boundary| record.settled_boundary().amount(boundary).unwrap())
            .sum::<BigRational>();
        if observed_increment != settled_total {
            return Ok(BoundaryVerdict::Violated(BoundaryViolation {
                sentence: sentence.id.clone(),
                transition,
                ledger: sentence.ledger.clone(),
                ledger_axis: ledger.axis.clone(),
                boundaries,
                observed_increment,
                settled_total,
            }));
        }
    }
    Ok(BoundaryVerdict::Satisfied(BoundaryWitness {
        sentence: sentence.id.clone(),
        ledger: sentence.ledger.clone(),
        ledger_axis: ledger.axis.clone(),
        boundaries,
        transitions_checked: trace.records.len(),
    }))
}

/// Named embedding of an existing graded state law.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GradedStateLaw<K> {
    id: SentenceId,
    law: GradedLaw<K>,
}

impl<K: Kind> GradedStateLaw<K> {
    /// Constructs a named graded state sentence without changing its semantics.
    pub fn new(id: SentenceId, law: GradedLaw<K>) -> Self {
        Self { id, law }
    }

    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }

    /// Existing graded law carried unchanged.
    pub fn law(&self) -> &GradedLaw<K> {
        &self.law
    }

    /// Validates every law axis and kind against stocks or projected ledgers.
    pub fn validate(&self, carrier: &StockFlowCarrier<K>) -> Result<(), StockFlowError<K>> {
        for (axis, _) in self.law.form().coefficients() {
            let actual = carrier
                .internal_effects()
                .axis_kind(axis)
                .or_else(|| {
                    carrier
                        .identity
                        .ledgers
                        .values()
                        .find(|ledger| &ledger.axis == axis)
                        .map(|ledger| ledger.kind)
                })
                .ok_or_else(|| StockFlowError::UnknownAxis(axis.clone()))?;
            if actual != self.law.form().kind() {
                return Err(StockFlowError::SentenceKindMismatch {
                    sentence: self.id.clone(),
                    symbol: axis.symbol_id(),
                    expected: self.law.form().kind(),
                    actual,
                });
            }
        }
        Ok(())
    }
}

/// Delegates an embedded graded sentence to the existing exact trace checker.
pub fn check_graded_state_law<K: Kind>(
    sentence: &GradedStateLaw<K>,
    trace: &TransitionTrace<K>,
) -> Result<LawVerdict<K>, StockFlowError<K>> {
    sentence.validate(trace.carrier())?;
    require_transitions(trace)?;
    Ok(check_law(sentence.law(), &trace.graded_states()?)?)
}

fn require_transitions<K: Kind>(trace: &TransitionTrace<K>) -> Result<(), StockFlowError<K>> {
    if trace.records.is_empty() {
        Err(StockFlowError::TooShort { transitions: 0 })
    } else {
        Ok(())
    }
}

/// Sealed exact evidence that one typed coefficient vector annihilates `S`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedNullspace<K> {
    carrier: CarrierIdentity<K>,
    law: BalanceLaw<K>,
    source: NullspaceSource,
    annihilation: BTreeMap<FlowId, BigRational>,
    boundary_coefficients: BTreeMap<BoundaryId, BigRational>,
}

impl<K: Kind> CheckedNullspace<K> {
    /// Exact carrier identity under which this evidence was certified.
    pub fn carrier_identity(&self) -> &CarrierIdentity<K> {
        &self.carrier
    }

    /// Checked canonical linear form.
    pub fn law(&self) -> &BalanceLaw<K> {
        &self.law
    }

    /// Structural interpretation used to derive provenance.
    pub fn source(&self) -> NullspaceSource {
        self.source
    }

    /// Exact zero product for every internal-flow column.
    pub fn annihilation(&self) -> &BTreeMap<FlowId, BigRational> {
        &self.annihilation
    }

    /// Exact `w^T B` coefficient for one boundary port.
    pub fn boundary_coefficient(&self, boundary: &BoundaryId) -> Option<&BigRational> {
        self.boundary_coefficients.get(boundary)
    }

    /// Derives a named direct open-balance sentence.
    pub fn open_balance(&self, id: SentenceId) -> OpenBalance<K> {
        OpenBalance {
            id,
            certificate: self.clone(),
        }
    }

    /// Derives an existing graded invariant over stocks and selected ledgers.
    ///
    /// Selected carrier-authoritative mappings must cover every nonzero
    /// boundary coefficient exactly once. Every port grouped by one ledger
    /// must have the same coefficient.
    pub fn graded_invariant(
        &self,
        carrier: &StockFlowCarrier<K>,
        selected_ledgers: impl IntoIterator<Item = LedgerId>,
    ) -> Result<GradedLaw<K>, StockFlowError<K>> {
        if &self.carrier != carrier.identity() {
            return Err(StockFlowError::CarrierMismatch);
        }
        let mut coefficients = self
            .law
            .coefficients()
            .map(|(axis, coefficient)| (axis.clone(), coefficient.clone()))
            .collect::<Vec<_>>();
        let mut covered = BTreeSet::new();
        for ledger_id in selected_ledgers {
            let ledger = carrier
                .ledger(&ledger_id)
                .ok_or_else(|| StockFlowError::UnknownLedger(ledger_id.clone()))?;
            let mut ledger_coefficient = None;
            for boundary in &ledger.boundaries {
                if !covered.insert(boundary.clone()) {
                    return Err(StockFlowError::DuplicateBoundaryCoverage(boundary.clone()));
                }
                let coefficient = self
                    .boundary_coefficients
                    .get(boundary)
                    .expect("certificate covers carrier boundary");
                match &ledger_coefficient {
                    Some(current) if current != coefficient => {
                        return Err(StockFlowError::IncompatibleLedgerCoefficient(ledger_id));
                    }
                    None => ledger_coefficient = Some(coefficient.clone()),
                    _ => {}
                }
            }
            if let Some(coefficient) = ledger_coefficient {
                coefficients.push((ledger.axis.clone(), -coefficient));
            }
        }
        for (boundary, coefficient) in &self.boundary_coefficients {
            if !coefficient.is_zero() && !covered.contains(boundary) {
                return Err(StockFlowError::UncoveredBoundary(boundary.clone()));
            }
        }
        Ok(GradedLaw::from(BalanceLaw::new(
            self.law.kind(),
            coefficients,
            self.source.provenance(),
        )?))
    }
}

/// Recomputes and seals one exact structural left-nullspace certificate.
pub fn certify_nullspace<K: Kind>(
    carrier: &StockFlowCarrier<K>,
    kind: K,
    coefficients: impl IntoIterator<Item = (AxisId, BigRational)>,
) -> Result<CheckedNullspace<K>, StockFlowError<K>> {
    let source = NullspaceSource::Incidence;
    let coefficients = coefficients.into_iter().collect::<Vec<_>>();
    for (axis, _) in &coefficients {
        let Some(actual) = carrier.internal_effects().axis_kind(axis) else {
            return Err(StockFlowError::UnknownAxis(axis.clone()));
        };
        if actual != kind {
            return Err(StockFlowError::SentenceKindMismatch {
                sentence: SentenceId::new("nullspace-certificate")
                    .expect("static certificate identifier is nonblank"),
                symbol: axis.symbol_id(),
                expected: kind,
                actual,
            });
        }
    }
    let law = BalanceLaw::new(kind, coefficients, source.provenance())?;
    let mut annihilation = BTreeMap::new();
    for flow in carrier.internal_effects().columns() {
        let residual = law
            .coefficients()
            .map(|(axis, coefficient)| {
                coefficient
                    * carrier
                        .internal_effects()
                        .coefficient(axis, flow)
                        .expect("matrix is total")
            })
            .sum::<BigRational>();
        if !residual.is_zero() {
            return Err(StockFlowError::NonNullCertificate {
                flow: flow.clone(),
                residual: Box::new(residual),
            });
        }
        annihilation.insert(flow.clone(), residual);
    }
    let boundary_coefficients = carrier
        .boundary_effects()
        .columns()
        .map(|boundary| {
            let coefficient = law
                .coefficients()
                .map(|(axis, weight)| {
                    weight
                        * carrier
                            .boundary_effects()
                            .coefficient(axis, boundary)
                            .expect("matrix is total")
                })
                .sum::<BigRational>();
            (boundary.clone(), coefficient)
        })
        .collect();
    Ok(CheckedNullspace {
        carrier: carrier.identity.clone(),
        law,
        source,
        annihilation,
        boundary_coefficients,
    })
}

/// Derives and seals a deterministic exact basis for one carrier kind.
pub fn derive_nullspace_basis<K: Kind>(
    carrier: &StockFlowCarrier<K>,
    kind: K,
) -> Result<Vec<CheckedNullspace<K>>, StockFlowError<K>> {
    let axes = carrier
        .internal_effects()
        .axes()
        .filter(|axis| carrier.internal_effects().axis_kind(axis) == Some(kind))
        .cloned()
        .collect::<Vec<_>>();
    if axes.is_empty() {
        return Err(StockFlowError::UnknownKind(kind));
    }
    let flows = carrier
        .internal_effects()
        .columns()
        .filter(|flow| carrier.internal_effects().column_kind(flow) == Some(kind))
        .cloned()
        .collect::<Vec<_>>();
    let matrix = if flows.is_empty() {
        TransitionMatrix::empty(axes.clone())?
    } else {
        let rows = axes
            .iter()
            .map(|axis| {
                flows
                    .iter()
                    .map(|flow| {
                        carrier
                            .internal_effects()
                            .coefficient(axis, flow)
                            .expect("matrix is total")
                            .clone()
                    })
                    .collect()
            })
            .collect();
        TransitionMatrix::new(axes, rows)?
    };
    derive_left_nullspace(&matrix, kind, NullspaceSource::Incidence)?
        .into_iter()
        .map(|law| {
            certify_nullspace(
                carrier,
                kind,
                law.coefficients()
                    .map(|(axis, coefficient)| (axis.clone(), coefficient.clone())),
            )
        })
        .collect()
}

/// Named direct open-balance sentence derived from checked `w^T S = 0`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBalance<K> {
    id: SentenceId,
    certificate: CheckedNullspace<K>,
}

impl<K: Kind> OpenBalance<K> {
    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }

    /// Sealed derivation evidence authorizing this sentence.
    pub fn certificate(&self) -> &CheckedNullspace<K> {
        &self.certificate
    }
}

/// Positive exact evidence for a derived direct open balance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBalanceWitness<K> {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// Number of transitions checked.
    pub transitions_checked: usize,
    /// Exact derivation certificate kept separate from runtime evidence.
    pub certificate: CheckedNullspace<K>,
}

/// First exact direct open-balance failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenBalanceViolation {
    /// Sentence checked.
    pub sentence: SentenceId,
    /// First offending transition.
    pub transition: usize,
    /// Exact observed weighted stock delta.
    pub observed_delta: BigRational,
    /// Exact weighted boundary contribution.
    pub boundary_delta: BigRational,
    /// `observed_delta - boundary_delta`.
    pub residual: BigRational,
}

/// Typed semantic outcome for a derived direct open balance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenBalanceVerdict<K> {
    /// Every transition satisfied the derived balance.
    Satisfied(OpenBalanceWitness<K>),
    /// The first mismatching transition.
    Violated(OpenBalanceViolation),
}

impl<K: Kind> OpenBalanceVerdict<K> {
    /// Returns whether this verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }
}

/// Checks a certificate-derived direct open balance on one exact trace.
pub fn check_open_balance<K: Kind>(
    sentence: &OpenBalance<K>,
    trace: &TransitionTrace<K>,
) -> Result<OpenBalanceVerdict<K>, StockFlowError<K>> {
    if &sentence.certificate.carrier != trace.carrier.identity() {
        return Err(StockFlowError::CarrierMismatch);
    }
    require_transitions(trace)?;
    for (transition, record) in trace.records.iter().enumerate() {
        let observed_delta = sentence
            .certificate
            .law
            .coefficients()
            .map(|(axis, coefficient)| {
                coefficient
                    * (record.after().amount(axis).unwrap() - record.before().amount(axis).unwrap())
            })
            .sum::<BigRational>();
        let boundary_delta = sentence
            .certificate
            .boundary_coefficients
            .iter()
            .map(|(boundary, coefficient)| {
                coefficient * record.settled_boundary().amount(boundary).unwrap()
            })
            .sum::<BigRational>();
        if observed_delta != boundary_delta {
            let residual = &observed_delta - &boundary_delta;
            return Ok(OpenBalanceVerdict::Violated(OpenBalanceViolation {
                sentence: sentence.id.clone(),
                transition,
                observed_delta,
                boundary_delta,
                residual,
            }));
        }
    }
    Ok(OpenBalanceVerdict::Satisfied(OpenBalanceWitness {
        sentence: sentence.id.clone(),
        transitions_checked: trace.records.len(),
        certificate: sentence.certificate.clone(),
    }))
}

/// Complete typed outcome for one named sentence in a carrier-layer suite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuiteVerdict<K> {
    /// Transition-equation outcome.
    Transition(TransitionVerdict<K>),
    /// Linear flow-constraint outcome.
    FlowConstraint(FlowConstraintVerdict),
    /// Boundary-correspondence outcome.
    Boundary(BoundaryVerdict),
    /// Certificate-derived direct open-balance outcome.
    OpenBalance(OpenBalanceVerdict<K>),
    /// Existing graded-state outcome.
    Graded {
        /// Stable sentence identifier retained beside existing evidence.
        sentence: SentenceId,
        /// Existing typed graded verdict.
        verdict: LawVerdict<K>,
    },
}

impl<K: Kind> SuiteVerdict<K> {
    /// Stable sentence identifier for this complete typed outcome.
    pub fn id(&self) -> &SentenceId {
        match self {
            Self::Transition(TransitionVerdict::Satisfied(witness)) => &witness.sentence,
            Self::Transition(TransitionVerdict::Violated(violation)) => &violation.sentence,
            Self::FlowConstraint(FlowConstraintVerdict::Satisfied(witness)) => &witness.sentence,
            Self::FlowConstraint(FlowConstraintVerdict::Violated(violation)) => &violation.sentence,
            Self::Boundary(BoundaryVerdict::Satisfied(witness)) => &witness.sentence,
            Self::Boundary(BoundaryVerdict::Violated(violation)) => &violation.sentence,
            Self::OpenBalance(OpenBalanceVerdict::Satisfied(witness)) => &witness.sentence,
            Self::OpenBalance(OpenBalanceVerdict::Violated(violation)) => &violation.sentence,
            Self::Graded { sentence, .. } => sentence,
        }
    }

    /// Returns whether this named typed outcome is satisfied.
    pub fn is_satisfied(&self) -> bool {
        match self {
            Self::Transition(verdict) => verdict.is_satisfied(),
            Self::FlowConstraint(verdict) => verdict.is_satisfied(),
            Self::Boundary(verdict) => verdict.is_satisfied(),
            Self::OpenBalance(verdict) => verdict.is_satisfied(),
            Self::Graded { verdict, .. } => matches!(verdict, LawVerdict::Satisfied(_)),
        }
    }
}

/// Canonical carrier-layer law suite retaining every named typed verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockFlowLawSuite<K> {
    transition: Option<TransitionEquation>,
    constraints: BTreeMap<SentenceId, LinearFlowConstraint<K>>,
    boundaries: BTreeMap<SentenceId, BoundaryCorrespondence>,
    open_balances: BTreeMap<SentenceId, OpenBalance<K>>,
    graded: BTreeMap<SentenceId, GradedStateLaw<K>>,
}

impl<K: Kind> StockFlowLawSuite<K> {
    /// Constructs a suite and rejects duplicate names across all families.
    pub fn new(
        transition: Option<TransitionEquation>,
        constraints: impl IntoIterator<Item = LinearFlowConstraint<K>>,
        boundaries: impl IntoIterator<Item = BoundaryCorrespondence>,
        open_balances: impl IntoIterator<Item = OpenBalance<K>>,
        graded: impl IntoIterator<Item = GradedStateLaw<K>>,
    ) -> Result<Self, StockFlowError<K>> {
        let mut names = BTreeSet::new();
        if let Some(sentence) = &transition {
            names.insert(sentence.id.clone());
        }
        let constraints = collect_named(constraints, &mut names, |law| &law.id)?;
        let boundaries = collect_named(boundaries, &mut names, |law| &law.id)?;
        let open_balances = collect_named(open_balances, &mut names, |law| &law.id)?;
        let graded = collect_named(graded, &mut names, |law| &law.id)?;
        Ok(Self {
            transition,
            constraints,
            boundaries,
            open_balances,
            graded,
        })
    }

    /// Checks every sentence and returns every typed outcome in name order.
    pub fn check(
        &self,
        trace: &TransitionTrace<K>,
    ) -> Result<Vec<SuiteVerdict<K>>, StockFlowError<K>> {
        let mut verdicts = Vec::with_capacity(
            usize::from(self.transition.is_some())
                + self.constraints.len()
                + self.boundaries.len()
                + self.open_balances.len()
                + self.graded.len(),
        );
        if let Some(sentence) = &self.transition {
            verdicts.push(SuiteVerdict::Transition(check_transition_equation(
                sentence, trace,
            )?));
        }
        verdicts.extend(
            self.constraints
                .values()
                .map(|sentence| {
                    check_linear_flow_constraint(sentence, trace).map(SuiteVerdict::FlowConstraint)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        verdicts.extend(
            self.boundaries
                .values()
                .map(|sentence| {
                    check_boundary_correspondence(sentence, trace).map(SuiteVerdict::Boundary)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        verdicts.extend(
            self.open_balances
                .values()
                .map(|sentence| check_open_balance(sentence, trace).map(SuiteVerdict::OpenBalance))
                .collect::<Result<Vec<_>, _>>()?,
        );
        verdicts.extend(
            self.graded
                .values()
                .map(|sentence| {
                    check_graded_state_law(sentence, trace).map(|verdict| SuiteVerdict::Graded {
                        sentence: sentence.id.clone(),
                        verdict,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        verdicts.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(verdicts)
    }
}

fn collect_named<T, K: Kind>(
    values: impl IntoIterator<Item = T>,
    names: &mut BTreeSet<SentenceId>,
    id: impl Fn(&T) -> &SentenceId,
) -> Result<BTreeMap<SentenceId, T>, StockFlowError<K>> {
    let mut collected = BTreeMap::new();
    for value in values {
        let sentence = id(&value).clone();
        if !names.insert(sentence.clone()) {
            return Err(StockFlowError::DuplicateSentence(sentence));
        }
        collected.insert(sentence, value);
    }
    Ok(collected)
}
