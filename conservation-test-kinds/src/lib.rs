#![forbid(unsafe_code)]

//! Test-only kinds. Fixtures, not physics: floors and dimensions are chosen to keep
//! conservation's tests Bridgman-free and their meaning unchanged.

use std::fmt;

use conservation_core::{Affine, DimensionAlgebra, Kind, KindRegistry};
use num_rational::BigRational;
use num_traits::Zero;

/// Variants are declared in name order, so `Ord` equals the former `KindId` string order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TestKind {
    Amount,
    Charge,
    Energy,
    EnergyPerMaterial,
    Entropy,
    Material,
    MaterialBalance,
    MaterialSquared,
    Matter,
    Ratio,
    Temperature,
    TemperatureDelta,
    Torque,
}

impl TestKind {
    /// Every variant in declaration order.
    pub const ALL: [TestKind; 13] = [
        Self::Amount,
        Self::Charge,
        Self::Energy,
        Self::EnergyPerMaterial,
        Self::Entropy,
        Self::Material,
        Self::MaterialBalance,
        Self::MaterialSquared,
        Self::Matter,
        Self::Ratio,
        Self::Temperature,
        Self::TemperatureDelta,
        Self::Torque,
    ];

    /// "amount", "charge", "energy", "energy_per_material", "entropy", "material",
    /// "material_balance", "material_squared", "matter", "ratio", "temperature",
    /// "temperature_delta", "torque".
    pub const fn name(self) -> &'static str {
        match self {
            Self::Amount => "amount",
            Self::Charge => "charge",
            Self::Energy => "energy",
            Self::EnergyPerMaterial => "energy_per_material",
            Self::Entropy => "entropy",
            Self::Material => "material",
            Self::MaterialBalance => "material_balance",
            Self::MaterialSquared => "material_squared",
            Self::Matter => "matter",
            Self::Ratio => "ratio",
            Self::Temperature => "temperature",
            Self::TemperatureDelta => "temperature_delta",
            Self::Torque => "torque",
        }
    }
}

impl fmt::Display for TestKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl Kind for TestKind {
    fn affine(self) -> Affine<Self> {
        match self {
            Self::Temperature => Affine::Point {
                difference: Self::TemperatureDelta,
            },
            Self::Amount
            | Self::Charge
            | Self::Energy
            | Self::EnergyPerMaterial
            | Self::Entropy
            | Self::Material
            | Self::MaterialBalance
            | Self::MaterialSquared
            | Self::Matter
            | Self::Ratio
            | Self::TemperatureDelta
            | Self::Torque => Affine::Linear,
        }
    }

    fn floor(self) -> Option<BigRational> {
        match self {
            Self::Amount | Self::Energy | Self::Material | Self::Matter | Self::Temperature => {
                Some(BigRational::zero())
            }
            Self::Charge
            | Self::EnergyPerMaterial
            | Self::Entropy
            | Self::MaterialBalance
            | Self::MaterialSquared
            | Self::Ratio
            | Self::TemperatureDelta
            | Self::Torque => None,
        }
    }
}

/// The base names, in the order of [`TestDimensions`] exponents.
const BASES: [&str; 7] = [
    "amount",
    "charge",
    "energy",
    "entropy",
    "material",
    "matter",
    "temperature",
];

/// Exponents over the bases [amount, charge, energy, entropy, material, matter, temperature].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TestDimensions(pub [i32; 7]);

impl TestDimensions {
    fn base(index: usize) -> Self {
        let mut powers = [0; 7];
        powers[index] = 1;
        Self(powers)
    }

    fn combine(
        left: &Self,
        right: &Self,
        operation: fn(i32, i32) -> Option<i32>,
    ) -> Result<Self, TestDimensionOverflow> {
        let mut powers = [0; 7];
        for (index, power) in powers.iter_mut().enumerate() {
            *power = operation(left.0[index], right.0[index]).ok_or(TestDimensionOverflow)?;
        }
        Ok(Self(powers))
    }
}

impl fmt::Display for TestDimensions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (base, power) in BASES.iter().zip(self.0) {
            if power == 0 {
                continue;
            }
            if !first {
                formatter.write_str("·")?;
            }
            write!(formatter, "{base}^{power}")?;
            first = false;
        }
        if first {
            formatter.write_str("1")?;
        }
        Ok(())
    }
}

/// A product or quotient whose exponent does not fit an `i32`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TestDimensionOverflow;

impl fmt::Display for TestDimensionOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("test dimension exponent overflow")
    }
}

impl std::error::Error for TestDimensionOverflow {}

impl DimensionAlgebra for TestKind {
    type Dimensions = TestDimensions;
    type AlgebraError = TestDimensionOverflow;

    fn dimensions(self) -> TestDimensions {
        match self {
            Self::Amount => TestDimensions::base(0),
            Self::Charge => TestDimensions::base(1),
            Self::Energy | Self::Torque => TestDimensions::base(2),
            Self::Entropy => TestDimensions::base(3),
            Self::Material | Self::MaterialBalance => TestDimensions::base(4),
            Self::Matter => TestDimensions::base(5),
            Self::Temperature | Self::TemperatureDelta => TestDimensions::base(6),
            Self::Ratio => TestDimensions([0; 7]),
            Self::EnergyPerMaterial => TestDimensions([0, 0, 1, 0, -1, 0, 0]),
            Self::MaterialSquared => TestDimensions([0, 0, 0, 0, 2, 0, 0]),
        }
    }

    fn product(
        left: &TestDimensions,
        right: &TestDimensions,
    ) -> Result<TestDimensions, TestDimensionOverflow> {
        TestDimensions::combine(left, right, i32::checked_add)
    }

    fn quotient(
        left: &TestDimensions,
        right: &TestDimensions,
    ) -> Result<TestDimensions, TestDimensionOverflow> {
        TestDimensions::combine(left, right, i32::checked_sub)
    }
}

/// The registry of every [`TestKind`], resolving each by its name.
pub struct TestKinds;

impl KindRegistry for TestKinds {
    type Kind = TestKind;

    fn resolve(&self, name: &str) -> Option<TestKind> {
        TestKind::ALL.into_iter().find(|kind| kind.name() == name)
    }
}
