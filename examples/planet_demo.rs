//! Demonstration of VoronoiPlanet generation and usage

use rust_voronoi_planet::*;

fn main() -> Result<()> {
    println!("Generating planet...");

    let config = PlanetConfigBuilder::new()
        .seed(42)
        .planet_size(PlanetSize::Tiny)
        .lloyd_iterations(5)?
        .build()?;

    let planet = VoronoiPlanet::generate(config)?;

    println!("Generated {} cells", planet.cell_count());
    println!("Radius: {}", planet.radius());

    // Show terrain distribution
    let mut counts = std::collections::HashMap::new();
    for cell in planet.cells() {
        *counts.entry(cell.terrain).or_insert(0) += 1;
    }

    println!("\nTerrain distribution:");
    for (terrain, count) in &counts {
        let pct = (*count as f32 / planet.cell_count() as f32) * 100.0;
        println!("  {:?}: {} ({:.1}%)", terrain, count, pct);
    }

    // Test spatial index if available
    #[cfg(feature = "spatial-index")]
    {
        use glam::Vec3;
        let pos = Vec3::new(planet.radius(), 0.0, 0.0);
        let cell_id = planet.find_cell_at(pos);
        println!("\nPosition {:?} is in cell {}", pos, cell_id);

        // Show neighbors
        let neighbors = planet.get_neighbors(cell_id);
        println!("Cell {} has {} neighbors", cell_id, neighbors.len());

        // Show cells within radius
        let nearby = planet.find_cells_within_radius(cell_id, 2);
        println!("Found {} cells within 2 hops of cell {}", nearby.len(), cell_id);
    }

    Ok(())
}
