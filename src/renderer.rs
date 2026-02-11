use glium::uniforms::{AsUniformValue, UniformValue, Uniforms};
use glium::{DrawParameters, Surface};
use cgmath::Matrix4;

use crate::light::Light;
use crate::model::Mesh;
use crate::scene::SceneObject;
use crate::types::{mat3_to_array, mat4_to_array, Material};

/// Maximum number of lights supported by the shaders.
const MAX_LIGHTS: usize = 8;

// ──────────── Dynamic Uniforms Container ────────────────

/// A container that stores uniform values by name and implements `Uniforms`.
struct DynamicUniforms {
    values: Vec<(String, UniformValueOwned)>,
}

/// Owned version of `UniformValue` so we can store them.
#[allow(dead_code)]
enum UniformValueOwned {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vec3([f32; 3]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
}

impl DynamicUniforms {
    fn new() -> Self {
        Self { values: Vec::new() }
    }
    fn add_float(&mut self, name: impl Into<String>, v: f32) {
        self.values.push((name.into(), UniformValueOwned::Float(v)));
    }
    fn add_int(&mut self, name: impl Into<String>, v: i32) {
        self.values.push((name.into(), UniformValueOwned::Int(v)));
    }
    fn add_bool(&mut self, name: impl Into<String>, v: bool) {
        self.values.push((name.into(), UniformValueOwned::Bool(v)));
    }
    fn add_vec3(&mut self, name: impl Into<String>, v: [f32; 3]) {
        self.values.push((name.into(), UniformValueOwned::Vec3(v)));
    }
    fn add_mat3(&mut self, name: impl Into<String>, v: [[f32; 3]; 3]) {
        self.values.push((name.into(), UniformValueOwned::Mat3(v)));
    }
    fn add_mat4(&mut self, name: impl Into<String>, v: [[f32; 4]; 4]) {
        self.values.push((name.into(), UniformValueOwned::Mat4(v)));
    }
}

impl Uniforms for DynamicUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        for (name, val) in &self.values {
            let uv = match val {
                UniformValueOwned::Float(v) => UniformValue::Float(*v),
                UniformValueOwned::Int(v) => UniformValue::SignedInt(*v),
                UniformValueOwned::Bool(v) => UniformValue::Bool(*v),
                UniformValueOwned::Vec3(v) => UniformValue::Vec3(*v),
                UniformValueOwned::Mat3(v) => UniformValue::Mat3(*v),
                UniformValueOwned::Mat4(v) => UniformValue::Mat4(*v),
            };
            f(name, uv);
        }
    }
}

// ──────────── Renderer ──────────────────────────────────

/// All rendering state needed per frame.
pub struct Renderer {
    pub phong_program: glium::Program,
    pub gouraud_program: glium::Program,
    pub use_phong: bool,
    /// 1×1 white texture used when no diffuse/normal map is bound.
    pub dummy_texture: glium::texture::Texture2d,
}

impl Renderer {
    pub fn new(display: &glium::Display<glium::glutin::surface::WindowSurface>) -> Self {
        let phong_vert = include_str!("../assets/shaders/phong.vert");
        let phong_frag = include_str!("../assets/shaders/phong.frag");
        let gouraud_vert = include_str!("../assets/shaders/gouraud.vert");
        let gouraud_frag = include_str!("../assets/shaders/gouraud.frag");

        let phong_program = glium::Program::from_source(display, phong_vert, phong_frag, None)
            .expect("Failed to compile Phong shaders");
        let gouraud_program =
            glium::Program::from_source(display, gouraud_vert, gouraud_frag, None)
                .expect("Failed to compile Gouraud shaders");

        let dummy_texture = {
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                &[255u8, 255, 255, 255],
                (1, 1),
            );
            glium::texture::Texture2d::new(display, image).unwrap()
        };

        Self {
            phong_program,
            gouraud_program,
            use_phong: true,
            dummy_texture,
        }
    }

    pub fn active_program(&self) -> &glium::Program {
        if self.use_phong {
            &self.phong_program
        } else {
            &self.gouraud_program
        }
    }

    /// Draw a single scene object.
    pub fn draw_object(
        &self,
        target: &mut glium::Frame,
        obj: &SceneObject,
        view: &Matrix4<f32>,
        projection: &Matrix4<f32>,
        lights: &[Light],
        fog_enabled: bool,
        fog_color: [f32; 3],
        fog_density: f32,
        ambient_strength: f32,
        params: &DrawParameters<'_>,
    ) {
        let model = obj.transform.model_matrix();
        let normal_mat = obj.transform.normal_matrix(view);

        let model_arr = mat4_to_array(&model);
        let view_arr = mat4_to_array(view);
        let proj_arr = mat4_to_array(projection);
        let normal_arr = mat3_to_array(&normal_mat);

        let program = self.active_program();

        for mesh in &obj.meshes {
            self.draw_mesh(
                target,
                mesh,
                program,
                &model_arr,
                &view_arr,
                &proj_arr,
                &normal_arr,
                &obj.material,
                lights,
                view,
                fog_enabled,
                fog_color,
                fog_density,
                ambient_strength,
                params,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_mesh(
        &self,
        target: &mut glium::Frame,
        mesh: &Mesh,
        program: &glium::Program,
        model: &[[f32; 4]; 4],
        view_arr: &[[f32; 4]; 4],
        projection: &[[f32; 4]; 4],
        normal_matrix: &[[f32; 3]; 3],
        material: &Material,
        lights: &[Light],
        view_mat: &Matrix4<f32>,
        fog_enabled: bool,
        fog_color: [f32; 3],
        fog_density: f32,
        ambient_strength: f32,
        params: &DrawParameters<'_>,
    ) {
        let num_lights = lights.len().min(MAX_LIGHTS) as i32;

        // Build dynamic uniforms
        let mut u = DynamicUniforms::new();

        u.add_mat4("model", *model);
        u.add_mat4("view", *view_arr);
        u.add_mat4("projection", *projection);
        u.add_mat3("normal_matrix", *normal_matrix);

        u.add_vec3("u_material_ambient", material.ambient);
        u.add_vec3("u_material_diffuse", material.diffuse);
        u.add_vec3("u_material_specular", material.specular);
        u.add_float("u_material_shininess", material.shininess);

        u.add_bool("u_has_diffuse_tex", false);
        u.add_bool("u_has_normal_map", false);

        u.add_int("u_num_lights", num_lights);

        // Add per-light uniforms with indexed struct names
        for i in 0..MAX_LIGHTS {
            let prefix = format!("u_lights[{}]", i);

            if i < lights.len() {
                let light = &lights[i];
                let (pos_v, dir_v) = light.to_view_space(view_mat);

                u.add_int(format!("{}.type", prefix), light.kind as i32);
                u.add_vec3(format!("{}.position_view", prefix), pos_v);
                u.add_vec3(format!("{}.direction_view", prefix), dir_v);
                u.add_vec3(format!("{}.ambient", prefix), light.ambient);
                u.add_vec3(format!("{}.diffuse", prefix), light.diffuse);
                u.add_vec3(format!("{}.specular", prefix), light.specular);
                u.add_float(format!("{}.constant_att", prefix), light.constant_att);
                u.add_float(format!("{}.linear_att", prefix), light.linear_att);
                u.add_float(format!("{}.quadratic_att", prefix), light.quadratic_att);
                u.add_float(format!("{}.cutoff", prefix), light.cutoff);
                u.add_float(format!("{}.outer_cutoff", prefix), light.outer_cutoff);
            } else {
                // Fill unused slots with defaults
                u.add_int(format!("{}.type", prefix), 0);
                u.add_vec3(format!("{}.position_view", prefix), [0.0; 3]);
                u.add_vec3(format!("{}.direction_view", prefix), [0.0; 3]);
                u.add_vec3(format!("{}.ambient", prefix), [0.0; 3]);
                u.add_vec3(format!("{}.diffuse", prefix), [0.0; 3]);
                u.add_vec3(format!("{}.specular", prefix), [0.0; 3]);
                u.add_float(format!("{}.constant_att", prefix), 1.0);
                u.add_float(format!("{}.linear_att", prefix), 0.0);
                u.add_float(format!("{}.quadratic_att", prefix), 0.0);
                u.add_float(format!("{}.cutoff", prefix), 0.0);
                u.add_float(format!("{}.outer_cutoff", prefix), 0.0);
            }
        }

        u.add_bool("u_fog_enabled", fog_enabled);
        u.add_vec3("u_fog_color", fog_color);
        u.add_float("u_fog_density", fog_density);
        u.add_float("u_ambient_strength", ambient_strength);

        // We also need the sampler uniforms. We'll use a combined approach:
        // wrap DynamicUniforms + texture samplers together.
        let combined = CombinedUniforms {
            dynamic: u,
            diffuse_tex: &self.dummy_texture,
            normal_map: &self.dummy_texture,
        };

        target
            .draw(
                &mesh.vertex_buffer,
                &mesh.index_buffer,
                program,
                &combined,
                params,
            )
            .expect("Draw call failed");
    }
}

/// Combines dynamic scalar/vector uniforms with texture samplers.
struct CombinedUniforms<'a> {
    dynamic: DynamicUniforms,
    diffuse_tex: &'a glium::texture::Texture2d,
    normal_map: &'a glium::texture::Texture2d,
}

impl<'a> Uniforms for CombinedUniforms<'a> {
    fn visit_values<'b, F: FnMut(&str, UniformValue<'b>)>(&'b self, mut f: F) {
        // Forward all dynamic uniforms
        self.dynamic.visit_values(&mut f);
        // Add texture samplers
        f("u_diffuse_tex", self.diffuse_tex.as_uniform_value());
        f("u_normal_map", self.normal_map.as_uniform_value());
    }
}
