//! Bevy visualization example for rust_voronoi_planet
//!
//! Displays 3 different planet types: Earth, Mars, and Alien.
//!
//! Controls:
//! - 1: Orbit Earth (center)
//! - 2: Orbit Mars (right)
//! - 3: Orbit Alien (left)
//! - Mouse drag: Orbit camera
//! - Scroll: Zoom in/out

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::mesh::{Indices, PrimitiveTopology};
use rust_voronoi_planet::{
    PlanetConfigBuilder, PlanetSize, VoronoiPlanet,
    generate_mesh, BasicColorMapper, MeshData, TerrainSampler,
    ColorMapper, TerrainColor,
    terrain::sample_perlin_fbm,
    Vec3 as LibVec3,
};
use std::time::Instant;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<OrbitTarget>()
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, switch_target))
        .run();
}

// ============================================================================
// MARS TERRAIN
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum MarsTerrain {
    Dust,
    Rock,
    Canyon,
    PolarIce,
    DarkRock,
}

struct MarsTerrainSampler {
    seed: u32,
}

impl TerrainSampler for MarsTerrainSampler {
    type Output = MarsTerrain;

    fn sample(&self, position: LibVec3, radius: f32) -> MarsTerrain {
        let latitude = (position.y / radius).abs();

        // Larger polar caps than Earth
        if latitude > 0.75 {
            return MarsTerrain::PolarIce;
        }

        // Sample elevation
        let pos = LibVec3::new(-position.x, position.y, -position.z);
        let elevation = sample_perlin_fbm(pos * 0.15, self.seed, 4, 0.5, 2.0);

        // Mars is mostly land with canyons
        if elevation < -0.4 {
            MarsTerrain::Canyon
        } else if elevation < -0.1 {
            MarsTerrain::DarkRock
        } else if elevation > 0.3 {
            MarsTerrain::Rock
        } else {
            MarsTerrain::Dust
        }
    }
}

struct MarsColorMapper;

impl ColorMapper<MarsTerrain> for MarsColorMapper {
    fn map_color(&self, terrain: &MarsTerrain) -> TerrainColor {
        match terrain {
            MarsTerrain::Dust => [0.76, 0.42, 0.26, 1.0],      // Rusty orange
            MarsTerrain::Rock => [0.6, 0.35, 0.25, 1.0],       // Dark rust
            MarsTerrain::Canyon => [0.4, 0.2, 0.15, 1.0],      // Deep brown
            MarsTerrain::PolarIce => [0.9, 0.88, 0.85, 1.0],   // Slightly tinted ice
            MarsTerrain::DarkRock => [0.5, 0.28, 0.2, 1.0],    // Dark rusty rock
        }
    }
}

// ============================================================================
// ALIEN TERRAIN
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum AlienTerrain {
    ToxicSea,
    Crystal,
    FungalForest,
    Volcanic,
    Bioluminescent,
}

struct AlienTerrainSampler {
    seed: u32,
}

impl TerrainSampler for AlienTerrainSampler {
    type Output = AlienTerrain;

    fn sample(&self, position: LibVec3, _radius: f32) -> AlienTerrain {
        let pos = LibVec3::new(-position.x, position.y, -position.z);

        // Multiple noise layers for varied terrain
        let elevation = sample_perlin_fbm(pos * 0.12, self.seed, 3, 0.5, 2.0);
        let variation = sample_perlin_fbm(pos * 0.3, self.seed.wrapping_add(500), 2, 0.5, 2.0);

        if elevation < -0.2 {
            AlienTerrain::ToxicSea
        } else if elevation > 0.4 {
            AlienTerrain::Volcanic
        } else if variation > 0.3 {
            AlienTerrain::Crystal
        } else if variation < -0.2 {
            AlienTerrain::Bioluminescent
        } else {
            AlienTerrain::FungalForest
        }
    }
}

struct AlienColorMapper;

impl ColorMapper<AlienTerrain> for AlienColorMapper {
    fn map_color(&self, terrain: &AlienTerrain) -> TerrainColor {
        match terrain {
            AlienTerrain::ToxicSea => [0.2, 0.05, 0.3, 1.0],       // Deep purple
            AlienTerrain::Crystal => [0.3, 0.8, 0.9, 1.0],         // Cyan crystal
            AlienTerrain::FungalForest => [0.6, 0.2, 0.5, 1.0],    // Magenta
            AlienTerrain::Volcanic => [0.9, 0.4, 0.1, 1.0],        // Orange glow
            AlienTerrain::Bioluminescent => [0.2, 0.9, 0.5, 1.0],  // Bright green
        }
    }
}

// ============================================================================
// CAMERA AND CONTROLS
// ============================================================================

#[derive(Component)]
struct PlanetMesh;

#[derive(Component)]
struct OrbitCamera {
    distance: f32,
    yaw: f32,
    pitch: f32,
}

#[derive(Resource, Default)]
struct OrbitTarget {
    position: Vec3,
    index: usize,
}

const PLANET_POSITIONS: [Vec3; 3] = [
    Vec3::new(0.0, 0.0, 0.0),      // Earth (center)
    Vec3::new(100.0, 0.0, 0.0),    // Mars (right)
    Vec3::new(-100.0, 0.0, 0.0),   // Alien (left)
];

const PLANET_NAMES: [&str; 3] = ["Earth", "Mars", "Alien"];

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut orbit_target: ResMut<OrbitTarget>,
) {
    info!("Generating 3 planets...");
    let total_start = Instant::now();

    let base_seed = 42u32;

    // Generate Earth
    info!("Generating Earth...");
    let start = Instant::now();
    let earth_config = PlanetConfigBuilder::new()
        .seed(base_seed)
        .planet_size(PlanetSize::Tiny)
        .lloyd_iterations(5).unwrap()
        .build().unwrap();
    let earth = VoronoiPlanet::generate(earth_config).unwrap();
    let generation_time = start.elapsed();
    let mesh_start = Instant::now();
    let earth_mesh = generate_mesh(&earth, &BasicColorMapper);
    let mesh_time = mesh_start.elapsed();
    info!("  Earth: {} cells, generation: {:?}, mesh: {:?}",
          earth.cell_count(), generation_time, mesh_time);
    spawn_planet(&mut commands, &mut meshes, &mut materials, earth_mesh, PLANET_POSITIONS[0]);

    // Generate Mars
    info!("Generating Mars...");
    let start = Instant::now();
    let mars_config = PlanetConfigBuilder::new()
        .seed(base_seed + 1000)
        .planet_size(PlanetSize::Tiny)
        .lloyd_iterations(5).unwrap()
        .build().unwrap();
    let mars_sampler = MarsTerrainSampler { seed: mars_config.terrain_seed };
    let mars: VoronoiPlanet<MarsTerrain> = VoronoiPlanet::generate_with_sampler(mars_config, &mars_sampler).unwrap();
    let generation_time = start.elapsed();
    let mesh_start = Instant::now();
    let mars_mesh = generate_mesh(&mars, &MarsColorMapper);
    let mesh_time = mesh_start.elapsed();
    info!("  Mars: {} cells, generation: {:?}, mesh: {:?}",
          mars.cell_count(), generation_time, mesh_time);
    spawn_planet(&mut commands, &mut meshes, &mut materials, mars_mesh, PLANET_POSITIONS[1]);

    // Generate Alien
    info!("Generating Alien planet...");
    let start = Instant::now();
    let alien_config = PlanetConfigBuilder::new()
        .seed(base_seed + 2000)
        .planet_size(PlanetSize::Tiny)
        .lloyd_iterations(5).unwrap()
        .build().unwrap();
    let alien_sampler = AlienTerrainSampler { seed: alien_config.terrain_seed };
    let alien: VoronoiPlanet<AlienTerrain> = VoronoiPlanet::generate_with_sampler(alien_config, &alien_sampler).unwrap();
    let generation_time = start.elapsed();
    let mesh_start = Instant::now();
    let alien_mesh = generate_mesh(&alien, &AlienColorMapper);
    let mesh_time = mesh_start.elapsed();
    info!("  Alien: {} cells, generation: {:?}, mesh: {:?}",
          alien.cell_count(), generation_time, mesh_time);
    spawn_planet(&mut commands, &mut meshes, &mut materials, alien_mesh, PLANET_POSITIONS[2]);

    let total_time = total_start.elapsed();
    info!("=== Total generation time: {:?} ===", total_time);

    // Set initial orbit target to Earth
    orbit_target.position = PLANET_POSITIONS[0];
    orbit_target.index = 0;

    // Spawn camera
    let camera_distance = 40.0;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(camera_distance, camera_distance * 0.5, camera_distance)
            .looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            distance: camera_distance,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 0.3,
        },
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    // UI text showing controls
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("1: Terrestrial  2: Mars  3: Alien"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    info!("=== Controls ===");
    info!("1: Orbit Earth | 2: Orbit Mars | 3: Orbit Alien");
    info!("Mouse drag: Rotate | Scroll: Zoom");
}

fn spawn_planet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mesh_data: MeshData,
    position: Vec3,
) {
    let mesh = mesh_data_to_bevy_mesh(mesh_data);
    let mesh_handle = meshes.add(mesh);

    let material = StandardMaterial {
        base_color: Color::WHITE,
        unlit: false,
        ..default()
    };
    let material_handle = materials.add(material);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_translation(position),
        PlanetMesh,
    ));
}

fn mesh_data_to_bevy_mesh(mesh_data: MeshData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_data.colors);
    mesh.insert_indices(Indices::U32(mesh_data.indices));

    mesh
}

fn switch_target(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut orbit_target: ResMut<OrbitTarget>,
) {
    let new_index = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else {
        None
    };

    if let Some(idx) = new_index {
        if idx != orbit_target.index {
            orbit_target.index = idx;
            orbit_target.position = PLANET_POSITIONS[idx];
            info!("Now orbiting: {}", PLANET_NAMES[idx]);
        }
    }
}

fn orbit_camera(
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    orbit_target: Res<OrbitTarget>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
) {
    let Ok((mut transform, mut orbit)) = query.single_mut() else {
        return;
    };

    // Handle mouse drag for orbiting
    if mouse_button.pressed(MouseButton::Left) {
        for motion in mouse_motion.read() {
            orbit.yaw -= motion.delta.x * 0.005;
            orbit.pitch += motion.delta.y * 0.005;
            orbit.pitch = orbit.pitch.clamp(-1.4, 1.4);
        }
    } else {
        mouse_motion.clear();
    }

    // Handle scroll for zooming
    for ev in scroll.read() {
        orbit.distance -= ev.y * orbit.distance * 0.1;
        orbit.distance = orbit.distance.clamp(15.0, 100.0);
    }

    // Calculate camera position relative to target
    let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.distance * orbit.pitch.sin();
    let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();

    let target = orbit_target.position;
    transform.translation = target + Vec3::new(x, y, z);
    transform.look_at(target, Vec3::Y);
}
