use std::collections::HashMap;
use std::path::Path;

use glium::Display;
use glium::index::PrimitiveType;
use tobj;

use crate::types::Material;
use crate::vertex::Vertex;

/// A single sub-mesh loaded from a model file.
pub struct Mesh {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u32>,
    /// Per-mesh material from the MTL file (if available).
    pub material: Option<Material>,
}

/// Load all meshes from an OBJ file, **merging** sub-meshes that share the
/// same material into a single draw call. This is critical for complex models
/// (e.g. a car with 1000+ parts but only ~38 materials).
#[allow(dead_code)]
pub fn load_obj(display: &Display<glium::glutin::surface::WindowSurface>, path: &Path) -> Vec<Mesh> {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    // Resolve to absolute path so tobj finds the MTL file next to the OBJ
    let abs_path = if path.is_relative() {
        std::env::current_dir().unwrap().join(path)
    } else {
        path.to_path_buf()
    };

    let (models, materials_result) = tobj::load_obj(&abs_path, &load_options)
        .unwrap_or_else(|e| panic!("Failed to load OBJ '{}': {}", abs_path.display(), e));

    // Parse MTL materials → our Material type
    let raw_mats = match materials_result {
        Ok(mats) => mats,
        Err(_) => Vec::new(),
    };

    let mtl_materials: Vec<Material> = raw_mats
        .iter()
        .map(|m| {
            let ka = m.ambient.unwrap_or([0.1, 0.1, 0.1]);
            let kd = m.diffuse.unwrap_or([0.8, 0.8, 0.8]);
            let ks = m.specular.unwrap_or([1.0, 1.0, 1.0]);
            let ns = m.shininess.unwrap_or(32.0);
            Material { ambient: ka, diffuse: kd, specular: ks, shininess: ns }
        })
        .collect();

    // ── Group all sub-mesh geometry by material_id ──
    // Key: material_id (None = no material)
    // Value: (accumulated vertices, accumulated indices)
    let mut groups: HashMap<Option<usize>, (Vec<Vertex>, Vec<u32>)> = HashMap::new();

    for model in &models {
        let mesh = &model.mesh;
        let num_vertices = mesh.positions.len() / 3;

        let has_normals = !mesh.normals.is_empty();
        let has_texcoords = !mesh.texcoords.is_empty();

        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);

        for i in 0..num_vertices {
            let px = mesh.positions[i * 3];
            let py = mesh.positions[i * 3 + 1];
            let pz = mesh.positions[i * 3 + 2];

            let (nx, ny, nz) = if has_normals {
                (
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                )
            } else {
                (0.0, 1.0, 0.0)
            };

            let (u, v) = if has_texcoords {
                (mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1])
            } else {
                (0.0, 0.0)
            };

            vertices.push(Vertex {
                position: [px, py, pz],
                normal: [nx, ny, nz],
                tex_coords: [u, v],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            });
        }

        // Compute tangent / bitangent per triangle
        let indices = &mesh.indices;
        for tri in indices.chunks(3) {
            if tri.len() < 3 { continue; }
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p0 = vertices[i0].position;
            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;
            let uv0 = vertices[i0].tex_coords;
            let uv1 = vertices[i1].tex_coords;
            let uv2 = vertices[i2].tex_coords;

            let edge1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
            let edge2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
            let duv1 = [uv1[0]-uv0[0], uv1[1]-uv0[1]];
            let duv2 = [uv2[0]-uv0[0], uv2[1]-uv0[1]];

            let denom = duv1[0]*duv2[1] - duv2[0]*duv1[1];
            let f = if denom.abs() < 1e-8 { 1.0 } else { 1.0 / denom };

            let tangent = [
                f * (duv2[1]*edge1[0] - duv1[1]*edge2[0]),
                f * (duv2[1]*edge1[1] - duv1[1]*edge2[1]),
                f * (duv2[1]*edge1[2] - duv1[1]*edge2[2]),
            ];
            let bitangent = [
                f * (-duv2[0]*edge1[0] + duv1[0]*edge2[0]),
                f * (-duv2[0]*edge1[1] + duv1[0]*edge2[1]),
                f * (-duv2[0]*edge1[2] + duv1[0]*edge2[2]),
            ];

            for &idx in &[i0, i1, i2] {
                for k in 0..3 {
                    vertices[idx].tangent[k] += tangent[k];
                    vertices[idx].bitangent[k] += bitangent[k];
                }
            }
        }

        // Normalize accumulated tangent/bitangent
        for v in &mut vertices {
            let len_t = (v.tangent[0]*v.tangent[0] + v.tangent[1]*v.tangent[1] + v.tangent[2]*v.tangent[2]).sqrt();
            if len_t > 1e-6 { v.tangent[0] /= len_t; v.tangent[1] /= len_t; v.tangent[2] /= len_t; }
            let len_b = (v.bitangent[0]*v.bitangent[0] + v.bitangent[1]*v.bitangent[1] + v.bitangent[2]*v.bitangent[2]).sqrt();
            if len_b > 1e-6 { v.bitangent[0] /= len_b; v.bitangent[1] /= len_b; v.bitangent[2] /= len_b; }
        }

        // Merge into the group for this material
        let mat_key = model.mesh.material_id;
        let group = groups.entry(mat_key).or_insert_with(|| (Vec::new(), Vec::new()));
        let base_index = group.0.len() as u32;
        group.0.extend_from_slice(&vertices);
        for &idx in indices {
            group.1.push(base_index + idx);
        }
    }

    // ── Build one Mesh per material group ──
    let mut meshes = Vec::with_capacity(groups.len());
    for (mat_key, (vertices, indices)) in groups {
        let vb = glium::VertexBuffer::new(display, &vertices)
            .expect("Failed to create vertex buffer");
        let ib = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices)
            .expect("Failed to create index buffer");
        let mesh_material = mat_key
            .and_then(|id| mtl_materials.get(id))
            .cloned();
        meshes.push(Mesh {
            vertex_buffer: vb,
            index_buffer: ib,
            material: mesh_material,
        });
    }

    println!("[OBJ] '{}': {} sub-meshes merged into {} draw calls ({} materials)",
        path.display(), models.len(), meshes.len(), mtl_materials.len());

    meshes
}

/// Create a `Mesh` from raw vertex + index data (used for procedural primitives).
pub fn mesh_from_data(
    display: &Display<glium::glutin::surface::WindowSurface>,
    vertices: &[Vertex],
    indices: &[u32],
) -> Mesh {
    Mesh {
        vertex_buffer: glium::VertexBuffer::new(display, vertices)
            .expect("Failed to create vertex buffer"),
        index_buffer: glium::IndexBuffer::new(
            display,
            PrimitiveType::TrianglesList,
            indices,
        )
        .expect("Failed to create index buffer"),
        material: None,
    }
}
