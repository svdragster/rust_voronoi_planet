//! Delaunay triangulation via convex hull
//!
//! For points on a sphere, the Delaunay triangulation is equivalent to the
//! 3D convex hull of those points. This module provides a thin wrapper around
//! parry3d's convex hull algorithm.
//!
//! Note: This module is currently unused but kept for potential future use.

use parry3d::math::Point;
use parry3d::transformation;

/// Result of Delaunay triangulation
///
/// Contains the vertices (which may be reordered from input) and triangle indices.
#[allow(dead_code)]
pub struct DelaunayResult {
    /// Vertices (may be reordered from input)
    pub vertices: Vec<Point<f32>>,
    /// Triangle indices (each [u32; 3] is a triangle)
    pub triangles: Vec<[u32; 3]>,
}

/// Compute Delaunay triangulation of points on sphere via convex hull
///
/// For points on a sphere, the Delaunay triangulation is the same as the
/// convex hull projected onto the sphere surface. This is computationally
/// efficient and numerically stable.
///
/// # Arguments
///
/// * `points` - Points on the sphere surface
///
/// # Returns
///
/// DelaunayResult containing reordered vertices and triangle connectivity
#[allow(dead_code)]
pub fn compute_delaunay(points: &[Point<f32>]) -> DelaunayResult {
    let (vertices, triangles) = transformation::convex_hull(points);
    DelaunayResult { vertices, triangles }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_delaunay() {
        // Create a simple tetrahedron (4 points)
        let points = vec![
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(0.0, 0.0, 1.0),
            Point::new(-1.0, -1.0, -1.0),
        ];

        let result = compute_delaunay(&points);

        // Should have 4 vertices and 4 triangular faces (tetrahedron)
        assert_eq!(result.vertices.len(), 4);
        assert!(result.triangles.len() >= 4);
    }
}
