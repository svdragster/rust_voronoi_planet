# rust_voronoi_planet

Voronoi-based planet mesh generation for games and simulations.

![bevy_voronoi_planets](https://github.com/user-attachments/assets/0802d7d0-f635-48e7-a3a8-13de1b715ac5)

## Features

- Generates Voronoi-tessellated sphere meshes for procedural planets
- Deterministic generation from seed values
- Engine-agnostic mesh output (positions, normals, colors, indices)
- Optional KD-tree spatial indexing for O(log n) position-to-cell lookups
- Configurable terrain system with Perlin noise

## Quick Start

```rust
use rust_voronoi_planet::prelude::*;

let config = PlanetConfigBuilder::new()
    .seed(42)
    .size(PlanetSize::Small)
    .point_distribution(PointDistribution::Fibonacci)
    .lloyd_iterations(1)?
    .build()?;

let planet: VoronoiPlanet<BasicTerrainType> = VoronoiPlanet::generate(
    &config,
    &PerlinTerrainSampler::new(config.seed()),
);

let mesh = MeshData::from_planet(&planet, &BasicColorMapper);
```

## Interacting with a Planet

### Querying Cells

```rust
// Get a specific cell by ID
let cell = planet.get_cell(42).unwrap();
println!("Terrain: {:?}", cell.terrain);
println!("Center: {:?}", cell.center);
println!("Neighbors: {:?}", cell.neighbors);

// Iterate over all cells
for cell in planet.cells() {
    // Process each cell
}
```

### Neighbor Navigation

```rust
// Get neighboring cell IDs (useful for pathfinding, flood fill)
let neighbors = planet.get_neighbors(cell_id);
for &neighbor_id in neighbors {
    let neighbor = planet.get_cell(neighbor_id).unwrap();
    // ...
}

// Check if two cells are adjacent
if cell.is_neighbor_of(other_cell_id) {
    // Cells share an edge
}
```

### Spatial Queries

```rust
// Find which cell contains a 3D position (requires spatial-index feature)
// Useful for raycasting, mouse clicks, entity placement
let position = Vec3::new(planet.radius(), 0.0, 0.0);
let cell_id = planet.find_cell_at(position);
```

### Finding Nearby Cells

```rust
// Get all cells within N hops using BFS
// Useful for area-of-effect, territory expansion
let nearby = planet.find_cells_within_radius(center_id, 3);
println!("Found {} cells within 3 hops", nearby.len());
```

### Terrain Analysis

```rust
// Count cells by terrain type
let ocean_count = planet.cells()
    .iter()
    .filter(|c| c.terrain == BasicTerrainType::Ocean)
    .count();

let land_pct = 100.0 * (planet.cell_count() - ocean_count) as f32
    / planet.cell_count() as f32;
println!("Land coverage: {:.1}%", land_pct);
```

## Installation

```toml
[dependencies]
rust_voronoi_planet = "0.1"
```

### Optional Features

- `spatial-index` (default): KD-tree for position-to-cell lookups
- `serde`: Serialization support for config

## Example

### Godot Example
Please see ![https://github.com/svdragster/godot_rust_voronoi_planet](https://github.com/svdragster/godot_rust_voronoi_planet)

### Bevy Example
See `examples/bevy_visualization/` for a full Bevy demo with 3 planets (Earth, Mars, Alien) showcasing custom terrain types and color mappers.

```bash
cd examples/bevy_visualization
cargo run --release
```

Controls: `1/2/3` to switch planets, mouse drag to orbit, scroll to zoom.

## License

MIT
