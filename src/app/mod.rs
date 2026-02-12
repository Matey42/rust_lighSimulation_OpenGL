use std::time::Instant;

use cgmath::{Deg, Matrix4, Point3, Vector3};
use egui::ViewportId;
use glium::Surface;
use glium::winit;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

use crate::core::camera::{Camera, FppCamera, FreeCamera, StaticCamera, TppCamera, TrackingCamera};
use crate::core::light::{Light, LightKind};
use crate::render::grid::Grid;
use crate::render::light_markers::LightMarkers;
use crate::render::renderer::Renderer;
use crate::scene::Scene;
use crate::ui::{draw_ui, UiState};

// ─────────────────────── Entry ───────────────────────────

pub fn run() {
    // ── Window + OpenGL context ──
    let event_loop = winit::event_loop::EventLoop::builder()
        .build()
        .expect("Failed to create event loop");

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("GK4 - 3D Scene (Phong/Gouraud, Fog, Day/Night)")
        .with_inner_size(1280, 720)
        .build(&event_loop);

    // ── Scene & renderer ──
    let mut scene = Scene::build_default(&display);
    let mut renderer = Renderer::new(&display);
    let mut grid = Grid::new(&display);
    let light_markers = LightMarkers::new(&display);
    let mut state = UiState::default();

    // ── egui ──
    let mut egui_glium =
        egui_glium::EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);

    // Dark theme for game-engine look
    egui_glium.egui_ctx().set_visuals(egui::Visuals::dark());

    // ── Cameras ──
    let static_cam = StaticCamera::new(
        Point3::new(12.0, 10.0, 12.0),
        Point3::new(0.0, 0.0, 0.0),
    );
    let mut tracking_cam = TrackingCamera::new(Point3::new(0.0, 12.0, -14.0));
    let mut tpp_cam = TppCamera::new(Vector3::new(0.0, 2.0, 5.0), 3.0);
    let mut fpp_cam = FppCamera::new(Vector3::new(0.0, 0.2, 0.55));
    let mut free_cam = FreeCamera::new(Point3::new(12.0, 10.0, 12.0));
    let mut prev_camera: usize = 0;

    let mut last_frame = Instant::now();

    // Print controls
    println!("═══════════════════════════════════════════════════════");
    println!("  GK4 – 3D Scene Application                         ");
    println!("═══════════════════════════════════════════════════════");
    println!("  1/2/3/4  – Switch camera (Static/Track/TPP/FPP)    ");
    println!("  P        – Toggle Phong / Gouraud shading           ");
    println!("  F        – Toggle fog                               ");
    println!("  +/-      – Adjust fog density                       ");
    println!("  N        – Toggle auto day/night cycle              ");
    println!("  O/L      – Manual day ↑ / night ↓                  ");
    println!("  Arrow keys – Adjust spotlight direction             ");
    println!("  Esc      – Quit                                     ");
    println!("═══════════════════════════════════════════════════════");

    // ── Main loop ──
    #[allow(deprecated)]
    let _ = event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    // Forward events to egui first
                    let response = egui_glium.on_event(&window, &event);

                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        // Track key presses AND releases for continuous WASD
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: key_state,
                                    logical_key,
                                    ..
                                },
                            ..
                        } => {
                            let pressed = key_state == ElementState::Pressed;

                            // Tab toggles panel visibility (always handled, press only)
                            if pressed && logical_key == Key::Named(NamedKey::Tab) {
                                state.show_panel = !state.show_panel;
                            }

                            // Track WASD / QE for free camera OR manual drive
                            if state.active_camera == 4 || state.manual_drive {
                                match &logical_key {
                                    Key::Character(c) => match c.as_str() {
                                        "w" | "W" => state.key_w = pressed,
                                        "a" | "A" => state.key_a = pressed,
                                        "s" | "S" => state.key_s = pressed,
                                        "d" | "D" => state.key_d = pressed,
                                        "q" | "Q" => state.key_q = pressed,
                                        "e" | "E" => state.key_e = pressed,
                                        _ => {}
                                    }
                                    _ => {}
                                }
                            }

                            // Track arrow keys for free-cam look
                            match &logical_key {
                                Key::Named(NamedKey::ArrowLeft) => state.key_arrow_left = pressed,
                                Key::Named(NamedKey::ArrowRight) => state.key_arrow_right = pressed,
                                Key::Named(NamedKey::ArrowUp) => state.key_arrow_up = pressed,
                                Key::Named(NamedKey::ArrowDown) => state.key_arrow_down = pressed,
                                _ => {}
                            }

                            // Only handle scene keys on press when egui doesn't want input
                            if pressed && !response.consumed {
                                handle_key(&logical_key, &mut state);
                            }
                        }
                        // Track right mouse button for free-camera look
                        WindowEvent::MouseInput {
                            state: btn_state,
                            button: winit::event::MouseButton::Right,
                            ..
                        } => {
                            state.mouse_look =
                                btn_state == ElementState::Pressed;
                        }
                        // Mouse motion for free-camera look
                        WindowEvent::CursorMoved { .. } => {}
                    
                        WindowEvent::RedrawRequested => {
                            let now = Instant::now();
                            let dt = (now - last_frame).as_secs_f32();
                            last_frame = now;

                            // Performance stats
                            if dt > 0.0 {
                                state.fps = 1.0 / dt;
                                state.frame_time_ms = dt * 1000.0;
                            }

                            // ── Update ──
                            if !state.manual_drive {
                                scene.moving_object.update(dt);
                            }

                            // Manual drive of the moving object
                            if state.manual_drive {
                                let fwd = if state.key_w { 1.0 } else { 0.0 }
                                    - if state.key_s { 1.0 } else { 0.0 };
                                let mut turn = if state.key_a { 1.0 } else { 0.0 }
                                    - if state.key_d { 1.0 } else { 0.0 };
                                // Reverse turning when going backward (like a real vehicle)
                                if fwd < 0.0 {
                                    turn = -turn;
                                }
                                // Only pass movement when NOT in free cam
                                // (free cam uses WASD for itself)
                                if state.active_camera != 4 {
                                    scene.moving_object.manual_drive_update(
                                        fwd,
                                        turn,
                                        state.drive_speed,
                                        state.drive_turn_speed,
                                        dt,
                                    );
                                }
                            }

                            // Free camera movement (WASD + QE)
                            if state.active_camera == 4 {
                                free_cam.speed = state.free_cam_speed;
                                free_cam.sensitivity = state.free_cam_sensitivity * 0.01;
                                let fwd = if state.key_w { 1.0 } else { 0.0 }
                                    - if state.key_s { 1.0 } else { 0.0 };
                                let right = if state.key_d { 1.0 } else { 0.0 }
                                    - if state.key_a { 1.0 } else { 0.0 };
                                let up = if state.key_e { 1.0 } else { 0.0 }
                                    - if state.key_q { 1.0 } else { 0.0 };
                                free_cam.process_keyboard(fwd, right, up, dt);

                                // Arrow keys rotate the camera view
                                let arrow_yaw = if state.key_arrow_right { 1.0 } else { 0.0 }
                                    - if state.key_arrow_left { 1.0 } else { 0.0 };
                                let arrow_pitch = if state.key_arrow_up { 1.0 } else { 0.0 }
                                    - if state.key_arrow_down { 1.0 } else { 0.0 };
                                if arrow_yaw != 0.0 || arrow_pitch != 0.0 {
                                    // Scale to feel similar to mouse look
                                    let look_speed = 220.0; // degrees-per-second feel
                                    free_cam.process_mouse(
                                        arrow_yaw * look_speed * dt,
                                        -arrow_pitch * look_speed * dt,
                                    );
                                }
                            }

                            // Continuously update spotlight aim from arrow keys
                            // (works in parallel with WASD driving)
                            if state.active_camera != 4 {
                                let spot_speed = 1.2; // radians per second
                                let yaw_input = if state.key_arrow_left { 1.0f32 } else { 0.0 }
                                    - if state.key_arrow_right { 1.0 } else { 0.0 };
                                let pitch_input = if state.key_arrow_up { 1.0f32 } else { 0.0 }
                                    - if state.key_arrow_down { 1.0 } else { 0.0 };
                                state.spotlight_yaw_offset = (state.spotlight_yaw_offset + yaw_input * spot_speed * dt).clamp(-1.0, 1.0);
                                state.spotlight_pitch_offset = (state.spotlight_pitch_offset + pitch_input * spot_speed * dt).clamp(-1.0, 1.0);
                            }

                            // Apply spotlight aim (in local cube space, like real headlights)
                            scene.moving_object.light_aim_yaw =
                                state.spotlight_yaw_offset * 0.5;
                            scene.moving_object.light_aim_pitch =
                                state.spotlight_pitch_offset * 0.5;

                            // Update cameras
                            let mover_pos = scene.moving_object.world_position();
                            let mover_yaw = scene.moving_object.yaw();
                            tracking_cam.update_target(mover_pos);
                            tpp_cam.update(mover_pos, mover_yaw);
                            fpp_cam.update(mover_pos, mover_yaw);

                            // Day/night
                            if state.day_night_auto {
                                state.day_factor +=
                                    state.day_night_speed * dt * 0.5;
                                if state.day_factor > 1.0 || state.day_factor < 0.0
                                {
                                    state.day_night_speed =
                                        -state.day_night_speed;
                                    state.day_factor =
                                        state.day_factor.clamp(0.0, 1.0);
                                }
                            }

                            // Modulate the sun (directional light) by day factor
                            if let Some(sun) =
                                scene.lights.iter_mut().find(|l| {
                                    l.kind == LightKind::Directional
                                })
                            {
                                let f = state.day_factor;
                                sun.diffuse = [f * 1.0, f * 0.95, f * 0.85];
                                sun.ambient = [f * 0.15, f * 0.15, f * 0.15];
                            }

                            // Sync grid config from UI state
                            grid.config.grid_size = state.grid_size;
                            grid.config.sub_grid_size = state.grid_sub_size;
                            grid.config.fade_radius = state.grid_fade_radius;
                            grid.config.line_width = state.grid_line_width;

                            // ── Build egui UI ──
                            egui_glium.run(&window, |ctx| {
                                draw_ui(ctx, &mut state);
                            });

                            // ── Render ──
                            renderer.use_phong = state.use_phong;

                            // When switching to free cam, inherit position & look direction
                            if state.active_camera == 4 && prev_camera != 4 {
                                let prev_pos: Point3<f32> = match prev_camera {
                                    0 => static_cam.position(),
                                    1 => tracking_cam.position(),
                                    2 => tpp_cam.position(),
                                    3 => fpp_cam.position(),
                                    _ => free_cam.eye,
                                };
                                // Compute the look-at target of the previous camera
                                let prev_view = match prev_camera {
                                    0 => static_cam.view_matrix(),
                                    1 => tracking_cam.view_matrix(),
                                    2 => tpp_cam.view_matrix(),
                                    3 => fpp_cam.view_matrix(),
                                    _ => free_cam.view_matrix(),
                                };
                                // Extract forward direction from view matrix (row 2 = -forward)
                                let fwd = cgmath::Vector3::new(
                                    -prev_view.x.z, -prev_view.y.z, -prev_view.z.z,
                                );
                                free_cam.eye = prev_pos;
                                free_cam.yaw = fwd.z.atan2(fwd.x);
                                free_cam.pitch = fwd.y.asin().clamp(
                                    -89.0_f32.to_radians(),
                                    89.0_f32.to_radians(),
                                );
                            }
                            prev_camera = state.active_camera;

                            let view: Matrix4<f32> = match state.active_camera {
                                0 => static_cam.view_matrix(),
                                1 => tracking_cam.view_matrix(),
                                2 => tpp_cam.view_matrix(),
                                3 => fpp_cam.view_matrix(),
                                4 => free_cam.view_matrix(),
                                _ => static_cam.view_matrix(),
                            };

                            let (width, height) = {
                                let size = window.inner_size();
                                (size.width as f32, size.height as f32)
                            };
                            let aspect = width / height;
                            let projection: Matrix4<f32> =
                                cgmath::perspective(Deg(55.0), aspect, 0.1, 200.0);

                            // Fog color = sky color for realistic vanishing
                            let df = state.day_factor;
                            let fog_color = [
                                0.05 + 0.45 * df,
                                0.05 + 0.55 * df,
                                0.1  + 0.6  * df,
                            ];

                            // Collect lights with on/off and intensity controls
                            let mut all_lights: Vec<Light> = Vec::new();
                            // 0: Point light
                            if state.light_point_enabled {
                                let mut l = scene.lights[0].clone();
                                let s = state.light_point_intensity;
                                l.diffuse = [l.diffuse[0]*s, l.diffuse[1]*s, l.diffuse[2]*s];
                                l.specular = [l.specular[0]*s, l.specular[1]*s, l.specular[2]*s];
                                all_lights.push(l);
                            }
                            // 1: Fixed spot
                            if state.light_spot_enabled {
                                let mut l = scene.lights[1].clone();
                                let s = state.light_spot_intensity;
                                l.diffuse = [l.diffuse[0]*s, l.diffuse[1]*s, l.diffuse[2]*s];
                                l.specular = [l.specular[0]*s, l.specular[1]*s, l.specular[2]*s];
                                all_lights.push(l);
                            }
                            // 2: Sun (directional)
                            if state.light_sun_enabled {
                                let mut l = scene.lights[2].clone();
                                let s = state.light_sun_intensity;
                                l.diffuse = [l.diffuse[0]*s, l.diffuse[1]*s, l.diffuse[2]*s];
                                l.specular = [l.specular[0]*s, l.specular[1]*s, l.specular[2]*s];
                                all_lights.push(l);
                            }
                            // Headlights
                            if state.light_headlights_enabled {
                                for hl in &scene.moving_object.spotlights {
                                    let mut l = hl.clone();
                                    let s = state.light_headlights_intensity;
                                    l.diffuse = [l.diffuse[0]*s, l.diffuse[1]*s, l.diffuse[2]*s];
                                    l.specular = [l.specular[0]*s, l.specular[1]*s, l.specular[2]*s];
                                    all_lights.push(l);
                                }
                            }

                            let ambient_strength =
                                0.15 + 0.85 * state.day_factor;

                            let params = glium::DrawParameters {
                                depth: glium::Depth {
                                    test: glium::draw_parameters::DepthTest::IfLess,
                                    write: true,
                                    ..Default::default()
                                },
                                backface_culling:
                                    glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                                blend: glium::Blend::alpha_blending(),
                                ..Default::default()
                            };

                            let mut target = display.draw();

                            // Clear with sky colour based on day factor
                            let sky_r = 0.05 + 0.45 * state.day_factor;
                            let sky_g = 0.05 + 0.55 * state.day_factor;
                            let sky_b = 0.1 + 0.6 * state.day_factor;
                            target.clear_color_and_depth(
                                (sky_r, sky_g, sky_b, 1.0),
                                1.0,
                            );

                            // Draw grid floor
                            if state.grid_visible {
                                grid.draw(
                                    &mut target,
                                    &view,
                                    &projection,
                                    state.fog_enabled,
                                    fog_color,
                                    state.fog_density,
                                    ambient_strength,
                                    &all_lights,
                                );
                            }

                            // Draw light source markers
                            light_markers.draw(
                                &mut target,
                                &all_lights,
                                &view,
                                &projection,
                                state.fog_enabled,
                                fog_color,
                                state.fog_density,
                            );

                            // Draw static objects
                            for obj in &scene.static_objects {
                                renderer.draw_object(
                                    &mut target,
                                    obj,
                                    &view,
                                    &projection,
                                    &all_lights,
                                    state.fog_enabled,
                                    fog_color,
                                    state.fog_density,
                                    ambient_strength,
                                    &params,
                                );
                            }

                            // Load / unload car model based on UI toggle
                            if state.show_car && !state.car_loaded {
                                scene.load_car(&display);
                                state.car_loaded = true;
                            } else if !state.show_car && state.car_loaded {
                                scene.unload_car();
                                state.car_loaded = false;
                            }

                            // Load / unload normal maps based on UI toggle
                            if state.show_normal_maps && !state.normal_maps_loaded {
                                scene.load_normal_maps(&display);
                                state.normal_maps_loaded = true;
                            } else if !state.show_normal_maps && state.normal_maps_loaded {
                                scene.unload_normal_maps();
                                state.normal_maps_loaded = false;
                            }

                            // Draw car model (if loaded)
                            if let Some(car) = &scene.car_model {
                                renderer.draw_object(
                                    &mut target,
                                    car,
                                    &view,
                                    &projection,
                                    &all_lights,
                                    state.fog_enabled,
                                    fog_color,
                                    state.fog_density,
                                    ambient_strength,
                                    &params,
                                );
                            }

                            // Draw the moving object
                            renderer.draw_object(
                                &mut target,
                                &scene.moving_object.obj,
                                &view,
                                &projection,
                                &all_lights,
                                state.fog_enabled,
                                fog_color,
                                state.fog_density,
                                ambient_strength,
                                &params,
                            );

                            // Draw egui on top of everything
                            egui_glium.paint(&display, &mut target);

                            target.finish().expect("Failed to swap buffers");
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if state.active_camera == 4 && state.mouse_look {
                        free_cam.process_mouse(delta.0 as f32, delta.1 as f32);
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}

fn handle_key(key: &Key, state: &mut UiState) {
    match key {
        Key::Named(NamedKey::Escape) => std::process::exit(0),
        Key::Character(c) => match c.as_str() {
            "1" => {
                state.active_camera = 0;
                println!("[Camera] Static");
            }
            "2" => {
                state.active_camera = 1;
                println!("[Camera] Tracking");
            }
            "3" => {
                state.active_camera = 2;
                println!("[Camera] Third-Person (TPP)");
            }
            "4" => {
                state.active_camera = 3;
                println!("[Camera] First-Person (FPP)");
            }
            "5" => {
                state.active_camera = 4;
                println!("[Camera] Free (WASD)");
            }
            "p" | "P" => {
                state.use_phong = !state.use_phong;
                println!(
                    "[Shading] {}",
                    if state.use_phong { "Phong" } else { "Gouraud" }
                );
            }
            "f" | "F" => {
                state.fog_enabled = !state.fog_enabled;
                println!("[Fog] {}", if state.fog_enabled { "ON" } else { "OFF" });
            }
            "+" | "=" => {
                state.fog_density = (state.fog_density + 0.005).min(0.1);
                println!("[Fog density] {:.3}", state.fog_density);
            }
            "-" => {
                state.fog_density = (state.fog_density - 0.005).max(0.0);
                println!("[Fog density] {:.3}", state.fog_density);
            }
            "n" | "N" => {
                state.day_night_auto = !state.day_night_auto;
                println!(
                    "[Day/Night auto] {}",
                    if state.day_night_auto { "ON" } else { "OFF" }
                );
            }
            "o" | "O" => {
                state.day_factor = (state.day_factor + 0.05).min(1.0);
                println!("[Day factor] {:.2}", state.day_factor);
            }
            "l" | "L" => {
                state.day_factor = (state.day_factor - 0.05).max(0.0);
                println!("[Day factor] {:.2}", state.day_factor);
            }
            _ => {}
        },
        Key::Named(NamedKey::Space) => {
            state.manual_drive = !state.manual_drive;
            println!(
                "[Drive] {}",
                if state.manual_drive { "Manual" } else { "Animation" }
            );
        }
        // Arrow keys now handled continuously per-frame in the update loop
        // so they work in parallel with WASD driving
        _ => {}
    }
}
