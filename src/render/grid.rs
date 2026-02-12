use cgmath::Matrix4;
use glium::uniforms::{UniformValue, Uniforms};
use glium::{DrawParameters, Surface};

use crate::core::light::{Light, LightKind};
use crate::core::types::mat4_to_array;
use crate::core::vertex::Vertex;

// ───────────────────── Grid Configuration ─────────────────────

/// Configuration for the procedural grid floor.
pub struct GridConfig {
    /// Major grid spacing in world units (default: 1.0).
    pub grid_size: f32,
    /// Sub-grid (subdivision) spacing (default: 0.25 → 4 subdivisions per cell).
    pub sub_grid_size: f32,
    /// Distance from camera at which the grid fully fades out.
    pub fade_radius: f32,
    /// Line width in world units.
    pub line_width: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            grid_size: 1.0,
            sub_grid_size: 0.25,
            fade_radius: 60.0,
            line_width: 0.02,
        }
    }
}

// ───────────────────── Grid Renderer ──────────────────────────

/// Holds GPU resources for the procedural grid floor.
pub struct Grid {
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u32>,
    pub config: GridConfig,
}

impl Grid {
    /// Create the grid. Generates a large subdivided quad so the fragment shader
    /// has enough geometry to rasterize over.
    pub fn new(display: &glium::Display<glium::glutin::surface::WindowSurface>) -> Self {
        let vert_src = include_str!("../../assets/shaders/grid.vert");
        let frag_src = include_str!("../../assets/shaders/grid.frag");

        let program = glium::Program::from_source(display, vert_src, frag_src, None)
            .expect("Failed to compile grid shaders");

        // Generate a large ground-plane quad (Y = 0) that extends far enough.
        let half = 100.0_f32;

        let vertices = vec![
            make_grid_vertex(-half, -half),
            make_grid_vertex(half, -half),
            make_grid_vertex(half, half),
            make_grid_vertex(-half, half),
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .unwrap();

        Self {
            program,
            vertex_buffer,
            index_buffer,
            config: GridConfig::default(),
        }
    }

    /// Draw the grid floor.
    ///
    /// Should be drawn **after** opaque geometry with blending enabled
    /// (or before other transparent objects).
    pub fn draw(
        &self,
        target: &mut glium::Frame,
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
        fog_enabled: bool,
        fog_color: [f32; 3],
        fog_density: f32,
        ambient_strength: f32,
        spotlights: &[Light],
    ) {
        let view_arr = mat4_to_array(view);
        let proj_arr = mat4_to_array(projection);

        // Collect spotlight data (up to 4)
        const MAX_SPOTS: usize = 4;
        let mut spot_positions = [[0.0_f32; 3]; MAX_SPOTS];
        let mut spot_directions = [[0.0_f32; 3]; MAX_SPOTS];
        let mut spot_colors = [[0.0_f32; 3]; MAX_SPOTS];
        let mut spot_cutoffs = [0.0_f32; MAX_SPOTS];
        let mut spot_outer_cutoffs = [0.0_f32; MAX_SPOTS];
        let mut num_spots = 0_i32;

        for light in spotlights {
            if light.kind != LightKind::Spot {
                continue;
            }
            if (num_spots as usize) >= MAX_SPOTS {
                break;
            }
            let i = num_spots as usize;
            spot_positions[i] = [light.position.x, light.position.y, light.position.z];
            spot_directions[i] = [light.direction.x, light.direction.y, light.direction.z];
            spot_colors[i] = light.diffuse;
            spot_cutoffs[i] = light.cutoff;
            spot_outer_cutoffs[i] = light.outer_cutoff;
            num_spots += 1;
        }

        let uniforms = GridUniforms {
            view: view_arr,
            projection: proj_arr,
            grid_size: self.config.grid_size,
            sub_grid_size: self.config.sub_grid_size,
            fade_radius: self.config.fade_radius,
            line_width: self.config.line_width,
            fog_enabled,
            fog_color,
            fog_density,
            ambient_strength,
            num_spots,
            spot_positions,
            spot_directions,
            spot_colors,
            spot_cutoffs,
            spot_outer_cutoffs,
        };

        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            // No backface culling so grid is visible from below too
            ..Default::default()
        };

        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &params,
            )
            .expect("Grid draw failed");
    }
}

// ─────────── Helpers ────────────

fn make_grid_vertex(x: f32, z: f32) -> Vertex {
    Vertex {
        position: [x, 0.0, z],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    }
}

/// Uniform struct for the grid shader.
struct GridUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    grid_size: f32,
    sub_grid_size: f32,
    fade_radius: f32,
    line_width: f32,
    fog_enabled: bool,
    fog_color: [f32; 3],
    fog_density: f32,
    ambient_strength: f32,
    num_spots: i32,
    spot_positions: [[f32; 3]; 4],
    spot_directions: [[f32; 3]; 4],
    spot_colors: [[f32; 3]; 4],
    spot_cutoffs: [f32; 4],
    spot_outer_cutoffs: [f32; 4],
}

impl Uniforms for GridUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        f("u_view", UniformValue::Mat4(self.view));
        f("u_projection", UniformValue::Mat4(self.projection));
        f("u_grid_size", UniformValue::Float(self.grid_size));
        f("u_sub_grid_size", UniformValue::Float(self.sub_grid_size));
        f("u_fade_radius", UniformValue::Float(self.fade_radius));
        f("u_line_width", UniformValue::Float(self.line_width));
        f("u_fog_enabled", UniformValue::Bool(self.fog_enabled));
        f("u_fog_color", UniformValue::Vec3(self.fog_color));
        f("u_fog_density", UniformValue::Float(self.fog_density));
        f("u_ambient_strength", UniformValue::Float(self.ambient_strength));
        f("u_num_spots", UniformValue::SignedInt(self.num_spots));

        // Per-spotlight uniforms (indexed arrays)
        for i in 0..4 {
            f(
                &format!("u_spot_position[{}]", i),
                UniformValue::Vec3(self.spot_positions[i]),
            );
            f(
                &format!("u_spot_direction[{}]", i),
                UniformValue::Vec3(self.spot_directions[i]),
            );
            f(
                &format!("u_spot_color[{}]", i),
                UniformValue::Vec3(self.spot_colors[i]),
            );
            f(
                &format!("u_spot_cutoff[{}]", i),
                UniformValue::Float(self.spot_cutoffs[i]),
            );
            f(
                &format!("u_spot_outer_cutoff[{}]", i),
                UniformValue::Float(self.spot_outer_cutoffs[i]),
            );
        }
    }
}
