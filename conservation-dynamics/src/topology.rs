use std::collections::{BTreeMap, BTreeSet};

use conservation_core::Kind;

use crate::{FlowRole, ProcessId, StockFlowError, StockId};

/// What settlement does when a batch would take a floored stock below its
/// kind's floor. Declared once per process.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Rationing {
    /// The batch is rejected with `StockFlowError::BelowFloor`.
    #[default]
    Refuse,
    /// Withdrawals share the proportional limit `(available − floor) / withdrawal`,
    /// but only when every process withdrawing from that stock is `Ration`.
    Ration,
}

/// One process's declaration. A process that is not declared is `Refuse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessDefinition {
    /// The declared process.
    pub id: ProcessId,
    /// What settlement does when this process would breach a floor.
    pub rationing: Rationing,
}

/// Immutable metadata for one stock in a compiled flow system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockDefinition<K> {
    /// Stable external identifier.
    pub id: StockId,
    /// Conserved quantity kind stored in the stock.
    pub kind: K,
}

/// Immutable metadata for one flow slot in a compiled flow system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowSpec<K> {
    /// Process responsible for this flow slot.
    pub process: ProcessId,
    /// The kind of the amount moved: the endpoint stocks' `kind.difference()`.
    pub kind: K,
    /// Source stock, absent for a boundary input.
    pub source: Option<StockId>,
    /// Target stock, absent for a boundary output.
    pub target: Option<StockId>,
}

/// One validated flow expressed entirely as stable topology indices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledFlow {
    process: usize,
    kind: usize,
    source: Option<usize>,
    target: Option<usize>,
    role: FlowRole,
}

impl CompiledFlow {
    /// Index into [`FlowTopology::processes`].
    pub fn process(&self) -> usize {
        self.process
    }

    /// Index into [`FlowTopology::kinds`] of the balance this flow enters.
    pub fn kind(&self) -> usize {
        self.kind
    }

    /// Source stock index, absent for a boundary input.
    pub fn source(&self) -> Option<usize> {
        self.source
    }

    /// Target stock index, absent for a boundary output.
    pub fn target(&self) -> Option<usize> {
        self.target
    }

    /// Boundary role determined during compilation.
    pub fn role(&self) -> FlowRole {
        self.role
    }
}

/// Immutable, validated stock-flow layout shared by numeric backends.
///
/// Every stock, conserved kind, process, source, and target is assigned an
/// index once. Settlement therefore operates on contiguous amount arrays and
/// cannot encounter an unknown stock or kind after compilation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowTopology<K> {
    stocks: Vec<StockId>,
    stock_kinds: Vec<usize>,
    stock_indices: BTreeMap<StockId, usize>,
    kinds: Vec<K>,
    kind_indices: BTreeMap<K, usize>,
    processes: Vec<ProcessId>,
    process_indices: BTreeMap<ProcessId, usize>,
    rationing: Vec<Rationing>,
    flows: Vec<CompiledFlow>,
}

impl<K: Kind> FlowTopology<K> {
    /// Compiles stock, flow and process declarations into a stable index layout.
    ///
    /// A process with a flow but no declaration is [`Rationing::Refuse`].
    pub fn new(
        stocks: impl IntoIterator<Item = StockDefinition<K>>,
        flows: impl IntoIterator<Item = FlowSpec<K>>,
        processes: impl IntoIterator<Item = ProcessDefinition>,
    ) -> Result<Self, StockFlowError<K>> {
        let stocks: Vec<_> = stocks.into_iter().collect();
        if stocks.is_empty() {
            return Err(StockFlowError::NoStocks);
        }

        let mut stock_ids = Vec::with_capacity(stocks.len());
        let mut stock_indices = BTreeMap::new();
        let mut kinds = Vec::new();
        let mut kind_indices = BTreeMap::new();
        let mut stock_kinds = Vec::with_capacity(stocks.len());

        for stock in stocks {
            let stock_index = stock_ids.len();
            if stock_indices
                .insert(stock.id.clone(), stock_index)
                .is_some()
            {
                return Err(StockFlowError::DuplicateStock(stock.id));
            }
            let kind_index = match kind_indices.get(&stock.kind) {
                Some(index) => *index,
                None => {
                    let index = kinds.len();
                    kind_indices.insert(stock.kind, index);
                    kinds.push(stock.kind);
                    index
                }
            };
            stock_ids.push(stock.id);
            stock_kinds.push(kind_index);
        }

        let mut process_ids = Vec::new();
        let mut process_indices = BTreeMap::new();
        let mut compiled = Vec::new();
        for flow in flows {
            if flow.source.is_none() && flow.target.is_none() {
                return Err(StockFlowError::DisconnectedFlow);
            }
            let source = resolve_stock(
                flow.source.as_ref(),
                flow.kind,
                &stock_indices,
                &stock_kinds,
                &kinds,
            )?;
            let target = resolve_stock(
                flow.target.as_ref(),
                flow.kind,
                &stock_indices,
                &stock_kinds,
                &kinds,
            )?;
            let (role, endpoint) = match (source, target) {
                (None, None) => unreachable!("disconnected flows were rejected before resolution"),
                (Some(source), Some(target)) if source == target => {
                    return Err(StockFlowError::SameStock(stock_ids[source].clone()));
                }
                (Some(source), Some(target)) if stock_kinds[source] != stock_kinds[target] => {
                    return Err(StockFlowError::TransferKinds {
                        source: stock_ids[source].clone(),
                        source_kind: kinds[stock_kinds[source]],
                        target: stock_ids[target].clone(),
                        target_kind: kinds[stock_kinds[target]],
                    });
                }
                (None, Some(target)) => (FlowRole::Input, target),
                (Some(source), None) => (FlowRole::Output, source),
                (Some(source), Some(_)) => (FlowRole::Transfer, source),
            };
            let kind = stock_kinds[endpoint];
            let process = match process_indices.get(&flow.process) {
                Some(index) => *index,
                None => {
                    let index = process_ids.len();
                    process_indices.insert(flow.process.clone(), index);
                    process_ids.push(flow.process);
                    index
                }
            };
            compiled.push(CompiledFlow {
                process,
                kind,
                source,
                target,
                role,
            });
        }

        let mut rationing = vec![Rationing::default(); process_ids.len()];
        let mut declared = BTreeSet::new();
        for process in processes {
            if !declared.insert(process.id.clone()) {
                return Err(StockFlowError::DuplicateProcess(process.id));
            }
            let Some(index) = process_indices.get(&process.id) else {
                return Err(StockFlowError::UnknownProcess(process.id));
            };
            rationing[*index] = process.rationing;
        }

        Ok(Self {
            stocks: stock_ids,
            stock_kinds,
            stock_indices,
            kinds,
            kind_indices,
            processes: process_ids,
            process_indices,
            rationing,
            flows: compiled,
        })
    }

    /// The declared rationing of a process with a flow in this topology.
    pub fn rationing(&self, process: &ProcessId) -> Option<Rationing> {
        self.process_index(process)
            .map(|index| self.process_rationing(index))
    }

    pub(crate) fn process_rationing(&self, process: usize) -> Rationing {
        self.rationing[process]
    }

    /// Stock identifiers in stable index order.
    pub fn stocks(&self) -> &[StockId] {
        &self.stocks
    }

    /// Conserved kinds in stable first-declaration order.
    pub fn kinds(&self) -> &[K] {
        &self.kinds
    }

    /// Process identifiers in stable first-flow order.
    pub fn processes(&self) -> &[ProcessId] {
        &self.processes
    }

    /// Compiled flows in declaration order.
    pub fn flows(&self) -> &[CompiledFlow] {
        &self.flows
    }

    /// Returns the stable index assigned to a stock.
    pub fn stock_index(&self, stock: &StockId) -> Option<usize> {
        self.stock_indices.get(stock).copied()
    }

    /// Returns the conserved kind stored by a declared stock.
    pub fn stock_kind_for(&self, stock: &StockId) -> Option<K> {
        self.stock_index(stock)
            .map(|index| self.kinds[self.stock_kinds[index]])
    }

    /// Returns the stable index assigned to a conserved kind.
    pub fn kind_index(&self, kind: K) -> Option<usize> {
        self.kind_indices.get(&kind).copied()
    }

    /// Returns the stable index assigned to a process.
    pub fn process_index(&self, process: &ProcessId) -> Option<usize> {
        self.process_indices.get(process).copied()
    }

    pub(crate) fn stock_kind(&self, stock: usize) -> usize {
        self.stock_kinds[stock]
    }
}

fn resolve_stock<K: Kind>(
    stock: Option<&StockId>,
    flow_kind: K,
    indices: &BTreeMap<StockId, usize>,
    stock_kinds: &[usize],
    kinds: &[K],
) -> Result<Option<usize>, StockFlowError<K>> {
    stock
        .map(|stock| {
            let index = indices
                .get(stock)
                .copied()
                .ok_or_else(|| StockFlowError::UnknownStock(stock.clone()))?;
            let stock_kind = kinds[stock_kinds[index]];
            if stock_kind.difference() != flow_kind {
                return Err(StockFlowError::KindMismatch {
                    stock: stock.clone(),
                    stock_kind,
                    flow_kind,
                });
            }
            Ok(index)
        })
        .transpose()
}
