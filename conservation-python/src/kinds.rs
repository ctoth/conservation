//! The native kind table behind the Python `KindRegistry`.
//!
//! A registry is validated once and leaked to `'static`, so a kind handle is a
//! `Copy` table pointer plus an index. A registry lives until the process exits.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ptr;

use conservation_core::{Affine, DimensionAlgebra, Kind, KindRegistry, nonblank};
use num_rational::BigRational;

pub(crate) struct NativeTable {
    /// Base dimension names, sorted.
    bases: Box<[Box<str>]>,
    /// Sorted by name, so `Ord` on indices is name order.
    kinds: Box<[NativeEntry]>,
    names: BTreeMap<Box<str>, u32>,
}

struct NativeEntry {
    name: Box<str>,
    /// Exponents indexed like `NativeTable::bases`.
    powers: Box<[i32]>,
    floor: Option<BigRational>,
    difference: Option<u32>,
}

/// One declaration as supplied by Python, before validation.
pub(crate) struct Declaration {
    pub(crate) dimensions: BTreeMap<String, i32>,
    pub(crate) floor: Option<String>,
    pub(crate) difference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DeclarationError {
    BlankKind { name: String },
    BlankBase { kind: String },
    ZeroExponent { kind: String, base: String },
    InvalidFloor { kind: String, value: String },
    UnknownDifference { kind: String, difference: String },
    DifferenceIsPoint { kind: String, difference: String },
    DifferenceDimensions { kind: String, difference: String },
}

impl fmt::Display for DeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankKind { name } => write!(formatter, "kind name {name:?} is blank"),
            Self::BlankBase { kind } => {
                write!(formatter, "kind {kind} declares a blank base dimension")
            }
            Self::ZeroExponent { kind, base } => write!(
                formatter,
                "kind {kind} declares a zero exponent for base {base}"
            ),
            Self::InvalidFloor { kind, value } => write!(
                formatter,
                "kind {kind} declares floor {value:?}, which is not an exact rational"
            ),
            Self::UnknownDifference { kind, difference } => write!(
                formatter,
                "kind {kind} names undeclared difference kind {difference}"
            ),
            Self::DifferenceIsPoint { kind, difference } => write!(
                formatter,
                "kind {kind} names difference kind {difference}, which is itself a point kind"
            ),
            Self::DifferenceDimensions { kind, difference } => write!(
                formatter,
                "kind {kind} and its difference kind {difference} have different dimensions"
            ),
        }
    }
}

impl std::error::Error for DeclarationError {}

impl NativeTable {
    /// Validates every declaration and leaks the resulting table.
    pub(crate) fn leak(
        declarations: BTreeMap<String, Declaration>,
        rational: impl Fn(&str) -> Option<BigRational>,
    ) -> Result<&'static Self, DeclarationError> {
        let mut bases = BTreeSet::new();
        for (name, declaration) in &declarations {
            if nonblank(name).is_err() {
                return Err(DeclarationError::BlankKind { name: name.clone() });
            }
            for (base, power) in &declaration.dimensions {
                if nonblank(base).is_err() {
                    return Err(DeclarationError::BlankBase { kind: name.clone() });
                }
                if *power == 0 {
                    return Err(DeclarationError::ZeroExponent {
                        kind: name.clone(),
                        base: base.clone(),
                    });
                }
                bases.insert(base.as_str());
            }
        }
        let bases: Box<[Box<str>]> = bases.into_iter().map(Box::from).collect();
        let names: BTreeMap<Box<str>, u32> = declarations
            .keys()
            .enumerate()
            .map(|(index, name)| (Box::from(name.as_str()), index as u32))
            .collect();
        let powers = |dimensions: &BTreeMap<String, i32>| -> Box<[i32]> {
            bases
                .iter()
                .map(|base| dimensions.get(base.as_ref()).copied().unwrap_or(0))
                .collect()
        };
        let mut kinds = Vec::with_capacity(declarations.len());
        for (name, declaration) in &declarations {
            let floor = declaration
                .floor
                .as_ref()
                .map(|value| {
                    rational(value).ok_or_else(|| DeclarationError::InvalidFloor {
                        kind: name.clone(),
                        value: value.clone(),
                    })
                })
                .transpose()?;
            let difference = declaration
                .difference
                .as_ref()
                .map(|difference| {
                    let target = declarations.get(difference).ok_or_else(|| {
                        DeclarationError::UnknownDifference {
                            kind: name.clone(),
                            difference: difference.clone(),
                        }
                    })?;
                    if target.difference.is_some() {
                        return Err(DeclarationError::DifferenceIsPoint {
                            kind: name.clone(),
                            difference: difference.clone(),
                        });
                    }
                    if powers(&target.dimensions) != powers(&declaration.dimensions) {
                        return Err(DeclarationError::DifferenceDimensions {
                            kind: name.clone(),
                            difference: difference.clone(),
                        });
                    }
                    Ok(names[difference.as_str()])
                })
                .transpose()?;
            kinds.push(NativeEntry {
                name: Box::from(name.as_str()),
                powers: powers(&declaration.dimensions),
                floor,
                difference,
            });
        }
        Ok(Box::leak(Box::new(Self {
            bases,
            kinds: kinds.into_boxed_slice(),
            names,
        })))
    }

    fn entry(&self, index: u32) -> &NativeEntry {
        &self.kinds[index as usize]
    }
}

/// A kind handle: a pointer to its registry's table plus an index.
#[derive(Clone, Copy)]
pub(crate) struct NativeKind {
    table: &'static NativeTable,
    index: u32,
}

impl NativeKind {
    pub(crate) fn name(self) -> &'static str {
        &self.table.entry(self.index).name
    }

    /// Nonzero exponents by base name.
    pub(crate) fn powers(self) -> BTreeMap<String, i32> {
        self.dimensions().powers()
    }

    fn address(self) -> usize {
        ptr::from_ref(self.table) as usize
    }
}

impl PartialEq for NativeKind {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.table, other.table) && self.index == other.index
    }
}

impl Eq for NativeKind {}

impl Hash for NativeKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state);
        self.index.hash(state);
    }
}

impl PartialOrd for NativeKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NativeKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.index, self.address()).cmp(&(other.index, other.address()))
    }
}

impl fmt::Debug for NativeKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Kind({:?})", self.name())
    }
}

impl fmt::Display for NativeKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl Kind for NativeKind {
    fn affine(self) -> Affine<Self> {
        match self.table.entry(self.index).difference {
            Some(index) => Affine::Point {
                difference: Self {
                    table: self.table,
                    index,
                },
            },
            None => Affine::Linear,
        }
    }

    fn floor(self) -> Option<BigRational> {
        self.table.entry(self.index).floor.clone()
    }
}

/// Exponents over one registry's bases. Dimensions from different registries
/// are never equal.
#[derive(Clone)]
pub(crate) struct NativeDimensions {
    table: &'static NativeTable,
    powers: Box<[i32]>,
}

impl NativeDimensions {
    fn powers(&self) -> BTreeMap<String, i32> {
        self.table
            .bases
            .iter()
            .zip(&self.powers)
            .filter(|(_, power)| **power != 0)
            .map(|(base, power)| (base.to_string(), *power))
            .collect()
    }

    fn combine(
        left: &Self,
        right: &Self,
        operation: fn(i32, i32) -> Option<i32>,
    ) -> Result<Self, DimensionOverflow> {
        let overflow = || DimensionOverflow {
            left: left.clone(),
            right: right.clone(),
        };
        if !ptr::eq(left.table, right.table) {
            return Err(overflow());
        }
        let powers = left
            .powers
            .iter()
            .zip(&right.powers)
            .map(|(a, b)| operation(*a, *b).ok_or_else(overflow))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            table: left.table,
            powers,
        })
    }
}

impl PartialEq for NativeDimensions {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.table, other.table) && self.powers == other.powers
    }
}

impl Eq for NativeDimensions {}

impl fmt::Debug for NativeDimensions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Dimensions({self})")
    }
}

impl fmt::Display for NativeDimensions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let powers = self.powers();
        if powers.is_empty() {
            return formatter.write_str("1");
        }
        let terms: Vec<_> = powers
            .iter()
            .map(|(base, power)| format!("{base}^{power}"))
            .collect();
        formatter.write_str(&terms.join("·"))
    }
}

/// A product or quotient with no representation: an exponent outside `i32`, or
/// operands from two different registries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DimensionOverflow {
    left: NativeDimensions,
    right: NativeDimensions,
}

impl fmt::Display for DimensionOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "dimensions {} and {} cannot be combined",
            self.left, self.right
        )
    }
}

impl std::error::Error for DimensionOverflow {}

impl DimensionAlgebra for NativeKind {
    type Dimensions = NativeDimensions;
    type AlgebraError = DimensionOverflow;

    fn dimensions(self) -> NativeDimensions {
        NativeDimensions {
            table: self.table,
            powers: self.table.entry(self.index).powers.clone(),
        }
    }

    fn product(
        left: &NativeDimensions,
        right: &NativeDimensions,
    ) -> Result<NativeDimensions, DimensionOverflow> {
        NativeDimensions::combine(left, right, i32::checked_add)
    }

    fn quotient(
        left: &NativeDimensions,
        right: &NativeDimensions,
    ) -> Result<NativeDimensions, DimensionOverflow> {
        NativeDimensions::combine(left, right, i32::checked_sub)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NativeRegistry(&'static NativeTable);

impl NativeRegistry {
    pub(crate) fn new(table: &'static NativeTable) -> Self {
        Self(table)
    }
}

impl KindRegistry for NativeRegistry {
    type Kind = NativeKind;

    fn resolve(&self, name: &str) -> Option<NativeKind> {
        self.0.names.get(name).map(|index| NativeKind {
            table: self.0,
            index: *index,
        })
    }
}
