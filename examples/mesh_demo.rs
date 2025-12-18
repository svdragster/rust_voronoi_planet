//! Demonstration of mesh generation

use rust_voronoi_planet::*;

fn main() -> Result<()> {
    println!("Generating planet...");

    let config = PlanetConfigBuilder::new()
        .seed(42)
        .planet_size(PlanetSize::Tiny)
        .lloyd_iterations(3)?
        .build()?;

    let planet = VoronoiPlanet::generate(config)?;
    println!("Generated {} cells", planet.cell_count());

    // Generate mesh with default colors
    let color_mapper = BasicColorMapper;
    let mesh = generate_mesh(&planet, &color_mapper);

    println!("\nMesh statistics:");
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());
    println!("  Indices: {}", mesh.indices.len());

    // Memory estimate
    let mem_positions = mesh.positions.len() * 12; // 3 floats * 4 bytes
    let mem_normals = mesh.normals.len() * 12;
    let mem_colors = mesh.colors.len() * 16; // 4 floats * 4 bytes
    let mem_indices = mesh.indices.len() * 4;
    let total = mem_positions + mem_normals + mem_colors + mem_indices;
    println!("\nMemory usage:");
    println!("  Positions: {} bytes", mem_positions);
    println!("  Normals: {} bytes", mem_normals);
    println!("  Colors: {} bytes", mem_colors);
    println!("  Indices: {} bytes", mem_indices);
    println!("  Total: {} bytes ({:.2} MB)", total, total as f32 / 1024.0 / 1024.0);

    // Test custom colors
    let custom = CustomColorMapper {
        ocean: [0.0, 0.2, 0.5, 1.0],
        land: [0.3, 0.5, 0.1, 1.0],
        ..Default::default()
    };
    let _mesh2 = generate_mesh(&planet, &custom);
    println!("\nCustom color mapper works!");

    // Test fog of war
    let visible: Vec<usize> = (0..100).collect();
    let fog_mesh = generate_mesh_with_visibility(&planet, &color_mapper, Some(&visible), [0.0, 0.0, 0.0, 1.0]);
    println!("Fog of war mesh: {} vertices", fog_mesh.vertex_count());

    // Test all planet sizes
    println!("\n=== Testing all planet sizes ===");
    for size in [PlanetSize::Tiny, PlanetSize::Small, PlanetSize::Medium, PlanetSize::Large] {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(size)
            .lloyd_iterations(2)?
            .build()?;

        let planet = VoronoiPlanet::generate(config)?;
        let mesh = generate_mesh(&planet, &color_mapper);

        let mem = mesh.positions.len() * 12
            + mesh.normals.len() * 12
            + mesh.colors.len() * 16
            + mesh.indices.len() * 4;

        println!("{:?}: {} cells, {} vertices, {} triangles, {:.2} MB",
            size,
            planet.cell_count(),
            mesh.vertex_count(),
            mesh.triangle_count(),
            mem as f32 / 1024.0 / 1024.0
        );
    }

    Ok(())
}
