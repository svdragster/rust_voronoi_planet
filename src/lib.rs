//! Voronoi-based planet mesh generation
//!
//! A standalone library for generating Voronoi-tessellated sphere meshes,
//! suitable for use with any game engine (Bevy, Godot, etc.)
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rust_voronoi_planet::*;
//!
//! // Generate a planet
//! let config = PlanetConfigBuilder::new()
//!     .seed(42)
//!     .planet_size(PlanetSize::Medium)
//!     .lloyd_iterations(5).unwrap()
//!     .build().unwrap();
//!
//! let planet = VoronoiPlanet::generate(config).unwrap();
//!
//! // Generate mesh for rendering
//! let mesh = generate_mesh(&planet, &BasicColorMapper);
//! println!("Generated {} triangles", mesh.triangle_count());
//! ```
//!
//! # Features
//!
//! - `spatial-index` (default): Enables O(log n) position-to-cell lookups using KD-tree
//! - `serde`: Enables serialization support for configuration and cells

// Modules
pub mod error;
pub mod config;
pub mod cell;
pub mod generation;
pub mod terrain;
pub mod planet;
pub mod mesh;

#[cfg(feature = "spatial-index")]
pub mod spatial;

// Re-export core types for convenience
pub use error::{VoronoiError, Result};
pub use config::{PlanetConfig, PlanetConfigBuilder, PlanetSize};
pub use cell::VoronoiCell;
pub use planet::VoronoiPlanet;
pub use terrain::{BasicTerrainType, TerrainSampler, PerlinTerrainSampler, PerlinConfig};
pub use mesh::{MeshData, generate_mesh, generate_mesh_with_visibility, ColorMapper, BasicColorMapper, CustomColorMapper, TerrainColor};
pub use generation::{RawCell, LloydOptions};

#[cfg(feature = "spatial-index")]
pub use spatial::SpatialIndex;

// Re-export glam::Vec3 for convenience
pub use glam::Vec3;
