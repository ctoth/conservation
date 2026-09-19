//! Direct bindings: all state, validation and publication live in conservation-exchange.
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use conservation_exchange as core;
use num_rational::BigRational;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

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

fn failure(error: core::Error) -> PyErr {
    let message = error.to_string();
    match error {
        core::Error::Invalid(_) => InvalidExchange::new_err(message),
        core::Error::Dimension { .. } => DimensionError::new_err(message),
        core::Error::Domain { .. } => DomainError::new_err(message),
        core::Error::Capacity { .. } => CapacityError::new_err(message),
        core::Error::Constraint { .. } => ConstraintError::new_err(message),
        core::Error::Participant { .. } => ParticipationError::new_err(message),
        core::Error::Duplicate { .. } => DuplicateExchangeError::new_err(message),
        core::Error::Stale => StalePreparationError::new_err(message),
        core::Error::ForeignPreparation => ForeignPreparationError::new_err(message),
        core::Error::Snapshot(_) => SnapshotError::new_err(message),
    }
}

fn rational(value: &str) -> PyResult<BigRational> {
    BigRational::from_str(value).map_err(|_| {
        InvalidExchange::new_err("expected an exact integer or numerator/denominator string")
    })
}

#[pyclass(name = "Dimension", module = "conservation_exchange._native", frozen)]
struct Dimension {
    inner: core::Dimension,
}
#[pymethods]
impl Dimension {
    #[new]
    #[pyo3(signature = (name=None))]
    fn new(name: Option<&str>) -> PyResult<Self> {
        Ok(Self {
            inner: match name {
                Some(name) => core::Dimension::base(name).map_err(failure)?,
                None => core::Dimension::dimensionless(),
            },
        })
    }
    fn product(&self, other: &Self) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.product(&other.inner).map_err(failure)?,
        })
    }
    fn quotient(&self, other: &Self) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.quotient(&other.inner).map_err(failure)?,
        })
    }
    #[getter]
    fn powers(&self) -> BTreeMap<String, i32> {
        self.inner.powers().clone()
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[pyclass(name = "Quantity", module = "conservation_exchange._native", frozen)]
struct Quantity {
    inner: core::Quantity,
}
#[pymethods]
impl Quantity {
    #[new]
    #[pyo3(signature = (value, dimension=None))]
    fn new(value: &str, dimension: Option<&Dimension>) -> PyResult<Self> {
        Ok(Self {
            inner: core::Quantity::new(
                rational(value)?,
                dimension.map(|d| d.inner.clone()).unwrap_or_default(),
            ),
        })
    }
    #[getter]
    fn fraction(&self) -> String {
        self.inner.amount.to_string()
    }
    #[getter]
    fn dimension(&self) -> Dimension {
        Dimension {
            inner: self.inner.dimension.clone(),
        }
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[pyclass(name = "Expr", module = "conservation_exchange._native", frozen)]
struct Expr {
    inner: core::Expr,
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
    inner: core::Constraint,
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
    inner: core::Fact,
}
#[pymethods]
impl Fact {
    #[new]
    fn new(owner: &str, dimension: &Dimension) -> Self {
        Self {
            inner: core::Fact {
                owner: owner.into(),
                dimension: dimension.inner.clone(),
            },
        }
    }
}

#[pyclass(name = "Capacity", module = "conservation_exchange._native", frozen)]
struct Capacity {
    inner: core::Capacity,
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
) -> BTreeMap<String, core::Quantity> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|(id, value)| (id, value.borrow(py).inner.clone()))
        .collect()
}

#[pyclass(name = "Stock", module = "conservation_exchange._native", frozen)]
struct Stock {
    inner: core::Stock,
}
#[pymethods]
impl Stock {
    #[new]
    #[pyo3(signature = (id, owner, dimension, *, signed=false, capacities=None))]
    fn new(
        py: Python<'_>,
        id: &str,
        owner: &str,
        dimension: &Dimension,
        signed: bool,
        capacities: Option<BTreeMap<String, Py<Quantity>>>,
    ) -> Self {
        Self {
            inner: core::Stock {
                id: id.into(),
                owner: owner.into(),
                dimension: dimension.inner.clone(),
                domain: if signed {
                    core::Domain::Signed
                } else {
                    core::Domain::Nonnegative
                },
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
    fn dimension(&self) -> Dimension {
        Dimension {
            inner: self.inner.dimension.clone(),
        }
    }
    #[getter]
    fn signed(&self) -> bool {
        self.inner.domain == core::Domain::Signed
    }
}

#[pyclass(name = "Placement", module = "conservation_exchange._native", frozen)]
struct Placement {
    inner: core::Placement,
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
    inner: core::Law,
}
#[pymethods]
impl Law {
    #[new]
    #[pyo3(signature = (id, slots, constraints, *, boundaries=None, facts=None, participants=None))]
    fn new(
        py: Python<'_>,
        id: &str,
        slots: BTreeMap<String, Py<Dimension>>,
        constraints: Vec<Py<Constraint>>,
        boundaries: Option<BTreeMap<String, Py<Dimension>>>,
        facts: Option<BTreeMap<String, Py<Fact>>>,
        participants: Option<BTreeSet<String>>,
    ) -> Self {
        Self {
            inner: core::Law {
                id: id.into(),
                slots: slots
                    .into_iter()
                    .map(|(id, d)| (id, d.borrow(py).inner.clone()))
                    .collect(),
                boundaries: boundaries
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(id, d)| (id, d.borrow(py).inner.clone()))
                    .collect(),
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

#[pyclass(name = "Exchange", module = "conservation_exchange._native", frozen)]
struct Exchange {
    inner: core::Exchange,
}
#[pymethods]
impl Exchange {
    #[new]
    #[pyo3(signature = (id, law, bindings, *, deltas=None, boundaries=None, creates=None, removes=None, moves=None))]
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
    inner: core::Participation,
}
#[pyclass(name = "Prepared", module = "conservation_exchange._native", frozen)]
struct Prepared {
    inner: core::Prepared,
}
#[pyclass(name = "Receipt", module = "conservation_exchange._native", frozen)]
struct Receipt {
    inner: core::Receipt,
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
    inner: core::Engine,
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
                self.inner.stock(stock).map_err(failure)?.dimension.clone(),
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
    fn restore(bytes: &Bound<'_, PyBytes>) -> PyResult<Self> {
        Ok(Self {
            inner: core::Engine::restore(bytes.as_bytes()).map_err(failure)?,
        })
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<Dimension>()?;
    module.add_class::<Quantity>()?;
    module.add_class::<Expr>()?;
    module.add_class::<Constraint>()?;
    module.add_class::<Capacity>()?;
    module.add_class::<Fact>()?;
    module.add_class::<Stock>()?;
    module.add_class::<Placement>()?;
    module.add_class::<Law>()?;
    module.add_class::<Exchange>()?;
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
    Ok(())
}
