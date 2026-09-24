use conservation_core::{Affine, DimensionAlgebra, Kind, KindRegistry};
use conservation_test_kinds::{TestDimensions, TestKind, TestKinds};
use num_bigint::BigInt;
use num_rational::BigRational;

fn q(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

#[test]
fn kinds_report_affine_role_and_floor() {
    assert_eq!(
        TestKind::Temperature.affine(),
        Affine::Point {
            difference: TestKind::TemperatureDelta
        }
    );
    assert_eq!(
        TestKind::Temperature.difference(),
        TestKind::TemperatureDelta
    );
    assert_eq!(TestKind::Material.floor(), Some(q(0)));
    assert_eq!(TestKind::MaterialBalance.floor(), None);
    assert_eq!(TestKind::Material.affine(), Affine::Linear);
    assert_eq!(TestKind::Material.difference(), TestKind::Material);
    assert_eq!(
        TestKind::Enthalpy.affine(),
        Affine::Point {
            difference: TestKind::Heat
        }
    );
    assert_eq!(TestKind::Enthalpy.floor(), None);
    assert_eq!(TestKind::Heat.floor(), None);
    assert_eq!(TestKind::Reserve.floor(), Some(q(10)));
}

#[test]
fn registry_round_trips_every_name() {
    for kind in TestKind::ALL {
        assert_eq!(TestKinds.resolve(&kind.to_string()), Some(kind));
    }
    assert_eq!(TestKinds.resolve(""), None);
    assert_eq!(TestKinds.resolve("misspelled"), None);
}

#[test]
fn declaration_order_is_name_order() {
    let mut sorted = TestKind::ALL;
    sorted.sort_by_key(|kind| kind.name());
    assert_eq!(sorted, TestKind::ALL);
    let mut ordered = TestKind::ALL;
    ordered.sort();
    assert_eq!(ordered, TestKind::ALL);
}

#[test]
fn energy_and_torque_share_dimensions_but_are_distinct_kinds() {
    assert_eq!(TestKind::Energy.dimensions(), TestKind::Torque.dimensions());
    assert_ne!(TestKind::Energy, TestKind::Torque);
}

#[test]
fn dimensions_multiply_divide_and_display() {
    let energy = TestKind::Energy.dimensions();
    let material = TestKind::Material.dimensions();
    assert_eq!(
        TestKind::quotient(&energy, &material),
        Ok(TestKind::EnergyPerMaterial.dimensions())
    );
    assert_eq!(
        TestKind::product(&material, &material),
        Ok(TestKind::MaterialSquared.dimensions())
    );
    assert_eq!(
        TestKind::EnergyPerMaterial.dimensions().to_string(),
        "energy^1·material^-1"
    );
    assert_eq!(TestKind::Ratio.dimensions().to_string(), "1");
    let huge = TestDimensions([i32::MAX, 0, 0, 0, 0, 0, 0]);
    assert!(TestKind::product(&huge, &TestKind::Amount.dimensions()).is_err());
}
