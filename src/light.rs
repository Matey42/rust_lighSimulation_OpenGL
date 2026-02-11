use cgmath::{InnerSpace, Matrix4, Point3, Vector3, Vector4};

// ───────────────────────── Light Types ─────────────────────────

/// The three kinds of lights supported by the shader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightKind {
    Point = 0,
    Spot = 1,
    Directional = 2,
}

/// A light source in *world* space.
///
/// The renderer will transform `position` and `direction` into view space
/// before uploading uniforms.
#[derive(Clone, Debug)]
pub struct Light {
    pub kind: LightKind,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,

    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],

    pub constant_att: f32,
    pub linear_att: f32,
    pub quadratic_att: f32,

    /// Cosine of the inner cone angle (spot only).
    pub cutoff: f32,
    /// Cosine of the outer cone angle (spot only).
    pub outer_cutoff: f32,
}

impl Light {
    // ─── Constructors ───

    pub fn point(position: Point3<f32>, diffuse: [f32; 3]) -> Self {
        Self {
            kind: LightKind::Point,
            position,
            direction: Vector3::new(0.0, -1.0, 0.0),
            ambient: [0.1, 0.1, 0.1],
            diffuse,
            specular: [1.0, 1.0, 1.0],
            constant_att: 1.0,
            linear_att: 0.09,
            quadratic_att: 0.032,
            cutoff: 0.0,
            outer_cutoff: 0.0,
        }
    }

    pub fn spot(
        position: Point3<f32>,
        direction: Vector3<f32>,
        diffuse: [f32; 3],
        inner_deg: f32,
        outer_deg: f32,
    ) -> Self {
        Self {
            kind: LightKind::Spot,
            position,
            direction: direction.normalize(),
            ambient: [0.05, 0.05, 0.05],
            diffuse,
            specular: [1.0, 1.0, 1.0],
            constant_att: 1.0,
            linear_att: 0.09,
            quadratic_att: 0.032,
            cutoff: inner_deg.to_radians().cos(),
            outer_cutoff: outer_deg.to_radians().cos(),
        }
    }

    pub fn directional(direction: Vector3<f32>, diffuse: [f32; 3]) -> Self {
        Self {
            kind: LightKind::Directional,
            position: Point3::new(0.0, 0.0, 0.0),
            direction: direction.normalize(),
            ambient: [0.15, 0.15, 0.15],
            diffuse,
            specular: [0.5, 0.5, 0.5],
            constant_att: 1.0,
            linear_att: 0.0,
            quadratic_att: 0.0,
            cutoff: 0.0,
            outer_cutoff: 0.0,
        }
    }

    // ─── Transform into view space ───

    /// Returns `(position_view, direction_view)` for this light given a view matrix.
    pub fn to_view_space(&self, view: &Matrix4<f32>) -> ([f32; 3], [f32; 3]) {
        let pos4 = view * Vector4::new(self.position.x, self.position.y, self.position.z, 1.0);
        let dir4 = view * Vector4::new(self.direction.x, self.direction.y, self.direction.z, 0.0);
        (
            [pos4.x, pos4.y, pos4.z],
            [dir4.x, dir4.y, dir4.z],
        )
    }
}
