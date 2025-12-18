//! Complete workflow demonstration for rust_voronoi_planet

use rust_voronoi_planet::*;

fn main() -> Result<()> {
    println!("=== rust_voronoi_planet Complete Demo ===\n");

    // Step 1: Configure planet
    println!("Step 1: Configuring planet...");
    let config = PlanetConfigBuilder::new()
        .seed(12345)
        .planet_size(PlanetSize::Small)
        .lloyd_iterations(5)?
        .build()?;

    println!("  Seed: {}", config.seed);
    println!("  Size: {:?} ({} cells)", config.planet_size, config.cell_count());
    println!("  Radius: {}", config.radius());

    // Step 2: Generate planet
    println!("\nStep 2: Generating planet...");
    let planet = VoronoiPlanet::generate(config)?;
    println!("  Generated {} cells", planet.cell_count());

    // Step 3: Analyze terrain
    println!("\nStep 3: Terrain distribution:");
    let mut terrain_counts = std::collections::HashMap::new();
    for cell in planet.cells() {
        *terrain_counts.entry(cell.terrain).or_insert(0usize) += 1;
    }
    let mut sorted_terrain: Vec<_> = terrain_counts.iter().collect();
    sorted_terrain.sort_by_key(|(terrain, _)| format!("{:?}", terrain));
    for (terrain, count) in sorted_terrain {
        let pct = (*count as f32 / planet.cell_count() as f32) * 100.0;
        println!("  {:?}: {} ({:.1}%)", terrain, count, pct);
    }

    // Step 4: Query spatial index
    #[cfg(feature = "spatial-index")]
    {
        println!("\nStep 4: Spatial queries:");
        let test_pos = Vec3::new(config.radius(), 0.0, 0.0);
        let cell_id = planet.find_cell_at(test_pos);
        let cell = planet.get_cell(cell_id).unwrap();
        println!("  Position {:?} -> Cell {} ({:?})", test_pos, cell_id, cell.terrain);
        println!("  Cell has {} neighbors", cell.neighbors.len());
    }

    // Step 5: Generate mesh
    println!("\nStep 5: Generating mesh...");
    let mesh = generate_mesh(&planet, &BasicColorMapper);
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());

    // Memory estimate
    let mem = (mesh.positions.len() * 12 + mesh.normals.len() * 12 +
               mesh.colors.len() * 16 + mesh.indices.len() * 4) as f32 / 1024.0 / 1024.0;
    println!("  Memory: {:.2} MB", mem);

    println!("\n=== Demo Complete ===");
    Ok(())
}
