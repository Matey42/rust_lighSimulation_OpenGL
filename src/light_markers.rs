use cgmath::{Matrix4, Vector3};
use glium::uniforms::{UniformValue, Uniforms};
use glium::{DrawParameters, Surface};

use crate::light::{Light, LightKind};
use crate::primitives;
use crate::types::mat4_to_array;
use crate::vertex::Vertex;

/// Renders small glowing spheres at each light source position
/// so the user can see where lights are.
pub struct LightMarkers {
    program: glium::Program,
    sphere_vb: glium::VertexBuffer<Vertex>,
    sphere_ib: glium::IndexBuffer<u32>,
}

impl LightMarkers {
    pub fn new(display: &glium::Display<glium::glutin::surface::WindowSurface>) -> Self {
        let vert_src = include_str!("../assets/shaders/light_marker.vert");
        let frag_src = include_str!("../assets/shaders/light_marker.frag");

        let program = glium::Program::from_source(display, vert_src, frag_src, None)
            .expect("Failed to compile light marker shaders");

        // Small sphere mesh for markers
        let (verts, indices) = primitives::generate_sphere(0.2, 12, 12);
        let sphere_vb = glium::VertexBuffer::new(display, &verts).unwrap();
        let sphere_ib = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .unwrap();

        Self {
            program,
            sphere_vb,
            sphere_ib,
        }
    }

    /// Draw a marker for each non-directional light.
    /// Directional lights have no position, so they are skipped.
    pub fn draw(
        &self,
        target: &mut glium::Frame,
        lights: &[Light],
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
    ) {
        let view_arr = mat4_to_array(view);
        let proj_arr = mat4_to_array(projection);

        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        for light in lights {
            // Skip directional lights (they have no position in the scene)
            if light.kind == LightKind::Directional {
                continue;
            }

            let pos = light.position;
            let model = Matrix4::from_translation(Vector3::new(pos.x, pos.y, pos.z));
            let model_arr = mat4_to_array(&model);

            let uniforms = MarkerUniforms {
                model: model_arr,
                view: view_arr,
                projection: proj_arr,
                light_color: light.diffuse,
            };

            target
                .draw(
                    &self.sphere_vb,
                    &self.sphere_ib,
                    &self.program,
                    &uniforms,
                    &params,
                )
                .expect("Light marker draw failed");
        }
    }
}

struct MarkerUniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    light_color: [f32; 3],
}

impl Uniforms for MarkerUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        f("u_model", UniformValue::Mat4(self.model));
        f("u_view", UniformValue::Mat4(self.view));
        f("u_projection", UniformValue::Mat4(self.projection));
        f("u_light_color", UniformValue::Vec3(self.light_color));
    }
}
