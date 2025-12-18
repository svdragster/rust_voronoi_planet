# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test

# Run tests with specific feature
cargo test --features serde

# Run a specific example
cargo run --example generate_planet
cargo run --example terrain_demo
cargo run --example mesh_demo
cargo run --example full_demo

# Run example with release optimizations (recommended for larger planets)
cargo run --example generate_planet --release

# Check without building
cargo check
```

## Features

- `spatial-index` (default): Enables O(log n) position-to-cell lookups using KD-tree (kiddo)
- `serde`: Enables serialization support for configuration and cells

## Architecture

This library generates Voronoi-tessellated sphere meshes for procedural planet generation.

### Generation Pipeline (`src/generation/`)

The Voronoi generation follows this pipeline:

1. **Point Generation** (`points.rs`): Random points uniformly distributed on sphere using ChaCha8Rng for determinism
2. **Lloyd's Relaxation** (`lloyd.rs`): Iterative refinement moving points to cell centroids for uniform distribution. Features convergence detection (stops early when points stabilize).
3. **Delaunay Triangulation** (`delaunay.rs`): Uses parry3d convex hull (convex hull of sphere points = Delaunay triangulation)
4. **Voronoi Construction** (`voronoi.rs`): Computes circumcenters, orders vertices CCW, finds neighbors

### Core Types

- `PlanetConfig` / `PlanetConfigBuilder` (`config.rs`): Serializable configuration (seed, size, lloyd iterations, lloyd_convergence)
- `VoronoiPlanet<T>` (`planet.rs`): Complete planet with cells, generic over terrain type T
- `VoronoiCell<T>` (`cell.rs`): Individual cell with id, center, terrain, neighbors, vertices
- `RawCell` (`generation/voronoi.rs`): Geometry-only cell before terrain is applied

### Terrain System (`src/terrain/`)

- `TerrainSampler` trait: Sample terrain at 3D positions
- `PerlinTerrainSampler`: Default sampler using 3D Perlin noise with domain warping
- `BasicTerrainType`: Ocean, Beach, Land, Mountain, Ice

### Mesh Generation (`src/mesh/`)

- `MeshData`: Engine-agnostic output (positions, normals, colors, indices)
- `ColorMapper` trait: Map terrain types to RGBA colors
- Cells are triangulated as triangle fans from center to boundary

### Spatial Queries (`src/spatial.rs`)

When `spatial-index` feature is enabled:
- `SpatialIndex`: KD-tree wrapper for O(log n) nearest-neighbor lookups
- `planet.find_cell_at(position)`: Convert 3D position to cell ID

## Key Design Decisions

- **Determinism**: Same seed produces identical planet (ChaCha8Rng, stable sort)
- **Engine-agnostic**: Raw mesh data compatible with Bevy, Godot, wgpu
- **Serialization**: Only config is serialized (~20 bytes), planets regenerated from config
- **Generic terrain**: `VoronoiCell<T>` allows custom terrain types via `TerrainSampler`

## Planet Sizes

| Size   | Cells  | Radius |
|--------|--------|--------|
| Tiny   | 5,000  | 11.3   |
| Small  | 11,000 | 16.7   |
| Medium | 17,000 | 20.9   |
| Large  | 26,000 | 25.8   |
