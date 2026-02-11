use cgmath::{Point3, Vector3};
use glium::Display;

use crate::light::Light;
use crate::model::{mesh_from_data, Mesh};
use crate::primitives;
use crate::types::{Material, Transform};

// ────────────────────── Scene Object ──────────────────────

/// A single renderable entity in the scene.
pub struct SceneObject {
    #[allow(dead_code)]
    pub name: String,
    pub meshes: Vec<Mesh>,
    pub transform: Transform,
    pub material: Material,
}

// ────────────────────── Moving Object ─────────────────────

/// The object that moves around the scene. Carries its own spotlights.
pub struct MovingObject {
    pub obj: SceneObject,
    /// Spotlights attached to this object (e.g., headlights).
    pub spotlights: Vec<Light>,
    /// The *relative* direction offset of each spotlight (can be adjusted at runtime).
    pub spotlight_offsets: Vec<Vector3<f32>>,
    /// Current orbit angle (radians).
    pub orbit_angle: f32,
    /// Orbit radius.
    pub orbit_radius: f32,
    /// Self-rotation speed (rad/s).
    pub rotation_speed: f32,
    /// Orbit speed (rad/s).
    pub orbit_speed: f32,
}

impl MovingObject {
    pub fn new(obj: SceneObject, orbit_radius: f32) -> Self {
        // Two headlights: slightly left and right, pointing forward
        let left_offset = Vector3::new(-0.4, -0.1, 1.0);
        let right_offset = Vector3::new(0.4, -0.1, 1.0);

        let left_light = Light::spot(
            Point3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -0.3, 1.0),
            [1.0, 1.0, 0.9],
            15.0,
            25.0,
        );
        let right_light = Light::spot(
            Point3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -0.3, 1.0),
            [1.0, 1.0, 0.9],
            15.0,
            25.0,
        );

        Self {
            obj,
            spotlights: vec![left_light, right_light],
            spotlight_offsets: vec![left_offset, right_offset],
            orbit_angle: 0.0,
            orbit_radius,
            rotation_speed: 1.5,
            orbit_speed: 0.4,
        }
    }

    /// Advance the object's animation by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.orbit_angle += self.orbit_speed * dt;
        self.obj.transform.rotation.y += self.rotation_speed * dt;

        // Orbit in the XZ plane
        let x = self.orbit_radius * self.orbit_angle.cos();
        let z = self.orbit_radius * self.orbit_angle.sin();
        self.obj.transform.position = Vector3::new(x, self.obj.transform.position.y, z);

        // Update spotlight world positions & directions
        // The +Z face direction after Y-rotation by orbit_angle
        let yaw = self.orbit_angle;
        let forward = Vector3::new(yaw.sin(), 0.0, yaw.cos());
        let right_dir = Vector3::new(yaw.cos(), 0.0, -yaw.sin());

        for (i, light) in self.spotlights.iter_mut().enumerate() {
            let offset = &self.spotlight_offsets[i];
            let world_offset = right_dir * offset.x
                + Vector3::new(0.0, offset.y, 0.0)
                + forward * offset.z;
            light.position = Point3::new(
                self.obj.transform.position.x + world_offset.x,
                self.obj.transform.position.y + 0.8 + world_offset.y,
                self.obj.transform.position.z + world_offset.z,
            );
            light.direction = forward + Vector3::new(0.0, -0.3, 0.0);
        }
    }

    /// Advance the object under manual control.
    /// `forward` = +1 (W) / −1 (S), `turn` = +1 (D=right) / −1 (A=left).
    pub fn manual_drive_update(
        &mut self,
        forward: f32,
        turn: f32,
        speed: f32,
        turn_speed: f32,
        dt: f32,
    ) {
        // Turn: positive turn = clockwise from above (right), negative = left
        self.orbit_angle += turn * turn_speed * dt;
        // The visual rotation of the cube: from_angle_y(Rad(rotation.y))
        // rotates the +Z face of the cube. We want +Z face = front.
        self.obj.transform.rotation.y = self.orbit_angle;

        // The +Z face after Y-rotation points in direction (sin(angle), 0, cos(angle))
        let heading = self.orbit_angle;
        let dir = Vector3::new(heading.sin(), 0.0, heading.cos());
        self.obj.transform.position += dir * forward * speed * dt;

        // Update spotlights to follow the new position/heading
        let fwd = dir;
        let right_dir = Vector3::new(heading.cos(), 0.0, -heading.sin());

        for (i, light) in self.spotlights.iter_mut().enumerate() {
            let offset = &self.spotlight_offsets[i];
            let world_offset = right_dir * offset.x
                + Vector3::new(0.0, offset.y, 0.0)
                + fwd * offset.z;
            light.position = Point3::new(
                self.obj.transform.position.x + world_offset.x,
                self.obj.transform.position.y + 0.8 + world_offset.y,
                self.obj.transform.position.z + world_offset.z,
            );
            light.direction = fwd + Vector3::new(0.0, -0.3, 0.0);
        }
    }

    pub fn world_position(&self) -> Point3<f32> {
        let p = self.obj.transform.position;
        Point3::new(p.x, p.y, p.z)
    }

    pub fn yaw(&self) -> f32 {
        self.orbit_angle
    }
}

// ────────────────────── Scene ─────────────────────────────

/// Holds every object and light in the world.
pub struct Scene {
    pub static_objects: Vec<SceneObject>,
    pub moving_object: MovingObject,
    pub lights: Vec<Light>,
}

impl Scene {
    /// Build the default demonstration scene.
    pub fn build_default(display: &Display<glium::glutin::surface::WindowSurface>) -> Self {
        // ── Sphere (smooth surface – required) ──
        let (sphere_v, sphere_i) = primitives::generate_sphere(1.0, 40, 40);
        let sphere = SceneObject {
            name: "Sphere".into(),
            meshes: vec![mesh_from_data(display, &sphere_v, &sphere_i)],
            transform: Transform {
                position: Vector3::new(4.0, 1.0, 3.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.1, 0.05, 0.05],
                diffuse: [0.8, 0.2, 0.2],
                specular: [1.0, 1.0, 1.0],
                shininess: 64.0,
            },
        };

        // ── Torus (another smooth surface) ──
        let (torus_v, torus_i) = primitives::generate_torus(1.5, 0.5, 48, 24);
        let torus = SceneObject {
            name: "Torus".into(),
            meshes: vec![mesh_from_data(display, &torus_v, &torus_i)],
            transform: Transform {
                position: Vector3::new(-4.0, 1.5, 2.0),
                rotation: Vector3::new(0.3, 0.0, 0.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.05, 0.05, 0.1],
                diffuse: [0.3, 0.3, 0.9],
                specular: [1.0, 1.0, 1.0],
                shininess: 48.0,
            },
        };

        // ── Static cubes ──
        let (cube_v, cube_i) = primitives::generate_cube(0.8);
        let cube1 = SceneObject {
            name: "Cube1".into(),
            meshes: vec![mesh_from_data(display, &cube_v, &cube_i)],
            transform: Transform {
                position: Vector3::new(-2.0, 0.3, -3.0),
                rotation: Vector3::new(0.0, 0.8, 0.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.05, 0.1, 0.05],
                diffuse: [0.2, 0.7, 0.3],
                specular: [0.5, 0.5, 0.5],
                shininess: 16.0,
            },
        };
        let cube2 = SceneObject {
            name: "Cube2".into(),
            meshes: vec![mesh_from_data(display, &cube_v, &cube_i)],
            transform: Transform {
                position: Vector3::new(3.0, 0.3, -4.0),
                rotation: Vector3::new(0.0, -0.5, 0.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.1, 0.1, 0.05],
                diffuse: [0.8, 0.7, 0.2],
                specular: [0.6, 0.6, 0.6],
                shininess: 24.0,
            },
        };

        // ── Large sphere (pedestal) ──
        let (big_sphere_v, big_sphere_i) = primitives::generate_sphere(2.0, 48, 48);
        let big_sphere = SceneObject {
            name: "BigSphere".into(),
            meshes: vec![mesh_from_data(display, &big_sphere_v, &big_sphere_i)],
            transform: Transform {
                position: Vector3::new(0.0, 1.5, -7.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.1, 0.1, 0.1],
                diffuse: [0.6, 0.6, 0.65],
                specular: [0.9, 0.9, 0.9],
                shininess: 96.0,
            },
        };

        // ── Moving object (a cube that orbits) ──
        let (mv_v, mv_i) = primitives::generate_cube(0.5);
        let moving_scene_obj = SceneObject {
            name: "Mover".into(),
            meshes: vec![mesh_from_data(display, &mv_v, &mv_i)],
            transform: Transform {
                position: Vector3::new(5.0, 0.5, 0.0),
                ..Default::default()
            },
            material: Material {
                ambient: [0.1, 0.1, 0.1],
                diffuse: [0.9, 0.6, 0.1],
                specular: [1.0, 1.0, 1.0],
                shininess: 32.0,
            },
        };
        let moving_object = MovingObject::new(moving_scene_obj, 6.0);

        // ── Lights ──

        // 1. Fixed point light (warm, above scene)
        let point_light = Light::point(
            Point3::new(0.0, 8.0, 0.0),
            [0.9, 0.85, 0.7],
        );

        // 2. Fixed spotlight (cool, illuminating the torus area)
        let fixed_spot = Light::spot(
            Point3::new(-4.0, 6.0, 2.0),
            Vector3::new(0.0, -1.0, 0.0),
            [0.4, 0.4, 0.8],
            20.0,
            35.0,
        );

        // 3. Directional "sun" – will be modulated by day/night cycle
        let sun = Light::directional(
            Vector3::new(-0.3, -1.0, -0.5),
            [1.0, 0.95, 0.85],
        );

        Scene {
            static_objects: vec![sphere, torus, cube1, cube2, big_sphere],
            moving_object,
            lights: vec![point_light, fixed_spot, sun],
        }
    }
}
