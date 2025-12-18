//! 3D Perlin noise implementation
//!
//! This module provides 3D Perlin noise sampling for procedural terrain generation.
//! The implementation is ported from the meridian_civilization reference implementation
//! and uses the standard Ken Perlin permutation table and algorithm.

use glam::Vec3;

/// Configuration for Perlin noise generation
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PerlinConfig {
    /// Base frequency controls feature size (lower = larger features)
    pub base_frequency: f32,
    /// Number of octaves for fractal detail layers
    pub octaves: usize,
    /// Amplitude decay per octave (controls roughness)
    pub persistence: f32,
    /// Frequency multiplier per octave
    pub lacunarity: f32,
}

impl Default for PerlinConfig {
    fn default() -> Self {
        Self {
            base_frequency: 0.8,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

// ============================================================================
// PERMUTATION TABLE
// ============================================================================
// Standard 256-element permutation table from Ken Perlin's reference implementation.
// This table must remain unchanged to maintain deterministic terrain generation.
const PERM: [u32; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

// ============================================================================
// PERLIN NOISE HELPER FUNCTIONS
// ============================================================================

/// Hash function: combines permutation table lookups with seed
///
/// # Algorithm
/// 1. Hash seed with linear congruential generator
/// 2. XOR coordinates with seed hash components
/// 3. Three-level permutation table lookup
#[inline]
fn hash(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let seed_hash = (seed.wrapping_mul(1103515245).wrapping_add(12345)) >> 16;
    let ix = ((x as u32) ^ seed_hash) & 255;
    let iy = ((y as u32) ^ (seed_hash >> 8)) & 255;
    let iz = ((z as u32) ^ (seed_hash >> 16)) & 255;
    let a = PERM[ix as usize];
    let b = PERM[((a + iy) & 255) as usize];
    PERM[((b + iz) & 255) as usize]
}

/// Generate gradient vector from hash value (12 edge vectors of a cube)
///
/// Uses hash value to select one of 12 edge directions of a unit cube,
/// then computes dot product with input vector (x, y, z).
#[inline]
fn gradient(hash_value: u32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash_value & 15;

    // Port of WGSL select() logic to Rust conditionals
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        z
    } else {
        x
    };

    let sign_u = if (h & 1) == 0 { -u } else { u };
    let sign_v = if (h & 2) == 0 { -v } else { v };

    sign_u + sign_v
}

/// Quintic smoothstep interpolation (Ken Perlin's improved fade function)
///
/// Formula: 6t⁵ - 15t⁴ + 10t³
/// Provides smooth C2-continuous interpolation with zero first and second derivatives at t=0 and t=1.
#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

// ============================================================================
// 3D PERLIN NOISE CORE FUNCTION
// ============================================================================

/// Sample 3D Perlin noise at a given position with seed
///
/// # Algorithm
/// 1. Find unit cube containing the point
/// 2. Compute relative position within cube [0,1]
/// 3. Apply fade curves for smooth interpolation
/// 4. Hash all 8 cube corners
/// 5. Compute gradient dot products for each corner
/// 6. Perform trilinear interpolation of gradients
///
/// # Returns
/// Value in range [-1, 1] (standard Perlin output)
fn perlin_3d(pos: Vec3, seed: u32) -> f32 {
    // Find unit cube containing the point
    let x0 = pos.x.floor() as i32;
    let y0 = pos.y.floor() as i32;
    let z0 = pos.z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    // Find relative position within cube (0.0 to 1.0)
    let xf = pos.x - pos.x.floor();
    let yf = pos.y - pos.y.floor();
    let zf = pos.z - pos.z.floor();

    // Compute fade curves for interpolation
    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    // Hash coordinates of 8 cube corners
    let aaa = hash(x0, y0, z0, seed);
    let aba = hash(x0, y1, z0, seed);
    let aab = hash(x0, y0, z1, seed);
    let abb = hash(x0, y1, z1, seed);
    let baa = hash(x1, y0, z0, seed);
    let bba = hash(x1, y1, z0, seed);
    let bab = hash(x1, y0, z1, seed);
    let bbb = hash(x1, y1, z1, seed);

    // Calculate gradient dot products for 8 corners
    let g_aaa = gradient(aaa, xf, yf, zf);
    let g_baa = gradient(baa, xf - 1.0, yf, zf);
    let g_aba = gradient(aba, xf, yf - 1.0, zf);
    let g_bba = gradient(bba, xf - 1.0, yf - 1.0, zf);
    let g_aab = gradient(aab, xf, yf, zf - 1.0);
    let g_bab = gradient(bab, xf - 1.0, yf, zf - 1.0);
    let g_abb = gradient(abb, xf, yf - 1.0, zf - 1.0);
    let g_bbb = gradient(bbb, xf - 1.0, yf - 1.0, zf - 1.0);

    // Trilinear interpolation of gradients
    let x00 = lerp(g_aaa, g_baa, u);
    let x10 = lerp(g_aba, g_bba, u);
    let x01 = lerp(g_aab, g_bab, u);
    let x11 = lerp(g_abb, g_bbb, u);
    let y0_val = lerp(x00, x10, v);
    let y1_val = lerp(x01, x11, v);

    lerp(y0_val, y1_val, w)
}

// ============================================================================
// FRACTAL BROWNIAN MOTION (FBM)
// ============================================================================

/// Sample 3D Perlin noise with Fractal Brownian Motion
///
/// Generates layered noise by accumulating multiple octaves at different
/// frequencies and amplitudes. Returns normalized value in range [0.0, 1.0].
///
/// # Arguments
/// * `position` - 3D position to sample
/// * `seed` - Random seed for deterministic generation
/// * `config` - Configuration for noise parameters
///
/// # Returns
/// Elevation value in range [0.0, 1.0]
pub fn sample_perlin_3d(position: Vec3, seed: u32, config: &PerlinConfig) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = config.base_frequency;
    let mut max_value = 0.0;

    // Fractal Brownian Motion: accumulate multiple octaves of noise
    // Each octave adds detail at higher frequency with lower amplitude
    for _ in 0..config.octaves {
        let sample = perlin_3d(position * frequency, seed);
        value += sample * amplitude;
        max_value += amplitude;

        // Update for next octave: higher frequency, lower amplitude
        frequency *= config.lacunarity;
        amplitude *= config.persistence;
    }

    // Normalize Perlin output [-1, 1] to [0, 1] range
    ((value / max_value) + 1.0) / 2.0
}

/// Sample FBM with explicit parameters (for internal use)
///
/// Returns raw FBM value in range [-1, 1] (approximately) for use in
/// domain warping and other advanced techniques.
///
/// # Arguments
/// * `position` - 3D position to sample
/// * `seed` - Random seed for deterministic generation
/// * `octaves` - Number of noise layers
/// * `persistence` - Amplitude decay per octave
/// * `lacunarity` - Frequency multiplier per octave
///
/// # Returns
/// Value in range [-1, 1] (raw FBM output)
pub fn sample_perlin_fbm(
    position: Vec3,
    seed: u32,
    octaves: usize,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0; // For normalization

    for _ in 0..octaves {
        total += perlin_3d(position * frequency, seed) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    // Normalize to approximately [-1, 1]
    total / max_value
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that identical seed and position always produce the same result (determinism)
    #[test]
    fn test_determinism() {
        let config = PerlinConfig::default();
        let position = Vec3::new(0.5, 0.7, 0.3);
        let seed = 42;

        let value1 = sample_perlin_3d(position, seed, &config);
        let value2 = sample_perlin_3d(position, seed, &config);

        assert_eq!(
            value1, value2,
            "Same seed and position must produce identical results"
        );
    }

    /// Test that output is always in valid [0.0, 1.0] range
    #[test]
    fn test_range() {
        let config = PerlinConfig::default();
        let seed = 12345;

        // Test multiple positions across the sphere
        let test_positions = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.577, 0.577, 0.577), // normalized corner
            Vec3::new(-0.5, 0.5, 0.707),
        ];

        for position in test_positions {
            let value = sample_perlin_3d(position, seed, &config);
            assert!(
                value >= 0.0 && value <= 1.0,
                "Value {} at position {:?} is outside [0.0, 1.0] range",
                value,
                position
            );
        }
    }

    /// Test that different seeds produce different terrain
    #[test]
    fn test_different_seeds() {
        let config = PerlinConfig::default();
        let position = Vec3::new(0.5, 0.5, 0.5);

        let value1 = sample_perlin_3d(position, 42, &config);
        let value2 = sample_perlin_3d(position, 999, &config);

        assert_ne!(
            value1, value2,
            "Different seeds should produce different values"
        );
    }

    /// Test that raw Perlin core returns values in [-1, 1]
    #[test]
    fn test_perlin_core() {
        let seed = 42;

        // Test basic Perlin output range [-1, 1]
        let value = perlin_3d(Vec3::new(0.0, 0.0, 0.0), seed);
        assert!(
            value >= -1.0 && value <= 1.0,
            "Raw Perlin should be in [-1, 1] range, got {}",
            value
        );

        // Test determinism at raw Perlin level
        let pos = Vec3::new(1.5, 2.3, 0.7);
        let v1 = perlin_3d(pos, seed);
        let v2 = perlin_3d(pos, seed);
        assert_eq!(v1, v2, "Raw Perlin must be deterministic");
    }

    /// Test that FBM helper returns values in approximate [-1, 1] range
    #[test]
    fn test_fbm_helper() {
        let position = Vec3::new(0.5, 0.5, 0.5);
        let seed = 123;

        let value = sample_perlin_fbm(position, seed, 4, 0.5, 2.0);

        // FBM output should be approximately in [-1, 1], though might slightly exceed
        assert!(
            value >= -1.5 && value <= 1.5,
            "FBM value {} is outside reasonable range",
            value
        );
    }
}
