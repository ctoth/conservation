#![forbid(unsafe_code)]

//! Bridgman kinds as conservation kinds. The crate holds no kind data of its
//! own: identity, affine role, difference, floor, dimensions and grade are read
//! from a Bridgman registry, once, when [`BridgmanKinds::new`] admits it.
//!
//! A coordinate of a stock of a Bridgman kind is its value in the kind's canonical
//! unit (`bridgman_core::Kind::canonical_unit`), the unit Bridgman holds computed
//! quantities and declared floors in.
//!
//! Handles are `'static`. Bridgman's bundled catalog already is; any other
//! catalog is leaked on purpose by [`BridgmanKinds::leak`], because catalogs load
//! once per process (the lifetime decision of conservation#6). The readings
//! admission takes are leaked beside the registry, once per admission, so a
//! handle reads them without failing.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

use bridgman_core::{
    AffineRole, DerivationError, Graded, Kind as Declared, Operand, ProductOp, Quantity,
    QuantityError, Registry, Side, derive,
};
use conservation_core::{Affine, DimensionAlgebra, Factor, Kind, KindRegistry};
use num_rational::BigRational;

/// What admission read of one kind of a registry.
#[derive(Debug)]
struct Admitted {
    kind: Declared<'static>,
    /// Position of `kind.difference()` in the admitted table.
    difference: usize,
    /// The declared floor as an exact coordinate in the canonical unit.
    floor: Option<BigRational>,
    /// Dimensions and grade (`operand.graded`, the kind's `Dimensions`) and
    /// affine role (`Declared::operand`).
    operand: Operand,
}

/// A Bridgman kind of an admitted registry, as a conservation kind. Identity,
/// order and hash are the Bridgman kind's.
#[derive(Clone, Copy)]
pub struct BridgmanKind {
    table: &'static [Admitted],
    index: usize,
}

impl BridgmanKind {
    /// The Bridgman kind this handle is.
    pub fn bridgman(self) -> Declared<'static> {
        self.admitted().kind
    }

    fn admitted(self) -> &'static Admitted {
        &self.table[self.index]
    }
}

impl PartialEq for BridgmanKind {
    fn eq(&self, other: &Self) -> bool {
        self.bridgman() == other.bridgman()
    }
}

impl Eq for BridgmanKind {}

impl Hash for BridgmanKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bridgman().hash(state);
    }
}

impl Ord for BridgmanKind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bridgman().cmp(&other.bridgman())
    }
}

impl PartialOrd for BridgmanKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for BridgmanKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("BridgmanKind")
            .field(&self.bridgman())
            .finish()
    }
}

impl fmt::Display for BridgmanKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bridgman().fmt(formatter)
    }
}

/// A `'static` Bridgman registry whose every kind has resolved dimensions and,
/// when it declares a floor, an exactly readable one. Admission reads each
/// kind's difference, floor and dimensions once and keeps them.
#[derive(Clone, Copy, Debug)]
pub struct BridgmanKinds {
    registry: &'static Registry,
    table: &'static [Admitted],
}

impl BridgmanKinds {
    /// Bridgman's bundled thermal catalog (`bridgman_core::thermal`), admitted
    /// once per process.
    pub fn thermal() -> Result<Self, BridgmanKindError> {
        static THERMAL: OnceLock<Result<BridgmanKinds, BridgmanKindError>> = OnceLock::new();
        THERMAL
            .get_or_init(|| Self::new(bridgman_core::thermal()))
            .clone()
    }

    /// Admits a registry the process already holds for its lifetime.
    pub fn new(registry: &'static Registry) -> Result<Self, BridgmanKindError> {
        let kinds: Vec<_> = registry.kinds().collect();
        let table = kinds
            .iter()
            .map(|&kind| {
                let operand = kind
                    .operand()
                    .map_err(|source| BridgmanKindError::Dimensions {
                        kind,
                        source: Box::new(source),
                    })?;
                let floor = kind
                    .minimum()
                    .map(|minimum| floor_coordinate(kind, minimum))
                    .transpose()?;
                let difference = kind.difference();
                let difference = kinds
                    .iter()
                    .position(|candidate| *candidate == difference)
                    .ok_or(BridgmanKindError::ForeignRegistry { kind: difference })?;
                Ok(Admitted {
                    kind,
                    difference,
                    floor,
                    operand,
                })
            })
            .collect::<Result<Vec<_>, BridgmanKindError>>()?;
        Ok(Self {
            registry,
            table: Vec::leak(table),
        })
    }

    /// Leaks `registry` for the process and admits it.
    pub fn leak(registry: Registry) -> Result<Self, BridgmanKindError> {
        Self::new(Box::leak(Box::new(registry)))
    }

    /// The admitted registry.
    pub fn registry(self) -> &'static Registry {
        self.registry
    }

    /// The conservation handle of a Bridgman kind the caller already holds.
    /// Typed callers use this, not a name lookup.
    pub fn of(self, kind: Declared<'static>) -> Result<BridgmanKind, BridgmanKindError> {
        self.table
            .iter()
            .position(|admitted| admitted.kind == kind)
            .map(|index| BridgmanKind {
                table: self.table,
                index,
            })
            .ok_or(BridgmanKindError::ForeignRegistry { kind })
    }

    /// Every kind, in the registry's declaration order.
    pub fn kinds(self) -> impl ExactSizeIterator<Item = BridgmanKind> {
        let table = self.table;
        (0..table.len()).map(move |index| BridgmanKind { table, index })
    }
}

impl KindRegistry for BridgmanKinds {
    type Kind = BridgmanKind;

    fn resolve(&self, name: &str) -> Option<BridgmanKind> {
        self.registry
            .kind(name)
            .ok()
            .and_then(|kind| self.of(kind).ok())
    }
}

/// `minimum` as an exact coordinate in `kind`'s canonical unit. The binary64
/// value is the one Bridgman itself enforces (`check_minimum`), so conservation
/// and Bridgman refuse at the same boundary.
fn floor_coordinate(
    kind: Declared<'static>,
    minimum: Quantity<'static>,
) -> Result<BigRational, BridgmanKindError> {
    let floor = |source| BridgmanKindError::Floor {
        kind,
        source: Box::new(source),
    };
    let unit = kind.canonical_unit().map_err(floor)?;
    let read = minimum.in_unit(unit).map_err(floor)?;
    let round_trip = Quantity::new(read, unit, kind).map_err(floor)?;
    match minimum.compare(round_trip).map_err(floor)? {
        Ordering::Equal => {
            BigRational::from_float(read).ok_or(BridgmanKindError::FloorNotExact { kind, read })
        }
        Ordering::Less | Ordering::Greater => Err(BridgmanKindError::FloorNotExact { kind, read }),
    }
}

impl Kind for BridgmanKind {
    fn affine(self) -> Affine<Self> {
        let admitted = self.admitted();
        match admitted.operand.role {
            AffineRole::Point => Affine::Point {
                difference: Self {
                    table: self.table,
                    index: admitted.difference,
                },
            },
            AffineRole::Linear | AffineRole::Difference => Affine::Linear,
        }
    }

    fn floor(self) -> Option<BigRational> {
        self.admitted().floor.clone()
    }
}

/// `left op right` by Bridgman's `derive`: a kind factor is read as its admitted
/// operand, a derived one as its dimensions and grade. Bridgman refuses a point
/// kind and an ungraded product; its refusal names the factors it was given.
fn derived(
    left: &Factor<BridgmanKind>,
    op: ProductOp,
    right: &Factor<BridgmanKind>,
) -> Result<Graded, DerivationError<Factor<BridgmanKind>>> {
    fn read(factor: &Factor<BridgmanKind>) -> bridgman_core::Factor<'_> {
        match factor {
            Factor::Kind(kind) => bridgman_core::Factor::Kind(&kind.admitted().operand),
            Factor::Derived(graded) => bridgman_core::Factor::Derived(graded),
        }
    }
    derive(read(left), op, read(right)).map_err(|refusal| {
        refusal.map_kinds(|side| match side {
            Side::Left => left.clone(),
            Side::Right => right.clone(),
        })
    })
}

impl DimensionAlgebra for BridgmanKind {
    type Dimensions = Graded;
    type AlgebraError = DerivationError<Factor<BridgmanKind>>;

    fn dimensions(self) -> Graded {
        self.admitted().operand.graded.clone()
    }

    fn product(
        left: &Factor<Self>,
        right: &Factor<Self>,
    ) -> Result<Graded, DerivationError<Factor<Self>>> {
        derived(left, ProductOp::Mul, right)
    }

    fn quotient(
        left: &Factor<Self>,
        right: &Factor<Self>,
    ) -> Result<Graded, DerivationError<Factor<Self>>> {
        derived(left, ProductOp::Div, right)
    }
}

/// Why a Bridgman kind cannot be a conservation kind. Each variant keeps the
/// Bridgman kind and Bridgman's own refusal, whole; the refusal is boxed because
/// `QuantityError` is large.
#[derive(Clone, Debug, PartialEq)]
pub enum BridgmanKindError {
    /// The kind's dimensions are unresolved.
    Dimensions {
        kind: Declared<'static>,
        source: Box<QuantityError<'static>>,
    },
    /// The declared floor cannot be read in the canonical unit.
    Floor {
        kind: Declared<'static>,
        source: Box<QuantityError<'static>>,
    },
    /// The floor read in the canonical unit (`read`) is not the declared floor.
    FloorNotExact { kind: Declared<'static>, read: f64 },
    /// The kind belongs to another registry than the one asked.
    ForeignRegistry { kind: Declared<'static> },
}

impl fmt::Display for BridgmanKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dimensions { kind, source } => write!(formatter, "kind {kind}: {source}"),
            Self::Floor { kind, source } => write!(formatter, "floor of kind {kind}: {source}"),
            Self::FloorNotExact { kind, read } => {
                write!(
                    formatter,
                    "floor of kind {kind} reads as {read}, not its declared value"
                )
            }
            Self::ForeignRegistry { kind } => {
                write!(
                    formatter,
                    "kind {kind} belongs to another Bridgman registry"
                )
            }
        }
    }
}

impl Error for BridgmanKindError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dimensions { source, .. } | Self::Floor { source, .. } => Some(source.as_ref()),
            Self::FloorNotExact { .. } | Self::ForeignRegistry { .. } => None,
        }
    }
}
