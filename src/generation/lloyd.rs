//! Lloyd's Relaxation for uniform point distribution
//!
//! Lloyd's Relaxation iteratively improves the uniformity of point distribution
//! on the sphere by moving each point to the centroid of its Voronoi cell.

use glam::Vec3;
use parry3d::math::Point;
use parry3d::transformation;
use std::collections::HashMap;
use std::time::Instant;

/// Type alias for vertex-triangle adjacency map
type VertexTriangleMap = HashMap<usize, Vec<usize>>;

/// Options for Lloyd's relaxation algorithm
#[derive(Debug, Clone, Copy)]
pub struct LloydOptions {
    /// Maximum number of iterations to run
    pub max_iterations: usize,
    /// Convergence threshold - stop when max displacement < this value
    /// Set to 0.0 to disable early termination
    pub convergence_threshold: f32,
}

impl Default for LloydOptions {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            // Threshold relative to radius. For radius=11.3 (Tiny planet), 0.01 means
            // stop when max displacement < 0.113 units. This typically triggers after
            // 3-4 iterations, providing good uniformity with ~40% speedup.
            convergence_threshold: 0.01,
        }
    }
}

/// Apply Lloyd's Relaxation to improve point distribution uniformity
///
/// Lloyd's Relaxation iteratively moves each seed point to the centroid of its
/// Voronoi cell, creating a more uniform, honeycomb-like distribution.
///
/// # Algorithm
///
/// For each iteration:
/// 1. Compute Delaunay triangulation from current points
/// 2. Build vertex-triangle adjacency
/// 3. For each point: calculate centroid of its Voronoi cell
/// 4. Move point to centroid (normalized to sphere surface)
///
/// # Arguments
///
/// * `points` - Initial point distribution
/// * `radius` - Sphere radius
/// * `iterations` - Number of relaxation iterations (typically 3-5)
///
/// # Returns
///
/// Relaxed points with improved uniformity
pub fn lloyd_relaxation(
    points: Vec<Point<f32>>,
    radius: f32,
    iterations: usize,
) -> Vec<Point<f32>> {
    let options = LloydOptions {
        max_iterations: iterations,
        ..Default::default()
    };
    lloyd_relaxation_with_options(points, radius, options)
}

/// Apply Lloyd's Relaxation with custom options
///
/// This variant allows fine-tuned control over convergence detection and
/// maximum iterations. Use `lloyd_relaxation` for the simple interface.
///
/// # Arguments
///
/// * `points` - Initial point distribution
/// * `radius` - Sphere radius
/// * `options` - Relaxation options (max iterations, convergence threshold)
///
/// # Returns
///
/// Relaxed points with improved uniformity
pub fn lloyd_relaxation_with_options(
    mut points: Vec<Point<f32>>,
    radius: f32,
    options: LloydOptions,
) -> Vec<Point<f32>> {
    let convergence_threshold = options.convergence_threshold * radius;
    let total_start = Instant::now();
    let num_points = points.len();

    eprintln!(
        "[Lloyd] Starting: {} points, max {} iterations, threshold {:.4} (abs: {:.4})",
        num_points, options.max_iterations, options.convergence_threshold, convergence_threshold
    );

    let mut iterations_run = 0;
    let mut converged = false;

    for iteration in 0..options.max_iterations {
        let iter_start = Instant::now();

        // Compute convex hull from current points (this is the bottleneck - ~97% of time)
        let hull_start = Instant::now();
        let (vertices, triangle_indices) = transformation::convex_hull(&points);
        let hull_time = hull_start.elapsed();

        // Build vertex-triangle adjacency map
        let map_start = Instant::now();
        let vertex_triangle_map = build_vertex_triangle_map(&triangle_indices);
        let map_time = map_start.elapsed();

        // Calculate new positions with displacement tracking
        let points_start = Instant::now();
        let (new_points, max_displacement) = compute_new_points(
            &vertices,
            &vertex_triangle_map,
            &triangle_indices,
            radius,
        );
        let points_time = points_start.elapsed();

        points = new_points;
        iterations_run = iteration + 1;

        eprintln!(
            "[Lloyd] Iter {}: hull={:?}, map={:?}, points={:?}, total={:?}, max_disp={:.4}",
            iteration + 1,
            hull_time,
            map_time,
            points_time,
            iter_start.elapsed(),
            max_displacement
        );

        // Early exit if converged
        if convergence_threshold > 0.0 && max_displacement < convergence_threshold {
            converged = true;
            eprintln!(
                "[Lloyd] Converged at iteration {} (max_disp {:.4} < threshold {:.4})",
                iteration + 1,
                max_displacement,
                convergence_threshold
            );
            break;
        }
    }

    let total_time = total_start.elapsed();
    eprintln!(
        "[Lloyd] Finished: {} iterations (of max {}), converged={}, total={:?}",
        iterations_run, options.max_iterations, converged, total_time
    );

    points
}

/// Compute new point positions and track maximum displacement
fn compute_new_points(
    vertices: &[Point<f32>],
    vertex_triangle_map: &VertexTriangleMap,
    triangle_indices: &[[u32; 3]],
    radius: f32,
) -> (Vec<Point<f32>>, f32) {
    let mut max_displacement: f32 = 0.0;

    let new_points: Vec<Point<f32>> = (0..vertices.len())
        .map(|vertex_idx| {
            let old_pos = &vertices[vertex_idx];

            // Get all triangles adjacent to this vertex
            let adjacent_triangles = &vertex_triangle_map[&vertex_idx];

            // Compute circumcenters of adjacent triangles
            let circumcenters: Vec<Vec3> = adjacent_triangles
                .iter()
                .map(|&tri_idx| {
                    compute_spherical_circumcenter(tri_idx, vertices, triangle_indices, radius)
                })
                .collect();

            // Calculate centroid (average position)
            let sum: Vec3 = circumcenters.iter().copied().sum();
            let centroid = sum / circumcenters.len() as f32;

            // Normalize back to sphere surface
            let normalized = centroid.normalize() * radius;
            let new_point = Point::new(normalized.x, normalized.y, normalized.z);

            // Track displacement
            let dx = new_point.x - old_pos.x;
            let dy = new_point.y - old_pos.y;
            let dz = new_point.z - old_pos.z;
            let displacement = (dx * dx + dy * dy + dz * dz).sqrt();
            if displacement > max_displacement {
                max_displacement = displacement;
            }

            new_point
        })
        .collect();

    (new_points, max_displacement)
}

/// Build map from vertex index to all triangles that include it
///
/// This adjacency map is essential for finding all triangles adjacent to each seed point.
fn build_vertex_triangle_map(triangle_indices: &[[u32; 3]]) -> VertexTriangleMap {
    let mut map: VertexTriangleMap = HashMap::new();

    for (tri_idx, triangle) in triangle_indices.iter().enumerate() {
        for &vertex_idx in triangle.iter() {
            map.entry(vertex_idx as usize)
                .or_insert_with(Vec::new)
                .push(tri_idx);
        }
    }

    map
}

/// Compute the circumcenter of a spherical triangle
///
/// For a triangle on a sphere, the circumcenter is perpendicular to the triangle's plane.
/// This point lies on the sphere surface and is equidistant from all three triangle vertices.
///
/// # Arguments
///
/// * `tri_idx` - Triangle index in the triangle_indices array
/// * `vertices` - All vertices (points on sphere)
/// * `triangle_indices` - Triangle connectivity
/// * `radius` - Sphere radius
///
/// # Returns
///
/// Circumcenter position on the sphere surface
fn compute_spherical_circumcenter(
    tri_idx: usize,
    vertices: &[Point<f32>],
    triangle_indices: &[[u32; 3]],
    radius: f32,
) -> Vec3 {
    let tri = triangle_indices[tri_idx];

    // Get the three vertices of the triangle
    let v0 = Vec3::new(
        vertices[tri[0] as usize].x,
        vertices[tri[0] as usize].y,
        vertices[tri[0] as usize].z,
    );
    let v1 = Vec3::new(
        vertices[tri[1] as usize].x,
        vertices[tri[1] as usize].y,
        vertices[tri[1] as usize].z,
    );
    let v2 = Vec3::new(
        vertices[tri[2] as usize].x,
        vertices[tri[2] as usize].y,
        vertices[tri[2] as usize].z,
    );

    // Compute edges
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;

    // The circumcenter is perpendicular to the triangle plane
    let normal = edge1.cross(edge2);

    // Normalize and scale to sphere radius
    normal.normalize() * radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::points::generate_sphere_points;

    #[test]
    fn test_lloyd_relaxation() {
        let points = generate_sphere_points(100, 10.0, 42);
        let relaxed = lloyd_relaxation(points, 10.0, 3);

        assert_eq!(relaxed.len(), 100);

        // Verify all relaxed points are on sphere surface
        for point in &relaxed {
            let distance = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
            assert!((distance - 10.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_lloyd_relaxation_determinism() {
        let points1 = generate_sphere_points(50, 10.0, 12345);
        let points2 = generate_sphere_points(50, 10.0, 12345);

        let relaxed1 = lloyd_relaxation(points1, 10.0, 2);
        let relaxed2 = lloyd_relaxation(points2, 10.0, 2);

        // Same input should produce identical output
        assert_eq!(relaxed1.len(), relaxed2.len());
        for (p1, p2) in relaxed1.iter().zip(relaxed2.iter()) {
            assert!((p1.x - p2.x).abs() < 0.0001);
            assert!((p1.y - p2.y).abs() < 0.0001);
            assert!((p1.z - p2.z).abs() < 0.0001);
        }
    }

    #[test]
    fn test_lloyd_relaxation_with_options() {
        let points = generate_sphere_points(100, 10.0, 42);
        let options = LloydOptions {
            max_iterations: 10,
            convergence_threshold: 0.0001,
        };
        let relaxed = lloyd_relaxation_with_options(points, 10.0, options);

        assert_eq!(relaxed.len(), 100);

        // Verify all relaxed points are on sphere surface
        for point in &relaxed {
            let distance = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
            assert!((distance - 10.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_lloyd_options_default() {
        let options = LloydOptions::default();
        assert_eq!(options.max_iterations, 5);
        assert!((options.convergence_threshold - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_lloyd_no_convergence_threshold() {
        // Test with convergence disabled (threshold = 0)
        let points = generate_sphere_points(50, 10.0, 42);
        let options = LloydOptions {
            max_iterations: 3,
            convergence_threshold: 0.0,
        };
        let relaxed = lloyd_relaxation_with_options(points, 10.0, options);

        assert_eq!(relaxed.len(), 50);
    }
}
