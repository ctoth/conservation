//! Direct bindings: all state, validation and publication live in conservation-exchange.
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use conservation_core::Kind as _;
use conservation_exchange as core;
use num_rational::BigRational;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod kinds;

use kinds::{Declaration, DeclarationError, NativeKind, NativeRegistry, NativeTable};

create_exception!(_native, ExchangeError, PyValueError);
create_exception!(_native, InvalidExchange, ExchangeError);
create_exception!(_native, DimensionError, ExchangeError);
create_exception!(_native, DomainError, ExchangeError);
create_exception!(_native, CapacityError, ExchangeError);
create_exception!(_native, ConstraintError, ExchangeError);
create_exception!(_native, ParticipationError, ExchangeError);
create_exception!(_native, DuplicateExchangeError, ExchangeError);
create_exception!(_native, StalePreparationError, ExchangeError);
create_exception!(_native, ForeignPreparationError, ExchangeError);
create_exception!(_native, SnapshotError, ExchangeError);
create_exception!(_native, KindError, ExchangeError);
create_exception!(_native, UnknownKindError, KindError);
create_exception!(_native, KindDeclarationError, KindError);

fn failure(error: core::Error<NativeKind>) -> PyErr {
    let message = error.to_string();
    match error {
        core::Error::Invalid(_) => InvalidExchange::new_err(message),
        core::Error::Kinds { .. } => DimensionError::new_err(message),
        core::Error::Dimensions { .. } => DimensionError::new_err(message),
        core::Error::Algebra(_) => DimensionError::new_err(message),
        core::Error::BelowFloor { .. } => DomainError::new_err(message),
        core::Error::UnknownKind { .. } => UnknownKindError::new_err(message),
        core::Error::UnsupportedSnapshot { .. } => SnapshotError::new_err(message),
        core::Error::Capacity { .. } => CapacityError::new_err(message),
        core::Error::Constraint { .. } => ConstraintError::new_err(message),
        core::Error::Participant { .. } => ParticipationError::new_err(message),
        core::Error::Duplicate { .. } => DuplicateExchangeError::new_err(message),
        core::Error::Stale => StalePreparationError::new_err(message),
        core::Error::ForeignPreparation => ForeignPreparationError::new_err(message),
        core::Error::Snapshot(_) => SnapshotError::new_err(message),
    }
}

fn declaration_failure(error: DeclarationError) -> PyErr {
    KindDeclarationError::new_err(error.to_string())
}

fn rational(value: &str) -> PyResult<BigRational> {
    BigRational::from_str(value).map_err(|_| {
        InvalidExchange::new_err("expected an exact integer or numerator/denominator string")
    })
}

/// How one kind is declared: its dimensions, an optional floor and, for a point
/// kind, the kind of its differences.
#[pyclass(
    name = "KindDeclaration",
    module = "conservation_exchange._native",
    frozen
)]
struct KindDeclaration {
    dimensions: BTreeMap<String, i32>,
    floor: Option<String>,
    difference: Option<String>,
}
#[pymethods]
impl KindDeclaration {
    #[new]
    #[pyo3(signature = (dimensions=None, *, floor=None, difference=None))]
    fn new(
        dimensions: Option<BTreeMap<String, i32>>,
        floor: Option<String>,
        difference: Option<String>,
    ) -> Self {
        Self {
            dimensions: dimensions.unwrap_or_default(),
            floor,
            difference,
        }
    }
}

/// Resolves kind names. A registry is validated once and lives until the
/// process exits.
#[pyclass(
    name = "KindRegistry",
    module = "conservation_exchange._native",
    frozen
)]
struct KindRegistry {
    inner: NativeRegistry,
}
#[pymethods]
impl KindRegistry {
    #[new]
    fn new(py: Python<'_>, declarations: BTreeMap<String, Py<KindDeclaration>>) -> PyResult<Self> {
        let declarations = declarations
            .into_iter()
            .map(|(name, declaration)| {
                let declaration = declaration.borrow(py);
                (
                    name,
                    Declaration {
                        dimensions: declaration.dimensions.clone(),
                        floor: declaration.floor.clone(),
                        difference: declaration.difference.clone(),
                    },
                )
            })
            .collect();
        let table = NativeTable::leak(declarations, |value| rational(value).ok())
            .map_err(declaration_failure)?;
        Ok(Self {
            inner: NativeRegistry::new(table),
        })
    }
    fn kind(&self, name: &str) -> PyResult<Kind> {
        use conservation_core::KindRegistry as _;
        self.inner
            .resolve(name)
            .map(|inner| Kind { inner })
            .ok_or_else(|| UnknownKindError::new_err(format!("unknown kind {name:?}")))
    }
}

/// One kind of one registry.
#[pyclass(name = "Kind", module = "conservation_exchange._native", frozen)]
struct Kind {
    inner: NativeKind,
}
#[pymethods]
impl Kind {
    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    #[getter]
    fn dimensions(&self) -> BTreeMap<String, i32> {
        self.inner.powers()
    }
    #[getter]
    fn floor(&self) -> Option<String> {
        self.inner.floor().map(|floor| floor.to_string())
    }
    #[getter]
    fn difference(&self) -> Option<Kind> {
        match self.inner.affine() {
            conservation_core::Affine::Point { difference } => Some(Kind { inner: difference }),
            conservation_core::Affine::Linear => None,
        }
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

#[pyclass(name = "Quantity", module = "conservation_exchange._native", frozen)]
struct Quantity {
    inner: core::Quantity<NativeKind>,
}
#[pymethods]
impl Quantity {
    #[new]
    fn new(value: &str, kind: &Kind) -> PyResult<Self> {
        Ok(Self {
            inner: core::Quantity::new(rational(value)?, kind.inner),
        })
    }
    #[getter]
    fn fraction(&self) -> String {
        self.inner.amount.to_string()
    }
    #[getter]
    fn kind(&self) -> Kind {
        Kind {
            inner: self.inner.kind,
        }
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[pyclass(name = "Expr", module = "conservation_exchange._native", frozen)]
struct Expr {
    inner: core::Expr<NativeKind>,
}
#[pymethods]
impl Expr {
    #[staticmethod]
    fn constant(value: &Quantity) -> Self {
        Self {
            inner: core::Expr::constant(value.inner.clone()),
        }
    }
    #[staticmethod]
    fn before(slot: &str) -> Self {
        Self {
            inner: core::Expr::before(slot),
        }
    }
    #[staticmethod]
    fn after(slot: &str) -> Self {
        Self {
            inner: core::Expr::after(slot),
        }
    }
    #[staticmethod]
    fn delta(slot: &str) -> Self {
        Self {
            inner: core::Expr::delta(slot),
        }
    }
    #[staticmethod]
    fn boundary(port: &str) -> Self {
        Self {
            inner: core::Expr::boundary(port),
        }
    }
    #[staticmethod]
    fn fact(name: &str) -> Self {
        Self {
            inner: core::Expr::fact(name),
        }
    }
    #[staticmethod]
    fn sum(py: Python<'_>, terms: Vec<Py<Expr>>) -> Self {
        Self {
            inner: core::Expr::sum(terms.iter().map(|e| e.borrow(py).inner.clone())),
        }
    }
    fn product(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.clone().product(other.inner.clone()),
        }
    }
    fn quotient(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.clone().quotient(other.inner.clone()),
        }
    }
}

#[pyclass(name = "Constraint", module = "conservation_exchange._native", frozen)]
struct Constraint {
    inner: core::Constraint<NativeKind>,
}
#[pymethods]
impl Constraint {
    #[new]
    #[pyo3(signature = (id, left, right, *, less_or_equal=false, absolute="0", relative="0"))]
    fn new(
        id: &str,
        left: &Expr,
        right: &Expr,
        less_or_equal: bool,
        absolute: &str,
        relative: &str,
    ) -> PyResult<Self> {
        let mut inner = core::Constraint::equal(id, left.inner.clone(), right.inner.clone());
        if less_or_equal {
            inner.relation = core::Relation::LessOrEqual;
        }
        inner.absolute = rational(absolute)?;
        inner.relative = rational(relative)?;
        Ok(Self { inner })
    }
}

#[pyclass(name = "Fact", module = "conservation_exchange._native", frozen)]
struct Fact {
    inner: core::Fact<NativeKind>,
}
#[pymethods]
impl Fact {
    #[new]
    fn new(owner: &str, kind: &Kind) -> Self {
        Self {
            inner: core::Fact {
                owner: owner.into(),
                kind: kind.inner,
            },
        }
    }
}

#[pyclass(name = "Capacity", module = "conservation_exchange._native", frozen)]
struct Capacity {
    inner: core::Capacity<NativeKind>,
}
#[pymethods]
impl Capacity {
    #[new]
    fn new(maximum: &Quantity) -> Self {
        Self {
            inner: core::Capacity {
                maximum: maximum.inner.clone(),
            },
        }
    }
}

fn quantities(
    py: Python<'_>,
    values: Option<BTreeMap<String, Py<Quantity>>>,
) -> BTreeMap<String, core::Quantity<NativeKind>> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|(id, value)| (id, value.borrow(py).inner.clone()))
        .collect()
}

fn kinds(
    py: Python<'_>,
    values: Option<BTreeMap<String, Py<Kind>>>,
) -> BTreeMap<String, NativeKind> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|(id, kind)| (id, kind.borrow(py).inner))
        .collect()
}

#[pyclass(name = "Stock", module = "conservation_exchange._native", frozen)]
struct Stock {
    inner: core::Stock<NativeKind>,
}
#[pymethods]
impl Stock {
    #[new]
    #[pyo3(signature = (id, owner, kind, *, capacities=None))]
    fn new(
        py: Python<'_>,
        id: &str,
        owner: &str,
        kind: &Kind,
        capacities: Option<BTreeMap<String, Py<Quantity>>>,
    ) -> Self {
        Self {
            inner: core::Stock {
                id: id.into(),
                owner: owner.into(),
                kind: kind.inner,
                capacities: quantities(py, capacities),
            },
        }
    }
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }
    #[getter]
    fn owner(&self) -> &str {
        &self.inner.owner
    }
    #[getter]
    fn kind(&self) -> Kind {
        Kind {
            inner: self.inner.kind,
        }
    }
}

#[pyclass(name = "Placement", module = "conservation_exchange._native", frozen)]
struct Placement {
    inner: core::Placement<NativeKind>,
}
#[pymethods]
impl Placement {
    #[new]
    #[pyo3(signature = (owner, capacities=None))]
    fn new(
        py: Python<'_>,
        owner: &str,
        capacities: Option<BTreeMap<String, Py<Quantity>>>,
    ) -> Self {
        Self {
            inner: core::Placement {
                owner: owner.into(),
                capacities: quantities(py, capacities),
            },
        }
    }
}

#[pyclass(name = "Law", module = "conservation_exchange._native", frozen)]
struct Law {
    inner: core::Law<NativeKind>,
}
#[pymethods]
impl Law {
    #[new]
    #[pyo3(signature = (id, slots, constraints, *, boundaries=None, facts=None, participants=None))]
    fn new(
        py: Python<'_>,
        id: &str,
        slots: BTreeMap<String, Py<Kind>>,
        constraints: Vec<Py<Constraint>>,
        boundaries: Option<BTreeMap<String, Py<Kind>>>,
        facts: Option<BTreeMap<String, Py<Fact>>>,
        participants: Option<BTreeSet<String>>,
    ) -> Self {
        Self {
            inner: core::Law {
                id: id.into(),
                slots: kinds(py, Some(slots)),
                boundaries: kinds(py, boundaries),
                facts: facts
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(id, f)| (id, f.borrow(py).inner.clone()))
                    .collect(),
                constraints: constraints
                    .iter()
                    .map(|c| c.borrow(py).inner.clone())
                    .collect(),
                participants: participants.unwrap_or_default(),
            },
        }
    }
}

#[pyclass(name = "Model", module = "conservation_exchange._native", frozen)]
struct Model {
    inner: core::Model<NativeKind>,
}
#[pymethods]
impl Model {
    #[new]
    #[pyo3(signature = (owners, laws, capacities=None))]
    fn new(
        py: Python<'_>,
        owners: BTreeSet<String>,
        laws: Vec<Py<Law>>,
        capacities: Option<BTreeMap<String, Py<Capacity>>>,
    ) -> Self {
        Self {
            inner: core::Model {
                owners,
                laws: laws
                    .iter()
                    .map(|law| law.borrow(py).inner.clone())
                    .collect(),
                capacities: capacities
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(id, capacity)| (id, capacity.borrow(py).inner.clone()))
                    .collect(),
            },
        }
    }
}

#[pyclass(name = "Exchange", module = "conservation_exchange._native", frozen)]
struct Exchange {
    inner: core::Exchange<NativeKind>,
}
#[pymethods]
impl Exchange {
    #[new]
    #[pyo3(signature = (id, law, bindings, *, deltas=None, boundaries=None, creates=None, removes=None, moves=None, declarations=None))]
    #[allow(clippy::too_many_arguments)] // Mirrors the one native request, with no second schema.
    fn new(
        py: Python<'_>,
        id: &str,
        law: &str,
        bindings: BTreeMap<String, String>,
        deltas: Option<BTreeMap<String, Py<Quantity>>>,
        boundaries: Option<BTreeMap<String, Py<Quantity>>>,
        creates: Option<Vec<Py<Stock>>>,
        removes: Option<BTreeSet<String>>,
        moves: Option<BTreeMap<String, Py<Placement>>>,
        declarations: Option<&Model>,
    ) -> Self {
        Self {
            inner: core::Exchange {
                id: id.into(),
                law: law.into(),
                bindings,
                deltas: quantities(py, deltas),
                boundaries: quantities(py, boundaries),
                creates: creates
                    .unwrap_or_default()
                    .iter()
                    .map(|s| s.borrow(py).inner.clone())
                    .collect(),
                removes: removes.unwrap_or_default(),
                moves: moves
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(id, p)| (id, p.borrow(py).inner.clone()))
                    .collect(),
                declarations: declarations.map(|model| model.inner.clone()),
            },
        }
    }
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }
}

#[pyclass(name = "RecordWrite", module = "conservation_exchange._native", frozen)]
struct RecordWrite {
    inner: core::RecordWrite,
}
#[pymethods]
impl RecordWrite {
    #[new]
    fn new(key: &str, value: Option<&Bound<'_, PyBytes>>) -> Self {
        Self {
            inner: core::RecordWrite {
                key: key.into(),
                value: value.map(|b| b.as_bytes().to_vec()),
            },
        }
    }
}

#[pyclass(
    name = "Participation",
    module = "conservation_exchange._native",
    frozen
)]
struct Participation {
    inner: core::Participation<NativeKind>,
}
#[pyclass(name = "Prepared", module = "conservation_exchange._native", frozen)]
struct Prepared {
    inner: core::Prepared<NativeKind>,
}
#[pyclass(name = "Receipt", module = "conservation_exchange._native", frozen)]
struct Receipt {
    inner: core::Receipt<NativeKind>,
}
#[pymethods]
impl Receipt {
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }
    #[getter]
    fn revision(&self) -> u64 {
        self.inner.revision
    }
    #[getter]
    fn law(&self) -> &str {
        &self.inner.law
    }
    #[getter]
    fn boundaries(&self) -> BTreeMap<String, Quantity> {
        self.inner
            .boundaries
            .iter()
            .map(|(id, q)| (id.clone(), Quantity { inner: q.clone() }))
            .collect()
    }
    #[getter]
    fn changes(&self) -> BTreeMap<String, (Quantity, Quantity)> {
        self.inner
            .changes
            .iter()
            .map(|(id, (a, b))| {
                (
                    id.clone(),
                    (Quantity { inner: a.clone() }, Quantity { inner: b.clone() }),
                )
            })
            .collect()
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[pyclass(name = "Engine", module = "conservation_exchange._native")]
struct Engine {
    inner: core::Engine<NativeKind>,
}
#[pymethods]
impl Engine {
    #[new]
    #[pyo3(signature = (owners, laws, capacities=None))]
    fn new(
        py: Python<'_>,
        owners: BTreeSet<String>,
        laws: Vec<Py<Law>>,
        capacities: Option<BTreeMap<String, Py<Capacity>>>,
    ) -> PyResult<Self> {
        let model = core::Model {
            owners,
            laws: laws.iter().map(|l| l.borrow(py).inner.clone()).collect(),
            capacities: capacities
                .unwrap_or_default()
                .into_iter()
                .map(|(id, c)| (id, c.borrow(py).inner.clone()))
                .collect(),
        };
        Ok(Self {
            inner: core::Engine::new(model).map_err(failure)?,
        })
    }
    #[getter]
    fn revision(&self) -> u64 {
        self.inner.revision()
    }
    fn amount(&self, stock: &str) -> PyResult<Quantity> {
        Ok(Quantity {
            inner: core::Quantity::new(
                self.inner.amount(stock).map_err(failure)?.clone(),
                self.inner.stock(stock).map_err(failure)?.kind,
            ),
        })
    }
    fn stock(&self, stock: &str) -> PyResult<Stock> {
        Ok(Stock {
            inner: self.inner.stock(stock).map_err(failure)?.clone(),
        })
    }
    fn record(&self, py: Python<'_>, owner: &str, key: &str) -> Option<Py<PyBytes>> {
        self.inner
            .record(owner, key)
            .map(|data| PyBytes::new(py, data).unbind())
    }
    fn receipt(&self, id: &str) -> Option<Receipt> {
        self.inner
            .receipt(id)
            .cloned()
            .map(|inner| Receipt { inner })
    }
    #[pyo3(signature = (owner, exchange, writes=None, facts=None))]
    fn participate(
        &self,
        py: Python<'_>,
        owner: &str,
        exchange: &Exchange,
        writes: Option<Vec<Py<RecordWrite>>>,
        facts: Option<BTreeMap<String, Py<Quantity>>>,
    ) -> PyResult<Participation> {
        Ok(Participation {
            inner: self
                .inner
                .participate(
                    owner,
                    &exchange.inner,
                    writes
                        .unwrap_or_default()
                        .iter()
                        .map(|w| w.borrow(py).inner.clone())
                        .collect(),
                    quantities(py, facts),
                )
                .map_err(failure)?,
        })
    }
    #[pyo3(signature = (exchange, participants=None))]
    fn prepare(
        &self,
        py: Python<'_>,
        exchange: &Exchange,
        participants: Option<Vec<Py<Participation>>>,
    ) -> PyResult<Prepared> {
        Ok(Prepared {
            inner: self
                .inner
                .prepare(
                    exchange.inner.clone(),
                    participants
                        .unwrap_or_default()
                        .iter()
                        .map(|p| p.borrow(py).inner.clone())
                        .collect(),
                )
                .map_err(failure)?,
        })
    }
    fn publish(&mut self, prepared: &Prepared) -> PyResult<Receipt> {
        Ok(Receipt {
            inner: self
                .inner
                .publish(prepared.inner.clone())
                .map_err(failure)?,
        })
    }
    fn snapshot(&self, py: Python<'_>) -> PyResult<Py<PyBytes>> {
        Ok(PyBytes::new(py, &self.inner.snapshot().map_err(failure)?).unbind())
    }
    #[staticmethod]
    fn restore(bytes: &Bound<'_, PyBytes>, registry: &KindRegistry) -> PyResult<Self> {
        Ok(Self {
            inner: core::Engine::restore(bytes.as_bytes(), &registry.inner).map_err(failure)?,
        })
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<KindDeclaration>()?;
    module.add_class::<KindRegistry>()?;
    module.add_class::<Kind>()?;
    module.add_class::<Quantity>()?;
    module.add_class::<Expr>()?;
    module.add_class::<Constraint>()?;
    module.add_class::<Capacity>()?;
    module.add_class::<Fact>()?;
    module.add_class::<Stock>()?;
    module.add_class::<Placement>()?;
    module.add_class::<Law>()?;
    module.add_class::<Exchange>()?;
    module.add_class::<Model>()?;
    module.add_class::<RecordWrite>()?;
    module.add_class::<Participation>()?;
    module.add_class::<Prepared>()?;
    module.add_class::<Receipt>()?;
    module.add_class::<Engine>()?;
    let py = module.py();
    module.add("ExchangeError", py.get_type::<ExchangeError>())?;
    module.add("InvalidExchange", py.get_type::<InvalidExchange>())?;
    module.add("DimensionError", py.get_type::<DimensionError>())?;
    module.add("DomainError", py.get_type::<DomainError>())?;
    module.add("CapacityError", py.get_type::<CapacityError>())?;
    module.add("ConstraintError", py.get_type::<ConstraintError>())?;
    module.add("ParticipationError", py.get_type::<ParticipationError>())?;
    module.add(
        "DuplicateExchangeError",
        py.get_type::<DuplicateExchangeError>(),
    )?;
    module.add(
        "StalePreparationError",
        py.get_type::<StalePreparationError>(),
    )?;
    module.add(
        "ForeignPreparationError",
        py.get_type::<ForeignPreparationError>(),
    )?;
    module.add("SnapshotError", py.get_type::<SnapshotError>())?;
    module.add("KindError", py.get_type::<KindError>())?;
    module.add("UnknownKindError", py.get_type::<UnknownKindError>())?;
    module.add(
        "KindDeclarationError",
        py.get_type::<KindDeclarationError>(),
    )?;
    Ok(())
}
