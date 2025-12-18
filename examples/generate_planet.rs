//! Example: Generate a Voronoi planet
//!
//! Demonstrates the basic usage of the generation pipeline.

use rust_voronoi_planet::*;
use rust_voronoi_planet::generation::generate_raw_cells;

fn main() {
    println!("Voronoi Planet Generation Example");
    println!("==================================\n");

    // Create a configuration for a small planet
    let config = PlanetConfigBuilder::new()
        .seed(42)
        .planet_size(PlanetSize::Tiny) // Use Tiny for faster generation in example
        .lloyd_iterations(5)
        .unwrap()
        .build()
        .unwrap();

    println!("Configuration:");
    println!("  Seed: {}", config.seed);
    println!("  Planet Size: {}", config.planet_size.name());
    println!("  Cell Count: {}", config.cell_count());
    println!("  Sphere Radius: {}", config.radius());
    println!("  Lloyd Iterations: {}", config.lloyd_iterations);
    println!();

    // Generate raw cells (geometry only, no terrain)
    println!("Generating planet...");
    let cells = generate_raw_cells(&config).expect("Failed to generate planet");
    println!("Generated {} cells\n", cells.len());

    // Analyze the generated cells
    let total_neighbors: usize = cells.iter().map(|c| c.neighbors.len()).sum();
    let avg_neighbors = total_neighbors as f32 / cells.len() as f32;

    let total_vertices: usize = cells.iter().map(|c| c.vertices.len()).sum();
    let avg_vertices = total_vertices as f32 / cells.len() as f32;

    println!("Statistics:");
    println!("  Average neighbors per cell: {:.2}", avg_neighbors);
    println!("  Average vertices per cell: {:.2}", avg_vertices);
    println!();

    // Show details for first few cells
    println!("Sample cells:");
    for cell in cells.iter().take(5) {
        println!(
            "  Cell {}: center=({:.2}, {:.2}, {:.2}), neighbors={}, vertices={}",
            cell.id,
            cell.center.x,
            cell.center.y,
            cell.center.z,
            cell.neighbors.len(),
            cell.vertices.len()
        );
    }

    println!("\nGeneration complete!");
}
