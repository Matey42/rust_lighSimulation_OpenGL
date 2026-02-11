use egui::{self, Align2, Color32, RichText, Vec2};

use crate::grid::GridConfig;

// ─────────────────── UI State ────────────────────────────

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

    // Grid
    pub grid_visible: bool,
    pub grid_size: f32,
    pub grid_sub_size: f32,
    pub grid_fade_radius: f32,
    pub grid_line_width: f32,

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
            grid_visible: true,
            grid_size: gc.grid_size,
            grid_sub_size: gc.sub_grid_size,
            grid_fade_radius: gc.fade_radius,
            grid_line_width: gc.line_width,
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

// ─────────────────── Drawing ─────────────────────────────

/// Build the options panel each frame.
pub fn draw_ui(ctx: &egui::Context, state: &mut UiState) {
    // Keybind hint at top-left when panel is hidden
    if !state.show_panel {
        egui::Area::new(egui::Id::new("hint_area"))
            .anchor(Align2::LEFT_TOP, Vec2::new(8.0, 8.0))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Press [Tab] to open options panel")
                        .color(Color32::from_white_alpha(160))
                        .size(13.0),
                );
            });
        return;
    }

    egui::SidePanel::left("options_panel")
        .default_width(260.0)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                // ── Header ──
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("⚙  Options").strong().size(18.0));
                });
                ui.separator();

                // ── Performance ──
                ui.label(
                    RichText::new(format!(
                        "FPS: {:.0}  ({:.1} ms)",
                        state.fps, state.frame_time_ms
                    ))
                    .color(Color32::from_rgb(160, 220, 160))
                    .size(13.0),
                );
                ui.separator();

                // ── Camera ──
                ui.collapsing(RichText::new("📷  Camera").size(15.0), |ui| {
                    ui.radio_value(&mut state.active_camera, 0, "Static");
                    ui.radio_value(&mut state.active_camera, 1, "Tracking");
                    ui.radio_value(&mut state.active_camera, 2, "Third-Person (TPP)");
                    ui.radio_value(&mut state.active_camera, 3, "First-Person (FPP)");
                    ui.radio_value(&mut state.active_camera, 4, "Free (WASD)");
                    if state.active_camera == 4 {
                        ui.separator();
                        ui.label(
                            RichText::new("Hold RMB to look around")
                                .italics()
                                .color(Color32::from_white_alpha(140))
                                .size(12.0),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.free_cam_speed, 1.0..=30.0)
                                .text("Speed")
                                .fixed_decimals(1),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.free_cam_sensitivity, 0.05..=1.0)
                                .text("Sensitivity")
                                .fixed_decimals(2),
                        );
                    }
                });

                // ── Shading ──
                ui.collapsing(RichText::new("🎨  Shading").size(15.0), |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.use_phong, true, "Phong");
                        ui.selectable_value(&mut state.use_phong, false, "Gouraud");
                    });
                });

                // ── Fog ──
                ui.collapsing(RichText::new("🌫  Fog").size(15.0), |ui| {
                    ui.checkbox(&mut state.fog_enabled, "Enable fog");
                    ui.add_enabled(
                        state.fog_enabled,
                        egui::Slider::new(&mut state.fog_density, 0.0..=0.2)
                            .text("Density")
                            .fixed_decimals(3),
                    );
                });

                // ── Animation ──
                ui.collapsing(RichText::new("▶  Animation").size(15.0), |ui| {
                    ui.checkbox(&mut state.manual_drive, "Manual drive (steer object)");
                    if state.manual_drive {
                        ui.label(
                            RichText::new("Auto-animation paused while driving")
                                .color(Color32::from_rgb(220, 180, 100))
                                .size(12.0),
                        );
                        ui.label(
                            RichText::new("W/S = forward/back, A/D = turn")
                                .italics()
                                .color(Color32::from_white_alpha(140))
                                .size(12.0),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.drive_speed, 1.0..=20.0)
                                .text("Drive speed")
                                .fixed_decimals(1),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.drive_turn_speed, 0.5..=5.0)
                                .text("Turn speed")
                                .fixed_decimals(1),
                        );
                    }
                });

                // ── Day / Night ──
                ui.collapsing(RichText::new("🌗  Day / Night").size(15.0), |ui| {
                    ui.checkbox(&mut state.day_night_auto, "Auto cycle");
                    if state.day_night_auto {
                        ui.add(
                            egui::Slider::new(&mut state.day_night_speed, 0.01..=1.0)
                                .text("Speed")
                                .logarithmic(true)
                                .fixed_decimals(2),
                        );
                    }
                    ui.add(
                        egui::Slider::new(&mut state.day_factor, 0.0..=1.0)
                            .text("Day factor")
                            .fixed_decimals(2),
                    );
                });

                // ── Spotlight ──
                ui.collapsing(RichText::new("🔦  Spotlight Aim").size(15.0), |ui| {
                    ui.add(
                        egui::Slider::new(&mut state.spotlight_yaw_offset, -1.0..=1.0)
                            .text("Yaw")
                            .fixed_decimals(2),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.spotlight_pitch_offset, -1.0..=1.0)
                            .text("Pitch")
                            .fixed_decimals(2),
                    );
                    if ui.button("Reset").clicked() {
                        state.spotlight_yaw_offset = 0.0;
                        state.spotlight_pitch_offset = 0.0;
                    }
                });

                // ── Grid ──
                ui.collapsing(RichText::new("▦  Grid").size(15.0), |ui| {
                    ui.checkbox(&mut state.grid_visible, "Show grid");
                    ui.add_enabled(
                        state.grid_visible,
                        egui::Slider::new(&mut state.grid_size, 0.25..=5.0)
                            .text("Cell size")
                            .fixed_decimals(2),
                    );
                    ui.add_enabled(
                        state.grid_visible,
                        egui::Slider::new(&mut state.grid_sub_size, 0.05..=2.0)
                            .text("Sub-grid size")
                            .fixed_decimals(2),
                    );
                    ui.add_enabled(
                        state.grid_visible,
                        egui::Slider::new(&mut state.grid_fade_radius, 10.0..=200.0)
                            .text("Fade radius")
                            .fixed_decimals(0),
                    );
                    ui.add_enabled(
                        state.grid_visible,
                        egui::Slider::new(&mut state.grid_line_width, 0.005..=0.1)
                            .text("Line width")
                            .fixed_decimals(3),
                    );
                });

                ui.separator();

                // ── Keyboard shortcuts ──
                ui.collapsing(RichText::new("⌨  Shortcuts").size(15.0), |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let shortcuts = [
                        ("Tab", "Toggle this panel"),
                        ("Space", "Toggle animation / manual"),
                        ("1–5", "Switch camera"),
                        ("WASD", "Move / Drive"),
                        ("Q / E", "Down / Up (Free cam)"),
                        ("RMB", "Look around (Free cam)"),
                        ("P", "Toggle shading"),
                        ("F", "Toggle fog"),
                        ("+/−", "Fog density"),
                        ("N", "Auto day/night"),
                        ("O / L", "Day ↑ / Night ↓"),
                        ("Arrows", "Spotlight aim"),
                        ("Esc", "Quit"),
                    ];
                    for (key, desc) in shortcuts {
                        ui.horizontal(|ui| {
                            ui.monospace(format!("{:>8}", key));
                            ui.label(desc);
                        });
                    }
                });

                ui.separator();
                ui.vertical_centered(|ui| {
                    if ui.small_button("Hide panel [Tab]").clicked() {
                        state.show_panel = false;
                    }
                });
            });
        });
}
