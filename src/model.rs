use std::path::Path;

use glium::Display;
use glium::index::PrimitiveType;
use tobj;

use crate::vertex::Vertex;

/// A single sub-mesh loaded from a model file.
pub struct Mesh {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u32>,
}

/// Load all meshes from an OBJ file. Returns one `Mesh` per shape in the file.
#[allow(dead_code)]
pub fn load_obj(display: &Display<glium::glutin::surface::WindowSurface>, path: &Path) -> Vec<Mesh> {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(path, &load_options)
        .unwrap_or_else(|e| panic!("Failed to load OBJ '{}': {}", path.display(), e));

    let mut meshes = Vec::new();

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

        // Compute tangent / bitangent per triangle and accumulate.
        let indices = &mesh.indices;
        for tri in indices.chunks(3) {
            if tri.len() < 3 {
                continue;
            }
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p0 = vertices[i0].position;
            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;

            let uv0 = vertices[i0].tex_coords;
            let uv1 = vertices[i1].tex_coords;
            let uv2 = vertices[i2].tex_coords;

            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
            let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

            let denom = duv1[0] * duv2[1] - duv2[0] * duv1[1];
            let f = if denom.abs() < 1e-8 { 1.0 } else { 1.0 / denom };

            let tangent = [
                f * (duv2[1] * edge1[0] - duv1[1] * edge2[0]),
                f * (duv2[1] * edge1[1] - duv1[1] * edge2[1]),
                f * (duv2[1] * edge1[2] - duv1[1] * edge2[2]),
            ];
            let bitangent = [
                f * (-duv2[0] * edge1[0] + duv1[0] * edge2[0]),
                f * (-duv2[0] * edge1[1] + duv1[0] * edge2[1]),
                f * (-duv2[0] * edge1[2] + duv1[0] * edge2[2]),
            ];

            for &idx in &[i0, i1, i2] {
                for k in 0..3 {
                    vertices[idx].tangent[k] += tangent[k];
                    vertices[idx].bitangent[k] += bitangent[k];
                }
            }
        }

        // Normalize accumulated tangent/bitangent.
        for v in &mut vertices {
            let len_t = (v.tangent[0] * v.tangent[0]
                + v.tangent[1] * v.tangent[1]
                + v.tangent[2] * v.tangent[2])
                .sqrt();
            if len_t > 1e-6 {
                v.tangent[0] /= len_t;
                v.tangent[1] /= len_t;
                v.tangent[2] /= len_t;
            }
            let len_b = (v.bitangent[0] * v.bitangent[0]
                + v.bitangent[1] * v.bitangent[1]
                + v.bitangent[2] * v.bitangent[2])
                .sqrt();
            if len_b > 1e-6 {
                v.bitangent[0] /= len_b;
                v.bitangent[1] /= len_b;
                v.bitangent[2] /= len_b;
            }
        }

        let vb = glium::VertexBuffer::new(display, &vertices)
            .expect("Failed to create vertex buffer");
        let ib = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, indices)
            .expect("Failed to create index buffer");

        meshes.push(Mesh {
            vertex_buffer: vb,
            index_buffer: ib,
        });
    }

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
    }
}
