# Voronoi Generation Module

This module implements the core algorithm for generating Voronoi cells on a sphere surface.

## Architecture

The generation pipeline consists of several stages:

1. **Point Generation** (`points.rs`)
   - Generates random points uniformly distributed on sphere surface
   - Uses deterministic RNG (ChaCha8Rng) for reproducibility
   - Points are normalized to sphere radius

2. **Lloyd's Relaxation** (`lloyd.rs`)
   - Optional iterative refinement for uniform distribution
   - Moves each point to the centroid of its Voronoi cell
   - Produces honeycomb-like hexagonal cells after 5+ iterations
   - Each iteration requires full Delaunay triangulation

3. **Delaunay Triangulation** (`delaunay.rs`)
   - Uses parry3d's convex hull algorithm
   - For points on a sphere, convex hull = Delaunay triangulation
   - Returns vertices (may be reordered) and triangle connectivity

4. **Voronoi Construction** (`voronoi.rs`)
   - Computes circumcenters of Delaunay triangles
   - Orders vertices counter-clockwise around each cell
   - Finds neighbor relationships via shared triangles
   - Returns `RawCell` structures (geometry only, no terrain)

## Usage

```rust
use rust_voronoi_planet::*;

// Create configuration
let config = PlanetConfigBuilder::new()
    .seed(42)
    .planet_size(PlanetSize::Small)
    .lloyd_iterations(5)
    .unwrap()
    .build()
    .unwrap();

// Generate cells (geometry only)
let cells = generate_raw_cells(&config)?;

// Cells contain:
// - id: unique identifier
// - center: position on sphere
// - neighbors: adjacent cell IDs
// - vertices: boundary polygon (ordered CCW)
```

## Key Data Structures

### RawCell

A Voronoi cell without terrain information:

```rust
pub struct RawCell {
    pub id: usize,              // Unique identifier
    pub center: Vec3,           // Center point on sphere
    pub neighbors: Vec<usize>,  // Adjacent cell IDs
    pub vertices: Vec<Vec3>,    // Boundary vertices (CCW)
}
```

### Adjacency Maps

Two key maps enable efficient neighbor finding:

- **VertexTriangleMap**: `vertex_id -> [triangle_ids]`
  - Maps each vertex to all triangles that include it
  - Used to find a vertex's Voronoi cell vertices (triangle circumcenters)

- **TriangleVertexMap**: `triangle_id -> [vertex_ids]`
  - Reverse lookup: which vertices are in each triangle
  - Enables O(1) neighbor finding (avoid O(N²) iteration)

## Algorithm Details

### Spherical Circumcenter

For a triangle on a sphere, the circumcenter is computed as:

```rust
let edge1 = v1 - v0;
let edge2 = v2 - v0;
let normal = edge1.cross(edge2);
let circumcenter = normal.normalize() * radius;
```

The circumcenter lies perpendicular to the triangle's plane, on the sphere surface.

### Vertex Ordering

Vertices are ordered counter-clockwise by:
1. Computing tangent plane basis at cell center
2. Projecting each vertex onto the tangent plane
3. Computing angle from reference direction
4. Sorting by angle

This ensures proper polygon winding for rendering.

### Neighbor Finding

Two cells are neighbors if they share a Delaunay triangle:
1. Get all triangles adjacent to cell A
2. For each triangle, find all vertices in that triangle
3. All vertices != A are neighbors

## Performance

Generation time scales with cell count and Lloyd iterations:

- **5,000 cells** (Tiny): ~0.1s with 5 iterations
- **11,000 cells** (Small): ~0.3s with 5 iterations
- **17,000 cells** (Medium): ~0.5s with 5 iterations
- **26,000 cells** (Large): ~1.0s with 5 iterations

Lloyd's Relaxation dominates runtime (requires full triangulation per iteration).

## Determinism

The same `PlanetConfig` always produces identical cells:
- Deterministic RNG (ChaCha8Rng)
- Stable floating-point operations
- Consistent ordering (neighbor lists are sorted)

This enables:
- Client-server planet synchronization
- Save file compatibility
- Reproducible testing

## Future Work

Phase 3 will add terrain sampling to create `VoronoiCell<TerrainType>` from `RawCell`.
