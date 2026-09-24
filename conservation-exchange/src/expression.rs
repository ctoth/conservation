use std::collections::{BTreeMap, BTreeSet};

use conservation_core::DimensionAlgebra;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::model::{DimensionContext, KindContext, invalid, rational};
use crate::{Error, Law, Quantity};

/// Closed, kind-checked expression language. Unsupported forms fail decoding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Expr<K> {
    Constant(Quantity<K>),
    Before(String),
    After(String),
    Delta(String),
    Boundary(String),
    Fact(String),
    Sum(Vec<Expr<K>>),
    Product(Box<Expr<K>>, Box<Expr<K>>),
    Quotient(Box<Expr<K>>, Box<Expr<K>>),
}

/// The type of an exchange expression: a declared kind, or dimensions computed by
/// a product or quotient (which has no kind of its own).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExprType<K: DimensionAlgebra> {
    Kind(K),
    Derived(K::Dimensions),
}

impl<K: DimensionAlgebra> ExprType<K> {
    fn dimensions(&self) -> K::Dimensions {
        match self {
            Self::Kind(kind) => kind.dimensions(),
            Self::Derived(dimensions) => dimensions.clone(),
        }
    }

    /// Two kinds compare by identity. A derived side has no kind, so any
    /// comparison with one compares dimensions.
    fn require_equal(
        &self,
        other: &Self,
        kind_context: impl FnOnce() -> KindContext,
        dimension_context: impl FnOnce() -> DimensionContext,
    ) -> Result<(), Error<K>> {
        match (self, other) {
            (Self::Kind(expected), Self::Kind(found)) => {
                if expected != found {
                    return Err(Error::Kinds {
                        context: kind_context(),
                        expected: *expected,
                        found: *found,
                    });
                }
            }
            _ => {
                let (left, right) = (self.dimensions(), other.dimensions());
                if left != right {
                    return Err(Error::Dimensions {
                        context: dimension_context(),
                        left,
                        right,
                    });
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct Evaluation<'a, K> {
    pub before: &'a BTreeMap<String, Quantity<K>>,
    pub after: &'a BTreeMap<String, Quantity<K>>,
    pub deltas: &'a BTreeMap<String, Quantity<K>>,
    pub boundaries: &'a BTreeMap<String, Quantity<K>>,
    pub facts: &'a BTreeMap<String, Quantity<K>>,
}

impl<K: DimensionAlgebra> Expr<K> {
    pub fn constant(value: Quantity<K>) -> Self {
        Self::Constant(value)
    }
    pub fn before(slot: impl Into<String>) -> Self {
        Self::Before(slot.into())
    }
    pub fn after(slot: impl Into<String>) -> Self {
        Self::After(slot.into())
    }
    pub fn delta(slot: impl Into<String>) -> Self {
        Self::Delta(slot.into())
    }
    pub fn boundary(port: impl Into<String>) -> Self {
        Self::Boundary(port.into())
    }
    pub fn fact(name: impl Into<String>) -> Self {
        Self::Fact(name.into())
    }
    pub fn sum(terms: impl IntoIterator<Item = Self>) -> Self {
        Self::Sum(terms.into_iter().collect())
    }
    pub fn product(self, other: Self) -> Self {
        Self::Product(Box::new(self), Box::new(other))
    }
    pub fn quotient(self, other: Self) -> Self {
        Self::Quotient(Box::new(self), Box::new(other))
    }

    /// Types the expression: leaves have declared kinds (a delta has its slot
    /// kind's difference), and products and quotients derive dimensions.
    fn dimensions(&self, law: &Law<K>, depth: usize) -> Result<ExprType<K>, Error<K>> {
        if depth > 64 {
            return Err(invalid("constraint expression nesting exceeds 64"));
        }
        let slot = |id: &String| {
            law.slots
                .get(id)
                .copied()
                .ok_or_else(|| invalid(format!("undeclared slot {id}")))
        };
        match self {
            Self::Constant(q) => {
                q.validate()?;
                Ok(ExprType::Kind(q.kind))
            }
            Self::Before(id) | Self::After(id) => slot(id).map(ExprType::Kind),
            Self::Delta(id) => slot(id).map(|kind| ExprType::Kind(kind.difference())),
            Self::Boundary(id) => law
                .boundaries
                .get(id)
                .copied()
                .map(ExprType::Kind)
                .ok_or_else(|| invalid(format!("undeclared boundary {id}"))),
            Self::Fact(id) => law
                .facts
                .get(id)
                .map(|fact| ExprType::Kind(fact.kind))
                .ok_or_else(|| invalid(format!("undeclared fact {id}"))),
            Self::Sum(terms) => {
                let first = terms
                    .first()
                    .ok_or_else(|| invalid("empty expression sum"))?
                    .dimensions(law, depth + 1)?;
                let mut every_kind = matches!(first, ExprType::Kind(_));
                for term in &terms[1..] {
                    let term = term.dimensions(law, depth + 1)?;
                    first.require_equal(&term, || KindContext::Sum, || DimensionContext::Sum)?;
                    every_kind &= matches!(term, ExprType::Kind(_));
                }
                Ok(if every_kind {
                    first
                } else {
                    ExprType::Derived(first.dimensions())
                })
            }
            Self::Product(a, b) => K::product(
                &a.dimensions(law, depth + 1)?.dimensions(),
                &b.dimensions(law, depth + 1)?.dimensions(),
            )
            .map(ExprType::Derived)
            .map_err(Error::Algebra),
            Self::Quotient(a, b) => K::quotient(
                &a.dimensions(law, depth + 1)?.dimensions(),
                &b.dimensions(law, depth + 1)?.dimensions(),
            )
            .map(ExprType::Derived)
            .map_err(Error::Algebra),
        }
    }

    fn evaluate(&self, input: &Evaluation<'_, K>) -> Result<BigRational, Error<K>> {
        let get = |values: &BTreeMap<String, Quantity<K>>, id: &str| {
            values
                .get(id)
                .map(|q| q.amount.clone())
                .ok_or_else(|| invalid(format!("missing evaluation input {id}")))
        };
        match self {
            Self::Constant(q) => Ok(q.amount.clone()),
            Self::Before(id) => get(input.before, id),
            Self::After(id) => get(input.after, id),
            Self::Delta(id) => get(input.deltas, id),
            Self::Boundary(id) => get(input.boundaries, id),
            Self::Fact(id) => get(input.facts, id),
            Self::Sum(terms) => terms.iter().try_fold(BigRational::zero(), |sum, term| {
                Ok(sum + term.evaluate(input)?)
            }),
            Self::Product(a, b) => Ok(a.evaluate(input)? * b.evaluate(input)?),
            Self::Quotient(a, b) => {
                let denominator = b.evaluate(input)?;
                if denominator.is_zero() {
                    return Err(invalid("division by zero in constraint"));
                }
                Ok(a.evaluate(input)? / denominator)
            }
        }
    }

    fn coverage(&self, result: &mut BTreeSet<(&'static str, String)>) {
        match self {
            Self::After(id) | Self::Delta(id) => {
                result.insert(("slot", id.clone()));
            }
            Self::Boundary(id) => {
                result.insert(("boundary", id.clone()));
            }
            Self::Sum(terms) => {
                for term in terms {
                    term.coverage(result);
                }
            }
            Self::Product(a, b) | Self::Quotient(a, b) => {
                a.coverage(result);
                b.coverage(result);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Relation {
    Equal,
    LessOrEqual,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint<K> {
    pub id: String,
    pub left: Expr<K>,
    pub right: Expr<K>,
    pub relation: Relation,
    /// Absolute tolerance in the dimension of the expression; exact by default.
    pub absolute: BigRational,
    pub relative: BigRational,
}

impl<K: DimensionAlgebra> Constraint<K> {
    pub fn equal(id: impl Into<String>, left: Expr<K>, right: Expr<K>) -> Self {
        Self {
            id: id.into(),
            left,
            right,
            relation: Relation::Equal,
            absolute: BigRational::zero(),
            relative: BigRational::zero(),
        }
    }

    pub(crate) fn validate(&self, law: &Law<K>) -> Result<(), Error<K>> {
        let left = self.left.dimensions(law, 0)?;
        let right = self.right.dimensions(law, 0)?;
        left.require_equal(
            &right,
            || KindContext::Constraint {
                id: self.id.clone(),
            },
            || DimensionContext::Constraint {
                id: self.id.clone(),
            },
        )?;
        rational(&self.absolute)?;
        rational(&self.relative)?;
        if self.absolute.is_negative()
            || self.relative.is_negative()
            || self.relative >= BigRational::one()
        {
            return Err(invalid(
                "tolerance requires absolute >= 0 and 0 <= relative < 1",
            ));
        }
        Ok(())
    }

    pub(crate) fn coverage(&self, result: &mut BTreeSet<(&'static str, String)>) {
        if self.relation == Relation::Equal {
            self.left.coverage(result);
            self.right.coverage(result);
        }
    }

    pub(crate) fn check(&self, exchange: &str, input: &Evaluation<'_, K>) -> Result<(), Error<K>> {
        let left = self.left.evaluate(input)?;
        let right = self.right.evaluate(input)?;
        let tolerance = &self.absolute + &self.relative * left.abs().max(right.abs());
        let residual = left - right;
        let exceeds = match self.relation {
            Relation::Equal => residual.abs() > tolerance,
            Relation::LessOrEqual => residual > tolerance,
        };
        if exceeds {
            return Err(Error::Constraint {
                exchange: exchange.into(),
                id: self.id.clone(),
                residual,
            });
        }
        Ok(())
    }
}
