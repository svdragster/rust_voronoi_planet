//! VoronoiPlanet main structure

use crate::cell::VoronoiCell;
use crate::config::PlanetConfig;
use crate::error::Result;
use crate::generation::generate_raw_cells;
use crate::terrain::{TerrainSampler, BasicTerrainType, PerlinTerrainSampler};

#[cfg(feature = "spatial-index")]
use crate::spatial::SpatialIndex;
#[cfg(feature = "spatial-index")]
use glam::Vec3;

/// A complete Voronoi-tessellated planet
///
/// Generic over terrain type `T` for maximum flexibility. The planet stores
/// all generated cells in memory for fast queries and pathfinding.
///
/// # Type Parameters
///
/// * `T` - Terrain type for cells (e.g., `BasicTerrainType` or custom enum)
///
/// # Examples
///
/// ```
/// use rust_voronoi_planet::*;
///
/// // Generate planet with default BasicTerrainType
/// let config = PlanetConfigBuilder::new()
///     .seed(42)
///     .planet_size(PlanetSize::Tiny)
///     .build()
///     .unwrap();
///
/// let planet = VoronoiPlanet::generate(config).unwrap();
/// println!("Generated {} cells", planet.cell_count());
///
/// // Query cells
/// if let Some(cell) = planet.get_cell(0) {
///     println!("Cell 0 terrain: {:?}", cell.terrain);
/// }
/// ```
#[derive(Clone)]
pub struct VoronoiPlanet<T> {
    /// Configuration used to generate this planet
    config: PlanetConfig,

    /// All Voronoi cells on the planet (indexed by cell ID)
    cells: Vec<VoronoiCell<T>>,

    /// Sphere radius for distance calculations
    radius: f32,

    /// Spatial index for fast position-to-cell lookups (optional, requires spatial-index feature)
    #[cfg(feature = "spatial-index")]
    spatial_index: SpatialIndex,
}

impl VoronoiPlanet<BasicTerrainType> {
    /// Generate a planet with default Perlin terrain sampling
    ///
    /// This is the most common way to create a planet. It uses `BasicTerrainType`
    /// with Perlin noise-based terrain generation.
    ///
    /// # Arguments
    ///
    /// * `config` - Planet configuration (seed, size, iterations)
    ///
    /// # Returns
    ///
    /// `Result<VoronoiPlanet<BasicTerrainType>>` - Generated planet or error
    ///
    /// # Example
    ///
    /// ```
    /// use rust_voronoi_planet::*;
    ///
    /// let config = PlanetConfigBuilder::new()
    ///     .seed(12345)
    ///     .planet_size(PlanetSize::Small)
    ///     .lloyd_iterations(5)
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// let planet = VoronoiPlanet::generate(config).unwrap();
    /// assert!(planet.cell_count() > 0);
    /// ```
    pub fn generate(config: PlanetConfig) -> Result<Self> {
        let sampler = PerlinTerrainSampler::new(config.terrain_seed);
        Self::generate_with_sampler(config, &sampler)
    }
}

impl<T: Clone> VoronoiPlanet<T> {
    /// Generate a planet with a custom terrain sampler
    ///
    /// This allows you to use custom terrain types and sampling logic.
    /// The sampler is called once for each cell center to determine terrain.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Terrain sampler implementing `TerrainSampler<Output = T>`
    ///
    /// # Arguments
    ///
    /// * `config` - Planet configuration
    /// * `sampler` - Terrain sampler to use for each cell
    ///
    /// # Returns
    ///
    /// `Result<VoronoiPlanet<T>>` - Generated planet or error
    ///
    /// # Example
    ///
    /// ```
    /// use rust_voronoi_planet::*;
    ///
    /// let config = PlanetConfig::default();
    /// let sampler = PerlinTerrainSampler::new(42);
    /// let planet = VoronoiPlanet::generate_with_sampler(config, &sampler).unwrap();
    /// ```
    pub fn generate_with_sampler<S>(config: PlanetConfig, sampler: &S) -> Result<Self>
    where
        S: TerrainSampler<Output = T>,
    {
        let radius = config.radius();

        // Generate raw cells (geometry only, no terrain)
        let raw_cells = generate_raw_cells(&config)?;

        // Apply terrain sampling to create full cells
        let cells: Vec<VoronoiCell<T>> = raw_cells
            .into_iter()
            .map(|raw| {
                // Sample terrain at cell center
                let terrain = sampler.sample(raw.center, radius);
                VoronoiCell::new(
                    raw.id,
                    raw.center,
                    terrain,
                    raw.neighbors,
                    raw.vertices,
                )
            })
            .collect();

        // Build spatial index (requires spatial-index feature)
        #[cfg(feature = "spatial-index")]
        let spatial_index = {
            let centers: Vec<Vec3> = cells.iter().map(|c| c.center).collect();
            SpatialIndex::new(&centers)
        };

        Ok(Self {
            config,
            cells,
            radius,
            #[cfg(feature = "spatial-index")]
            spatial_index,
        })
    }

    /// Get the configuration used to generate this planet
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// let config = planet.config();
    /// println!("Planet seed: {}", config.seed);
    /// ```
    #[inline]
    pub fn config(&self) -> &PlanetConfig {
        &self.config
    }

    /// Get the number of cells on this planet
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// println!("Planet has {} cells", planet.cell_count());
    /// ```
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get the sphere radius
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// println!("Planet radius: {}", planet.radius());
    /// ```
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get a cell by ID
    ///
    /// Returns `None` if the cell ID is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `id` - Cell ID (0 to cell_count-1)
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// if let Some(cell) = planet.get_cell(0) {
    ///     println!("Cell 0 has {} neighbors", cell.neighbor_count());
    /// }
    /// ```
    #[inline]
    pub fn get_cell(&self, id: usize) -> Option<&VoronoiCell<T>> {
        self.cells.get(id)
    }

    /// Get all cells as a slice
    ///
    /// Useful for iteration or bulk operations.
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// for cell in planet.cells() {
    ///     // Process each cell
    /// }
    /// ```
    #[inline]
    pub fn cells(&self) -> &[VoronoiCell<T>] {
        &self.cells
    }

    /// Get neighbor IDs for a cell
    ///
    /// Returns a slice of cell IDs that are adjacent to the given cell.
    /// Returns empty slice if cell ID is invalid.
    ///
    /// # Arguments
    ///
    /// * `cell_id` - Cell ID to query
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// let neighbors = planet.get_neighbors(0);
    /// println!("Cell 0 has {} neighbors", neighbors.len());
    /// ```
    pub fn get_neighbors(&self, cell_id: usize) -> &[usize] {
        self.cells
            .get(cell_id)
            .map(|c| c.neighbors.as_slice())
            .unwrap_or(&[])
    }

    /// Find the cell containing a position (requires spatial-index feature)
    ///
    /// Uses KD-tree spatial index for O(log n) nearest-neighbor lookup.
    /// This is essential for converting 3D positions (from raycasting, clicks, etc.)
    /// into cell IDs.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position on sphere surface
    ///
    /// # Returns
    ///
    /// Cell ID of the nearest cell
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # use glam::Vec3;
    /// # #[cfg(feature = "spatial-index")]
    /// # {
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// let position = Vec3::new(planet.radius(), 0.0, 0.0);
    /// let cell_id = planet.find_cell_at(position);
    /// println!("Position is in cell {}", cell_id);
    /// # }
    /// ```
    #[cfg(feature = "spatial-index")]
    pub fn find_cell_at(&self, position: Vec3) -> usize {
        self.spatial_index.find_nearest(position)
    }

    /// Find cells within a given hop count from a center cell (BFS)
    ///
    /// Uses breadth-first search to find all cells reachable within the
    /// specified number of hops from the center cell.
    ///
    /// # Arguments
    ///
    /// * `center_id` - Starting cell ID
    /// * `hops` - Maximum number of cell hops (0 = just the center cell)
    ///
    /// # Returns
    ///
    /// Vector of cell IDs within radius, including the center cell.
    /// Returns empty vec if center_id is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// # use rust_voronoi_planet::*;
    /// # let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();
    /// // Get all cells within 3 hops of cell 0
    /// let nearby_cells = planet.find_cells_within_radius(0, 3);
    /// println!("Found {} cells within 3 hops", nearby_cells.len());
    /// ```
    pub fn find_cells_within_radius(&self, center_id: usize, hops: usize) -> Vec<usize> {
        if center_id >= self.cells.len() {
            return vec![];
        }

        let mut visited = std::collections::HashSet::new();
        let mut current = vec![center_id];
        visited.insert(center_id);

        // BFS with hop limit
        for _ in 0..hops {
            let mut next = Vec::new();
            for &cell_id in &current {
                for &neighbor in self.get_neighbors(cell_id) {
                    if visited.insert(neighbor) {
                        next.push(neighbor);
                    }
                }
            }
            current = next;
        }

        visited.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlanetSize, PlanetConfigBuilder};

    #[test]
    fn test_planet_generation() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .lloyd_iterations(2)
            .unwrap()
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();

        assert!(planet.cell_count() > 0);
        assert_eq!(planet.radius(), config.radius());
    }

    #[test]
    fn test_get_cell() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();

        assert!(planet.get_cell(0).is_some());
        assert!(planet.get_cell(planet.cell_count()).is_none());
    }

    #[test]
    fn test_get_neighbors() {
        let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();

        let neighbors = planet.get_neighbors(0);
        assert!(!neighbors.is_empty());
        assert!(neighbors.len() >= 3); // Should have at least 3 neighbors
        assert!(neighbors.len() <= 10); // Shouldn't have more than ~10
    }

    #[cfg(feature = "spatial-index")]
    #[test]
    fn test_find_cell_at() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();

        // Get a cell's center and verify we find that cell
        let cell_center = planet.get_cell(0).unwrap().center;
        let found_cell_id = planet.find_cell_at(cell_center);

        assert_eq!(found_cell_id, 0);
    }

    #[test]
    fn test_find_cells_within_radius() {
        let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();

        // Radius 0 should return just the center cell
        let cells_r0 = planet.find_cells_within_radius(0, 0);
        assert_eq!(cells_r0.len(), 1);
        assert!(cells_r0.contains(&0));

        // Radius 1 should return center + neighbors
        let cells_r1 = planet.find_cells_within_radius(0, 1);
        let neighbors = planet.get_neighbors(0);
        assert_eq!(cells_r1.len(), 1 + neighbors.len());

        // Radius 2 should be larger
        let cells_r2 = planet.find_cells_within_radius(0, 2);
        assert!(cells_r2.len() > cells_r1.len());
    }

    #[test]
    fn test_invalid_cell_id() {
        let planet = VoronoiPlanet::generate(PlanetConfig::default()).unwrap();

        // Invalid cell ID should return empty neighbors
        let neighbors = planet.get_neighbors(999999);
        assert!(neighbors.is_empty());

        // Invalid cell ID should return empty radius
        let cells = planet.find_cells_within_radius(999999, 5);
        assert!(cells.is_empty());
    }

    #[test]
    fn test_terrain_distribution() {
        use crate::terrain::BasicTerrainType;

        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();

        // Count terrain types
        let mut counts = std::collections::HashMap::new();
        for cell in planet.cells() {
            *counts.entry(cell.terrain).or_insert(0) += 1;
        }

        // Should have multiple terrain types
        assert!(counts.len() > 1, "Should have varied terrain");

        // Should have some water and some land
        let water_count: usize = counts.get(&BasicTerrainType::Ocean).copied().unwrap_or(0);
        let land_count: usize = counts.values().sum::<usize>() - water_count;

        assert!(water_count > 0, "Should have some ocean");
        assert!(land_count > 0, "Should have some land");
    }
}
