//! Terrain sampling and classification
//!
//! Provides traits and implementations for sampling terrain on sphere surfaces.

mod perlin;

pub use perlin::{PerlinConfig, sample_perlin_3d, sample_perlin_fbm};

use glam::Vec3;

/// Basic terrain types for planet surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BasicTerrainType {
    /// Deep water (ocean)
    Ocean,
    /// Shallow coastal water/beach transition
    Beach,
    /// General land (grassland, plains)
    #[default]
    Land,
    /// Elevated terrain (hills, mountains)
    Mountain,
    /// Frozen polar regions
    Ice,
}

impl BasicTerrainType {
    /// Check if this terrain is water
    pub fn is_water(&self) -> bool {
        matches!(self, BasicTerrainType::Ocean)
    }

    /// Check if this terrain is walkable land
    pub fn is_land(&self) -> bool {
        !self.is_water()
    }
}

/// Trait for sampling terrain at positions on a sphere
pub trait TerrainSampler {
    /// The terrain type produced by this sampler
    type Output;

    /// Sample terrain at a 3D position on the sphere surface
    fn sample(&self, position: Vec3, radius: f32) -> Self::Output;
}

/// Default terrain sampler using 3D Perlin noise
pub struct PerlinTerrainSampler {
    /// Seed for noise generation
    pub seed: u32,
    /// Threshold below which terrain is ocean (default: -0.12)
    pub ocean_threshold: f32,
    /// Threshold above which terrain is mountain (default: 0.4)
    pub mountain_threshold: f32,
    /// Latitude above which terrain is ice (default: 0.85)
    pub ice_cap_latitude: f32,
    /// Width of beach band above ocean threshold (default: 0.05)
    pub beach_band: f32,
    /// Perlin noise configuration
    pub config: PerlinConfig,
}

impl Default for PerlinTerrainSampler {
    fn default() -> Self {
        Self {
            seed: 0,
            ocean_threshold: -0.12,
            mountain_threshold: 0.4,
            ice_cap_latitude: 0.85,
            beach_band: 0.05,
            config: PerlinConfig::default(),
        }
    }
}

impl PerlinTerrainSampler {
    /// Create a new sampler with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Create a sampler with custom configuration
    pub fn with_config(seed: u32, config: PerlinConfig) -> Self {
        Self {
            seed,
            config,
            ..Default::default()
        }
    }
}

impl TerrainSampler for PerlinTerrainSampler {
    type Output = BasicTerrainType;

    fn sample(&self, position: Vec3, radius: f32) -> BasicTerrainType {
        // Check for polar ice caps
        let latitude = (position.y / radius).abs();
        if latitude > self.ice_cap_latitude {
            return BasicTerrainType::Ice;
        }

        // Apply coordinate transformation for consistency
        let sampling_pos = Vec3::new(-position.x, position.y, -position.z);

        // Domain warping for organic coastlines
        let warp_freq = 0.15;
        let warp_strength = 1.75;
        let warp_x = sample_perlin_fbm(sampling_pos * warp_freq, self.seed.wrapping_add(1000), 3, 0.5, 2.0);
        let warp_y = sample_perlin_fbm(sampling_pos * warp_freq, self.seed.wrapping_add(2000), 3, 0.5, 2.0);
        let warp_z = sample_perlin_fbm(sampling_pos * warp_freq, self.seed.wrapping_add(3000), 3, 0.5, 2.0);
        let warped_pos = sampling_pos + Vec3::new(warp_x, warp_y, warp_z) * warp_strength;

        // Sample continent base
        let continent_freq = 0.125;
        let elevation = sample_perlin_fbm(warped_pos * continent_freq, self.seed, 1, 0.5, 2.0);

        // Classify terrain
        if elevation < self.ocean_threshold {
            BasicTerrainType::Ocean
        } else if elevation < self.ocean_threshold + self.beach_band {
            BasicTerrainType::Beach
        } else if elevation > self.mountain_threshold {
            BasicTerrainType::Mountain
        } else {
            BasicTerrainType::Land
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that terrain sampler produces valid terrain types
    #[test]
    fn test_terrain_classification() {
        let sampler = PerlinTerrainSampler::new(42);
        let radius = 1.0;

        // Test multiple positions
        let positions = vec![
            Vec3::new(1.0, 0.0, 0.0), // Equator
            Vec3::new(0.0, 1.0, 0.0), // North pole
            Vec3::new(0.0, -1.0, 0.0), // South pole
            Vec3::new(0.577, 0.577, 0.577), // Mid-latitude
        ];

        for pos in positions {
            let terrain = sampler.sample(pos, radius);
            // Just verify we get a valid terrain type (no panic)
            match terrain {
                BasicTerrainType::Ocean
                | BasicTerrainType::Beach
                | BasicTerrainType::Land
                | BasicTerrainType::Mountain
                | BasicTerrainType::Ice => {}
            }
        }
    }

    /// Test that polar regions produce ice
    #[test]
    fn test_polar_ice() {
        let sampler = PerlinTerrainSampler::new(42);
        let radius = 1.0;

        // North pole should be ice
        let north_pole = Vec3::new(0.0, 1.0, 0.0);
        let terrain = sampler.sample(north_pole, radius);
        assert_eq!(terrain, BasicTerrainType::Ice);

        // South pole should be ice
        let south_pole = Vec3::new(0.0, -1.0, 0.0);
        let terrain = sampler.sample(south_pole, radius);
        assert_eq!(terrain, BasicTerrainType::Ice);
    }

    /// Test determinism of terrain sampling
    #[test]
    fn test_terrain_determinism() {
        let sampler = PerlinTerrainSampler::new(123);
        let radius = 1.0;
        let position = Vec3::new(0.5, 0.5, 0.5);

        let terrain1 = sampler.sample(position, radius);
        let terrain2 = sampler.sample(position, radius);

        assert_eq!(terrain1, terrain2, "Same position should produce same terrain");
    }

    /// Test that different seeds produce potentially different terrain
    #[test]
    fn test_different_seeds_terrain() {
        let sampler1 = PerlinTerrainSampler::new(42);
        let sampler2 = PerlinTerrainSampler::new(999);
        let radius = 1.0;
        let position = Vec3::new(0.5, 0.3, 0.4);

        let terrain1 = sampler1.sample(position, radius);
        let terrain2 = sampler2.sample(position, radius);

        // Note: Different seeds might produce same terrain type, so we just verify no panic
        // This test primarily ensures the seed is being used
        let _ = (terrain1, terrain2);
    }

    /// Test is_water and is_land helper methods
    #[test]
    fn test_terrain_helpers() {
        assert!(BasicTerrainType::Ocean.is_water());
        assert!(!BasicTerrainType::Ocean.is_land());

        assert!(!BasicTerrainType::Beach.is_water());
        assert!(BasicTerrainType::Beach.is_land());

        assert!(!BasicTerrainType::Land.is_water());
        assert!(BasicTerrainType::Land.is_land());

        assert!(!BasicTerrainType::Mountain.is_water());
        assert!(BasicTerrainType::Mountain.is_land());

        assert!(!BasicTerrainType::Ice.is_water());
        assert!(BasicTerrainType::Ice.is_land());
    }
}
