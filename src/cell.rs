//! Voronoi Cell Structure
//!
//! Represents an individual cell on the Voronoi planet with terrain, neighbors, and geometry.

use glam::Vec3;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single Voronoi cell on the planet surface
///
/// Each cell represents a discrete region of the planet with:
/// - A unique ID for identification
/// - A center point for positioning entities
/// - Terrain type (generic T) for gameplay rules
/// - Neighbor connectivity for pathfinding
/// - Vertices for rendering the cell boundary
///
/// # Design Notes
///
/// Cells are NOT serialized individually. They are regenerated from PlanetConfig
/// when loading a save file, ensuring consistency and compact save files.
///
/// # Memory Usage
///
/// Approximate size per cell:
/// - id: 8 bytes (usize)
/// - center: 12 bytes (Vec3)
/// - terrain: sizeof(T) bytes
/// - neighbors: ~48 bytes (`Vec<usize>` with ~6 neighbors avg)
/// - vertices: ~72 bytes (`Vec<Vec3>` with ~6 vertices avg)
/// - **Total: ~140 bytes + sizeof(T) per cell**
///
/// For 26,000 cells (Large planet): ~3.6 MB in RAM (+ terrain data)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VoronoiCell<T> {
    /// Unique identifier for this cell (0 to cell_count-1)
    ///
    /// Cell IDs are stable and deterministic - the same configuration
    /// will always produce the same cell IDs in the same positions.
    pub id: usize,

    /// Center point of the cell on the sphere surface
    ///
    /// This is where units, cities, and buildings are positioned when placed in this cell.
    /// The center is computed as the centroid of the cell's Voronoi region.
    pub center: Vec3,

    /// Terrain type of this cell
    ///
    /// Sampled from 3D Perlin noise at the cell's center during generation.
    /// Determines movement costs, building placement rules, and habitability.
    pub terrain: T,

    /// IDs of adjacent cells (neighbors in the Voronoi graph)
    ///
    /// Cells are neighbors if they share an edge on the Voronoi diagram.
    /// Average cell has ~6 neighbors (hexagonal-like distribution after Lloyd's Relaxation).
    ///
    /// Used for:
    /// - A* pathfinding (graph edges)
    /// - Territory expansion
    /// - Flood-fill algorithms
    pub neighbors: Vec<usize>,

    /// Vertices defining the cell's boundary polygon (for rendering)
    ///
    /// Ordered counter-clockwise around the cell center.
    /// These are the circumcenters of the Delaunay triangles adjacent to this cell's seed point.
    ///
    /// Used for:
    /// - Rendering cell boundaries
    /// - Highlighting selected cells
    /// - Visualizing territories
    pub vertices: Vec<Vec3>,
}

impl<T> VoronoiCell<T> {
    /// Create a new Voronoi cell
    ///
    /// This is typically called during planet generation, not by user code.
    pub fn new(
        id: usize,
        center: Vec3,
        terrain: T,
        neighbors: Vec<usize>,
        vertices: Vec<Vec3>,
    ) -> Self {
        Self {
            id,
            center,
            terrain,
            neighbors,
            vertices,
        }
    }

    /// Get the number of neighboring cells
    ///
    /// Typically 5-7 for cells after Lloyd's Relaxation (hexagonal-like),
    /// but can vary especially at poles or with less relaxation.
    #[inline]
    pub fn neighbor_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Check if this cell is a neighbor of another cell
    #[inline]
    pub fn is_neighbor_of(&self, other_cell_id: usize) -> bool {
        self.neighbors.contains(&other_cell_id)
    }

    /// Get the vertex count (polygon complexity)
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Calculate the approximate surface area of this cell
    ///
    /// Uses the spherical polygon area formula. This is an approximation
    /// suitable for gameplay purposes, not a precise geometric calculation.
    pub fn approximate_area(&self) -> f32 {
        if self.vertices.len() < 3 {
            return 0.0;
        }

        // Simple approximation: treat as flat polygon projected onto sphere
        // Real formula would use spherical excess, but this is sufficient for gameplay
        let mut area = 0.0;
        for i in 0..self.vertices.len() {
            let v1 = self.vertices[i];
            let v2 = self.vertices[(i + 1) % self.vertices.len()];

            // Cross product gives area of triangle (center, v1, v2)
            let triangle_vec1 = v1 - self.center;
            let triangle_vec2 = v2 - self.center;
            area += triangle_vec1.cross(triangle_vec2).length() * 0.5;
        }

        area
    }

    /// Get distance to another cell (great circle distance between centers)
    ///
    /// Returns the arc distance along the sphere surface, not Euclidean distance.
    pub fn distance_to(&self, other: &VoronoiCell<T>, sphere_radius: f32) -> f32 {
        // Use arc distance formula: d = R * arccos(dot(v1, v2) / (|v1| * |v2|))
        let dot = self.center.dot(other.center);
        let cos_angle = dot / (self.center.length() * other.center.length());

        // Clamp to avoid numerical issues with acos
        let cos_angle = cos_angle.clamp(-1.0, 1.0);

        sphere_radius * cos_angle.acos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple terrain type for testing
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum TestTerrain {
        Ocean,
        Grassland,
    }

    #[test]
    fn test_cell_creation() {
        let cell = VoronoiCell::new(
            0,
            Vec3::new(1.0, 0.0, 0.0),
            TestTerrain::Grassland,
            vec![1, 2, 3],
            vec![
                Vec3::new(1.0, 0.1, 0.1),
                Vec3::new(1.0, 0.1, -0.1),
                Vec3::new(1.0, -0.1, 0.0),
            ],
        );

        assert_eq!(cell.id, 0);
        assert_eq!(cell.neighbor_count(), 3);
        assert_eq!(cell.vertex_count(), 3);
        assert!(cell.is_neighbor_of(1));
        assert!(!cell.is_neighbor_of(99));
    }

    #[test]
    fn test_approximate_area() {
        let cell = VoronoiCell::new(
            0,
            Vec3::new(10.0, 0.0, 0.0),
            TestTerrain::Ocean,
            vec![],
            vec![
                Vec3::new(10.0, 1.0, 0.0),
                Vec3::new(10.0, 0.0, 1.0),
                Vec3::new(10.0, -1.0, 0.0),
                Vec3::new(10.0, 0.0, -1.0),
            ],
        );

        let area = cell.approximate_area();
        assert!(area > 0.0);
        assert!(area < 10.0); // Should be reasonable
    }

    #[test]
    fn test_distance_to() {
        let cell1 = VoronoiCell::new(
            0,
            Vec3::new(10.0, 0.0, 0.0),
            TestTerrain::Ocean,
            vec![],
            vec![],
        );

        let cell2 = VoronoiCell::new(
            1,
            Vec3::new(0.0, 10.0, 0.0),
            TestTerrain::Grassland,
            vec![],
            vec![],
        );

        let distance = cell1.distance_to(&cell2, 10.0);
        // 90 degree arc on sphere with radius 10
        let expected = 10.0 * std::f32::consts::FRAC_PI_2;
        assert!((distance - expected).abs() < 0.01);
    }
}
