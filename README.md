# rust_voronoi_planet

Voronoi-based planet mesh generation for games and simulations.

## Features

- Generates Voronoi-tessellated sphere meshes for procedural planets
- Deterministic generation from seed values
- Engine-agnostic mesh output (positions, normals, colors, indices)
- Optional spatial indexing for O(log n) cell lookups
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

## Installation

```toml
[dependencies]
rust_voronoi_planet = "0.1"
```

### Optional Features

- `spatial-index` (default): KD-tree for position-to-cell lookups
- `serde`: Serialization support for config

## Example

See `examples/bevy_visualization/` for a full Bevy demo with 3 planets (Earth, Mars, Alien) showcasing custom terrain types and color mappers.

```bash
cd examples/bevy_visualization
cargo run --release
```

Controls: `1/2/3` to switch planets, mouse drag to orbit, scroll to zoom.

## License

MIT
