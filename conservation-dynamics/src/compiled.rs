use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;
use std::{fmt, mem};

use conservation_core::Kind;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::{
    AppliedFlow, FlowTopology, ProcessId, Rationing, SettlementReport, StockFlowError, StockId,
    source_scale,
};

/// Applied amounts from one compiled settlement, indexed like the topology's flows.
#[derive(Clone)]
pub struct CompiledSettlementReport<N, K> {
    topology: Arc<FlowTopology<K>>,
    requested: Vec<N>,
    applied: Vec<N>,
}

impl<N: fmt::Debug, K: Kind> fmt::Debug for CompiledSettlementReport<N, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledSettlementReport")
            .field("requested", &self.requested)
            .field("applied", &self.applied)
            .finish()
    }
}

impl<N: PartialEq, K: Kind> PartialEq for CompiledSettlementReport<N, K> {
    fn eq(&self, other: &Self) -> bool {
        self.requested == other.requested && self.applied == other.applied
    }
}

impl<N, K: Kind> CompiledSettlementReport<N, K> {
    /// Requested amounts in stable flow-slot order.
    pub fn requested(&self) -> &[N] {
        &self.requested
    }

    /// Applied amounts in stable flow-slot order.
    pub fn applied(&self) -> &[N] {
        &self.applied
    }
}

impl<K: Kind> FlowTopology<K> {
    /// Restores identifier-rich exact flows from a compiled settlement report.
    ///
    /// The report must originate from a structurally equal topology; equal
    /// vector lengths alone are not sufficient compatibility evidence.
    pub fn materialize_exact_report(
        &self,
        report: &CompiledSettlementReport<BigRational, K>,
    ) -> Result<SettlementReport<K>, StockFlowError<K>> {
        if self != report.topology.as_ref()
            || self.flows().len() != report.requested.len()
            || report.requested.len() != report.applied.len()
        {
            return Err(StockFlowError::TopologyMismatch);
        }
        let applied = self
            .flows()
            .iter()
            .zip(report.requested.iter().zip(&report.applied))
            .map(|(flow, (requested, applied))| AppliedFlow {
                process: self.processes()[flow.process()].clone(),
                kind: self.kinds()[flow.kind()].difference(),
                source: flow.source().map(|index| self.stocks()[index].clone()),
                target: flow.target().map(|index| self.stocks()[index].clone()),
                requested: requested.clone(),
                applied: applied.clone(),
                role: flow.role(),
            })
            .collect();
        Ok(SettlementReport { applied })
    }
}

/// Explicit absolute and relative tolerances for binary64 comparisons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DenseTolerance {
    /// Absolute error allowed near zero.
    pub absolute: f64,
    /// Error allowed in proportion to the compared magnitude.
    pub relative: f64,
}

impl DenseTolerance {
    /// Tests two finite values using `absolute + relative * max(|a|, |b|)`.
    pub fn contains(self, left: f64, right: f64) -> bool {
        if !left.is_finite()
            || !right.is_finite()
            || !self.absolute.is_finite()
            || !self.relative.is_finite()
            || self.absolute < 0.0
            || self.relative < 0.0
        {
            return false;
        }
        let scale = left.abs().max(right.abs());
        if scale == 0.0 {
            return true;
        }
        (left / scale - right / scale).abs() <= self.absolute / scale + self.relative
    }
}

impl Default for DenseTolerance {
    fn default() -> Self {
        // One settlement performs several additions and a multiply/divide per
        // flow. 256 epsilon leaves headroom for those roundings without
        // treating model-scale discrepancies as representation noise.
        Self {
            absolute: 256.0 * f64::EPSILON,
            relative: 256.0 * f64::EPSILON,
        }
    }
}

/// Exact mutable amounts and boundary accounts over an immutable topology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactState<K> {
    topology: Arc<FlowTopology<K>>,
    amounts: Vec<BigRational>,
    initial: Vec<BigRational>,
    inputs: Vec<BigRational>,
    outputs: Vec<BigRational>,
}

impl<K: Kind> ExactState<K> {
    /// Creates exact state from amounts in stable stock-index order.
    pub fn new(
        topology: Arc<FlowTopology<K>>,
        amounts: Vec<BigRational>,
    ) -> Result<Self, StockFlowError<K>> {
        validate_count(topology.stocks().len(), amounts.len())?;
        for (stock, amount) in amounts.iter().enumerate() {
            if let Some(floor) = topology.kinds()[topology.stock_kind(stock)].floor() {
                if *amount < floor {
                    return Err(StockFlowError::InitialBelowFloor {
                        stock: topology.stocks()[stock].clone(),
                        floor: Box::new(floor),
                        amount: Box::new(amount.clone()),
                    });
                }
            }
        }
        let initial = totals(&topology, &amounts);
        let kind_count = topology.kinds().len();
        Ok(Self {
            topology,
            amounts,
            initial,
            inputs: vec![BigRational::zero(); kind_count],
            outputs: vec![BigRational::zero(); kind_count],
        })
    }

    /// Shared immutable topology.
    pub fn topology(&self) -> &Arc<FlowTopology<K>> {
        &self.topology
    }

    /// Exact stock amounts in stable stock-index order.
    pub fn amounts(&self) -> &[BigRational] {
        &self.amounts
    }

    /// Looks up one exact stock amount.
    pub fn amount(&self, stock: &StockId) -> Option<&BigRational> {
        self.topology
            .stock_index(stock)
            .map(|index| &self.amounts[index])
    }

    /// Current total of a conserved kind.
    pub fn total(&self, kind: K) -> BigRational {
        self.topology
            .kind_index(kind)
            .map(|index| totals(&self.topology, &self.amounts)[index].clone())
            .unwrap_or_default()
    }

    /// Cumulative boundary input of a conserved kind.
    pub fn inputs(&self, kind: K) -> BigRational {
        self.account(kind, &self.inputs)
    }

    /// Cumulative boundary output of a conserved kind.
    pub fn outputs(&self, kind: K) -> BigRational {
        self.account(kind, &self.outputs)
    }

    /// `initial + inputs - outputs - current`, exactly.
    pub fn balance_residual(&self, kind: K) -> BigRational {
        self.topology
            .kind_index(kind)
            .map_or_else(BigRational::zero, |index| {
                self.initial[index].clone() + self.inputs[index].clone()
                    - self.outputs[index].clone()
                    - self.total(kind)
            })
    }

    /// Atomically settles requested amounts in stable flow-slot order.
    pub fn settle(
        &mut self,
        requested: &[BigRational],
    ) -> Result<CompiledSettlementReport<BigRational, K>, StockFlowError<K>> {
        validate_count(self.topology.flows().len(), requested.len())?;
        if let Some((index, amount)) = requested
            .iter()
            .enumerate()
            .find(|(_, amount)| amount.is_negative())
        {
            return Err(StockFlowError::NegativeFlow {
                index,
                process: self.topology.processes()[self.topology.flows()[index].process()].clone(),
                amount: Box::new(amount.clone()),
            });
        }
        let batch = compute_exact_batch(&self.topology, &self.amounts, requested)?;
        self.amounts = batch.amounts;
        add_accounts(&mut self.inputs, batch.inputs);
        add_accounts(&mut self.outputs, batch.outputs);
        debug_assert!(self.amounts.iter().enumerate().all(|(stock, amount)| {
            self.topology.kinds()[self.topology.stock_kind(stock)]
                .floor()
                .is_none_or(|floor| *amount >= floor)
        }));
        Ok(CompiledSettlementReport {
            topology: self.topology.clone(),
            requested: requested.to_vec(),
            applied: batch.applied,
        })
    }

    fn account(&self, kind: K, accounts: &[BigRational]) -> BigRational {
        self.topology
            .kind_index(kind)
            .map(|index| accounts[index].clone())
            .unwrap_or_default()
    }
}

/// Fast binary64 amounts and boundary accounts over an immutable topology.
///
/// Every stored value is finite, and at or above the kind's floor for floored
/// kinds. A settlement that would overflow is rejected before the state is changed.
pub struct DenseState<K> {
    topology: Arc<FlowTopology<K>>,
    floors: Vec<Option<DenseFloor>>,
    amounts: Vec<f64>,
    initial: Vec<f64>,
    inputs: Vec<f64>,
    outputs: Vec<f64>,
    scratch: DenseScratch,
}

/// A kind's floor, converted once for binary64 settlement.
#[derive(Clone)]
struct DenseFloor {
    value: f64,
    exact: BigRational,
}

impl<K: Kind> Clone for DenseState<K> {
    fn clone(&self) -> Self {
        Self {
            topology: self.topology.clone(),
            floors: self.floors.clone(),
            amounts: self.amounts.clone(),
            initial: self.initial.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            scratch: DenseScratch::new(&self.topology),
        }
    }
}

impl<K: Kind> fmt::Debug for DenseState<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DenseState")
            .field("topology", &self.topology)
            .field("amounts", &self.amounts)
            .field("initial", &self.initial)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish()
    }
}

impl<K: Kind> PartialEq for DenseState<K> {
    fn eq(&self, other: &Self) -> bool {
        self.topology == other.topology
            && self.amounts == other.amounts
            && self.initial == other.initial
            && self.inputs == other.inputs
            && self.outputs == other.outputs
    }
}

impl<K: Kind> DenseState<K> {
    /// Creates dense state from amounts in stable stock-index order.
    pub fn new(
        topology: Arc<FlowTopology<K>>,
        amounts: Vec<f64>,
    ) -> Result<Self, StockFlowError<K>> {
        validate_count(topology.stocks().len(), amounts.len())?;
        let floors = topology
            .kinds()
            .iter()
            .map(|kind| {
                kind.floor()
                    .map(|floor| match floor.to_f64() {
                        Some(value) if value.is_finite() => Ok(DenseFloor {
                            value,
                            exact: floor,
                        }),
                        Some(_) | None => Err(StockFlowError::FloorNotRepresentable {
                            kind: *kind,
                            floor: Box::new(floor),
                        }),
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        validate_input_finite(&amounts)?;
        for (stock, amount) in amounts.iter().enumerate() {
            if let Some(floor) = &floors[topology.stock_kind(stock)] {
                if *amount < floor.value {
                    return Err(StockFlowError::InitialBelowFloor {
                        stock: topology.stocks()[stock].clone(),
                        floor: Box::new(floor.exact.clone()),
                        amount: Box::new(from_float(*amount)),
                    });
                }
            }
        }
        let initial = totals(&topology, &amounts);
        validate_intermediate(&initial)?;
        let kind_count = topology.kinds().len();
        let scratch = DenseScratch::new(&topology);
        Ok(Self {
            topology,
            floors,
            amounts,
            initial,
            inputs: vec![0.0; kind_count],
            outputs: vec![0.0; kind_count],
            scratch,
        })
    }

    /// Shared immutable topology.
    pub fn topology(&self) -> &Arc<FlowTopology<K>> {
        &self.topology
    }

    /// Stock amounts in stable stock-index order: finite, and at or above the
    /// kind's floor for floored kinds.
    pub fn amounts(&self) -> &[f64] {
        &self.amounts
    }

    /// Looks up one stock amount.
    pub fn amount(&self, stock: &StockId) -> Option<f64> {
        self.topology
            .stock_index(stock)
            .map(|index| self.amounts[index])
    }

    /// Current total of a conserved kind.
    pub fn total(&self, kind: K) -> f64 {
        self.topology
            .kind_index(kind)
            .map(|index| totals(&self.topology, &self.amounts)[index])
            .unwrap_or(0.0)
    }

    /// Cumulative boundary input of a conserved kind.
    pub fn inputs(&self, kind: K) -> f64 {
        self.account(kind, &self.inputs)
    }

    /// Cumulative boundary output of a conserved kind.
    pub fn outputs(&self, kind: K) -> f64 {
        self.account(kind, &self.outputs)
    }

    /// Floating-point `initial + inputs - outputs - current`.
    pub fn balance_residual(&self, kind: K) -> f64 {
        self.topology.kind_index(kind).map_or(0.0, |index| {
            stable_residual(
                [self.initial[index], self.inputs[index]],
                [self.outputs[index], self.total(kind)],
            )
        })
    }

    /// Tests the balance residual against an error budget scaled to all terms.
    pub fn balance_within(&self, kind: K, tolerance: DenseTolerance) -> bool {
        if !tolerance.absolute.is_finite()
            || !tolerance.relative.is_finite()
            || tolerance.absolute < 0.0
            || tolerance.relative < 0.0
        {
            return false;
        }
        self.topology.kind_index(kind).is_some_and(|index| {
            let current = self.total(kind);
            let terms = [
                self.initial[index],
                self.inputs[index],
                self.outputs[index],
                current,
            ];
            let scale = terms.iter().map(|term| term.abs()).fold(0.0_f64, f64::max);
            let residual = self.balance_residual(kind);
            residual.is_finite()
                && (scale == 0.0
                    || (residual / scale).abs()
                        <= tolerance.absolute / scale
                            + tolerance.relative
                                * terms.iter().map(|term| term.abs() / scale).sum::<f64>())
        })
    }

    /// Atomically settles requested amounts in stable flow-slot order.
    ///
    /// The returned report owns copies of both the requested and applied
    /// vectors. Use [`Self::settle_discard`] when no report is needed.
    pub fn settle(
        &mut self,
        requested: &[f64],
    ) -> Result<CompiledSettlementReport<f64, K>, StockFlowError<K>> {
        self.settle_reusing_scratch(requested)?;
        Ok(CompiledSettlementReport {
            topology: self.topology.clone(),
            requested: requested.to_vec(),
            applied: self.scratch.applied.clone(),
        })
    }

    /// Atomically settles a batch without allocating an owned report.
    ///
    /// Applied values remain only in private reusable workspace and are
    /// overwritten by the next settlement.
    pub fn settle_discard(&mut self, requested: &[f64]) -> Result<(), StockFlowError<K>> {
        self.settle_reusing_scratch(requested)
    }

    fn settle_reusing_scratch(&mut self, requested: &[f64]) -> Result<(), StockFlowError<K>> {
        validate_count(self.topology.flows().len(), requested.len())?;
        validate_input_finite(requested)?;
        if let Some((index, amount)) = requested
            .iter()
            .enumerate()
            .find(|(_, amount)| **amount < 0.0)
        {
            return Err(StockFlowError::NegativeFlow {
                index,
                process: self.topology.processes()[self.topology.flows()[index].process()].clone(),
                amount: Box::new(from_float(*amount)),
            });
        }
        compute_dense_batch(
            &self.topology,
            &self.floors,
            &self.amounts,
            requested,
            &mut self.scratch,
        )?;

        // A proportional binary64 sum can exceed its source by a few ulps.
        // The exact kernel reaches the floor; clamp only that representation noise.
        let tol = DenseTolerance::default();
        for (index, amount) in self.scratch.next_amounts.iter_mut().enumerate() {
            if let Some(floor) = &self.floors[self.topology.stock_kind(index)] {
                if *amount < floor.value {
                    let allowance = tol.absolute
                        + tol.relative * (self.amounts[index].abs() + floor.value.abs());
                    if floor.value - *amount > allowance {
                        return Err(StockFlowError::ArithmeticOverflow);
                    }
                    *amount = floor.value;
                }
            }
        }
        validate_intermediate(&self.scratch.next_amounts)?;
        validate_intermediate(&self.scratch.batch_inputs)?;
        validate_intermediate(&self.scratch.batch_outputs)?;

        checked_accounts_into(
            &self.inputs,
            &self.scratch.batch_inputs,
            &mut self.scratch.next_inputs,
        )?;
        checked_accounts_into(
            &self.outputs,
            &self.scratch.batch_outputs,
            &mut self.scratch.next_outputs,
        )?;
        dense_totals_into(
            &self.topology,
            &self.scratch.next_amounts,
            &mut self.scratch.total_terms,
            &mut self.scratch.next_totals,
        );
        validate_intermediate(&self.scratch.next_totals)?;

        mem::swap(&mut self.amounts, &mut self.scratch.next_amounts);
        mem::swap(&mut self.inputs, &mut self.scratch.next_inputs);
        mem::swap(&mut self.outputs, &mut self.scratch.next_outputs);
        Ok(())
    }

    fn account(&self, kind: K, accounts: &[f64]) -> f64 {
        self.topology
            .kind_index(kind)
            .map(|index| accounts[index])
            .unwrap_or(0.0)
    }
}

struct DenseScratch {
    requested_terms: Vec<Vec<f64>>,
    outgoing_terms: Vec<Vec<f64>>,
    incoming_terms: Vec<Vec<f64>>,
    input_terms: Vec<Vec<f64>>,
    output_terms: Vec<Vec<f64>>,
    total_terms: Vec<Vec<f64>>,
    scales: Vec<f64>,
    batch_inputs: Vec<f64>,
    batch_outputs: Vec<f64>,
    next_amounts: Vec<f64>,
    next_inputs: Vec<f64>,
    next_outputs: Vec<f64>,
    next_totals: Vec<f64>,
    applied: Vec<f64>,
}

impl DenseScratch {
    fn new<K: Kind>(topology: &FlowTopology<K>) -> Self {
        let stock_count = topology.stocks().len();
        let kind_count = topology.kinds().len();
        let mut source_counts = vec![0; stock_count];
        let mut target_counts = vec![0; stock_count];
        let mut input_counts = vec![0; kind_count];
        let mut output_counts = vec![0; kind_count];
        let mut total_counts = vec![0; kind_count];
        for stock in 0..stock_count {
            total_counts[topology.stock_kind(stock)] += 1;
        }
        for flow in topology.flows() {
            if let Some(source) = flow.source() {
                source_counts[source] += 1;
            } else {
                input_counts[flow.kind()] += 1;
            }
            if let Some(target) = flow.target() {
                target_counts[target] += 1;
            } else {
                output_counts[flow.kind()] += 1;
            }
        }
        Self {
            requested_terms: groups_with_capacities(&source_counts),
            outgoing_terms: groups_with_capacities(&source_counts),
            incoming_terms: groups_with_capacities(&target_counts),
            input_terms: groups_with_capacities(&input_counts),
            output_terms: groups_with_capacities(&output_counts),
            total_terms: groups_with_capacities(&total_counts),
            scales: Vec::with_capacity(stock_count),
            batch_inputs: Vec::with_capacity(kind_count),
            batch_outputs: Vec::with_capacity(kind_count),
            next_amounts: Vec::with_capacity(stock_count),
            next_inputs: Vec::with_capacity(kind_count),
            next_outputs: Vec::with_capacity(kind_count),
            next_totals: Vec::with_capacity(kind_count),
            applied: Vec::with_capacity(topology.flows().len()),
        }
    }
}

fn groups_with_capacities(capacities: &[usize]) -> Vec<Vec<f64>> {
    capacities
        .iter()
        .map(|capacity| Vec::with_capacity(*capacity))
        .collect()
}

fn clear_groups(groups: &mut [Vec<f64>]) {
    for group in groups {
        group.clear();
    }
}

struct Batch<N> {
    amounts: Vec<N>,
    inputs: Vec<N>,
    outputs: Vec<N>,
    applied: Vec<N>,
}

/// The least `Refuse` process with a positive request drawing on `stock`.
fn refusing_process<K: Kind, N>(
    topology: &FlowTopology<K>,
    stock: usize,
    requested: &[N],
    positive: impl Fn(&N) -> bool,
) -> Option<ProcessId> {
    topology
        .flows()
        .iter()
        .zip(requested)
        .filter(|(flow, request)| {
            flow.source() == Some(stock)
                && positive(request)
                && topology.process_rationing(flow.process()) == Rationing::Refuse
        })
        .map(|(flow, _)| &topology.processes()[flow.process()])
        .min()
        .cloned()
}

fn compute_exact_batch<K: Kind>(
    topology: &FlowTopology<K>,
    amounts: &[BigRational],
    requested: &[BigRational],
) -> Result<Batch<BigRational>, StockFlowError<K>> {
    let mut requested_terms = vec![Vec::new(); amounts.len()];
    for (flow, amount) in topology.flows().iter().zip(requested) {
        if let Some(source) = flow.source() {
            requested_terms[source].push(amount.clone());
        }
    }
    let requested_by_source: Vec<_> = requested_terms
        .into_iter()
        .map(BigRational::sum_terms)
        .collect();
    let floors = topology
        .kinds()
        .iter()
        .map(|kind| kind.floor())
        .collect::<Vec<_>>();
    let scales = requested_by_source
        .iter()
        .zip(amounts)
        .enumerate()
        .map(|(stock, (withdrawal, available))| {
            source_scale(
                &topology.stocks()[stock],
                floors[topology.stock_kind(stock)].clone(),
                available,
                withdrawal,
                || refusing_process(topology, stock, requested, |request| request.is_positive()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut incoming_terms = vec![Vec::new(); amounts.len()];
    let mut outgoing_terms = vec![Vec::new(); amounts.len()];
    let mut input_terms = vec![Vec::new(); topology.kinds().len()];
    let mut output_terms = vec![Vec::new(); topology.kinds().len()];
    let mut applied = Vec::with_capacity(requested.len());

    for (flow, request) in topology.flows().iter().zip(requested) {
        let amount = flow.source().map_or_else(
            || request.clone(),
            |source| request.clone() * scales[source].clone(),
        );
        if let Some(source) = flow.source() {
            outgoing_terms[source].push(amount.clone());
        } else {
            input_terms[flow.kind()].push(amount.clone());
        }
        if let Some(target) = flow.target() {
            incoming_terms[target].push(amount.clone());
        } else {
            output_terms[flow.kind()].push(amount.clone());
        }
        applied.push(amount);
    }

    let outgoing = outgoing_terms.into_iter().map(BigRational::sum_terms);
    let incoming = incoming_terms.into_iter().map(BigRational::sum_terms);
    let inputs = input_terms
        .into_iter()
        .map(BigRational::sum_terms)
        .collect();
    let outputs = output_terms
        .into_iter()
        .map(BigRational::sum_terms)
        .collect();

    let amounts = amounts
        .iter()
        .cloned()
        .zip(outgoing)
        .zip(incoming)
        .map(|((available, outgoing), incoming)| available - outgoing + incoming)
        .collect();
    Ok(Batch {
        amounts,
        inputs,
        outputs,
        applied,
    })
}

fn compute_dense_batch<K: Kind>(
    topology: &FlowTopology<K>,
    floors: &[Option<DenseFloor>],
    amounts: &[f64],
    requested: &[f64],
    scratch: &mut DenseScratch,
) -> Result<(), StockFlowError<K>> {
    clear_groups(&mut scratch.requested_terms);
    scratch.scales.clear();
    for (flow, amount) in topology.flows().iter().zip(requested) {
        if let Some(source) = flow.source() {
            scratch.requested_terms[source].push(*amount);
        }
    }
    for (stock, (terms, available)) in scratch.requested_terms.iter_mut().zip(amounts).enumerate() {
        let total = sum_dense_terms(terms);
        if !total.is_finite() {
            return Err(StockFlowError::ArithmeticOverflow);
        }
        let scale = match &floors[topology.stock_kind(stock)] {
            None => 1.0,
            Some(floor) => {
                let headroom = *available - floor.value;
                if total == 0.0 || total <= headroom {
                    1.0
                } else if let Some(process) =
                    refusing_process(topology, stock, requested, |request| *request > 0.0)
                {
                    return Err(StockFlowError::BelowFloor {
                        stock: topology.stocks()[stock].clone(),
                        floor: Box::new(floor.exact.clone()),
                        process,
                        available: Box::new(from_float(*available)),
                        withdrawal: Box::new(from_float(total)),
                    });
                } else {
                    headroom / total
                }
            }
        };
        scratch.scales.push(scale);
    }

    clear_groups(&mut scratch.incoming_terms);
    clear_groups(&mut scratch.outgoing_terms);
    clear_groups(&mut scratch.input_terms);
    clear_groups(&mut scratch.output_terms);
    scratch.applied.clear();
    for (flow, request) in topology.flows().iter().zip(requested) {
        let amount = flow
            .source()
            .map_or(*request, |source| *request * scratch.scales[source]);
        if let Some(source) = flow.source() {
            scratch.outgoing_terms[source].push(amount);
        } else {
            scratch.input_terms[flow.kind()].push(amount);
        }
        if let Some(target) = flow.target() {
            scratch.incoming_terms[target].push(amount);
        } else {
            scratch.output_terms[flow.kind()].push(amount);
        }
        scratch.applied.push(amount);
    }

    scratch.next_amounts.clear();
    for (stock, ((available, outgoing), incoming)) in amounts
        .iter()
        .zip(&mut scratch.outgoing_terms)
        .zip(&mut scratch.incoming_terms)
        .enumerate()
    {
        let outgoing = sum_dense_terms(outgoing);
        // A rationed group takes the stock exactly to its floor. Reusing
        // `available - sum(request * scale)` would leave a model-scale-dependent
        // binary64 cancellation residue.
        let remaining = match (
            scratch.scales[stock] < 1.0,
            &floors[topology.stock_kind(stock)],
        ) {
            (true, Some(floor)) => floor.value,
            (false, _) | (true, None) => *available - outgoing,
        };
        scratch
            .next_amounts
            .push(remaining + sum_dense_terms(incoming));
    }
    sums_into(&mut scratch.input_terms, &mut scratch.batch_inputs);
    sums_into(&mut scratch.output_terms, &mut scratch.batch_outputs);
    Ok(())
}

fn sums_into(groups: &mut [Vec<f64>], values: &mut Vec<f64>) {
    values.clear();
    values.extend(groups.iter_mut().map(|terms| sum_dense_terms(terms)));
}

fn sum_dense_terms(terms: &mut [f64]) -> f64 {
    // A canonical magnitude order makes each aggregate independent of flow
    // declaration order. Neumaier compensation retains low-order contributions
    // without making the settlement allowance depend on the number of flows.
    terms.sort_by(|left, right| {
        left.abs()
            .total_cmp(&right.abs())
            .then_with(|| left.total_cmp(right))
    });
    let mut sum = 0.0;
    let mut correction = 0.0;
    for value in terms.iter().copied() {
        let next = sum + value;
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        sum = next;
    }
    sum + correction
}

fn totals<N, K: Kind>(topology: &FlowTopology<K>, amounts: &[N]) -> Vec<N>
where
    N: SettlementNumber,
{
    let mut terms = vec![Vec::new(); topology.kinds().len()];
    for (stock, amount) in amounts.iter().enumerate() {
        let kind = topology.stock_kind(stock);
        terms[kind].push(amount.clone());
    }
    terms.into_iter().map(N::sum_terms).collect()
}

trait SettlementNumber:
    Clone
    + One
    + PartialOrd
    + Zero
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn sum_terms(terms: Vec<Self>) -> Self;
}

impl SettlementNumber for BigRational {
    fn sum_terms(terms: Vec<Self>) -> Self {
        terms.into_iter().sum()
    }
}

impl SettlementNumber for f64 {
    fn sum_terms(mut terms: Vec<Self>) -> Self {
        sum_dense_terms(&mut terms)
    }
}

fn stable_residual(mut positive: [f64; 2], mut negative: [f64; 2]) -> f64 {
    // Cancel terms before summing so a finite balance such as
    // MAX + MAX - MAX - MAX does not overflow during diagnosis.
    for positive_term in &mut positive {
        for negative_term in &mut negative {
            let cancelled = positive_term.min(*negative_term);
            *positive_term -= cancelled;
            *negative_term -= cancelled;
        }
    }
    sum_dense_terms(&mut positive) - sum_dense_terms(&mut negative)
}

fn add_accounts<N>(accounts: &mut [N], increments: Vec<N>)
where
    N: Clone + Add<Output = N>,
{
    for (account, increment) in accounts.iter_mut().zip(increments) {
        *account = account.clone() + increment;
    }
}

fn checked_accounts_into<K: Kind>(
    accounts: &[f64],
    increments: &[f64],
    values: &mut Vec<f64>,
) -> Result<(), StockFlowError<K>> {
    values.clear();
    values.extend(
        accounts
            .iter()
            .zip(increments)
            .map(|(account, increment)| account + increment),
    );
    validate_intermediate(values)?;
    Ok(())
}

fn dense_totals_into<K: Kind>(
    topology: &FlowTopology<K>,
    amounts: &[f64],
    terms: &mut [Vec<f64>],
    totals: &mut Vec<f64>,
) {
    clear_groups(terms);
    for (stock, amount) in amounts.iter().enumerate() {
        terms[topology.stock_kind(stock)].push(*amount);
    }
    sums_into(terms, totals);
}

fn validate_count<K: Kind>(expected: usize, actual: usize) -> Result<(), StockFlowError<K>> {
    if expected == actual {
        Ok(())
    } else {
        Err(StockFlowError::AmountCount { expected, actual })
    }
}

/// The exact value of a finite binary64 input or intermediate.
fn from_float(value: f64) -> BigRational {
    BigRational::from_float(value).expect("validated finite")
}

fn validate_input_finite<K: Kind>(amounts: &[f64]) -> Result<(), StockFlowError<K>> {
    if amounts.iter().all(|amount| amount.is_finite()) {
        Ok(())
    } else {
        Err(StockFlowError::NonFiniteAmount)
    }
}

fn validate_intermediate<K: Kind>(amounts: &[f64]) -> Result<(), StockFlowError<K>> {
    if amounts.iter().all(|amount| amount.is_finite()) {
        Ok(())
    } else {
        Err(StockFlowError::ArithmeticOverflow)
    }
}
