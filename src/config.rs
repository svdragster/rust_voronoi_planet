//! Voronoi Planet Configuration and Builder
//!
//! This module provides configuration types for deterministic Voronoi planet generation.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::{Result, VoronoiError};

/// Planet size presets matching the existing game's size system
///
/// Each size maps to a specific cell count and sphere radius for consistent gameplay scaling.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetSize {
    /// Tiny planet: ~5,000 cells, radius ~11.3 units (Size 1)
    Tiny,
    /// Small planet: ~11,000 cells, radius ~16.7 units (Size 3)
    Small,
    /// Medium planet: ~17,000 cells, radius ~20.9 units (Size 5)
    Medium,
    /// Large planet: ~26,000 cells, radius ~25.8 units (Size 8, default)
    Large,
    /// Custom planet size with specific cell count and radius
    Custom {
        /// Number of Voronoi cells to generate
        cell_count: usize,
        /// Sphere radius in world units
        radius: f32,
    },
}

impl PlanetSize {
    /// Get the approximate number of Voronoi cells for this planet size
    ///
    /// Cell counts are approximate as the actual count depends on Lloyd's relaxation
    /// and the random point distribution.
    pub fn cell_count(self) -> usize {
        match self {
            PlanetSize::Tiny => 5_000,
            PlanetSize::Small => 11_000,
            PlanetSize::Medium => 17_000,
            PlanetSize::Large => 26_000,
            PlanetSize::Custom { cell_count, .. } => cell_count,
        }
    }

    /// Get the sphere radius for this planet size
    ///
    /// The radius scales roughly with sqrt(cell_count) to maintain similar cell density.
    pub fn sphere_radius(self) -> f32 {
        match self {
            PlanetSize::Tiny => 11.3,
            PlanetSize::Small => 16.7,
            PlanetSize::Medium => 20.9,
            PlanetSize::Large => 25.8,
            PlanetSize::Custom { radius, .. } => radius,
        }
    }

    /// Get a human-readable name for this planet size
    pub fn name(self) -> &'static str {
        match self {
            PlanetSize::Tiny => "Tiny",
            PlanetSize::Small => "Small",
            PlanetSize::Medium => "Medium",
            PlanetSize::Large => "Large",
            PlanetSize::Custom { .. } => "Custom",
        }
    }
}

impl Default for PlanetSize {
    fn default() -> Self {
        PlanetSize::Large // Match existing game default (size 8)
    }
}

/// Configuration for deterministic Voronoi planet generation
///
/// This configuration is serializable and can be shared between client and server.
/// The same configuration will always produce the identical planet.
///
/// # Serialization
///
/// Only the configuration is serialized (~20 bytes), not the generated cells.
/// The planet is regenerated from the configuration when loading a save file.
///
/// # Example
///
/// ```rust
/// use rust_voronoi_planet::*;
///
/// let config = PlanetConfigBuilder::new()
///     .seed(42)
///     .planet_size(PlanetSize::Medium)
///     .build()
///     .unwrap();
///
/// // Config is serializable (with "serde" feature)
/// # #[cfg(feature = "serde")]
/// # {
/// let json = serde_json::to_string(&config).unwrap();
/// let restored: PlanetConfig = serde_json::from_str(&json).unwrap();
/// assert_eq!(config.seed, restored.seed);
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetConfig {
    /// Random seed for deterministic planet generation
    ///
    /// The same seed (with same planet_size and lloyd_iterations) will always
    /// produce the exact same planet with identical cell positions and terrain.
    pub seed: u32,

    /// Planet size preset (determines cell count and sphere radius)
    pub planet_size: PlanetSize,

    /// Number of Lloyd's Relaxation iterations for uniform cell distribution
    ///
    /// - 0: Random Voronoi cells (irregular)
    /// - 2-3: Decent uniformity
    /// - 5: Good uniformity (recommended, default)
    /// - 10+: Diminishing returns, slower generation
    pub lloyd_iterations: usize,

    /// Convergence threshold for Lloyd's relaxation (fraction of radius)
    ///
    /// Lloyd's relaxation will stop early when the maximum point displacement
    /// falls below this threshold multiplied by the sphere radius.
    ///
    /// - 0.0: Disable early termination (run all iterations)
    /// - 0.01: Default, stops when points move less than 1% of radius (~3-4 iterations)
    /// - 0.001: Stricter, may run all iterations for very uniform distribution
    pub lloyd_convergence: f32,

    /// Random seed for terrain generation (separate from cell placement seed)
    ///
    /// This allows the same cell layout with different terrain distributions.
    pub terrain_seed: u32,

    /// Override the sphere radius from the planet_size preset
    ///
    /// If set, this radius will be used instead of the preset radius.
    /// Useful for fine-tuning cell density without creating a custom size.
    pub radius_override: Option<f32>,
}

impl PlanetConfig {
    /// Get the cell count for this configuration
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.planet_size.cell_count()
    }

    /// Get the sphere radius for this configuration
    ///
    /// Returns the radius_override if set, otherwise the planet_size preset radius.
    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius_override
            .unwrap_or_else(|| self.planet_size.sphere_radius())
    }

    /// Get the sphere radius for this configuration (legacy method)
    ///
    /// Deprecated: Use `radius()` instead for clarity.
    #[inline]
    #[deprecated(since = "0.1.0", note = "use radius() instead")]
    pub fn sphere_radius(&self) -> f32 {
        self.radius()
    }
}

impl Default for PlanetConfig {
    fn default() -> Self {
        PlanetConfigBuilder::new().build().unwrap()
    }
}

/// Builder for creating PlanetConfig with validation
///
/// Uses the builder pattern to create configurations with sensible defaults
/// and compile-time guarantees of validity.
///
/// # Example
///
/// ```rust
/// use rust_voronoi_planet::*;
///
/// // Use defaults
/// let config = PlanetConfigBuilder::new().build().unwrap();
///
/// // Customize
/// let config = PlanetConfigBuilder::new()
///     .seed(12345)
///     .planet_size(PlanetSize::Small)
///     .lloyd_iterations(3)
///     .unwrap()
///     .terrain_seed(67890)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct PlanetConfigBuilder {
    seed: Option<u32>,
    planet_size: PlanetSize,
    lloyd_iterations: usize,
    lloyd_convergence: f32,
    terrain_seed: Option<u32>,
    radius_override: Option<f32>,
}

impl PlanetConfigBuilder {
    /// Create a new builder with default values
    ///
    /// Defaults:
    /// - seed: Random (generated from thread_rng)
    /// - planet_size: Large (~26,000 cells)
    /// - lloyd_iterations: 5 (good uniformity)
    /// - lloyd_convergence: 0.01 (stop when points move < 1% of radius)
    /// - terrain_seed: Same as seed
    /// - radius_override: None
    pub fn new() -> Self {
        Self {
            seed: None,
            planet_size: PlanetSize::default(),
            lloyd_iterations: 5,
            lloyd_convergence: 0.01,
            terrain_seed: None,
            radius_override: None,
        }
    }

    /// Set the random seed for planet generation
    ///
    /// Using the same seed with the same other parameters will produce
    /// an identical planet every time.
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the planet size preset
    ///
    /// This determines the number of cells and sphere radius.
    pub fn planet_size(mut self, size: PlanetSize) -> Self {
        self.planet_size = size;
        self
    }

    /// Set the number of Lloyd's Relaxation iterations
    ///
    /// More iterations create more uniform cell distributions but take longer.
    /// Recommended: 3-5 iterations for good uniformity.
    ///
    /// # Errors
    ///
    /// Returns `InvalidConfig` if iterations > 20 (excessive and impractical)
    pub fn lloyd_iterations(mut self, iterations: usize) -> Result<Self> {
        if iterations > 20 {
            return Err(VoronoiError::InvalidConfig(format!(
                "Lloyd iterations must be <= 20 (got {})",
                iterations
            )));
        }
        self.lloyd_iterations = iterations;
        Ok(self)
    }

    /// Set the convergence threshold for Lloyd's relaxation
    ///
    /// The threshold is a fraction of the sphere radius. Lloyd's relaxation
    /// will stop early when the maximum point displacement falls below
    /// `threshold * radius`.
    ///
    /// - 0.0: Disable early termination (run all iterations)
    /// - 0.0001: Default, good balance of quality and performance
    /// - 0.001: Stop earlier, faster but potentially less uniform
    ///
    /// # Errors
    ///
    /// Returns `InvalidConfig` if threshold is negative
    pub fn lloyd_convergence(mut self, threshold: f32) -> Result<Self> {
        if threshold < 0.0 {
            return Err(VoronoiError::InvalidConfig(format!(
                "Lloyd convergence threshold must be >= 0 (got {})",
                threshold
            )));
        }
        self.lloyd_convergence = threshold;
        Ok(self)
    }

    /// Set a separate terrain seed
    ///
    /// If not set, the terrain seed will match the planet seed.
    /// Setting a different terrain seed allows the same cell layout
    /// with different terrain distributions.
    pub fn terrain_seed(mut self, seed: u32) -> Self {
        self.terrain_seed = Some(seed);
        self
    }

    /// Override the sphere radius
    ///
    /// If set, this radius will be used instead of the planet_size preset radius.
    /// Useful for fine-tuning cell density.
    ///
    /// # Errors
    ///
    /// Returns `InvalidConfig` if radius <= 0.0
    pub fn radius_override(mut self, radius: f32) -> Result<Self> {
        if radius <= 0.0 {
            return Err(VoronoiError::InvalidConfig(format!(
                "Radius override must be positive (got {})",
                radius
            )));
        }
        self.radius_override = Some(radius);
        Ok(self)
    }

    /// Build the configuration
    ///
    /// If no seed was provided, generates a random seed using thread_rng.
    pub fn build(self) -> Result<PlanetConfig> {
        let seed = self.seed.unwrap_or_else(|| rand::random());
        let terrain_seed = self.terrain_seed.unwrap_or(seed);

        Ok(PlanetConfig {
            seed,
            planet_size: self.planet_size,
            lloyd_iterations: self.lloyd_iterations,
            lloyd_convergence: self.lloyd_convergence,
            terrain_seed,
            radius_override: self.radius_override,
        })
    }
}

impl Default for PlanetConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planet_size_cell_counts() {
        assert_eq!(PlanetSize::Tiny.cell_count(), 5_000);
        assert_eq!(PlanetSize::Small.cell_count(), 11_000);
        assert_eq!(PlanetSize::Medium.cell_count(), 17_000);
        assert_eq!(PlanetSize::Large.cell_count(), 26_000);
    }

    #[test]
    fn test_planet_size_radii() {
        assert_eq!(PlanetSize::Tiny.sphere_radius(), 11.3);
        assert_eq!(PlanetSize::Small.sphere_radius(), 16.7);
        assert_eq!(PlanetSize::Medium.sphere_radius(), 20.9);
        assert_eq!(PlanetSize::Large.sphere_radius(), 25.8);
    }

    #[test]
    fn test_planet_size_custom() {
        let custom = PlanetSize::Custom {
            cell_count: 50_000,
            radius: 30.0,
        };
        assert_eq!(custom.cell_count(), 50_000);
        assert_eq!(custom.sphere_radius(), 30.0);
        assert_eq!(custom.name(), "Custom");
    }

    #[test]
    fn test_builder_defaults() {
        let config = PlanetConfigBuilder::new().build().unwrap();
        assert_eq!(config.planet_size, PlanetSize::Large);
        assert_eq!(config.lloyd_iterations, 5);
        assert_eq!(config.radius_override, None);
        // seed and terrain_seed are random, so just verify they were set
        let _seed = config.seed; // Just verify seed exists
    }

    #[test]
    fn test_builder_custom() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Small)
            .lloyd_iterations(3)
            .unwrap()
            .terrain_seed(99)
            .build()
            .unwrap();

        assert_eq!(config.seed, 42);
        assert_eq!(config.planet_size, PlanetSize::Small);
        assert_eq!(config.lloyd_iterations, 3);
        assert_eq!(config.terrain_seed, 99);
    }

    #[test]
    fn test_radius_override() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .radius_override(100.0)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(config.radius(), 100.0);
        assert_eq!(config.radius_override, Some(100.0));
    }

    #[test]
    fn test_radius_no_override() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Medium)
            .build()
            .unwrap();

        assert_eq!(config.radius(), PlanetSize::Medium.sphere_radius());
        assert_eq!(config.radius_override, None);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_config_serialization() {
        let config = PlanetConfigBuilder::new()
            .seed(12345)
            .planet_size(PlanetSize::Medium)
            .build()
            .unwrap();

        let json = serde_json::to_string(&config).unwrap();
        let restored: PlanetConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.seed, restored.seed);
        assert_eq!(config.planet_size, restored.planet_size);
    }

    #[test]
    fn test_builder_too_many_iterations() {
        let result = PlanetConfigBuilder::new().lloyd_iterations(21);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_radius() {
        let result = PlanetConfigBuilder::new().radius_override(0.0);
        assert!(result.is_err());

        let result = PlanetConfigBuilder::new().radius_override(-5.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_terrain_seed_defaults_to_planet_seed() {
        let config = PlanetConfigBuilder::new().seed(42).build().unwrap();
        assert_eq!(config.terrain_seed, 42);
    }

    #[test]
    fn test_separate_terrain_seed() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .terrain_seed(99)
            .build()
            .unwrap();
        assert_eq!(config.seed, 42);
        assert_eq!(config.terrain_seed, 99);
    }
}
