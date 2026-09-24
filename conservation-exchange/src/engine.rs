use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::sync::Arc;

use conservation_core::{DimensionAlgebra, KindRegistry};
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::expression::Evaluation;
use crate::model::{KindContext, ValidatedModel, identifier, invalid, zero};
use crate::{
    Capacity, Constraint, Error, Exchange, Expr, Fact, Law, Model, Placement, Quantity,
    RecordWrite, Stock,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contribution<K> {
    owner: String,
    writes: Vec<RecordWrite>,
    facts: BTreeMap<String, Quantity<K>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request<K> {
    exchange: Exchange<K>,
    contributions: Vec<Contribution<K>>,
}

/// Evidence of the committed transition. Boundary amounts are signed net inputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt<K> {
    pub id: String,
    pub revision: u64,
    pub law: String,
    pub boundaries: BTreeMap<String, Quantity<K>>,
    pub changes: BTreeMap<String, (Quantity<K>, Quantity<K>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StockState<K> {
    definition: Arc<Stock<K>>,
    amount: BigRational,
}

type OwnerRecords = BTreeMap<String, Arc<Vec<u8>>>;
/// One committed request and the receipt it produced.
type Commit<K> = (Arc<Request<K>>, Arc<Receipt<K>>);

#[derive(Clone, Debug)]
struct State<K> {
    model: Arc<ValidatedModel<K>>,
    revision: u64,
    stocks: BTreeMap<String, Arc<StockState<K>>>,
    retired: BTreeSet<String>,
    records: BTreeMap<String, Arc<OwnerRecords>>,
    commits: Vec<Commit<K>>,
    committed: BTreeMap<String, usize>,
}

impl<K> Default for State<K> {
    fn default() -> Self {
        Self {
            model: Arc::default(),
            revision: 0,
            stocks: BTreeMap::new(),
            retired: BTreeSet::new(),
            records: BTreeMap::new(),
            commits: Vec::new(),
            committed: BTreeMap::new(),
        }
    }
}

/// An owner-authored contribution tied to one exact proposal and observed root.
/// It contains no callbacks and cannot mutate either the engine or external state.
#[derive(Clone, Debug)]
pub struct Participation<K> {
    lineage: Arc<()>,
    base: Arc<State<K>>,
    exchange: Exchange<K>,
    contribution: Contribution<K>,
}

/// Opaque, process-local preparation. Restore deliberately invalidates all handles.
#[derive(Clone, Debug)]
pub struct Prepared<K> {
    lineage: Arc<()>,
    base: Arc<State<K>>,
    request: Arc<Request<K>>,
    next: Option<Arc<State<K>>>,
    receipt: Receipt<K>,
}

/// One quantity/participant authority. Publication replaces one immutable root.
///
/// Domain participants store their canonical immutable records here. Mutating a
/// foreign database/object is not participation and is not made atomic by this API.
#[derive(Debug)]
pub struct Engine<K> {
    lineage: Arc<()>,
    initial_model: Model<K>,
    state: Arc<State<K>>,
}

impl<K: DimensionAlgebra> Engine<K> {
    pub fn new(mut model: Model<K>) -> Result<Self, Error<K>> {
        let validated = Arc::new(ValidatedModel::new(&model)?);
        model.laws.sort_by(|a, b| a.id.cmp(&b.id));
        let state = State {
            model: validated,
            ..State::default()
        };
        Ok(Self {
            lineage: Arc::new(()),
            initial_model: model,
            state: Arc::new(state),
        })
    }

    pub fn revision(&self) -> u64 {
        self.state.revision
    }
    pub fn amount(&self, stock: &str) -> Result<&BigRational, Error<K>> {
        self.state
            .stocks
            .get(stock)
            .map(|state| &state.amount)
            .ok_or_else(|| invalid(format!("unknown stock {stock}")))
    }
    pub fn stock(&self, stock: &str) -> Result<&Stock<K>, Error<K>> {
        self.state
            .stocks
            .get(stock)
            .map(|state| state.definition.as_ref())
            .ok_or_else(|| invalid(format!("unknown stock {stock}")))
    }
    pub fn record(&self, owner: &str, key: &str) -> Option<&[u8]> {
        self.state
            .records
            .get(owner)?
            .get(key)
            .map(|value| value.as_slice())
    }
    pub fn receipt(&self, id: &str) -> Option<&Receipt<K>> {
        self.state
            .committed
            .get(id)
            .map(|index| self.state.commits[*index].1.as_ref())
    }

    /// Stage an owner's immutable canonical records and evaluated facts.
    /// The caller is the trusted domain owner, responsible for schema/authorization.
    /// Empty or absent required participation is rejection, not implicit approval.
    pub fn participate(
        &self,
        owner: &str,
        exchange: &Exchange<K>,
        mut writes: Vec<RecordWrite>,
        facts: BTreeMap<String, Quantity<K>>,
    ) -> Result<Participation<K>, Error<K>> {
        let model = self.model_for(exchange)?;
        model.owner(owner)?;
        let exchange = Self::canonical(exchange.clone(), &model)?;
        writes.sort_by(|a, b| a.key.cmp(&b.key));
        for write in &writes {
            identifier(&write.key)?;
        }
        if writes.windows(2).any(|pair| pair[0].key == pair[1].key) {
            return Err(participant(owner, "duplicate record key"));
        }
        if writes.is_empty() && facts.is_empty() {
            return Err(participant(owner, "empty participation"));
        }
        for (id, value) in &facts {
            identifier(id)?;
            value.validate()?;
        }
        Ok(Participation {
            lineage: self.lineage.clone(),
            base: self.state.clone(),
            exchange,
            contribution: Contribution {
                owner: owner.into(),
                writes,
                facts,
            },
        })
    }

    /// Validate without publishing. Preparing never reserves resources or emits evidence.
    pub fn prepare(
        &self,
        exchange: Exchange<K>,
        participation: Vec<Participation<K>>,
    ) -> Result<Prepared<K>, Error<K>> {
        let model = self.model_for(&exchange)?;
        let exchange = Self::canonical(exchange, &model)?;
        for part in &participation {
            if !Arc::ptr_eq(&part.lineage, &self.lineage) {
                return Err(Error::ForeignPreparation);
            }
            if part.exchange != exchange {
                return Err(participant(
                    &part.contribution.owner,
                    "contribution belongs to another proposal",
                ));
            }
        }
        let mut contributions: Vec<_> = participation
            .iter()
            .map(|part| part.contribution.clone())
            .collect();
        contributions.sort_by(|a, b| a.owner.cmp(&b.owner));
        if contributions
            .windows(2)
            .any(|pair| pair[0].owner == pair[1].owner)
        {
            return Err(invalid("duplicate participant owner"));
        }
        let request = Arc::new(Request {
            exchange,
            contributions,
        });
        // An identical duplicate returns the original result even at a later revision.
        if let Some(receipt) = self.duplicate(&request)? {
            return Ok(Prepared {
                lineage: self.lineage.clone(),
                base: self.state.clone(),
                request,
                next: None,
                receipt: receipt.clone(),
            });
        }
        for part in &participation {
            if !Arc::ptr_eq(&part.base, &self.state) {
                return Err(Error::Stale);
            }
        }
        let (next, receipt) = self.transition(&request, model)?;
        Ok(Prepared {
            lineage: self.lineage.clone(),
            base: self.state.clone(),
            request,
            next: Some(Arc::new(next)),
            receipt,
        })
    }

    /// The only mutation boundary. All allocation/validation precedes the root swap.
    /// No user code, IO, or observer runs while publishing.
    pub fn publish(&mut self, prepared: Prepared<K>) -> Result<Receipt<K>, Error<K>> {
        if !Arc::ptr_eq(&prepared.lineage, &self.lineage) {
            return Err(Error::ForeignPreparation);
        }
        if let Some(receipt) = self.duplicate(&prepared.request)? {
            return Ok(receipt.clone());
        }
        if !Arc::ptr_eq(&prepared.base, &self.state) {
            return Err(Error::Stale);
        }
        let next = prepared
            .next
            .ok_or_else(|| invalid("duplicate preparation lost its committed identity"))?;
        self.state = next;
        Ok(prepared.receipt)
    }

    /// Versioned replay snapshot of the authoritative model and committed cut.
    /// Each kind is written as its registry name. Preparations are intentionally
    /// absent; restoring creates a new handle lineage.
    pub fn snapshot(&self) -> Result<Vec<u8>, Error<K>> {
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            model: self.initial_model.clone(),
            requests: self
                .state
                .commits
                .iter()
                .map(|(request, _)| request.clone())
                .collect(),
        }
        .map_kinds(&mut |kind: K| Ok::<_, Infallible>(kind.to_string()))
        .unwrap_or_else(|never| match never {});
        serde_json::to_vec(&snapshot).map_err(|error| Error::Snapshot(error.to_string()))
    }

    /// Resolve every kind name once through `registry`, then revalidate every
    /// recorded transition; never trust separately serialized balances.
    pub fn restore<R: KindRegistry<Kind = K>>(
        bytes: &[u8],
        registry: &R,
    ) -> Result<Self, Error<K>> {
        let SnapshotVersion { version } =
            serde_json::from_slice(bytes).map_err(|error| Error::Snapshot(error.to_string()))?;
        if version != SNAPSHOT_VERSION {
            return Err(Error::UnsupportedSnapshot { version });
        }
        let snapshot: Snapshot<String> =
            serde_json::from_slice(bytes).map_err(|error| Error::Snapshot(error.to_string()))?;
        let snapshot = snapshot.map_kinds(&mut |name: String| {
            registry.resolve(&name).ok_or(Error::UnknownKind { name })
        })?;
        let mut engine = Self::new(snapshot.model)?;
        for request in snapshot.requests {
            let request = Arc::unwrap_or_clone(request);
            if engine.receipt(&request.exchange.id).is_some() {
                return Err(Error::Snapshot("duplicate commit in snapshot".into()));
            }
            let parts = request
                .contributions
                .into_iter()
                .map(|part| {
                    engine.participate(&part.owner, &request.exchange, part.writes, part.facts)
                })
                .collect::<Result<_, _>>()?;
            let prepared = engine.prepare(request.exchange, parts)?;
            engine.publish(prepared)?;
        }
        Ok(engine)
    }

    fn model_for(&self, exchange: &Exchange<K>) -> Result<Arc<ValidatedModel<K>>, Error<K>> {
        match &exchange.declarations {
            Some(declarations) => self.state.model.extended(declarations).map(Arc::new),
            None => Ok(self.state.model.clone()),
        }
    }

    fn law<'a>(model: &'a ValidatedModel<K>, id: &str) -> Result<&'a Law<K>, Error<K>> {
        model
            .laws
            .get(id)
            .map(AsRef::as_ref)
            .ok_or_else(|| invalid(format!("unknown law {id}")))
    }

    fn canonical(
        mut exchange: Exchange<K>,
        model: &ValidatedModel<K>,
    ) -> Result<Exchange<K>, Error<K>> {
        identifier(&exchange.id)?;
        if let Some(declarations) = &mut exchange.declarations {
            declarations.laws.sort_by(|a, b| a.id.cmp(&b.id));
        }
        let law = Self::law(model, &exchange.law)?;
        if exchange.bindings.keys().ne(law.slots.keys()) {
            return Err(invalid("bindings must exactly match law slots"));
        }
        let mut stocks = BTreeSet::new();
        for id in exchange.bindings.values() {
            identifier(id)?;
            if !stocks.insert(id) {
                return Err(invalid("one stock cannot alias multiple law slots"));
            }
        }
        for (id, value) in &exchange.deltas {
            let slot = law
                .slots
                .get(id)
                .ok_or_else(|| invalid(format!("undeclared delta slot {id}")))?;
            value.same_kind(slot.difference(), KindContext::Delta { slot: id.clone() })?;
        }
        for (id, value) in &exchange.boundaries {
            let port = law
                .boundaries
                .get(id)
                .ok_or_else(|| invalid(format!("undeclared boundary {id}")))?;
            value.same_kind(*port, KindContext::Boundary { port: id.clone() })?;
        }
        for (id, kind) in &law.slots {
            exchange
                .deltas
                .entry(id.clone())
                .or_insert_with(|| zero(kind.difference()));
        }
        for (id, kind) in &law.boundaries {
            exchange
                .boundaries
                .entry(id.clone())
                .or_insert_with(|| zero(*kind));
        }
        exchange.creates.sort_by(|a, b| a.id.cmp(&b.id));
        if exchange
            .creates
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
        {
            return Err(invalid("duplicate stock creation"));
        }
        for stock in &exchange.creates {
            model.stock(stock)?;
            if !stocks.contains(&stock.id) {
                return Err(invalid("created stock must have a law slot"));
            }
        }
        for id in exchange.removes.iter().chain(exchange.moves.keys()) {
            if !stocks.contains(id) {
                return Err(invalid("topology change must have a law slot"));
            }
        }
        if exchange.removes.iter().any(|id| {
            exchange.moves.contains_key(id) || exchange.creates.iter().any(|stock| &stock.id == id)
        }) {
            return Err(invalid("conflicting topology operations"));
        }
        Ok(exchange)
    }

    fn duplicate(&self, request: &Request<K>) -> Result<Option<&Receipt<K>>, Error<K>> {
        if let Some(index) = self.state.committed.get(&request.exchange.id) {
            let (original, receipt) = &self.state.commits[*index];
            if original.as_ref() != request {
                return Err(Error::Duplicate {
                    id: request.exchange.id.clone(),
                });
            }
            return Ok(Some(receipt));
        }
        Ok(None)
    }

    fn transition(
        &self,
        request: &Arc<Request<K>>,
        model: Arc<ValidatedModel<K>>,
    ) -> Result<(State<K>, Receipt<K>), Error<K>> {
        let exchange = &request.exchange;
        let law = Self::law(&model, &exchange.law)?;
        let present: BTreeSet<_> = request
            .contributions
            .iter()
            .map(|part| part.owner.clone())
            .collect();
        if let Some(missing) = law.participants.difference(&present).next() {
            return Err(participant(missing, "required participation missing"));
        }
        let mut facts = BTreeMap::new();
        for part in &request.contributions {
            for (id, value) in &part.facts {
                let declaration = law
                    .facts
                    .get(id)
                    .ok_or_else(|| participant(&part.owner, format!("undeclared fact {id}")))?;
                if declaration.owner != part.owner {
                    return Err(participant(
                        &part.owner,
                        format!("wrong owner for fact {id}"),
                    ));
                }
                value.same_kind(declaration.kind, KindContext::Fact { fact: id.clone() })?;
                if facts.insert(id.clone(), value.clone()).is_some() {
                    return Err(participant(&part.owner, "duplicate fact"));
                }
            }
        }
        if facts.keys().ne(law.facts.keys()) {
            return Err(invalid("missing required evaluated facts"));
        }
        let mut next = (*self.state).clone();
        for stock in &exchange.creates {
            if next.stocks.contains_key(&stock.id) || next.retired.contains(&stock.id) {
                return Err(invalid(format!("stock identity {} already used", stock.id)));
            }
            next.stocks.insert(
                stock.id.clone(),
                Arc::new(StockState {
                    definition: Arc::new(stock.clone()),
                    amount: BigRational::zero(),
                }),
            );
        }
        let mut before = BTreeMap::new();
        let mut after = BTreeMap::new();
        let mut changes = BTreeMap::new();
        for (slot, id) in &exchange.bindings {
            let stock = Arc::make_mut(
                next.stocks
                    .get_mut(id)
                    .ok_or_else(|| invalid(format!("unknown stock {id}")))?,
            );
            let kind = law.slots[slot];
            if stock.definition.kind != kind {
                return Err(Error::Kinds {
                    context: KindContext::Slot {
                        slot: slot.clone(),
                        stock: id.clone(),
                    },
                    expected: kind,
                    found: stock.definition.kind,
                });
            }
            let prior = Quantity::new(stock.amount.clone(), kind);
            stock.amount += &exchange.deltas[slot].amount;
            if let Some(floor) = kind.floor() {
                if stock.amount < floor {
                    return Err(Error::BelowFloor {
                        stock: id.clone(),
                        kind,
                        amount: stock.amount.clone(),
                    });
                }
            }
            let following = Quantity::new(stock.amount.clone(), kind);
            before.insert(slot.clone(), prior.clone());
            after.insert(slot.clone(), following.clone());
            changes.insert(id.clone(), (prior, following));
        }
        for constraint in &law.constraints {
            constraint.check(
                &exchange.id,
                &Evaluation {
                    before: &before,
                    after: &after,
                    deltas: &exchange.deltas,
                    boundaries: &exchange.boundaries,
                    facts: &facts,
                },
            )?;
        }
        for (id, placement) in &exchange.moves {
            let stock = Arc::make_mut(
                next.stocks
                    .get_mut(id)
                    .ok_or_else(|| invalid(format!("unknown stock {id}")))?,
            );
            let definition = Arc::make_mut(&mut stock.definition);
            definition.owner = placement.owner.clone();
            definition.capacities = placement.capacities.clone();
            model.stock(&stock.definition)?;
        }
        for id in &exchange.removes {
            if !next.stocks[id].amount.is_zero() {
                return Err(invalid(format!("cannot remove nonempty stock {id}")));
            }
            next.stocks.remove(id);
            next.retired.insert(id.clone());
        }
        let mut loads: BTreeMap<String, BigRational> = BTreeMap::new();
        for stock in next.stocks.values() {
            for (id, weight) in &stock.definition.capacities {
                *loads.entry(id.clone()).or_default() += &weight.amount * &stock.amount;
            }
        }
        for (id, load) in loads {
            let residual = load - &model.capacities[&id].maximum.amount;
            if residual.is_positive() {
                return Err(Error::Capacity { id, residual });
            }
        }
        for part in &request.contributions {
            for write in &part.writes {
                let records = Arc::make_mut(next.records.entry(part.owner.clone()).or_default());
                match &write.value {
                    Some(value) => {
                        records.insert(write.key.clone(), Arc::new(value.clone()));
                    }
                    None => {
                        if records.remove(&write.key).is_none() {
                            return Err(participant(
                                &part.owner,
                                format!("cannot remove missing record {}", write.key),
                            ));
                        }
                    }
                }
            }
        }
        next.model = model;
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("revision overflow"))?;
        let receipt = Receipt {
            id: exchange.id.clone(),
            revision: next.revision,
            law: exchange.law.clone(),
            boundaries: exchange.boundaries.clone(),
            changes,
        };
        next.committed
            .insert(exchange.id.clone(), next.commits.len());
        next.commits
            .push((request.clone(), Arc::new(receipt.clone())));
        Ok((next, receipt))
    }
}

fn participant<K: DimensionAlgebra>(owner: &str, reason: impl Into<String>) -> Error<K> {
    Error::Participant {
        owner: owner.into(),
        reason: reason.into(),
    }
}

const SNAPSHOT_VERSION: u32 = 3;

/// Read first, so an unsupported version is reported before its fields are parsed.
#[derive(Deserialize)]
struct SnapshotVersion {
    version: u32,
}

/// The wire form is `Snapshot<String>`: every kind is its registry name.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot<K> {
    version: u32,
    model: Model<K>,
    requests: Vec<Arc<Request<K>>>,
}

/// Replaces every kind in a document, one at a time. The only way kinds cross
/// the snapshot byte boundary.
pub(crate) trait MapKinds<K>: Sized {
    type Mapped<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Self::Mapped<L>, E>;
}

fn map_named<T: MapKinds<K>, K, L, E>(
    values: BTreeMap<String, T>,
    f: &mut impl FnMut(K) -> Result<L, E>,
) -> Result<BTreeMap<String, T::Mapped<L>>, E> {
    values
        .into_iter()
        .map(|(id, value)| Ok((id, value.map_kinds(f)?)))
        .collect()
}

fn map_named_kinds<K, L, E>(
    values: BTreeMap<String, K>,
    f: &mut impl FnMut(K) -> Result<L, E>,
) -> Result<BTreeMap<String, L>, E> {
    values
        .into_iter()
        .map(|(id, kind)| Ok((id, f(kind)?)))
        .collect()
}

fn map_each<T: MapKinds<K>, K, L, E>(
    values: Vec<T>,
    f: &mut impl FnMut(K) -> Result<L, E>,
) -> Result<Vec<T::Mapped<L>>, E> {
    values.into_iter().map(|value| value.map_kinds(f)).collect()
}

impl<K> MapKinds<K> for Quantity<K> {
    type Mapped<L> = Quantity<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Quantity<L>, E> {
        Ok(Quantity {
            amount: self.amount,
            kind: f(self.kind)?,
        })
    }
}

impl<K> MapKinds<K> for Stock<K> {
    type Mapped<L> = Stock<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Stock<L>, E> {
        Ok(Stock {
            id: self.id,
            owner: self.owner,
            kind: f(self.kind)?,
            capacities: map_named(self.capacities, f)?,
        })
    }
}

impl<K> MapKinds<K> for Placement<K> {
    type Mapped<L> = Placement<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Placement<L>, E> {
        Ok(Placement {
            owner: self.owner,
            capacities: map_named(self.capacities, f)?,
        })
    }
}

impl<K> MapKinds<K> for Capacity<K> {
    type Mapped<L> = Capacity<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Capacity<L>, E> {
        Ok(Capacity {
            maximum: self.maximum.map_kinds(f)?,
        })
    }
}

impl<K> MapKinds<K> for Fact<K> {
    type Mapped<L> = Fact<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Fact<L>, E> {
        Ok(Fact {
            owner: self.owner,
            kind: f(self.kind)?,
        })
    }
}

impl<K> MapKinds<K> for Expr<K> {
    type Mapped<L> = Expr<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Expr<L>, E> {
        Ok(match self {
            Self::Constant(value) => Expr::Constant(value.map_kinds(f)?),
            Self::Before(id) => Expr::Before(id),
            Self::After(id) => Expr::After(id),
            Self::Delta(id) => Expr::Delta(id),
            Self::Boundary(id) => Expr::Boundary(id),
            Self::Fact(id) => Expr::Fact(id),
            Self::Sum(terms) => Expr::Sum(map_each(terms, f)?),
            Self::Product(a, b) => {
                Expr::Product(Box::new(a.map_kinds(f)?), Box::new(b.map_kinds(f)?))
            }
            Self::Quotient(a, b) => {
                Expr::Quotient(Box::new(a.map_kinds(f)?), Box::new(b.map_kinds(f)?))
            }
        })
    }
}

impl<K> MapKinds<K> for Constraint<K> {
    type Mapped<L> = Constraint<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Constraint<L>, E> {
        Ok(Constraint {
            id: self.id,
            left: self.left.map_kinds(f)?,
            right: self.right.map_kinds(f)?,
            relation: self.relation,
            absolute: self.absolute,
            relative: self.relative,
        })
    }
}

impl<K> MapKinds<K> for Law<K> {
    type Mapped<L> = Law<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Law<L>, E> {
        Ok(Law {
            id: self.id,
            slots: map_named_kinds(self.slots, f)?,
            boundaries: map_named_kinds(self.boundaries, f)?,
            facts: map_named(self.facts, f)?,
            participants: self.participants,
            constraints: map_each(self.constraints, f)?,
        })
    }
}

impl<K> MapKinds<K> for Model<K> {
    type Mapped<L> = Model<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Model<L>, E> {
        Ok(Model {
            owners: self.owners,
            capacities: map_named(self.capacities, f)?,
            laws: map_each(self.laws, f)?,
        })
    }
}

impl<K> MapKinds<K> for Exchange<K> {
    type Mapped<L> = Exchange<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Exchange<L>, E> {
        Ok(Exchange {
            id: self.id,
            law: self.law,
            bindings: self.bindings,
            deltas: map_named(self.deltas, f)?,
            boundaries: map_named(self.boundaries, f)?,
            creates: map_each(self.creates, f)?,
            removes: self.removes,
            moves: map_named(self.moves, f)?,
            declarations: self
                .declarations
                .map(|model| model.map_kinds(f))
                .transpose()?,
        })
    }
}

impl<K> MapKinds<K> for Contribution<K> {
    type Mapped<L> = Contribution<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Contribution<L>, E> {
        Ok(Contribution {
            owner: self.owner,
            writes: self.writes,
            facts: map_named(self.facts, f)?,
        })
    }
}

impl<K> MapKinds<K> for Request<K> {
    type Mapped<L> = Request<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Request<L>, E> {
        Ok(Request {
            exchange: self.exchange.map_kinds(f)?,
            contributions: map_each(self.contributions, f)?,
        })
    }
}

impl<K: Clone> MapKinds<K> for Snapshot<K> {
    type Mapped<L> = Snapshot<L>;
    fn map_kinds<L, E>(self, f: &mut impl FnMut(K) -> Result<L, E>) -> Result<Snapshot<L>, E> {
        Ok(Snapshot {
            version: self.version,
            model: self.model.map_kinds(f)?,
            requests: self
                .requests
                .into_iter()
                .map(|request| Ok(Arc::new(Arc::unwrap_or_clone(request).map_kinds(f)?)))
                .collect::<Result<_, E>>()?,
        })
    }
}

#[cfg(test)]
mod sharing_tests {
    use super::*;
    use conservation_test_kinds::{TestKind, TestKinds};

    #[test]
    fn unchanged_payloads_are_shared_across_prepared_roots() {
        let kind = TestKind::Material;
        let law = Law {
            id: "hold".into(),
            slots: BTreeMap::from([("stock".into(), kind)]),
            boundaries: BTreeMap::new(),
            facts: BTreeMap::new(),
            participants: BTreeSet::from(["owner".into()]),
            constraints: vec![Constraint::equal(
                "zero",
                Expr::delta("stock"),
                Expr::constant(zero(kind)),
            )],
        };
        let mut engine = Engine::new(Model {
            owners: BTreeSet::from(["owner".into()]),
            capacities: BTreeMap::new(),
            laws: vec![law],
        })
        .unwrap();
        let mut first = Exchange::new("first", "hold");
        first.bindings.insert("stock".into(), "item".into());
        first.creates.push(Stock {
            id: "item".into(),
            owner: "owner".into(),
            kind,
            capacities: BTreeMap::new(),
        });
        let part = engine
            .participate(
                "owner",
                &first,
                vec![RecordWrite {
                    key: "large-record".into(),
                    value: Some(vec![7; 65536]),
                }],
                BTreeMap::new(),
            )
            .unwrap();
        let prepared = engine.prepare(first, vec![part]).unwrap();
        engine.publish(prepared).unwrap();
        let old = Engine {
            lineage: engine.lineage.clone(),
            initial_model: engine.initial_model.clone(),
            state: engine.state.clone(),
        };
        let before = old.snapshot().unwrap();
        let mut second = Exchange::new("second", "hold");
        second.bindings.insert("stock".into(), "item".into());
        let part = engine
            .participate(
                "owner",
                &second,
                vec![RecordWrite {
                    key: "other".into(),
                    value: Some(vec![1]),
                }],
                BTreeMap::new(),
            )
            .unwrap();
        let prepared = engine.prepare(second, vec![part]).unwrap();
        engine.publish(prepared).unwrap();
        assert_eq!(old.snapshot().unwrap(), before);
        assert!(
            std::ptr::eq(old.stock("item").unwrap(), engine.stock("item").unwrap()),
            "unchanged stock definition was deep-copied"
        );
        assert!(
            std::ptr::eq(
                old.record("owner", "large-record").unwrap().as_ptr(),
                engine.record("owner", "large-record").unwrap().as_ptr()
            ),
            "unchanged record payload was deep-copied"
        );
        assert!(
            std::ptr::eq(
                old.receipt("first").unwrap(),
                engine.receipt("first").unwrap()
            ),
            "historical receipt was deep-copied"
        );
        let old_request: &Request<TestKind> = &old.state.commits[0].0;
        let new_request: &Request<TestKind> = &engine.state.commits[0].0;
        assert!(
            std::ptr::eq(old_request, new_request),
            "historical request was deep-copied"
        );
        assert!(
            std::ptr::eq(
                Engine::law(&old.state.model, "hold").unwrap(),
                Engine::law(&engine.state.model, "hold").unwrap()
            ),
            "unchanged law was deep-copied"
        );
        let snapshot = engine.snapshot().unwrap();
        assert_eq!(
            Engine::restore(&snapshot, &TestKinds)
                .unwrap()
                .snapshot()
                .unwrap(),
            snapshot
        );
    }
}
