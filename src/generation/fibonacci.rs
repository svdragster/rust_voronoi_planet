//! Fibonacci Lattice Point Distribution
//!
//! Generates near-uniform point distributions on a sphere using the Fibonacci
//! spiral (golden angle) method with added jitter for natural-looking Voronoi cells.
//!
//! # Algorithm
//!
//! The Fibonacci lattice uses the golden ratio to create a spiral pattern that
//! naturally avoids clustering. Points are placed at:
//! - Longitude: `2π * i / φ` (golden angle increments)
//! - Latitude: Evenly spaced in z-coordinate with pole offset
//!
//! Small random jitter is added to break up the regular spiral pattern,
//! producing more natural-looking Voronoi cells while maintaining uniformity.
//!
//! # References
//!
//! - [Fibonacci Lattice Optimization](https://extremelearning.com.au/how-to-evenly-distribute-points-on-a-sphere-more-effectively-than-the-canonical-fibonacci-lattice/)

use glam::Vec3;
use parry3d::math::Point;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use std::f32::consts::PI;

/// Golden ratio φ = (1 + √5) / 2
const PHI: f32 = 1.618033988749895;

/// Jitter strength as fraction of average cell spacing
/// 0.0 = no jitter (pure Fibonacci spiral)
/// 0.5 = strong jitter (breaks up spiral pattern well)
const JITTER_STRENGTH: f32 = 0.5;

/// Get the optimal epsilon offset for a given point count
///
/// The epsilon parameter offsets points away from the poles, improving
/// uniformity by up to 8.3% compared to the canonical Fibonacci lattice.
fn get_epsilon(n: usize) -> f32 {
    match n {
        0..=23 => 0.33,
        24..=176 => 1.33,
        177..=889 => 3.33,
        890..=10999 => 10.0,
        _ => 27.5,
    }
}

/// Generate points on a sphere using the Fibonacci lattice (golden spiral) with jitter
///
/// This method produces a near-uniform distribution in O(n) time, making it
/// significantly faster than random points + Lloyd's relaxation for large
/// point counts. Small random jitter is added to break up the regular spiral
/// pattern and produce natural-looking Voronoi cells.
///
/// # Arguments
///
/// * `count` - Number of points to generate
/// * `radius` - Sphere radius
/// * `seed` - Random seed for deterministic jitter
///
/// # Returns
///
/// Vector of points distributed on the sphere surface
///
/// # Example
///
/// ```rust
/// use rust_voronoi_planet::generation::generate_fibonacci_sphere_points;
///
/// let points = generate_fibonacci_sphere_points(1000, 10.0, 42);
/// assert_eq!(points.len(), 1000);
/// ```
pub fn generate_fibonacci_sphere_points(count: usize, radius: f32, seed: u32) -> Vec<Point<f32>> {
    if count == 0 {
        return Vec::new();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    let epsilon = get_epsilon(count);
    let n = count as f32;

    // Average angular spacing between points (approximate)
    let avg_spacing = (4.0 * PI / n).sqrt();
    let jitter_amount = avg_spacing * JITTER_STRENGTH;

    (0..count)
        .map(|i| {
            let i_f = i as f32;

            // Golden angle increment for longitude
            let theta = 2.0 * PI * i_f / PHI;

            // Latitude with epsilon offset for better pole distribution
            // Maps i from [0, n-1] to cos(phi) from [1, -1] with offset
            let cos_phi = 1.0 - 2.0 * (i_f + epsilon) / (n - 1.0 + 2.0 * epsilon);
            let sin_phi = (1.0 - cos_phi * cos_phi).sqrt();

            // Base position on sphere
            let base = Vec3::new(
                sin_phi * theta.cos(),
                sin_phi * theta.sin(),
                cos_phi,
            );

            // Add tangential jitter (perpendicular to radius)
            // Generate random offset in tangent plane
            let jitter_theta: f32 = rng.gen_range(0.0..2.0 * PI);
            let jitter_mag: f32 = rng.gen_range(0.0..jitter_amount);

            // Create orthonormal basis for tangent plane
            let up = if base.z.abs() < 0.9 {
                Vec3::Z
            } else {
                Vec3::X
            };
            let tangent1 = base.cross(up).normalize();
            let tangent2 = base.cross(tangent1).normalize();

            // Apply jitter in tangent plane
            let jittered = base
                + tangent1 * jitter_mag * jitter_theta.cos()
                + tangent2 * jitter_mag * jitter_theta.sin();

            // Normalize back to sphere surface
            let normalized = jittered.normalize() * radius;

            Point::new(normalized.x, normalized.y, normalized.z)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_point_count() {
        for count in [10, 100, 1000, 5000] {
            let points = generate_fibonacci_sphere_points(count, 10.0, 42);
            assert_eq!(points.len(), count);
        }
    }

    #[test]
    fn test_fibonacci_empty() {
        let points = generate_fibonacci_sphere_points(0, 10.0, 42);
        assert!(points.is_empty());
    }

    #[test]
    fn test_fibonacci_points_on_sphere() {
        let radius = 15.0;
        let points = generate_fibonacci_sphere_points(500, radius, 42);

        for point in &points {
            let distance = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
            assert!(
                (distance - radius).abs() < 0.0001,
                "Point distance {} should be {} (diff: {})",
                distance,
                radius,
                (distance - radius).abs()
            );
        }
    }

    #[test]
    fn test_fibonacci_determinism() {
        // Same seed should produce identical results
        let points1 = generate_fibonacci_sphere_points(100, 10.0, 42);
        let points2 = generate_fibonacci_sphere_points(100, 10.0, 42);

        for (p1, p2) in points1.iter().zip(points2.iter()) {
            assert!((p1.x - p2.x).abs() < 0.0001);
            assert!((p1.y - p2.y).abs() < 0.0001);
            assert!((p1.z - p2.z).abs() < 0.0001);
        }
    }

    #[test]
    fn test_fibonacci_different_seeds() {
        // Different seeds should produce different jitter
        let points1 = generate_fibonacci_sphere_points(100, 10.0, 12345);
        let points2 = generate_fibonacci_sphere_points(100, 10.0, 67890);

        // At least some points should differ (due to jitter)
        let mut any_different = false;
        for (p1, p2) in points1.iter().zip(points2.iter()) {
            if (p1.x - p2.x).abs() > 0.01 {
                any_different = true;
                break;
            }
        }
        assert!(any_different, "Different seeds should produce different points");
    }

    #[test]
    fn test_fibonacci_has_poles() {
        // First and last points should be near the poles
        let points = generate_fibonacci_sphere_points(1000, 10.0, 42);

        // First point near north pole (z close to radius)
        let first = &points[0];
        assert!(first.z > 9.0, "First point z={} should be near north pole", first.z);

        // Last point near south pole (z close to -radius)
        let last = &points[999];
        assert!(last.z < -9.0, "Last point z={} should be near south pole", last.z);
    }

    #[test]
    fn test_epsilon_ranges() {
        assert_eq!(get_epsilon(10), 0.33);
        assert_eq!(get_epsilon(23), 0.33);
        assert_eq!(get_epsilon(24), 1.33);
        assert_eq!(get_epsilon(176), 1.33);
        assert_eq!(get_epsilon(177), 3.33);
        assert_eq!(get_epsilon(889), 3.33);
        assert_eq!(get_epsilon(890), 10.0);
        assert_eq!(get_epsilon(5000), 10.0);
        assert_eq!(get_epsilon(10999), 10.0);
        assert_eq!(get_epsilon(11000), 27.5);
        assert_eq!(get_epsilon(50000), 27.5);
    }
}
