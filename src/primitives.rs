use std::f32::consts::PI;

use crate::vertex::Vertex;

/// Generate a UV sphere with the given number of stacks (latitude) and
/// sectors (longitude). Returns a list of vertices with tangent/bitangent
/// computed from the parametric surface.
pub fn generate_sphere(radius: f32, stacks: u32, sectors: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let stack_angle = PI / 2.0 - (i as f32) * PI / (stacks as f32);
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = 2.0 * PI * (j as f32) / (sectors as f32);

            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            let nx = x / radius;
            let ny = y / radius;
            let nz = z / radius;

            let s = j as f32 / sectors as f32;
            let t = i as f32 / stacks as f32;

            // Tangent: partial derivative w.r.t. sector angle
            let tx = -sector_angle.sin();
            let ty = sector_angle.cos();
            let tz = 0.0;

            // Bitangent = normal × tangent
            let bx = ny * tz - nz * ty;
            let by = nz * tx - nx * tz;
            let bz = nx * ty - ny * tx;

            vertices.push(Vertex {
                position: [x, z, y], // Y-up
                normal: [nx, nz, ny],
                tex_coords: [s, t],
                tangent: [tx, tz, ty],
                bitangent: [bx, bz, by],
            });
        }
    }

    for i in 0..stacks {
        for j in 0..sectors {
            let first = i * (sectors + 1) + j;
            let second = first + sectors + 1;

            // Reversed winding to match the Y↔Z swizzle (which flips handedness)
            indices.push(first);
            indices.push(first + 1);
            indices.push(second);

            indices.push(first + 1);
            indices.push(second + 1);
            indices.push(second);
        }
    }

    (vertices, indices)
}

/// Generate a torus centred at the origin lying in the XZ plane.
pub fn generate_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segments {
        let u = 2.0 * PI * (i as f32) / (major_segments as f32);
        let cos_u = u.cos();
        let sin_u = u.sin();

        for j in 0..=minor_segments {
            let v = 2.0 * PI * (j as f32) / (minor_segments as f32);
            let cos_v = v.cos();
            let sin_v = v.sin();

            let x = (major_radius + minor_radius * cos_v) * cos_u;
            let y = minor_radius * sin_v;
            let z = (major_radius + minor_radius * cos_v) * sin_u;

            // Normal: direction from centre of tube to surface
            let nx = cos_v * cos_u;
            let ny = sin_v;
            let nz = cos_v * sin_u;

            let s = i as f32 / major_segments as f32;
            let t = j as f32 / minor_segments as f32;

            // Tangent along the major circle
            let tx = -sin_u;
            let ty = 0.0;
            let tz = cos_u;

            // Bitangent = normal × tangent
            let bx = ny * tz - nz * ty;
            let by = nz * tx - nx * tz;
            let bz = nx * ty - ny * tx;

            vertices.push(Vertex {
                position: [x, y, z],
                normal: [nx, ny, nz],
                tex_coords: [s, t],
                tangent: [tx, ty, tz],
                bitangent: [bx, by, bz],
            });
        }
    }

    for i in 0..major_segments {
        for j in 0..minor_segments {
            let first = i * (minor_segments + 1) + j;
            let second = first + minor_segments + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(first + 1);
            indices.push(second);
            indices.push(second + 1);
        }
    }

    (vertices, indices)
}

/// Generate a simple cube of the given `half_size`.
pub fn generate_cube(half: f32) -> (Vec<Vertex>, Vec<u32>) {
    // 6 faces, each with 4 vertices, proper normals & tangents
    let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
        // normal,        tangent,       sign for bitangent
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),  // +Z
        ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), // -Z
        ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),  // +X
        ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),  // -X
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),  // +Y
        ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),  // -Y
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (face_i, (n, t, b)) in faces.iter().enumerate() {
        // Build a local frame
        let right = [t[0] * half, t[1] * half, t[2] * half];
        let up = [b[0] * half, b[1] * half, b[2] * half];
        let center = [n[0] * half, n[1] * half, n[2] * half];

        let offsets: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let base = (face_i * 4) as u32;
        for (vi, (off, uv)) in offsets.iter().zip(uvs.iter()).enumerate() {
            let _ = vi;
            let pos = [
                center[0] + right[0] * off[0] + up[0] * off[1],
                center[1] + right[1] * off[0] + up[1] * off[1],
                center[2] + right[2] * off[0] + up[2] * off[1],
            ];
            vertices.push(Vertex {
                position: pos,
                normal: *n,
                tex_coords: *uv,
                tangent: *t,
                bitangent: *b,
            });
        }

        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    (vertices, indices)
}

/// Generate a flat ground plane.
#[allow(dead_code)]
pub fn generate_plane(half_size: f32, subdivisions: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = (2.0 * half_size) / subdivisions as f32;
    let tex_step = 1.0 / subdivisions as f32;

    for i in 0..=subdivisions {
        for j in 0..=subdivisions {
            let x = -half_size + j as f32 * step;
            let z = -half_size + i as f32 * step;
            let u = j as f32 * tex_step;
            let v = i as f32 * tex_step;

            vertices.push(Vertex {
                position: [x, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [u * half_size, v * half_size], // tile texture
                tangent: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 1.0],
            });
        }
    }

    for i in 0..subdivisions {
        for j in 0..subdivisions {
            let tl = i * (subdivisions + 1) + j;
            let tr = tl + 1;
            let bl = tl + subdivisions + 1;
            let br = bl + 1;
            indices.push(tl);
            indices.push(bl);
            indices.push(tr);
            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    (vertices, indices)
}

/// Generate a cone pointing along -Z (tip at origin, base at z = -length).
/// This orientation makes it easy to align with a direction vector.
pub fn generate_cone(radius: f32, length: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Tip vertex at origin
    vertices.push(Vertex {
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.5, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 1.0, 0.0],
    });

    // Base ring vertices
    let slope_len = (radius * radius + length * length).sqrt();
    let nz = radius / slope_len;
    let nr = length / slope_len;

    for i in 0..=segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        vertices.push(Vertex {
            position: [radius * cos_a, radius * sin_a, -length],
            normal: [nr * cos_a, nr * sin_a, nz],
            tex_coords: [i as f32 / segments as f32, 1.0],
            tangent: [-sin_a, cos_a, 0.0],
            bitangent: [0.0, 0.0, -1.0],
        });
    }

    // Side triangles (tip to base ring)
    for i in 0..segments {
        indices.push(0); // tip
        indices.push(1 + i);
        indices.push(2 + i);
    }

    // Base cap center
    let base_center_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [0.0, 0.0, -length],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [0.5, 1.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 1.0, 0.0],
    });

    // Base cap triangles
    let base_ring_start = 1;
    for i in 0..segments {
        indices.push(base_center_idx);
        indices.push(base_ring_start + i + 1);
        indices.push(base_ring_start + i);
    }

    (vertices, indices)
}
