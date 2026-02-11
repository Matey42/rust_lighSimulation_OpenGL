use cgmath::{Matrix, Matrix3, Matrix4, SquareMatrix, Vector3};

/// Material properties for Phong / Gouraud shading.
#[derive(Clone, Debug)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: [0.2, 0.2, 0.2],
            diffuse: [0.8, 0.8, 0.8],
            specular: [1.0, 1.0, 1.0],
            shininess: 32.0,
        }
    }
}

// ───────────────────── Transform ─────────────────────

/// A 3-D transform: position + rotation (Euler angles in radians) + scale.
#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, // Euler angles (yaw, pitch, roll)
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    /// Compute the model matrix from position, rotation, and scale.
    pub fn model_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.position);
        let rot_x = Matrix4::from_angle_x(cgmath::Rad(self.rotation.x));
        let rot_y = Matrix4::from_angle_y(cgmath::Rad(self.rotation.y));
        let rot_z = Matrix4::from_angle_z(cgmath::Rad(self.rotation.z));
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        translation * rot_y * rot_x * rot_z * scale
    }

    /// Compute the normal matrix from model and view matrices.
    pub fn normal_matrix(&self, view: &Matrix4<f32>) -> Matrix3<f32> {
        let mv = view * self.model_matrix();
        // Take the 3×3 upper-left, invert, transpose.
        let m3: Matrix3<f32> = Matrix3::new(
            mv.x.x, mv.x.y, mv.x.z,
            mv.y.x, mv.y.y, mv.y.z,
            mv.z.x, mv.z.y, mv.z.z,
        );
        m3.invert().unwrap_or(Matrix3::identity()).transpose()
    }
}

// ───────────────── Helper conversions ────────────────

/// Convert a cgmath `Matrix4` to the `[[f32; 4]; 4]` glium expects.
pub fn mat4_to_array(m: &Matrix4<f32>) -> [[f32; 4]; 4] {
    [
        [m.x.x, m.x.y, m.x.z, m.x.w],
        [m.y.x, m.y.y, m.y.z, m.y.w],
        [m.z.x, m.z.y, m.z.z, m.z.w],
        [m.w.x, m.w.y, m.w.z, m.w.w],
    ]
}

/// Convert a cgmath `Matrix3` to `[[f32; 3]; 3]`.
pub fn mat3_to_array(m: &Matrix3<f32>) -> [[f32; 3]; 3] {
    [
        [m.x.x, m.x.y, m.x.z],
        [m.y.x, m.y.y, m.y.z],
        [m.z.x, m.z.y, m.z.z],
    ]
}
