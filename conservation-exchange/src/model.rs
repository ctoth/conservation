use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use conservation_core::{DimensionAlgebra, nonblank};
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::Constraint;

/// An exact rational in canonical base units. Binary floats are never implicit inputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quantity<K> {
    pub amount: BigRational,
    pub kind: K,
}

impl<K: DimensionAlgebra> Quantity<K> {
    pub fn new(amount: BigRational, kind: K) -> Self {
        Self { amount, kind }
    }

    pub(crate) fn validate(&self) -> Result<(), Error<K>> {
        rational(&self.amount)
    }

    /// Requires this value to have exactly the `expected` kind.
    pub(crate) fn same_kind(&self, expected: K, context: KindContext) -> Result<(), Error<K>> {
        self.validate()?;
        if self.kind != expected {
            return Err(Error::Kinds {
                context,
                expected,
                found: self.kind,
            });
        }
        Ok(())
    }
}

/// A stock's physical placement. Capacity weights map stock units to constraint units.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement<K> {
    pub owner: String,
    pub capacities: BTreeMap<String, Quantity<K>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stock<K> {
    pub id: String,
    pub owner: String,
    pub kind: K,
    pub capacities: BTreeMap<String, Quantity<K>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capacity<K> {
    pub maximum: Quantity<K>,
}

/// A named domain evaluation. The declared owner supplies it for this proposal/revision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact<K> {
    pub owner: String,
    pub kind: K,
}

/// Registered transition law over named stock roles, signed boundary inputs, and facts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Law<K> {
    pub id: String,
    pub slots: BTreeMap<String, K>,
    pub boundaries: BTreeMap<String, K>,
    pub facts: BTreeMap<String, Fact<K>>,
    pub participants: BTreeSet<String>,
    pub constraints: Vec<Constraint<K>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model<K> {
    pub owners: BTreeSet<String>,
    pub capacities: BTreeMap<String, Capacity<K>>,
    pub laws: Vec<Law<K>>,
}

impl<K> Default for Model<K> {
    fn default() -> Self {
        Self {
            owners: BTreeSet::new(),
            capacities: BTreeMap::new(),
            laws: Vec::new(),
        }
    }
}

impl<K: DimensionAlgebra> Model<K> {
    pub(crate) fn validate(&self) -> Result<(), Error<K>> {
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

    pub(crate) fn owner(&self, owner: &str) -> Result<(), Error<K>> {
        if !self.owners.contains(owner) {
            return Err(invalid(format!("undeclared owner {owner}")));
        }
        Ok(())
    }
}

/// Validated internal definitions. Payloads survive immutable root transitions.
#[derive(Clone, Debug)]
pub(crate) struct ValidatedModel<K> {
    pub(crate) owners: BTreeSet<String>,
    pub(crate) capacities: BTreeMap<String, Arc<Capacity<K>>>,
    pub(crate) laws: BTreeMap<String, Arc<Law<K>>>,
}

impl<K> Default for ValidatedModel<K> {
    fn default() -> Self {
        Self {
            owners: BTreeSet::new(),
            capacities: BTreeMap::new(),
            laws: BTreeMap::new(),
        }
    }
}

impl<K: DimensionAlgebra> ValidatedModel<K> {
    pub(crate) fn new(model: &Model<K>) -> Result<Self, Error<K>> {
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

    pub(crate) fn extended(&self, declarations: &Model<K>) -> Result<Self, Error<K>> {
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

    pub(crate) fn owner(&self, owner: &str) -> Result<(), Error<K>> {
        if !self.owners.contains(owner) {
            return Err(invalid(format!("undeclared owner {owner}")));
        }
        Ok(())
    }

    pub(crate) fn stock(&self, stock: &Stock<K>) -> Result<(), Error<K>> {
        identifier(&stock.id)?;
        self.owner(&stock.owner)?;
        for (id, weight) in &stock.capacities {
            let capacity = self
                .capacities
                .get(id)
                .ok_or_else(|| invalid(format!("undeclared capacity {id}")))?;
            weight.validate()?;
            let floored = stock.kind.floor().is_some_and(|floor| !floor.is_negative());
            if weight.amount.is_negative() || !floored {
                return Err(invalid(
                    "capacity weights require nonnegative stocks and weights",
                ));
            }
            let dimensions = K::product(&weight.kind.dimensions(), &stock.kind.dimensions())
                .map_err(Error::Algebra)?;
            let maximum = capacity.maximum.kind.dimensions();
            if dimensions != maximum {
                return Err(Error::Dimensions {
                    context: DimensionContext::Capacity {
                        capacity: id.clone(),
                        stock: stock.id.clone(),
                    },
                    left: dimensions,
                    right: maximum,
                });
            }
        }
        Ok(())
    }
}

fn validate_law<K: DimensionAlgebra>(
    law: &Law<K>,
    check_owner: impl Fn(&str) -> Result<(), Error<K>>,
) -> Result<(), Error<K>> {
    for id in law.slots.keys().chain(law.boundaries.keys()) {
        identifier(id)?;
    }
    for (id, fact) in &law.facts {
        identifier(id)?;
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
    use conservation_test_kinds::TestKind;

    fn law(id: &str) -> Law<TestKind> {
        let mass = TestKind::Material;
        Law {
            id: id.into(),
            slots: BTreeMap::from([("stock".into(), mass)]),
            boundaries: BTreeMap::new(),
            facts: BTreeMap::new(),
            participants: BTreeSet::from(["owner".into()]),
            constraints: vec![Constraint::equal(
                "zero",
                Expr::delta("stock"),
                Expr::constant(zero(mass)),
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
                    maximum: zero(TestKind::Material),
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
                maximum: Quantity::new(BigRational::from_integer((-1).into()), TestKind::Material),
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
pub struct Exchange<K> {
    pub id: String,
    pub law: String,
    pub bindings: BTreeMap<String, String>,
    pub deltas: BTreeMap<String, Quantity<K>>,
    pub boundaries: BTreeMap<String, Quantity<K>>,
    pub creates: Vec<Stock<K>>,
    pub removes: BTreeSet<String>,
    pub moves: BTreeMap<String, Placement<K>>,
    pub declarations: Option<Model<K>>,
}

impl<K: DimensionAlgebra> Exchange<K> {
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

/// Where two kinds were required to be identical.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KindContext {
    /// An exchange leg: the stock bound to a law slot.
    Slot {
        slot: String,
        stock: String,
    },
    Delta {
        slot: String,
    },
    Boundary {
        port: String,
    },
    Fact {
        fact: String,
    },
    Sum,
    Constraint {
        id: String,
    },
}

/// Where a computed quantity's dimensions were compared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DimensionContext {
    Sum,
    Constraint { id: String },
    Capacity { capacity: String, stock: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error<K: DimensionAlgebra> {
    Invalid(String),
    /// Two kinds that must be identical differ. Both are named.
    Kinds {
        context: KindContext,
        expected: K,
        found: K,
    },
    /// A computed (derived) quantity's dimensions differ from the other side's.
    Dimensions {
        context: DimensionContext,
        left: K::Dimensions,
        right: K::Dimensions,
    },
    Algebra(K::AlgebraError),
    /// The amount lies below the kind's floor. `kind.floor()` recovers the floor.
    BelowFloor {
        stock: String,
        kind: K,
        amount: BigRational,
    },
    UnknownKind {
        name: String,
    },
    UnsupportedSnapshot {
        version: u32,
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

impl<K: DimensionAlgebra> fmt::Display for Error<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl<K: DimensionAlgebra> std::error::Error for Error<K> {}

pub(crate) fn identifier<K: DimensionAlgebra>(value: &str) -> Result<(), Error<K>> {
    nonblank(value).map_err(|error| invalid(error.to_string()))
}
pub(crate) fn invalid<K: DimensionAlgebra>(message: impl Into<String>) -> Error<K> {
    Error::Invalid(message.into())
}
pub(crate) fn rational<K: DimensionAlgebra>(value: &BigRational) -> Result<(), Error<K>> {
    if value.denom() <= &0.into() {
        return Err(invalid("rational denominator must be positive"));
    }
    let canonical = BigRational::new(value.numer().clone(), value.denom().clone());
    if value.numer() != canonical.numer() || value.denom() != canonical.denom() {
        return Err(invalid("rational must be reduced"));
    }
    Ok(())
}

pub(crate) fn zero<K: DimensionAlgebra>(kind: K) -> Quantity<K> {
    Quantity::new(BigRational::zero(), kind)
}
