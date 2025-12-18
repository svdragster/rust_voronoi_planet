//! Voronoi cell construction from Delaunay triangulation
//!
//! Constructs Voronoi cells by computing circumcenters of Delaunay triangles
//! and finding neighbor relationships.

use glam::Vec3;
use parry3d::math::Point;
use parry3d::transformation;
use std::collections::{HashMap, HashSet};

use crate::error::Result;

/// Type alias for vertex-triangle adjacency map
type VertexTriangleMap = HashMap<usize, Vec<usize>>;

/// A Voronoi cell without terrain (geometry only)
///
/// This is an intermediate representation used during generation.
/// Terrain is added later to create the final VoronoiCell.
#[derive(Debug, Clone)]
pub struct RawCell {
    /// Unique cell identifier
    pub id: usize,
    /// Center point on sphere surface
    pub center: Vec3,
    /// IDs of neighboring cells
    pub neighbors: Vec<usize>,
    /// Vertices defining the cell boundary (ordered counter-clockwise)
    pub vertices: Vec<Vec3>,
}

/// Generate Voronoi cells from points on a sphere
///
/// This is the main entry point for Voronoi cell construction.
/// It computes the Delaunay triangulation, builds adjacency maps,
/// and constructs each cell's geometry and neighbor list.
///
/// # Arguments
///
/// * `points` - Seed points on the sphere surface
/// * `radius` - Sphere radius
///
/// # Returns
///
/// Vector of raw cells (without terrain), one per input point
pub fn generate_cells(points: &[Point<f32>], radius: f32) -> Result<Vec<RawCell>> {
    // Step 3: Compute convex hull (Delaunay triangulation)
    let (vertices, triangle_indices) = transformation::convex_hull(points);

    // Step 4: Build vertex-triangle adjacency map
    let vertex_triangle_map = build_vertex_triangle_map(&triangle_indices);

    // Step 4b: Build triangle-vertex reverse lookup for O(1) neighbor finding
    let triangle_vertex_map = build_triangle_vertex_map(&vertex_triangle_map);

    // Step 5: Construct Voronoi cells
    let cells: Vec<RawCell> = (0..vertices.len())
        .map(|vertex_idx| {
            // Get all triangles adjacent to this vertex
            let adjacent_triangles = &vertex_triangle_map[&vertex_idx];

            // Compute circumcenters of adjacent triangles (Voronoi cell vertices)
            let circumcenters: Vec<Vec3> = adjacent_triangles
                .iter()
                .map(|&tri_idx| {
                    compute_spherical_circumcenter(
                        tri_idx,
                        &vertices,
                        &triangle_indices,
                        radius,
                    )
                })
                .collect();

            // Convert seed point to Vec3
            let center = Vec3::new(
                vertices[vertex_idx].x,
                vertices[vertex_idx].y,
                vertices[vertex_idx].z,
            );

            // Order circumcenters counter-clockwise to form proper polygon
            let ordered_vertices = order_voronoi_vertices(circumcenters, center, radius);

            // Determine neighbors (cells that share circumcenters/edges)
            let neighbors = find_cell_neighbors(vertex_idx, &vertex_triangle_map, &triangle_vertex_map);

            RawCell {
                id: vertex_idx,
                center,
                neighbors,
                vertices: ordered_vertices,
            }
        })
        .collect();

    Ok(cells)
}

/// Build map from vertex index to all triangles that include it
///
/// This adjacency map is essential for finding all triangles adjacent to each seed point.
fn build_vertex_triangle_map(triangle_indices: &[[u32; 3]]) -> VertexTriangleMap {
    let mut map: VertexTriangleMap = HashMap::new();

    for (tri_idx, triangle) in triangle_indices.iter().enumerate() {
        for &vertex_idx in triangle.iter() {
            map.entry(vertex_idx as usize)
                .or_insert_with(Vec::new)
                .push(tri_idx);
        }
    }

    map
}

/// Build reverse lookup from triangle index to vertices in that triangle
///
/// This allows O(1) lookup of which vertices are in a given triangle,
/// avoiding the O(N²) iteration over all vertices.
fn build_triangle_vertex_map(vertex_triangle_map: &VertexTriangleMap) -> HashMap<usize, Vec<usize>> {
    let mut triangle_vertex_map: HashMap<usize, Vec<usize>> = HashMap::new();

    for (&vertex_idx, triangle_list) in vertex_triangle_map.iter() {
        for &tri_idx in triangle_list {
            triangle_vertex_map
                .entry(tri_idx)
                .or_insert_with(Vec::new)
                .push(vertex_idx);
        }
    }

    triangle_vertex_map
}

/// Compute the circumcenter of a spherical triangle
///
/// For a triangle on a sphere, the circumcenter is perpendicular to the triangle's plane.
fn compute_spherical_circumcenter(
    tri_idx: usize,
    vertices: &[Point<f32>],
    triangle_indices: &[[u32; 3]],
    radius: f32,
) -> Vec3 {
    let tri = triangle_indices[tri_idx];

    // Get the three vertices of the triangle
    let v0 = Vec3::new(
        vertices[tri[0] as usize].x,
        vertices[tri[0] as usize].y,
        vertices[tri[0] as usize].z,
    );
    let v1 = Vec3::new(
        vertices[tri[1] as usize].x,
        vertices[tri[1] as usize].y,
        vertices[tri[1] as usize].z,
    );
    let v2 = Vec3::new(
        vertices[tri[2] as usize].x,
        vertices[tri[2] as usize].y,
        vertices[tri[2] as usize].z,
    );

    // Compute edges
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;

    // The circumcenter is perpendicular to the triangle plane
    let normal = edge1.cross(edge2);

    // Normalize and scale to sphere radius
    normal.normalize() * radius
}

/// Order Voronoi cell vertices counter-clockwise around seed point
///
/// Projects circumcenters onto the tangent plane and sorts by angle.
fn order_voronoi_vertices(circumcenters: Vec<Vec3>, seed_point: Vec3, _radius: f32) -> Vec<Vec3> {
    if circumcenters.len() < 3 {
        return circumcenters;
    }

    // Get normal at seed point
    let normal = seed_point.normalize();

    // Choose reference direction in tangent plane
    let reference = if normal.x.abs() > 0.5 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };

    // Create orthogonal basis on tangent plane
    let tangent_u = reference.cross(normal).normalize();
    let tangent_v = normal.cross(tangent_u).normalize();

    // Compute angle for each circumcenter
    let mut vertices_with_angles: Vec<(Vec3, f32)> = circumcenters
        .iter()
        .map(|&cc| {
            let to_cc = cc - seed_point;
            let u = to_cc.dot(tangent_u);
            let v = to_cc.dot(tangent_v);
            let angle = v.atan2(u);
            (cc, angle)
        })
        .collect();

    // Sort by angle (counter-clockwise)
    vertices_with_angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    vertices_with_angles.into_iter().map(|(v, _)| v).collect()
}

/// Find neighbor cells by checking shared triangles
///
/// Two cells are neighbors if they share at least one Delaunay triangle.
///
/// OPTIMIZED: Instead of iterating over all vertices for each triangle (O(N²)),
/// we build a reverse lookup from triangles to vertices once.
fn find_cell_neighbors(
    cell_idx: usize,
    vertex_triangle_map: &VertexTriangleMap,
    triangle_vertex_map: &HashMap<usize, Vec<usize>>,
) -> Vec<usize> {
    let my_triangles = &vertex_triangle_map[&cell_idx];

    // Find all vertices that share triangles with this cell
    let mut neighbors = HashSet::new();

    for &tri_idx in my_triangles {
        // Get all vertices in this triangle directly (O(1) lookup)
        if let Some(triangle_vertices) = triangle_vertex_map.get(&tri_idx) {
            for &vertex in triangle_vertices {
                if vertex != cell_idx {
                    neighbors.insert(vertex);
                }
            }
        }
    }

    let mut neighbor_list: Vec<usize> = neighbors.into_iter().collect();
    neighbor_list.sort(); // Deterministic ordering
    neighbor_list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::points::generate_sphere_points;

    #[test]
    fn test_generate_cells() {
        let points = generate_sphere_points(100, 10.0, 42);
        let cells = generate_cells(&points, 10.0).unwrap();

        assert_eq!(cells.len(), 100);

        // Verify all cells have valid geometry
        for cell in &cells {
            assert!(cell.vertices.len() >= 3, "Cell should have at least 3 vertices");
            assert!(cell.neighbors.len() > 0, "Cell should have neighbors");

            // Verify center is on sphere surface
            let distance = cell.center.length();
            assert!((distance - 10.0).abs() < 0.01);

            // Verify vertices are on sphere surface
            for vertex in &cell.vertices {
                let vertex_distance = vertex.length();
                assert!((vertex_distance - 10.0).abs() < 0.01);
            }
        }
    }

    #[test]
    fn test_neighbor_symmetry() {
        let points = generate_sphere_points(50, 10.0, 12345);
        let cells = generate_cells(&points, 10.0).unwrap();

        // If A is a neighbor of B, then B should be a neighbor of A
        for cell in &cells {
            for &neighbor_id in &cell.neighbors {
                let neighbor = &cells[neighbor_id];
                assert!(
                    neighbor.neighbors.contains(&cell.id),
                    "Neighbor relationship should be symmetric"
                );
            }
        }
    }
}
