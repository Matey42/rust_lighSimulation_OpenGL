use egui::{self, Align2, Color32, RichText, Vec2};

use super::UiState;

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

                // ── Fog ──
                ui.collapsing(RichText::new("🌫  Fog").size(15.0), |ui| {
                    ui.checkbox(&mut state.fog_enabled, "Enable fog");
                    ui.add_enabled(
                        state.fog_enabled,
                        egui::Slider::new(&mut state.fog_density, 0.0..=0.1)
                            .text("Density")
                            .fixed_decimals(3),
                    );
                });

                // ── Shading ──
                ui.collapsing(RichText::new("🎨  Shading").size(15.0), |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.use_phong, true, "Phong");
                        ui.selectable_value(&mut state.use_phong, false, "Gouraud");
                    });
                });

                // ── Spotlight Aim ──
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

                // ── Lights ──
                ui.collapsing(RichText::new("💡  Lights").size(15.0), |ui| {
                    // Point light
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.light_point_enabled, "");
                        ui.label("Point light (warm)");
                    });
                    ui.add_enabled(
                        state.light_point_enabled,
                        egui::Slider::new(&mut state.light_point_intensity, 0.0..=2.0)
                            .text("Intensity")
                            .fixed_decimals(2),
                    );
                    ui.separator();

                    // Spotlight (torus)
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.light_spot_enabled, "");
                        ui.label("Spotlight (blue)");
                    });
                    ui.add_enabled(
                        state.light_spot_enabled,
                        egui::Slider::new(&mut state.light_spot_intensity, 0.0..=2.0)
                            .text("Intensity")
                            .fixed_decimals(2),
                    );
                    ui.separator();

                    // Sun / directional
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.light_sun_enabled, "");
                        ui.label("Sun (directional)");
                    });
                    ui.add_enabled(
                        state.light_sun_enabled,
                        egui::Slider::new(&mut state.light_sun_intensity, 0.0..=2.0)
                            .text("Intensity")
                            .fixed_decimals(2),
                    );
                    ui.separator();

                    // Headlights
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut state.light_headlights_enabled, "");
                        ui.label("Headlights");
                    });
                    ui.add_enabled(
                        state.light_headlights_enabled,
                        egui::Slider::new(&mut state.light_headlights_intensity, 0.0..=2.0)
                            .text("Intensity")
                            .fixed_decimals(2),
                    );
                });

                // ── Models ──
                ui.collapsing(RichText::new("🚗  Models").size(15.0), |ui| {
                    ui.checkbox(&mut state.show_car, "Show car model");
                    if state.show_car && !state.car_loaded {
                        ui.label(RichText::new("Loading...").color(Color32::YELLOW));
                    } else if state.show_car && state.car_loaded {
                        ui.label(RichText::new("Loaded (38 draw calls)").color(Color32::GREEN));
                    }
                    ui.separator();
                    ui.checkbox(&mut state.show_normal_maps, "Normal maps on spheres");
                    if state.show_normal_maps {
                        ui.label(
                            RichText::new("Sphere → brick_normalmap.png\nBigSphere → normal_map.jpg")
                                .color(Color32::from_rgb(140, 200, 255))
                                .size(12.0),
                        );
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
