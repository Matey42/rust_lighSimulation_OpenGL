use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};

// ─────────────────────── Camera Trait ───────────────────────

/// Common interface for every camera variant.
pub trait Camera {
    /// Build the view matrix for the current frame.
    fn view_matrix(&self) -> Matrix4<f32>;
    /// Camera position in world space (needed for shader uniforms).
    #[allow(dead_code)]
    fn position(&self) -> Point3<f32>;
}

// ─────────────────────── Static Camera ──────────────────────

/// A camera with a fixed position and orientation.
pub struct StaticCamera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
}

impl StaticCamera {
    pub fn new(eye: Point3<f32>, target: Point3<f32>) -> Self {
        Self {
            eye,
            target,
            up: Vector3::unit_y(),
        }
    }
}

impl Camera for StaticCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
    fn position(&self) -> Point3<f32> {
        self.eye
    }
}

// ─────────────────── Tracking Camera ────────────────────────

/// Camera with a fixed position that always looks at a target that moves.
pub struct TrackingCamera {
    pub eye: Point3<f32>,
    pub up: Vector3<f32>,
    target: Point3<f32>,
}

impl TrackingCamera {
    pub fn new(eye: Point3<f32>) -> Self {
        Self {
            eye,
            up: Vector3::unit_y(),
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Call each frame with the new position of the tracked object.
    pub fn update_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }
}

impl Camera for TrackingCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
    fn position(&self) -> Point3<f32> {
        self.eye
    }
}

// ──────────────── Third-Person (TPP) Camera ─────────────────

/// Camera that follows behind and above the moving object.
pub struct TppCamera {
    /// Offset *behind* the object in the object's local frame.
    pub offset: Vector3<f32>,
    /// Height above the object.
    pub height: f32,
    /// Current computed eye position (world space).
    eye: Point3<f32>,
    /// Current look-at target (world space).
    target: Point3<f32>,
}

impl TppCamera {
    pub fn new(offset: Vector3<f32>, height: f32) -> Self {
        Self {
            offset,
            height,
            eye: Point3::new(0.0, 5.0, -10.0),
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }

    /// Update every frame with the object's world position and its yaw rotation.
    pub fn update(&mut self, obj_pos: Point3<f32>, obj_yaw: f32) {
        // Compute a direction vector from the object's yaw
        let forward = Vector3::new(obj_yaw.sin(), 0.0, obj_yaw.cos());
        let right = forward.cross(Vector3::unit_y());

        let behind = -forward * self.offset.z + right * self.offset.x;
        self.eye = obj_pos + behind + Vector3::new(0.0, self.height + self.offset.y, 0.0);
        self.target = obj_pos + Vector3::new(0.0, 1.0, 0.0); // look slightly above object
    }
}

impl Camera for TppCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, Vector3::unit_y())
    }
    fn position(&self) -> Point3<f32> {
        self.eye
    }
}

// ────────────── First-Person (FPP) Camera ───────────────────

/// Camera sitting at the object's position looking forward.
pub struct FppCamera {
    /// Small offset above the object centre (eye height).
    pub eye_offset: Vector3<f32>,
    eye: Point3<f32>,
    target: Point3<f32>,
}

impl FppCamera {
    pub fn new(eye_offset: Vector3<f32>) -> Self {
        Self {
            eye_offset,
            eye: Point3::new(0.0, 1.5, 0.0),
            target: Point3::new(0.0, 1.5, 1.0),
        }
    }

    pub fn update(&mut self, obj_pos: Point3<f32>, obj_yaw: f32) {
        let forward = Vector3::new(obj_yaw.sin(), 0.0, obj_yaw.cos());
        let right = Vector3::new(obj_yaw.cos(), 0.0, -obj_yaw.sin());
        let local_offset = right * self.eye_offset.x
            + Vector3::new(0.0, self.eye_offset.y, 0.0)
            + forward * self.eye_offset.z;
        self.eye = obj_pos + local_offset;
        self.target = self.eye + forward * 10.0;
    }
}

impl Camera for FppCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, Vector3::unit_y())
    }
    fn position(&self) -> Point3<f32> {
        self.eye
    }
}

// ─────────────── Free Camera (bonus) ────────────────────────

/// A free-look camera controlled with keyboard + mouse.
pub struct FreeCamera {
    pub eye: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl FreeCamera {
    pub fn new(eye: Point3<f32>) -> Self {
        Self {
            eye,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            speed: 8.0,
            sensitivity: 0.002,
        }
    }

    pub fn front(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    /// Move with WASD.
    pub fn process_keyboard(&mut self, forward: f32, right: f32, up: f32, dt: f32) {
        let front = self.front();
        let world_up = Vector3::unit_y();
        let right_v = front.cross(world_up).normalize();

        self.eye += front * forward * self.speed * dt;
        self.eye += right_v * right * self.speed * dt;
        self.eye += world_up * up * self.speed * dt;
    }

    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * self.sensitivity;
        self.pitch -= dy * self.sensitivity;
        self.pitch = self.pitch.clamp(
            -Rad(89.0_f32.to_radians()).0,
            Rad(89.0_f32.to_radians()).0,
        );
    }
}

impl Camera for FreeCamera {
    fn view_matrix(&self) -> Matrix4<f32> {
        let target = self.eye + self.front();
        Matrix4::look_at_rh(self.eye, target, Vector3::unit_y())
    }
    fn position(&self) -> Point3<f32> {
        self.eye
    }
}
