/// Vertex type used across all meshes in the application.
///
/// Matches the layout expected by the GLSL shaders (locations 0–4).
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

glium::implement_vertex!(Vertex, position, normal, tex_coords, tangent, bitangent);
