//! Mesh generation for VoronoiPlanet
//!
//! Generates engine-agnostic mesh data from VoronoiPlanet cells.

mod colors;

pub use colors::{ColorMapper, BasicColorMapper, CustomColorMapper, TerrainColor};

use crate::planet::VoronoiPlanet;
use glam::Vec3;

/// Engine-agnostic mesh data output
///
/// Contains raw vertex data suitable for any rendering engine:
/// - Bevy: Convert to `Mesh` with attributes
/// - Godot: Convert to `ArrayMesh`
/// - wgpu: Use directly as vertex buffers
#[derive(Debug, Clone, Default)]
pub struct MeshData {
    /// Vertex positions (3D coordinates)
    pub positions: Vec<[f32; 3]>,
    /// Vertex normals (normalized direction from sphere center)
    pub normals: Vec<[f32; 3]>,
    /// Vertex colors (RGBA)
    pub colors: Vec<[f32; 4]>,
    /// Triangle indices
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if mesh is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Generate mesh from planet with color mapping
///
/// Each cell is triangulated as a triangle fan from center to boundary vertices.
/// All vertices in a cell get the same color based on terrain type.
pub fn generate_mesh<T, C>(planet: &VoronoiPlanet<T>, color_mapper: &C) -> MeshData
where
    T: Clone,
    C: ColorMapper<T>,
{
    generate_mesh_with_visibility(planet, color_mapper, None, [0.0, 0.0, 0.0, 1.0])
}

/// Generate mesh with fog of war support
///
/// # Arguments
/// * `planet` - The planet to generate mesh for
/// * `color_mapper` - Maps terrain types to colors
/// * `visible_cells` - Optional slice of visible cell IDs. If None, all cells are visible.
/// * `hidden_color` - Color for hidden cells (typically black)
pub fn generate_mesh_with_visibility<T, C>(
    planet: &VoronoiPlanet<T>,
    color_mapper: &C,
    visible_cells: Option<&[usize]>,
    hidden_color: TerrainColor,
) -> MeshData
where
    T: Clone,
    C: ColorMapper<T>,
{
    let mut mesh = MeshData::default();

    // Convert visible_cells to HashSet for O(1) lookup
    let visible_set: Option<std::collections::HashSet<usize>> =
        visible_cells.map(|cells| cells.iter().copied().collect());

    for cell in planet.cells() {
        // Skip degenerate cells
        if cell.vertices.len() < 3 {
            continue;
        }

        // Check visibility
        let is_visible = visible_set
            .as_ref()
            .map(|set| set.contains(&cell.id))
            .unwrap_or(true);

        // Get color
        let color = if is_visible {
            color_mapper.map_color(&cell.terrain)
        } else {
            hidden_color
        };

        // Triangulate cell
        triangulate_cell(
            cell.center,
            &cell.vertices,
            color,
            &mut mesh,
        );
    }

    mesh
}

/// Triangulate a single cell as a triangle fan
fn triangulate_cell(
    center: Vec3,
    vertices: &[Vec3],
    color: TerrainColor,
    mesh: &mut MeshData,
) {
    let base_idx = mesh.positions.len() as u32;

    // Add center vertex
    mesh.positions.push([center.x, center.y, center.z]);
    let center_normal = center.normalize();
    mesh.normals.push([center_normal.x, center_normal.y, center_normal.z]);
    mesh.colors.push(color);

    // Add boundary vertices
    for vertex in vertices {
        mesh.positions.push([vertex.x, vertex.y, vertex.z]);
        let normal = vertex.normalize();
        mesh.normals.push([normal.x, normal.y, normal.z]);
        mesh.colors.push(color);
    }

    // Create triangle fan indices
    let num_vertices = vertices.len();
    for i in 0..num_vertices {
        let next_i = (i + 1) % num_vertices;
        mesh.indices.push(base_idx);                          // Center
        mesh.indices.push(base_idx + 1 + i as u32);           // Current vertex
        mesh.indices.push(base_idx + 1 + next_i as u32);      // Next vertex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlanetConfigBuilder, PlanetSize};

    #[test]
    fn test_generate_mesh() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();
        let color_mapper = BasicColorMapper;
        let mesh = generate_mesh(&planet, &color_mapper);

        assert!(!mesh.is_empty());
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len(), mesh.colors.len());
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_mesh_with_fog_of_war() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();
        let color_mapper = BasicColorMapper;

        // Only first 100 cells visible
        let visible: Vec<usize> = (0..100).collect();
        let mesh = generate_mesh_with_visibility(
            &planet,
            &color_mapper,
            Some(&visible),
            [0.0, 0.0, 0.0, 1.0],
        );

        assert!(!mesh.is_empty());
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_mesh_consistency() {
        let config = PlanetConfigBuilder::new()
            .seed(42)
            .planet_size(PlanetSize::Tiny)
            .build()
            .unwrap();

        let planet = VoronoiPlanet::generate(config).unwrap();
        let color_mapper = BasicColorMapper;

        // Generate twice with same input
        let mesh1 = generate_mesh(&planet, &color_mapper);
        let mesh2 = generate_mesh(&planet, &color_mapper);

        assert_eq!(mesh1.vertex_count(), mesh2.vertex_count());
        assert_eq!(mesh1.triangle_count(), mesh2.triangle_count());
    }
}
