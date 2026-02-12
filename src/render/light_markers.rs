use cgmath::{InnerSpace, Matrix4, Vector3};
use glium::uniforms::{UniformValue, Uniforms};
use glium::{DrawParameters, Surface};

use crate::core::light::{Light, LightKind};
use crate::core::types::mat4_to_array;
use crate::core::vertex::Vertex;
use crate::scene::primitives;

/// Renders small glowing spheres at each light source position
/// and direction cones for spot lights.
pub struct LightMarkers {
    program: glium::Program,
    sphere_vb: glium::VertexBuffer<Vertex>,
    sphere_ib: glium::IndexBuffer<u32>,
    cone_vb: glium::VertexBuffer<Vertex>,
    cone_ib: glium::IndexBuffer<u32>,
}

impl LightMarkers {
    pub fn new(display: &glium::Display<glium::glutin::surface::WindowSurface>) -> Self {
        let vert_src = include_str!("../../assets/shaders/light_marker.vert");
        let frag_src = include_str!("../../assets/shaders/light_marker.frag");

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

        // Unit cone mesh (radius=1, length=1) — scaled per-light in draw()
        let (cone_v, cone_i) = primitives::generate_cone(1.0, 1.0, 32);
        let cone_vb = glium::VertexBuffer::new(display, &cone_v).unwrap();
        let cone_ib = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &cone_i,
        )
        .unwrap();

        Self {
            program,
            sphere_vb,
            sphere_ib,
            cone_vb,
            cone_ib,
        }
    }

    /// Draw a marker for each non-directional light.
    /// Spotlights additionally get a direction cone.
    pub fn draw(
        &self,
        target: &mut glium::Frame,
        lights: &[Light],
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
        fog_enabled: bool,
        fog_color: [f32; 3],
        fog_density: f32,
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

            // Draw sphere marker
            let uniforms = MarkerUniforms {
                model: model_arr,
                view: view_arr,
                projection: proj_arr,
                light_color: light.diffuse,
                alpha: 1.0,
                fog_enabled,
                fog_color,
                fog_density,
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

            // Draw direction cone for spot lights
            if light.kind == LightKind::Spot {
                let dir = light.direction.normalize();

                // Compute realistic cone dimensions from light properties
                let outer_angle = light.outer_cutoff.acos(); // outer_cutoff = cos(angle)
                let effective_range = compute_light_range(
                    light.constant_att,
                    light.linear_att,
                    light.quadratic_att,
                );
                let base_radius = effective_range * outer_angle.tan() * 0.5;

                // Build transform: translate to light pos, rotate to aim along dir,
                // then scale the unit cone to real light dimensions
                let cone_model = Matrix4::from_translation(Vector3::new(pos.x, pos.y, pos.z))
                    * look_rotation(dir)
                    * Matrix4::from_nonuniform_scale(base_radius, base_radius, effective_range);

                let cone_model_arr = mat4_to_array(&cone_model);

                // Cone tinted with light color, kept fairly bright for visibility
                let cone_color = [
                    light.diffuse[0] * 0.5,
                    light.diffuse[1] * 0.5,
                    light.diffuse[2] * 0.5,
                ];

                let cone_uniforms = MarkerUniforms {
                    model: cone_model_arr,
                    view: view_arr,
                    projection: proj_arr,
                    light_color: cone_color,
                    alpha: 0.13,
                    fog_enabled,
                    fog_color,
                    fog_density,
                };

                // Transparent cone: depth test ON but depth write OFF
                // so objects behind are still visible
                let cone_params = DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                        write: false,
                        ..Default::default()
                    },
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                };

                target
                    .draw(
                        &self.cone_vb,
                        &self.cone_ib,
                        &self.program,
                        &cone_uniforms,
                        &cone_params,
                    )
                    .expect("Light cone draw failed");
            }
        }
    }
}

/// Compute the effective range of a light source where intensity drops to ~5%.
/// Uses the attenuation formula: I = 1 / (c + l*d + q*d²).
fn compute_light_range(constant: f32, linear: f32, quadratic: f32) -> f32 {
    let threshold = 20.0; // 1 / 0.05
    let a = quadratic;
    let b = linear;
    let c = constant - threshold;
    if a <= 0.0 {
        // Linear-only falloff
        if b > 0.0 {
            return (-c / b).clamp(2.0, 15.0);
        }
        return 10.0;
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return 10.0;
    }
    let d = (-b + discriminant.sqrt()) / (2.0 * a);
    d.clamp(2.0, 15.0)
}

/// Build a rotation matrix that aligns the -Z axis with the given direction.
fn look_rotation(dir: Vector3<f32>) -> Matrix4<f32> {
    // We want -Z to become `dir`, so the "forward" for the matrix is `dir`
    let forward = dir.normalize();

    // Choose an up vector that isn't parallel to forward
    let world_up = if forward.y.abs() > 0.99 {
        Vector3::new(0.0, 0.0, 1.0)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };

    let right = forward.cross(world_up).normalize();
    let up = right.cross(forward).normalize();

    // Columns: right, up, -forward (because our cone points -Z)
    // We want -Z -> forward, so Z column = -forward
    #[rustfmt::skip]
    let m = Matrix4::new(
        right.x,    right.y,    right.z,    0.0,
        up.x,       up.y,       up.z,       0.0,
        -forward.x, -forward.y, -forward.z, 0.0,
        0.0,        0.0,        0.0,        1.0,
    );
    m
}

struct MarkerUniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    light_color: [f32; 3],
    alpha: f32,
    fog_enabled: bool,
    fog_color: [f32; 3],
    fog_density: f32,
}

impl Uniforms for MarkerUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        f("u_model", UniformValue::Mat4(self.model));
        f("u_view", UniformValue::Mat4(self.view));
        f("u_projection", UniformValue::Mat4(self.projection));
        f("u_light_color", UniformValue::Vec3(self.light_color));
        f("u_alpha", UniformValue::Float(self.alpha));
        f("u_fog_enabled", UniformValue::Bool(self.fog_enabled));
        f("u_fog_color", UniformValue::Vec3(self.fog_color));
        f("u_fog_density", UniformValue::Float(self.fog_density));
    }
}
