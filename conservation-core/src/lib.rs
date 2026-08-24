#![forbid(unsafe_code)]

//! Typed, exact primitives shared by conservation derivation and checking.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
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

impl AxisId {
    /// Creates an axis identifier.
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
