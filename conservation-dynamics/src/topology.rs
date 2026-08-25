use std::collections::BTreeMap;

use conservation_core::KindId;

use crate::{FlowRole, ProcessId, StockFlowError, StockId};

/// Immutable metadata for one stock in a compiled flow system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockDefinition {
    /// Stable external identifier.
    pub id: StockId,
    /// Conserved quantity kind stored in the stock.
    pub kind: KindId,
}

/// Immutable metadata for one flow slot in a compiled flow system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowSpec {
    /// Process responsible for this flow slot.
    pub process: ProcessId,
    /// Conserved quantity kind moved by the flow.
    pub kind: KindId,
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

    /// Index into [`FlowTopology::kinds`].
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
pub struct FlowTopology {
    stocks: Vec<StockId>,
    stock_kinds: Vec<usize>,
    stock_indices: BTreeMap<StockId, usize>,
    kinds: Vec<KindId>,
    kind_indices: BTreeMap<KindId, usize>,
    processes: Vec<ProcessId>,
    process_indices: BTreeMap<ProcessId, usize>,
    flows: Vec<CompiledFlow>,
}

impl FlowTopology {
    /// Compiles stock and flow declarations into a stable index layout.
    pub fn new(
        stocks: impl IntoIterator<Item = StockDefinition>,
        flows: impl IntoIterator<Item = FlowSpec>,
    ) -> Result<Self, StockFlowError> {
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
                    kind_indices.insert(stock.kind.clone(), index);
                    kinds.push(stock.kind);
                    index
                }
            };
            stock_ids.push(stock.id);
            stock_kinds.push(kind_index);
        }

        let mut processes = Vec::new();
        let mut process_indices = BTreeMap::new();
        let mut compiled = Vec::new();
        for flow in flows {
            if flow.source.is_none() && flow.target.is_none() {
                return Err(StockFlowError::DisconnectedFlow);
            }
            let source = resolve_stock(
                flow.source.as_ref(),
                &flow.kind,
                &stock_indices,
                &stock_kinds,
                &kinds,
            )?;
            let target = resolve_stock(
                flow.target.as_ref(),
                &flow.kind,
                &stock_indices,
                &stock_kinds,
                &kinds,
            )?;
            let kind = *kind_indices
                .get(&flow.kind)
                .expect("a connected, kind-valid flow references a declared kind");
            let role = match (source, target) {
                (None, None) => unreachable!("disconnected flows were rejected before resolution"),
                (Some(source), Some(target)) if source == target => {
                    return Err(StockFlowError::SameStock(stock_ids[source].clone()));
                }
                (None, Some(_)) => FlowRole::Input,
                (Some(_), None) => FlowRole::Output,
                (Some(_), Some(_)) => FlowRole::Transfer,
            };
            let process = match process_indices.get(&flow.process) {
                Some(index) => *index,
                None => {
                    let index = processes.len();
                    process_indices.insert(flow.process.clone(), index);
                    processes.push(flow.process);
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

        Ok(Self {
            stocks: stock_ids,
            stock_kinds,
            stock_indices,
            kinds,
            kind_indices,
            processes,
            process_indices,
            flows: compiled,
        })
    }

    /// Stock identifiers in stable index order.
    pub fn stocks(&self) -> &[StockId] {
        &self.stocks
    }

    /// Conserved kinds in stable first-declaration order.
    pub fn kinds(&self) -> &[KindId] {
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

    /// Returns the stable index assigned to a conserved kind.
    pub fn kind_index(&self, kind: &KindId) -> Option<usize> {
        self.kind_indices.get(kind).copied()
    }

    /// Returns the stable index assigned to a process.
    pub fn process_index(&self, process: &ProcessId) -> Option<usize> {
        self.process_indices.get(process).copied()
    }

    pub(crate) fn stock_kind(&self, stock: usize) -> usize {
        self.stock_kinds[stock]
    }
}

fn resolve_stock(
    stock: Option<&StockId>,
    flow_kind: &KindId,
    indices: &BTreeMap<StockId, usize>,
    stock_kinds: &[usize],
    kinds: &[KindId],
) -> Result<Option<usize>, StockFlowError> {
    stock
        .map(|stock| {
            let index = indices
                .get(stock)
                .copied()
                .ok_or_else(|| StockFlowError::UnknownStock(stock.clone()))?;
            let stock_kind = &kinds[stock_kinds[index]];
            if stock_kind != flow_kind {
                return Err(StockFlowError::KindMismatch {
                    stock: stock.clone(),
                    stock_kind: stock_kind.clone(),
                    flow_kind: flow_kind.clone(),
                });
            }
            Ok(index)
        })
        .transpose()
}
