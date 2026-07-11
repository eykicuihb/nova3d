#[path = "../src/math/vec3.rs"]
mod vec3;

use vec3::Vec3;

#[test]
fn supports_construction_and_arithmetic() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(
        a,
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(a.add(b), Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(a.sub(b), Vec3::new(-3.0, -3.0, -3.0));
    assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
    assert_eq!(a.scale(2.0), Vec3::new(2.0, 4.0, 6.0));
}

#[test]
fn computes_dot_and_cross_products() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(a.dot(b), 32.0);
    assert_eq!(a.cross(b), Vec3::new(-3.0, 6.0, -3.0));
    assert_eq!(a.cross(b).dot(a), 0.0);
    assert_eq!(a.cross(b).dot(b), 0.0);
}

#[test]
fn computes_length_and_normalizes_nonzero_vectors() {
    let vector = Vec3::new(3.0, 4.0, 0.0);

    assert_eq!(vector.length(), 5.0);

    let normalized = vector.normalize();
    assert!((normalized.x - 0.6).abs() < f32::EPSILON);
    assert!((normalized.y - 0.8).abs() < f32::EPSILON);
    assert_eq!(normalized.z, 0.0);
    assert!((normalized.length() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn normalizing_zero_returns_zero_without_nan() {
    let normalized = Vec3::ZERO.normalize();

    assert_eq!(normalized, Vec3::ZERO);
    assert!(!normalized.x.is_nan());
    assert!(!normalized.y.is_nan());
    assert!(!normalized.z.is_nan());
}
