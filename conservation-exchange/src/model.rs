use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use conservation_core::nonblank;
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::Constraint;

/// Dimensional exponents. Names identify base dimensions, not display units.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dimension(BTreeMap<String, i32>);

impl Dimension {
    pub fn base(name: impl Into<String>) -> Result<Self, Error> {
        let name = name.into();
        identifier(&name)?;
        Ok(Self(BTreeMap::from([(name, 1)])))
    }

    pub fn dimensionless() -> Self {
        Self::default()
    }

    pub fn powers(&self) -> &BTreeMap<String, i32> {
        &self.0
    }

    pub fn product(&self, other: &Self) -> Result<Self, Error> {
        self.combine(other, 1)
    }

    pub fn quotient(&self, other: &Self) -> Result<Self, Error> {
        self.combine(other, -1)
    }

    fn combine(&self, other: &Self, sign: i32) -> Result<Self, Error> {
        self.validate()?;
        other.validate()?;
        let mut result = self.0.clone();
        for (key, power) in &other.0 {
            let delta = power
                .checked_mul(sign)
                .ok_or_else(|| invalid("dimension exponent overflow"))?;
            let sum = result
                .get(key)
                .copied()
                .unwrap_or_default()
                .checked_add(delta)
                .ok_or_else(|| invalid("dimension exponent overflow"))?;
            if sum == 0 {
                result.remove(key);
            } else {
                result.insert(key.clone(), sum);
            }
        }
        Ok(Self(result))
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        for (name, power) in &self.0 {
            identifier(name)?;
            if *power == 0 {
                return Err(invalid("zero dimension exponents are not canonical"));
            }
        }
        Ok(())
    }
}

/// An exact rational in canonical base units. Binary floats are never implicit inputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quantity {
    pub amount: BigRational,
    pub dimension: Dimension,
}

impl Quantity {
    pub fn new(amount: BigRational, dimension: Dimension) -> Self {
        Self { amount, dimension }
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        self.dimension.validate()?;
        rational(&self.amount)
    }

    pub(crate) fn same_dimension(&self, dimension: &Dimension, context: &str) -> Result<(), Error> {
        self.validate()?;
        if &self.dimension != dimension {
            return Err(Error::Dimension {
                context: context.into(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Domain {
    Nonnegative,
    Signed,
}

/// A stock's physical placement. Capacity weights map stock units to constraint units.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub owner: String,
    pub capacities: BTreeMap<String, Quantity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stock {
    pub id: String,
    pub owner: String,
    pub dimension: Dimension,
    pub domain: Domain,
    pub capacities: BTreeMap<String, Quantity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capacity {
    pub maximum: Quantity,
}

/// A named domain evaluation. The declared owner supplies it for this proposal/revision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub owner: String,
    pub dimension: Dimension,
}

/// Registered transition law over named stock roles, signed boundary inputs, and facts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Law {
    pub id: String,
    pub slots: BTreeMap<String, Dimension>,
    pub boundaries: BTreeMap<String, Dimension>,
    pub facts: BTreeMap<String, Fact>,
    pub participants: BTreeSet<String>,
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub owners: BTreeSet<String>,
    pub capacities: BTreeMap<String, Capacity>,
    pub laws: Vec<Law>,
}

impl Model {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.owners.is_empty() {
            return Err(invalid("model requires at least one owner"));
        }
        for owner in &self.owners {
            identifier(owner)?;
        }
        for (id, capacity) in &self.capacities {
            identifier(id)?;
            capacity.maximum.validate()?;
            if capacity.maximum.amount.is_negative() {
                return Err(invalid("negative capacity"));
            }
        }
        let mut laws = BTreeSet::new();
        for law in &self.laws {
            identifier(&law.id)?;
            if !laws.insert(&law.id) {
                return Err(invalid("duplicate law"));
            }
            validate_law(law, |id| self.owner(id))?;
        }
        Ok(())
    }

    pub(crate) fn owner(&self, owner: &str) -> Result<(), Error> {
        if !self.owners.contains(owner) {
            return Err(invalid(format!("undeclared owner {owner}")));
        }
        Ok(())
    }
}

/// Validated internal definitions. Payloads survive immutable root transitions.
#[derive(Clone, Debug, Default)]
pub(crate) struct ValidatedModel {
    pub(crate) owners: BTreeSet<String>,
    pub(crate) capacities: BTreeMap<String, Arc<Capacity>>,
    pub(crate) laws: BTreeMap<String, Arc<Law>>,
}

impl ValidatedModel {
    pub(crate) fn new(model: &Model) -> Result<Self, Error> {
        model.validate()?;
        Ok(Self {
            owners: model.owners.clone(),
            capacities: model
                .capacities
                .iter()
                .map(|(id, value)| (id.clone(), Arc::new(value.clone())))
                .collect(),
            laws: model
                .laws
                .iter()
                .map(|law| (law.id.clone(), Arc::new(law.clone())))
                .collect(),
        })
    }

    pub(crate) fn extended(&self, declarations: &Model) -> Result<Self, Error> {
        let mut next = self.clone();
        for owner in &declarations.owners {
            identifier(owner)?;
            next.owners.insert(owner.clone());
        }
        for (id, capacity) in &declarations.capacities {
            if let Some(existing) = next.capacities.get(id) {
                if existing.as_ref() != capacity {
                    return Err(invalid(format!("cannot replace capacity {id}")));
                }
            } else {
                identifier(id)?;
                capacity.maximum.validate()?;
                if capacity.maximum.amount.is_negative() {
                    return Err(invalid("negative capacity"));
                }
                next.capacities
                    .insert(id.clone(), Arc::new(capacity.clone()));
            }
        }
        let mut seen = BTreeSet::new();
        for law in &declarations.laws {
            if !seen.insert(&law.id) {
                return Err(invalid("duplicate law declaration"));
            }
            if let Some(existing) = next.laws.get(&law.id) {
                if existing.as_ref() != law {
                    return Err(invalid(format!("cannot replace law {}", law.id)));
                }
            } else {
                identifier(&law.id)?;
                validate_law(law, |owner| next.owner(owner))?;
                next.laws.insert(law.id.clone(), Arc::new(law.clone()));
            }
        }
        Ok(next)
    }

    pub(crate) fn owner(&self, owner: &str) -> Result<(), Error> {
        if !self.owners.contains(owner) {
            return Err(invalid(format!("undeclared owner {owner}")));
        }
        Ok(())
    }

    pub(crate) fn stock(&self, stock: &Stock) -> Result<(), Error> {
        identifier(&stock.id)?;
        self.owner(&stock.owner)?;
        stock.dimension.validate()?;
        for (id, weight) in &stock.capacities {
            let capacity = self
                .capacities
                .get(id)
                .ok_or_else(|| invalid(format!("undeclared capacity {id}")))?;
            weight.validate()?;
            if weight.amount.is_negative() || stock.domain == Domain::Signed {
                return Err(invalid(
                    "capacity weights require nonnegative stocks and weights",
                ));
            }
            let dimension = weight.dimension.product(&stock.dimension)?;
            if dimension != capacity.maximum.dimension {
                return Err(Error::Dimension {
                    context: format!("capacity {id}"),
                });
            }
        }
        Ok(())
    }
}

fn validate_law(law: &Law, check_owner: impl Fn(&str) -> Result<(), Error>) -> Result<(), Error> {
    for (id, dimension) in law.slots.iter().chain(&law.boundaries) {
        identifier(id)?;
        dimension.validate()?;
    }
    for (id, fact) in &law.facts {
        identifier(id)?;
        fact.dimension.validate()?;
        check_owner(&fact.owner)?;
        if !law.participants.contains(&fact.owner) {
            return Err(invalid("fact owner must be a required participant"));
        }
    }
    for owner in &law.participants {
        check_owner(owner)?;
    }
    let mut names = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for constraint in &law.constraints {
        identifier(&constraint.id)?;
        if !names.insert(&constraint.id) {
            return Err(invalid("duplicate constraint"));
        }
        constraint.validate(law)?;
        constraint.coverage(&mut covered);
    }
    for id in law.slots.keys() {
        if !covered.contains(&("slot", id.clone())) {
            return Err(invalid(format!(
                "slot {id} has no equality constraint on its transition"
            )));
        }
    }
    for id in law.boundaries.keys() {
        if !covered.contains(&("boundary", id.clone())) {
            return Err(invalid(format!("boundary {id} has no equality constraint")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::Expr;

    fn law(id: &str) -> Law {
        let mass = Dimension::base("mass").unwrap();
        Law {
            id: id.into(),
            slots: BTreeMap::from([("stock".into(), mass.clone())]),
            boundaries: BTreeMap::new(),
            facts: BTreeMap::new(),
            participants: BTreeSet::from(["owner".into()]),
            constraints: vec![Constraint::equal(
                "zero",
                Expr::delta("stock"),
                Expr::constant(zero(&mass)),
            )],
        }
    }

    #[test]
    fn incremental_validation_rejects_invalid_new_definitions_without_revalidating_old_payloads() {
        let public = Model {
            owners: BTreeSet::from(["owner".into()]),
            capacities: BTreeMap::from([(
                "hold".into(),
                Capacity {
                    maximum: zero(&Dimension::base("mass").unwrap()),
                },
            )]),
            laws: vec![law("hold")],
        };
        let model = ValidatedModel::new(&public).unwrap();
        let valid = Model {
            owners: BTreeSet::from(["new-owner".into()]),
            laws: vec![law("new")],
            ..Model::default()
        };
        let extended = model.extended(&valid).unwrap();
        assert!(Arc::ptr_eq(&model.laws["hold"], &extended.laws["hold"]));
        assert!(Arc::ptr_eq(
            &model.capacities["hold"],
            &extended.capacities["hold"]
        ));
        let mut cases = Vec::new();
        let mut invalid = valid.clone();
        invalid.owners.insert(" ".into());
        cases.push(invalid);
        let mut invalid = valid.clone();
        invalid.capacities.insert(
            "negative".into(),
            Capacity {
                maximum: Quantity::new(
                    BigRational::from_integer((-1).into()),
                    Dimension::base("mass").unwrap(),
                ),
            },
        );
        cases.push(invalid);
        let mut invalid = valid.clone();
        invalid.laws[0].id = " ".into();
        cases.push(invalid);
        let mut invalid = valid.clone();
        invalid.laws[0].participants.insert("missing".into());
        cases.push(invalid);
        let mut invalid = valid.clone();
        invalid.laws[0].constraints.clear();
        cases.push(invalid);
        let mut invalid = valid.clone();
        invalid.laws.push(invalid.laws[0].clone());
        cases.push(invalid);
        for declarations in cases {
            let mut combined = public.clone();
            combined.owners.extend(declarations.owners.clone());
            combined.capacities.extend(declarations.capacities.clone());
            combined.laws.extend(declarations.laws.clone());
            assert!(combined.validate().is_err());
            assert!(model.extended(&declarations).is_err());
        }
        let mut replacement = public.clone();
        replacement.laws[0].constraints.clear();
        assert!(model.extended(&replacement).is_err());
        let mut replacement = public.clone();
        replacement
            .capacities
            .get_mut("hold")
            .unwrap()
            .maximum
            .amount = BigRational::from_integer(1.into());
        assert!(model.extended(&replacement).is_err());
        assert!(model.extended(&public).is_ok());
        assert_eq!(model.laws.len(), 1);
        assert_eq!(model.owners.len(), 1);
    }
}

/// One declarative request. All new stocks start at zero; funding requires a law.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exchange {
    pub id: String,
    pub law: String,
    pub bindings: BTreeMap<String, String>,
    pub deltas: BTreeMap<String, Quantity>,
    pub boundaries: BTreeMap<String, Quantity>,
    pub creates: Vec<Stock>,
    pub removes: BTreeSet<String>,
    pub moves: BTreeMap<String, Placement>,
    pub declarations: Option<Model>,
}

impl Exchange {
    pub fn new(id: impl Into<String>, law: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            law: law.into(),
            bindings: BTreeMap::new(),
            deltas: BTreeMap::new(),
            boundaries: BTreeMap::new(),
            creates: Vec::new(),
            removes: BTreeSet::new(),
            moves: BTreeMap::new(),
            declarations: None,
        }
    }
}

/// Immutable owner-authored bytes. The kernel does not interpret the domain schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordWrite {
    pub key: String,
    pub value: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid(String),
    Dimension {
        context: String,
    },
    Domain {
        stock: String,
        amount: BigRational,
    },
    Capacity {
        id: String,
        residual: BigRational,
    },
    Constraint {
        exchange: String,
        id: String,
        residual: BigRational,
    },
    Participant {
        owner: String,
        reason: String,
    },
    Duplicate {
        id: String,
    },
    Stale,
    ForeignPreparation,
    Snapshot(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for Error {}

pub(crate) fn identifier(value: &str) -> Result<(), Error> {
    nonblank(value).map_err(|error| invalid(error.to_string()))
}
pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
pub(crate) fn rational(value: &BigRational) -> Result<(), Error> {
    if value.denom() <= &0.into() {
        return Err(invalid("rational denominator must be positive"));
    }
    let canonical = BigRational::new(value.numer().clone(), value.denom().clone());
    if value.numer() != canonical.numer() || value.denom() != canonical.denom() {
        return Err(invalid("rational must be reduced"));
    }
    Ok(())
}

pub(crate) fn zero(dimension: &Dimension) -> Quantity {
    Quantity::new(BigRational::zero(), dimension.clone())
}
