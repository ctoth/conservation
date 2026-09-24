#![forbid(unsafe_code)]

//! Bridgman kinds as conservation kinds. The crate holds no kind data: identity,
//! affine role, floor, dimensions and grade are read from a Bridgman registry.
//!
//! A coordinate of a stock of a Bridgman kind is its value in the kind's canonical
//! unit (`bridgman_core::Kind::canonical_unit`), the unit Bridgman holds computed
//! quantities and declared floors in.
//!
//! Handles are `'static`. Bridgman's bundled profile already is; any other
//! catalog is leaked on purpose by [`BridgmanKinds::leak`], because catalogs load
//! once per process (the lifetime decision of conservation#6).

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use bridgman_core::{
    AffineRole, Dimensions, Grade, Kind as Declared, Op, ProductOp, Quantity, QuantityError,
    Registry,
};
use conservation_core::{Affine, DimensionAlgebra, Kind, KindRegistry};
use num_rational::BigRational;

/// Why a kind's reading was established when its registry was admitted; a
/// `BridgmanKind` exists only for kinds of an admitted registry.
const ADMITTED: &str = "BridgmanKinds::new admitted every kind of this registry";

/// A Bridgman kind of an admitted registry, as a conservation kind.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BridgmanKind(Declared<'static>);

impl BridgmanKind {
    /// The Bridgman kind this handle is.
    pub fn bridgman(self) -> Declared<'static> {
        self.0
    }
}

impl fmt::Display for BridgmanKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A `'static` Bridgman registry whose every kind has been checked to have
/// resolved dimensions, a difference kind when it is a point, and an exactly
/// readable floor when it declares one.
#[derive(Clone, Copy, Debug)]
pub struct BridgmanKinds {
    registry: &'static Registry,
}

impl BridgmanKinds {
    /// Bridgman's bundled thermal profile (`bridgman_core::profile::registry`).
    pub fn thermal() -> Result<Self, BridgmanKindError> {
        Self::new(bridgman_core::profile::registry())
    }

    /// Admits a registry the process already holds for its lifetime.
    pub fn new(registry: &'static Registry) -> Result<Self, BridgmanKindError> {
        for kind in registry.kinds() {
            kind.dimensions()
                .map_err(|source| BridgmanKindError::Dimensions {
                    kind,
                    source: Box::new(source),
                })?;
            match kind.role() {
                AffineRole::Point => {
                    difference_of(kind)
                        .map_err(|source| BridgmanKindError::Difference { kind, source })?;
                }
                AffineRole::Linear | AffineRole::Difference => {}
            }
            if let Some(minimum) = kind.minimum() {
                floor_coordinate(kind, minimum)?;
            }
        }
        Ok(Self { registry })
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
        if std::ptr::eq(kind.registry(), self.registry) {
            Ok(BridgmanKind(kind))
        } else {
            Err(BridgmanKindError::ForeignRegistry { kind })
        }
    }

    /// Every kind, in the registry's declaration order.
    pub fn kinds(self) -> impl ExactSizeIterator<Item = BridgmanKind> {
        self.registry.kinds().map(BridgmanKind)
    }
}

impl KindRegistry for BridgmanKinds {
    type Kind = BridgmanKind;

    fn resolve(&self, name: &str) -> Option<BridgmanKind> {
        self.registry.kind(name).ok().map(BridgmanKind)
    }
}

/// The kind of a difference of two values of `kind`: Bridgman's own subtraction
/// rule (a point minus itself is its declared difference; any other kind minus
/// itself is itself). Bridgman exposes no other public reading of it. Bridgman's
/// refusal is boxed whole, because `QuantityError` is large.
fn difference_of(kind: Declared<'static>) -> Result<Declared<'static>, Box<QuantityError>> {
    kind.combine(Op::Sub, kind).map_err(Box::new)
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
        match self.0.role() {
            AffineRole::Point => Affine::Point {
                difference: BridgmanKind(difference_of(self.0).expect(ADMITTED)),
            },
            AffineRole::Linear | AffineRole::Difference => Affine::Linear,
        }
    }

    fn floor(self) -> Option<BigRational> {
        self.0
            .minimum()
            .map(|minimum| floor_coordinate(self.0, minimum).expect(ADMITTED))
    }
}

/// What a product or quotient in conservation needs to know of a Bridgman kind:
/// its dimensions, its grade in G3, and whether its values are points.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindDimensions {
    pub dimensions: Dimensions,
    pub grade: Grade,
    /// Bridgman `AffineRole::Point`. A point takes part in no product.
    pub point: bool,
}

impl fmt::Display for KindDimensions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} grade {}", self.dimensions, self.grade)?;
        if self.point {
            formatter.write_str(" point")?;
        }
        Ok(())
    }
}

/// Why Bridgman gives a product or quotient no dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgmanAlgebraError {
    /// An operand is a point kind (Bridgman `derive`: `Underived::Point`).
    Point {
        op: ProductOp,
        left: KindDimensions,
        right: KindDimensions,
    },
    /// G3 gives the product no single grade (`Grade::product` is `None`).
    Ungraded {
        op: ProductOp,
        left: Grade,
        right: Grade,
    },
}

impl fmt::Display for BridgmanAlgebraError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Point { op, left, right } => {
                write!(
                    formatter,
                    "a point takes part in no {op:?}: ({left}) and ({right})"
                )
            }
            Self::Ungraded { op, left, right } => {
                write!(
                    formatter,
                    "{op:?} of grades {left} and {right} has no single grade"
                )
            }
        }
    }
}

impl Error for BridgmanAlgebraError {}

/// The grade of `left op right`, refusing points as Bridgman's `derive` does.
fn graded(
    left: &KindDimensions,
    op: ProductOp,
    right: &KindDimensions,
) -> Result<Grade, BridgmanAlgebraError> {
    if left.point || right.point {
        return Err(BridgmanAlgebraError::Point {
            op,
            left: left.clone(),
            right: right.clone(),
        });
    }
    left.grade
        .product(op, right.grade)
        .ok_or(BridgmanAlgebraError::Ungraded {
            op,
            left: left.grade,
            right: right.grade,
        })
}

impl DimensionAlgebra for BridgmanKind {
    type Dimensions = KindDimensions;
    type AlgebraError = BridgmanAlgebraError;

    fn dimensions(self) -> KindDimensions {
        KindDimensions {
            dimensions: self.0.dimensions().expect(ADMITTED).clone(),
            grade: self.0.grade(),
            point: matches!(self.0.role(), AffineRole::Point),
        }
    }

    fn product(
        left: &KindDimensions,
        right: &KindDimensions,
    ) -> Result<KindDimensions, BridgmanAlgebraError> {
        let grade = graded(left, ProductOp::Mul, right)?;
        Ok(KindDimensions {
            dimensions: &left.dimensions * &right.dimensions,
            grade,
            point: false,
        })
    }

    fn quotient(
        left: &KindDimensions,
        right: &KindDimensions,
    ) -> Result<KindDimensions, BridgmanAlgebraError> {
        let grade = graded(left, ProductOp::Div, right)?;
        Ok(KindDimensions {
            dimensions: &left.dimensions / &right.dimensions,
            grade,
            point: false,
        })
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
        source: Box<QuantityError>,
    },
    /// A point kind whose difference kind Bridgman does not give.
    Difference {
        kind: Declared<'static>,
        source: Box<QuantityError>,
    },
    /// The declared floor cannot be read in the canonical unit.
    Floor {
        kind: Declared<'static>,
        source: Box<QuantityError>,
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
            Self::Difference { kind, source } => {
                write!(
                    formatter,
                    "point kind {kind} has no difference kind: {source}"
                )
            }
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
            Self::Dimensions { source, .. }
            | Self::Difference { source, .. }
            | Self::Floor { source, .. } => Some(source.as_ref()),
            Self::FloorNotExact { .. } | Self::ForeignRegistry { .. } => None,
        }
    }
}
