#![forbid(unsafe_code)]

//! Typed, exact primitives shared by conservation derivation and checking.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::sync::LazyLock;

use num_rational::BigRational;
use num_traits::Zero;

static ZERO: LazyLock<BigRational> = LazyLock::new(BigRational::zero);

/// Identifies one coordinate of a quantitative state.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AxisId(String);

/// An error constructing a typed identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// The supplied identifier was empty or contained only whitespace.
    Blank,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blank => formatter.write_str("identifier must contain non-whitespace text"),
        }
    }
}

impl Error for IdentifierError {}

/// Rejects empty or whitespace-only identifiers. The one declaration of that rule.
pub fn nonblank(value: &str) -> Result<(), IdentifierError> {
    if value.trim().is_empty() {
        Err(IdentifierError::Blank)
    } else {
        Ok(())
    }
}

/// How a kind's values combine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Affine<K> {
    /// Values and their differences share this kind (mass, energy, money).
    Linear,
    /// Values are points; the difference of two points has kind `difference`
    /// (temperature and temperature_delta).
    Point { difference: K },
}

/// A conserved kind: a small `Copy` handle into a registry shared by every stock,
/// flow and law that names it. `Display` writes the kind's registry name, the
/// identity [`KindRegistry::resolve`] accepts.
pub trait Kind: Copy + Eq + Ord + Hash + fmt::Debug + fmt::Display {
    /// Whether this is a point kind, and which kind its differences have.
    fn affine(self) -> Affine<Self>;
    /// The least coordinate a stock of this kind may hold, in the kind's chart;
    /// `None` when the kind has no floor.
    fn floor(self) -> Option<BigRational>;
    /// The kind of a difference of two values of this kind.
    fn difference(self) -> Self {
        match self.affine() {
            Affine::Linear => self,
            Affine::Point { difference } => difference,
        }
    }
}

/// Dimensional comparison, product and quotient. Only conservation-exchange requires it.
pub trait DimensionAlgebra: Kind {
    /// A kind's dimensions. Equality is dimensional comparison.
    type Dimensions: Clone + Eq + fmt::Debug + fmt::Display;
    /// Why a product or quotient has no representation (for example exponent overflow).
    type AlgebraError: Error + Clone + Eq;
    /// Returns this kind's dimensions.
    fn dimensions(self) -> Self::Dimensions;
    /// Returns the dimensions of a product.
    fn product(
        left: &Self::Dimensions,
        right: &Self::Dimensions,
    ) -> Result<Self::Dimensions, Self::AlgebraError>;
    /// Returns the dimensions of a quotient.
    fn quotient(
        left: &Self::Dimensions,
        right: &Self::Dimensions,
    ) -> Result<Self::Dimensions, Self::AlgebraError>;
}

/// Resolves kind names once, at a document or foreign-language boundary.
pub trait KindRegistry {
    /// The handle type this registry resolves names to.
    type Kind: Kind;
    /// The handle whose `Display` is `name`, if the registry declares it.
    fn resolve(&self, name: &str) -> Option<Self::Kind>;
}

impl AxisId {
    /// Creates an axis identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        nonblank(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AxisId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Identifies the physical or logical kind measured by a law.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KindId(String);

impl KindId {
    /// Creates a kind identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(IdentifierError::Blank);
        }
        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KindId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Records an asserted origin for a conservation law.
///
/// This is metadata, not a certificate that the law was correctly derived or
/// that any trace satisfies it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Provenance {
    /// Derived from the left nullspace of an incidence matrix.
    IncidenceNullspace,
    /// Derived from the left nullspace of a stoichiometric matrix.
    StoichiometricNullspace,
    /// Required as an externally forced balance.
    ForcedBalance,
    /// Declared directly by a model author.
    Declared,
    /// Tagged as Noether-derived; this crate does not implement that derivation.
    Noether,
}

/// A canonical exact linear balance law over named axes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BalanceLaw {
    kind: KindId,
    coefficients: BTreeMap<AxisId, BigRational>,
    provenance: Provenance,
}

/// An error constructing a balance law.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BalanceLawError {
    /// No nonzero coefficient remains after exact canonicalization.
    Empty,
}

impl fmt::Display for BalanceLawError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("balance law must contain a nonzero coefficient"),
        }
    }
}

impl Error for BalanceLawError {}

impl BalanceLaw {
    /// Constructs a law, combining repeated axes and discarding exact zero terms.
    pub fn new(
        kind: KindId,
        coefficients: impl IntoIterator<Item = (AxisId, BigRational)>,
        provenance: Provenance,
    ) -> Result<Self, BalanceLawError> {
        let mut canonical = BTreeMap::<AxisId, BigRational>::new();
        for (axis, coefficient) in coefficients {
            *canonical.entry(axis).or_default() += coefficient;
        }
        canonical.retain(|_, coefficient| !coefficient.is_zero());

        if canonical.is_empty() {
            return Err(BalanceLawError::Empty);
        }

        Ok(Self {
            kind,
            coefficients: canonical,
            provenance,
        })
    }

    /// Returns the kind conserved by this law.
    pub fn kind(&self) -> &KindId {
        &self.kind
    }

    /// Returns an axis coefficient, or exact zero when the axis is absent.
    pub fn coefficient(&self, axis: &AxisId) -> &BigRational {
        self.coefficients.get(axis).unwrap_or(&ZERO)
    }

    /// Iterates through nonzero coefficients in deterministic axis order.
    pub fn coefficients(&self) -> impl ExactSizeIterator<Item = (&AxisId, &BigRational)> {
        self.coefficients.iter()
    }

    /// Returns asserted origin metadata, which is not a correctness certificate.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}

/// How a graded sentence reads its exact linear form along a finite trace.
///
/// Every grade interprets the same canonical carrier, a [`BalanceLaw`], so
/// sentence translation is identical for all grades: rename the form, keep the
/// grade. Only satisfaction differs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Grade {
    /// The form's value is identical in every state.
    Invariant,
    /// The form's value is at least zero in every state.
    Nonnegative,
    /// The form's value never decreases between consecutive states.
    Nondecreasing,
}

impl fmt::Display for Grade {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Invariant => "invariant",
            Self::Nonnegative => "nonnegative",
            Self::Nondecreasing => "nondecreasing",
        })
    }
}

/// A graded sentence: one exact linear form read under one grade.
///
/// [`Grade::Invariant`] recovers the classic balance sentence. The other
/// grades reuse the same form as an inequality along the trace: sign and
/// authority constraints are [`Grade::Nonnegative`] sentences, and monotone
/// dissipation axes are [`Grade::Nondecreasing`] sentences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GradedLaw {
    form: BalanceLaw,
    grade: Grade,
}

impl GradedLaw {
    /// Constructs a graded sentence over an already-canonical form.
    pub fn new(form: BalanceLaw, grade: Grade) -> Self {
        Self { form, grade }
    }

    /// Returns the exact linear form this sentence reads.
    pub fn form(&self) -> &BalanceLaw {
        &self.form
    }

    /// Returns how the form is read along a trace.
    pub fn grade(&self) -> Grade {
        self.grade
    }
}

impl From<BalanceLaw> for GradedLaw {
    /// Reads a balance law under its classic grade, [`Grade::Invariant`].
    fn from(form: BalanceLaw) -> Self {
        Self::new(form, Grade::Invariant)
    }
}
