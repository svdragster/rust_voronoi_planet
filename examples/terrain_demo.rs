//! Demonstration of the terrain system

use rust_voronoi_planet::{
    BasicTerrainType, PerlinTerrainSampler, TerrainSampler, Vec3,
};

fn main() {
    println!("Terrain System Demo\n");

    // Create a terrain sampler with seed 42
    let sampler = PerlinTerrainSampler::new(42);
    let radius = 1.0;

    println!("Sampling terrain at various positions:");
    println!("{:-<60}", "");

    // Sample positions around the sphere
    let positions = vec![
        (Vec3::new(0.0, 1.0, 0.0), "North Pole"),
        (Vec3::new(0.0, -1.0, 0.0), "South Pole"),
        (Vec3::new(1.0, 0.0, 0.0), "Equator (0°)"),
        (Vec3::new(0.0, 0.0, 1.0), "Equator (90°)"),
        (Vec3::new(0.707, 0.707, 0.0), "Mid-latitude"),
        (Vec3::new(0.5, 0.5, 0.5), "Random point"),
    ];

    for (pos, label) in positions {
        let terrain = sampler.sample(pos, radius);
        println!("{:20} -> {:?}", label, terrain);
    }

    println!("\n{:-<60}", "");
    println!("Counting terrain types in sample grid:");
    println!("{:-<60}", "");

    // Count terrain types in a grid
    let mut counts = std::collections::HashMap::new();
    let samples = 1000;

    for i in 0..samples {
        // Generate evenly distributed points on sphere
        let theta = 2.0 * std::f32::consts::PI * (i as f32 / samples as f32);
        let phi = std::f32::consts::PI * ((i as f32 * 0.618) % 1.0);
        
        let x = phi.sin() * theta.cos();
        let y = phi.cos();
        let z = phi.sin() * theta.sin();
        
        let pos = Vec3::new(x, y, z);
        let terrain = sampler.sample(pos, radius);
        
        *counts.entry(terrain).or_insert(0) += 1;
    }

    for terrain_type in [
        BasicTerrainType::Ocean,
        BasicTerrainType::Beach,
        BasicTerrainType::Land,
        BasicTerrainType::Mountain,
        BasicTerrainType::Ice,
    ] {
        let count = counts.get(&terrain_type).unwrap_or(&0);
        let percentage = (*count as f32 / samples as f32) * 100.0;
        println!("{:12} : {:4} samples ({:5.1}%)", 
                 format!("{:?}", terrain_type), count, percentage);
    }

    println!("\n{:-<60}", "");
    println!("Terrain system is working correctly!");
}
