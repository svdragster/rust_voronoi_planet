//! Core Voronoi generation algorithm
//!
//! Generates Voronoi cells on a sphere surface using Lloyd's relaxation
//! and Delaunay triangulation via convex hull.

mod points;
mod delaunay;
mod lloyd;
mod voronoi;

pub use points::generate_sphere_points;
pub use lloyd::{lloyd_relaxation, lloyd_relaxation_with_options, LloydOptions};
pub use voronoi::{generate_cells, RawCell};

use crate::config::PlanetConfig;
use crate::error::Result;

/// Generate raw Voronoi cells from configuration (without terrain)
///
/// Returns cells with geometry only (center, vertices, neighbors).
/// Terrain must be sampled separately.
pub fn generate_raw_cells(config: &PlanetConfig) -> Result<Vec<RawCell>> {
    let radius = config.radius();
    let cell_count = config.cell_count();

    // Step 1: Generate random points on sphere
    let points = points::generate_sphere_points(cell_count, radius, config.seed);

    // Step 2: Apply Lloyd's relaxation with convergence detection
    let points = if config.lloyd_iterations > 0 {
        let options = LloydOptions {
            max_iterations: config.lloyd_iterations,
            convergence_threshold: config.lloyd_convergence,
        };
        lloyd::lloyd_relaxation_with_options(points, radius, options)
    } else {
        points
    };

    // Step 3-5: Generate cells from points
    voronoi::generate_cells(&points, radius)
}
