//! Spatial indexing for fast position-to-cell lookups
//!
//! This module is only available with the `spatial-index` feature.

#[cfg(feature = "spatial-index")]
use kiddo::immutable::float::kdtree::ImmutableKdTree;
#[cfg(feature = "spatial-index")]
use kiddo::SquaredEuclidean;
#[cfg(feature = "spatial-index")]
use glam::Vec3;

/// Wrapper around KD-tree for spatial queries
///
/// Provides O(log n) nearest-neighbor lookups to convert 3D positions
/// into cell IDs. This is essential for raycasting, unit placement,
/// and position queries.
///
/// # Performance
///
/// - Construction: O(n log n), ~50-200ms for 5K-26K cells
/// - Query: O(log n), extremely fast (~15 comparisons for 26K cells)
/// - Memory: ~24 bytes per cell
#[cfg(feature = "spatial-index")]
#[derive(Clone)]
pub struct SpatialIndex {
    tree: ImmutableKdTree<f32, usize, 3, 32>,
}

#[cfg(feature = "spatial-index")]
impl SpatialIndex {
    /// Build spatial index from cell centers
    ///
    /// Creates an immutable KD-tree from the provided cell center positions.
    /// This is called once during planet generation.
    ///
    /// # Arguments
    ///
    /// * `centers` - Slice of Vec3 positions representing cell centers
    ///
    /// # Example
    ///
    /// ```
    /// use rust_voronoi_planet::*;
    /// use glam::Vec3;
    ///
    /// # #[cfg(feature = "spatial-index")]
    /// # {
    /// let centers = vec![
    ///     Vec3::new(1.0, 0.0, 0.0),
    ///     Vec3::new(0.0, 1.0, 0.0),
    ///     Vec3::new(0.0, 0.0, 1.0),
    /// ];
    ///
    /// let index = SpatialIndex::new(&centers);
    /// let cell_id = index.find_nearest(Vec3::new(1.0, 0.1, 0.0));
    /// assert_eq!(cell_id, 0); // Closest to first center
    /// # }
    /// ```
    pub fn new(centers: &[Vec3]) -> Self {
        // Convert Vec3 to [f32; 3] array format for kiddo
        let points: Vec<[f32; 3]> = centers
            .iter()
            .map(|c| [c.x, c.y, c.z])
            .collect();

        Self {
            tree: ImmutableKdTree::new_from_slice(&points),
        }
    }

    /// Find the nearest cell to a position
    ///
    /// Uses KD-tree nearest-neighbor search to find which cell contains
    /// the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position to query
    ///
    /// # Returns
    ///
    /// Cell ID (index) of the nearest cell
    ///
    /// # Performance
    ///
    /// O(log n) lookup, extremely fast even for large planets.
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # use glam::Vec3;
    /// # #[cfg(feature = "spatial-index")]
    /// # {
    /// # let centers = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
    /// # let index = SpatialIndex::new(&centers);
    /// let position = Vec3::new(0.9, 0.1, 0.0);
    /// let cell_id = index.find_nearest(position);
    /// // cell_id is the index of the closest cell center
    /// # }
    /// ```
    pub fn find_nearest(&self, position: Vec3) -> usize {
        let query = [position.x, position.y, position.z];
        let result = self.tree.nearest_one::<SquaredEuclidean>(&query);
        result.item as usize
    }
}

#[cfg(test)]
#[cfg(feature = "spatial-index")]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_index_basic() {
        let centers = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, 0.0, 0.0),
        ];

        let index = SpatialIndex::new(&centers);

        // Query near first center
        let result = index.find_nearest(Vec3::new(0.9, 0.1, 0.0));
        assert_eq!(result, 0);

        // Query near second center
        let result = index.find_nearest(Vec3::new(0.0, 0.95, 0.0));
        assert_eq!(result, 1);

        // Query near third center
        let result = index.find_nearest(Vec3::new(0.0, 0.1, 0.9));
        assert_eq!(result, 2);

        // Query near fourth center
        let result = index.find_nearest(Vec3::new(-0.8, 0.0, 0.0));
        assert_eq!(result, 3);
    }

    #[test]
    fn test_spatial_index_exact_match() {
        let centers = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 10.0, 0.0),
        ];

        let index = SpatialIndex::new(&centers);

        // Query at exact center positions
        let result = index.find_nearest(centers[0]);
        assert_eq!(result, 0);

        let result = index.find_nearest(centers[1]);
        assert_eq!(result, 1);
    }
}
