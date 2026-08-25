use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;
use std::{fmt, mem};

use conservation_core::KindId;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::{AppliedFlow, FlowTopology, SettlementReport, StockFlowError, StockId};

/// Applied amounts from one compiled settlement, indexed like the topology's flows.
#[derive(Clone)]
pub struct CompiledSettlementReport<N> {
    topology: Arc<FlowTopology>,
    requested: Vec<N>,
    applied: Vec<N>,
}

impl<N: fmt::Debug> fmt::Debug for CompiledSettlementReport<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledSettlementReport")
            .field("requested", &self.requested)
            .field("applied", &self.applied)
            .finish()
    }
}

impl<N: PartialEq> PartialEq for CompiledSettlementReport<N> {
    fn eq(&self, other: &Self) -> bool {
        self.requested == other.requested && self.applied == other.applied
    }
}

impl<N> CompiledSettlementReport<N> {
    /// Requested amounts in stable flow-slot order.
    pub fn requested(&self) -> &[N] {
        &self.requested
    }

    /// Applied amounts in stable flow-slot order.
    pub fn applied(&self) -> &[N] {
        &self.applied
    }
}

impl FlowTopology {
    /// Restores identifier-rich exact flows from a compiled settlement report.
    ///
    /// The report must originate from a structurally equal topology; equal
    /// vector lengths alone are not sufficient compatibility evidence.
    pub fn materialize_exact_report(
        &self,
        report: &CompiledSettlementReport<BigRational>,
    ) -> Result<SettlementReport, StockFlowError> {
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
                kind: self.kinds()[flow.kind()].clone(),
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
pub struct ExactState {
    topology: Arc<FlowTopology>,
    amounts: Vec<BigRational>,
    initial: Vec<BigRational>,
    inputs: Vec<BigRational>,
    outputs: Vec<BigRational>,
}

impl ExactState {
    /// Creates exact state from amounts in stable stock-index order.
    pub fn new(
        topology: Arc<FlowTopology>,
        amounts: Vec<BigRational>,
    ) -> Result<Self, StockFlowError> {
        validate_count(topology.stocks().len(), amounts.len())?;
        if amounts.iter().any(Signed::is_negative) {
            return Err(StockFlowError::NegativeAmount);
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
    pub fn topology(&self) -> &Arc<FlowTopology> {
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
    pub fn total(&self, kind: &KindId) -> BigRational {
        self.topology
            .kind_index(kind)
            .map(|index| totals(&self.topology, &self.amounts)[index].clone())
            .unwrap_or_default()
    }

    /// Cumulative boundary input of a conserved kind.
    pub fn inputs(&self, kind: &KindId) -> BigRational {
        self.account(kind, &self.inputs)
    }

    /// Cumulative boundary output of a conserved kind.
    pub fn outputs(&self, kind: &KindId) -> BigRational {
        self.account(kind, &self.outputs)
    }

    /// `initial + inputs - outputs - current`, exactly.
    pub fn balance_residual(&self, kind: &KindId) -> BigRational {
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
    ) -> Result<CompiledSettlementReport<BigRational>, StockFlowError> {
        validate_count(self.topology.flows().len(), requested.len())?;
        if requested.iter().any(Signed::is_negative) {
            return Err(StockFlowError::NegativeAmount);
        }
        let batch = compute_batch(&self.topology, &self.amounts, requested);
        self.amounts = batch.amounts;
        add_accounts(&mut self.inputs, batch.inputs);
        add_accounts(&mut self.outputs, batch.outputs);
        debug_assert!(self.amounts.iter().all(|amount| !amount.is_negative()));
        Ok(CompiledSettlementReport {
            topology: self.topology.clone(),
            requested: requested.to_vec(),
            applied: batch.applied,
        })
    }

    fn account(&self, kind: &KindId, accounts: &[BigRational]) -> BigRational {
        self.topology
            .kind_index(kind)
            .map(|index| accounts[index].clone())
            .unwrap_or_default()
    }
}

/// Fast binary64 amounts and boundary accounts over an immutable topology.
///
/// Every stored value is finite and nonnegative. A settlement that would
/// overflow is rejected before the state is changed.
pub struct DenseState {
    topology: Arc<FlowTopology>,
    amounts: Vec<f64>,
    initial: Vec<f64>,
    inputs: Vec<f64>,
    outputs: Vec<f64>,
    scratch: DenseScratch,
}

impl Clone for DenseState {
    fn clone(&self) -> Self {
        Self {
            topology: self.topology.clone(),
            amounts: self.amounts.clone(),
            initial: self.initial.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            scratch: DenseScratch::new(&self.topology),
        }
    }
}

impl fmt::Debug for DenseState {
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

impl PartialEq for DenseState {
    fn eq(&self, other: &Self) -> bool {
        self.topology == other.topology
            && self.amounts == other.amounts
            && self.initial == other.initial
            && self.inputs == other.inputs
            && self.outputs == other.outputs
    }
}

impl DenseState {
    /// Creates dense state from amounts in stable stock-index order.
    pub fn new(topology: Arc<FlowTopology>, amounts: Vec<f64>) -> Result<Self, StockFlowError> {
        validate_count(topology.stocks().len(), amounts.len())?;
        validate_dense(&amounts)?;
        let initial = totals(&topology, &amounts);
        validate_intermediate(&initial)?;
        let kind_count = topology.kinds().len();
        let scratch = DenseScratch::new(&topology);
        Ok(Self {
            topology,
            amounts,
            initial,
            inputs: vec![0.0; kind_count],
            outputs: vec![0.0; kind_count],
            scratch,
        })
    }

    /// Shared immutable topology.
    pub fn topology(&self) -> &Arc<FlowTopology> {
        &self.topology
    }

    /// Finite, nonnegative stock amounts in stable stock-index order.
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
    pub fn total(&self, kind: &KindId) -> f64 {
        self.topology
            .kind_index(kind)
            .map(|index| totals(&self.topology, &self.amounts)[index])
            .unwrap_or(0.0)
    }

    /// Cumulative boundary input of a conserved kind.
    pub fn inputs(&self, kind: &KindId) -> f64 {
        self.account(kind, &self.inputs)
    }

    /// Cumulative boundary output of a conserved kind.
    pub fn outputs(&self, kind: &KindId) -> f64 {
        self.account(kind, &self.outputs)
    }

    /// Floating-point `initial + inputs - outputs - current`.
    pub fn balance_residual(&self, kind: &KindId) -> f64 {
        self.topology.kind_index(kind).map_or(0.0, |index| {
            stable_residual(
                [self.initial[index], self.inputs[index]],
                [self.outputs[index], self.total(kind)],
            )
        })
    }

    /// Tests the balance residual against an error budget scaled to all terms.
    pub fn balance_within(&self, kind: &KindId, tolerance: DenseTolerance) -> bool {
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
            let scale = terms.iter().copied().fold(0.0_f64, f64::max);
            let residual = self.balance_residual(kind);
            residual.is_finite()
                && (scale == 0.0
                    || (residual / scale).abs()
                        <= tolerance.absolute / scale
                            + tolerance.relative
                                * terms.iter().map(|term| term / scale).sum::<f64>())
        })
    }

    /// Atomically settles requested amounts in stable flow-slot order.
    ///
    /// The returned report owns copies of both the requested and applied
    /// vectors. Use [`Self::settle_discard`] when no report is needed.
    pub fn settle(
        &mut self,
        requested: &[f64],
    ) -> Result<CompiledSettlementReport<f64>, StockFlowError> {
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
    pub fn settle_discard(&mut self, requested: &[f64]) -> Result<(), StockFlowError> {
        self.settle_reusing_scratch(requested)
    }

    fn settle_reusing_scratch(&mut self, requested: &[f64]) -> Result<(), StockFlowError> {
        validate_count(self.topology.flows().len(), requested.len())?;
        validate_dense(requested)?;
        compute_dense_batch(&self.topology, &self.amounts, requested, &mut self.scratch)?;

        // A proportional binary64 sum can exceed its source by a few ulps.
        // The exact kernel reaches zero; clamp only that representation noise.
        for (index, amount) in self.scratch.next_amounts.iter_mut().enumerate() {
            if *amount < 0.0 {
                let allowance = DenseTolerance::default().absolute
                    + DenseTolerance::default().relative * self.amounts[index].abs();
                if amount.abs() > allowance {
                    return Err(StockFlowError::ArithmeticOverflow);
                }
                *amount = 0.0;
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

    fn account(&self, kind: &KindId, accounts: &[f64]) -> f64 {
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
    fn new(topology: &FlowTopology) -> Self {
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

fn compute_batch<N>(topology: &FlowTopology, amounts: &[N], requested: &[N]) -> Batch<N>
where
    N: SettlementNumber,
{
    let mut requested_terms = vec![Vec::new(); amounts.len()];
    for (flow, amount) in topology.flows().iter().zip(requested) {
        if let Some(source) = flow.source() {
            requested_terms[source].push(amount.clone());
        }
    }
    let requested_by_source: Vec<_> = requested_terms.into_iter().map(N::sum_terms).collect();
    let scales: Vec<_> = requested_by_source
        .iter()
        .zip(amounts)
        .map(|(requested, available)| {
            if requested.is_zero() || requested <= available {
                N::one()
            } else {
                available.clone() / requested.clone()
            }
        })
        .collect();

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

    let outgoing = outgoing_terms.into_iter().map(N::sum_terms);
    let incoming = incoming_terms.into_iter().map(N::sum_terms);
    let inputs = input_terms.into_iter().map(N::sum_terms).collect();
    let outputs = output_terms.into_iter().map(N::sum_terms).collect();

    let amounts = amounts
        .iter()
        .cloned()
        .zip(outgoing)
        .zip(incoming)
        .map(|((available, outgoing), incoming)| available - outgoing + incoming)
        .collect();
    Batch {
        amounts,
        inputs,
        outputs,
        applied,
    }
}

fn compute_dense_batch(
    topology: &FlowTopology,
    amounts: &[f64],
    requested: &[f64],
    scratch: &mut DenseScratch,
) -> Result<(), StockFlowError> {
    clear_groups(&mut scratch.requested_terms);
    scratch.scales.clear();
    for (flow, amount) in topology.flows().iter().zip(requested) {
        if let Some(source) = flow.source() {
            scratch.requested_terms[source].push(*amount);
        }
    }
    for (terms, available) in scratch.requested_terms.iter_mut().zip(amounts) {
        let total = sum_dense_terms(terms);
        if !total.is_finite() {
            return Err(StockFlowError::ArithmeticOverflow);
        }
        scratch.scales.push(if total == 0.0 || total <= *available {
            1.0
        } else {
            *available / total
        });
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
    for ((available, outgoing), incoming) in amounts
        .iter()
        .zip(&mut scratch.outgoing_terms)
        .zip(&mut scratch.incoming_terms)
    {
        scratch
            .next_amounts
            .push(*available - sum_dense_terms(outgoing) + sum_dense_terms(incoming));
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

fn totals<N>(topology: &FlowTopology, amounts: &[N]) -> Vec<N>
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

fn checked_accounts_into(
    accounts: &[f64],
    increments: &[f64],
    values: &mut Vec<f64>,
) -> Result<(), StockFlowError> {
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

fn dense_totals_into(
    topology: &FlowTopology,
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

fn validate_count(expected: usize, actual: usize) -> Result<(), StockFlowError> {
    if expected == actual {
        Ok(())
    } else {
        Err(StockFlowError::AmountCount { expected, actual })
    }
}

fn validate_dense(amounts: &[f64]) -> Result<(), StockFlowError> {
    validate_input_finite(amounts)?;
    if amounts.iter().any(|amount| *amount < 0.0) {
        return Err(StockFlowError::NegativeAmount);
    }
    Ok(())
}

fn validate_input_finite(amounts: &[f64]) -> Result<(), StockFlowError> {
    if amounts.iter().all(|amount| amount.is_finite()) {
        Ok(())
    } else {
        Err(StockFlowError::NonFiniteAmount)
    }
}

fn validate_intermediate(amounts: &[f64]) -> Result<(), StockFlowError> {
    if amounts.iter().all(|amount| amount.is_finite()) {
        Ok(())
    } else {
        Err(StockFlowError::ArithmeticOverflow)
    }
}
