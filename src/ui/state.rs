use crate::render::grid::GridConfig;

/// State that the options panel can read and modify.
pub struct UiState {
    // Camera
    pub active_camera: usize,

    // Shading
    pub use_phong: bool,

    // Fog
    pub fog_enabled: bool,
    pub fog_density: f32,

    // Day / Night
    pub day_factor: f32,
    pub day_night_speed: f32,
    pub day_night_auto: bool,

    // Spotlight
    pub spotlight_yaw_offset: f32,
    pub spotlight_pitch_offset: f32,

    // Light controls
    pub light_point_enabled: bool,
    pub light_point_intensity: f32,
    pub light_spot_enabled: bool,
    pub light_spot_intensity: f32,
    pub light_sun_enabled: bool,
    pub light_sun_intensity: f32,
    pub light_headlights_enabled: bool,
    pub light_headlights_intensity: f32,

    // Grid
    pub grid_visible: bool,
    pub grid_size: f32,
    pub grid_sub_size: f32,
    pub grid_fade_radius: f32,
    pub grid_line_width: f32,

    // Models
    pub show_car: bool,
    pub car_loaded: bool,

    // Normal maps
    pub show_normal_maps: bool,
    pub normal_maps_loaded: bool,

    // Manual driving of the moving object
    pub manual_drive: bool,
    pub drive_speed: f32,
    pub drive_turn_speed: f32,

    // Free camera
    pub free_cam_speed: f32,
    pub free_cam_sensitivity: f32,

    // Continuous key state for free camera (WASD + QE)
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_q: bool,
    pub key_e: bool,
    pub key_arrow_left: bool,
    pub key_arrow_right: bool,
    pub key_arrow_up: bool,
    pub key_arrow_down: bool,
    /// Whether the right mouse button is held (enables free-cam look)
    pub mouse_look: bool,

    // UI visibility
    pub show_panel: bool,

    // Performance
    pub fps: f32,
    pub frame_time_ms: f32,
}

impl Default for UiState {
    fn default() -> Self {
        let gc = GridConfig::default();
        Self {
            active_camera: 0,
            use_phong: true,
            fog_enabled: true,
            fog_density: 0.02,
            day_factor: 0.8,
            day_night_speed: 0.15,
            day_night_auto: true,
            spotlight_yaw_offset: 0.0,
            spotlight_pitch_offset: 0.0,
            light_point_enabled: true,
            light_point_intensity: 1.0,
            light_spot_enabled: true,
            light_spot_intensity: 1.0,
            light_sun_enabled: true,
            light_sun_intensity: 1.0,
            light_headlights_enabled: true,
            light_headlights_intensity: 1.0,
            grid_visible: true,
            grid_size: gc.grid_size,
            grid_sub_size: gc.sub_grid_size,
            grid_fade_radius: gc.fade_radius,
            grid_line_width: gc.line_width,
            show_car: false,
            car_loaded: false,
            show_normal_maps: false,
            normal_maps_loaded: false,
            manual_drive: false,
            drive_speed: 5.0,
            drive_turn_speed: 2.0,
            free_cam_speed: 8.0,
            free_cam_sensitivity: 0.3,
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_q: false,
            key_e: false,
            key_arrow_left: false,
            key_arrow_right: false,
            key_arrow_up: false,
            key_arrow_down: false,
            mouse_look: false,
            show_panel: true,
            fps: 0.0,
            frame_time_ms: 0.0,
        }
    }
}
