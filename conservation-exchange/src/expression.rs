use std::collections::{BTreeMap, BTreeSet};

use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::model::{invalid, rational};
use crate::{Dimension, Error, Law, Quantity};

/// Closed, dimension-checked expression language. Unsupported forms fail decoding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Constant(Quantity),
    Before(String),
    After(String),
    Delta(String),
    Boundary(String),
    Fact(String),
    Sum(Vec<Expr>),
    Product(Box<Expr>, Box<Expr>),
    Quotient(Box<Expr>, Box<Expr>),
}

pub(crate) struct Evaluation<'a> {
    pub before: &'a BTreeMap<String, Quantity>,
    pub after: &'a BTreeMap<String, Quantity>,
    pub deltas: &'a BTreeMap<String, Quantity>,
    pub boundaries: &'a BTreeMap<String, Quantity>,
    pub facts: &'a BTreeMap<String, Quantity>,
}

impl Expr {
    pub fn constant(value: Quantity) -> Self {
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

    fn dimension(&self, law: &Law, depth: usize) -> Result<Dimension, Error> {
        if depth > 64 {
            return Err(invalid("constraint expression nesting exceeds 64"));
        }
        match self {
            Self::Constant(q) => {
                q.validate()?;
                Ok(q.dimension.clone())
            }
            Self::Before(id) | Self::After(id) | Self::Delta(id) => law
                .slots
                .get(id)
                .cloned()
                .ok_or_else(|| invalid(format!("undeclared slot {id}"))),
            Self::Boundary(id) => law
                .boundaries
                .get(id)
                .cloned()
                .ok_or_else(|| invalid(format!("undeclared boundary {id}"))),
            Self::Fact(id) => law
                .facts
                .get(id)
                .map(|fact| fact.dimension.clone())
                .ok_or_else(|| invalid(format!("undeclared fact {id}"))),
            Self::Sum(terms) => {
                let first = terms
                    .first()
                    .ok_or_else(|| invalid("empty expression sum"))?
                    .dimension(law, depth + 1)?;
                for term in &terms[1..] {
                    if term.dimension(law, depth + 1)? != first {
                        return Err(Error::Dimension {
                            context: "expression sum".into(),
                        });
                    }
                }
                Ok(first)
            }
            Self::Product(a, b) => a
                .dimension(law, depth + 1)?
                .product(&b.dimension(law, depth + 1)?),
            Self::Quotient(a, b) => a
                .dimension(law, depth + 1)?
                .quotient(&b.dimension(law, depth + 1)?),
        }
    }

    fn evaluate(&self, input: &Evaluation<'_>) -> Result<BigRational, Error> {
        let get = |values: &BTreeMap<String, Quantity>, id: &str| {
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
pub struct Constraint {
    pub id: String,
    pub left: Expr,
    pub right: Expr,
    pub relation: Relation,
    /// Absolute tolerance in the dimension of the expression; exact by default.
    pub absolute: BigRational,
    pub relative: BigRational,
}

impl Constraint {
    pub fn equal(id: impl Into<String>, left: Expr, right: Expr) -> Self {
        Self {
            id: id.into(),
            left,
            right,
            relation: Relation::Equal,
            absolute: BigRational::zero(),
            relative: BigRational::zero(),
        }
    }

    pub(crate) fn validate(&self, law: &Law) -> Result<(), Error> {
        if self.left.dimension(law, 0)? != self.right.dimension(law, 0)? {
            return Err(Error::Dimension {
                context: self.id.clone(),
            });
        }
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

    pub(crate) fn check(&self, exchange: &str, input: &Evaluation<'_>) -> Result<(), Error> {
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
