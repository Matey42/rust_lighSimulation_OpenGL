mod camera;
mod grid;
mod light;
mod model;
mod primitives;
mod renderer;
mod scene;
mod types;
mod vertex;

use std::time::Instant;

use cgmath::{Deg, Matrix4, Point3, Vector3};
use glium::Surface;
use glium::winit;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

use camera::{Camera, FppCamera, StaticCamera, TppCamera, TrackingCamera};
use grid::Grid;
use renderer::Renderer;
use scene::Scene;

// ─────────────────── Application State ───────────────────

struct AppState {
    /// 0 = Static, 1 = Tracking, 2 = TPP, 3 = FPP
    active_camera: usize,
    use_phong: bool,
    fog_enabled: bool,
    fog_density: f32,
    /// 0.0 = night, 1.0 = full day
    day_factor: f32,
    day_night_speed: f32,
    day_night_auto: bool,
    /// Spotlight direction offset adjustment (for manual control)
    spotlight_yaw_offset: f32,
    spotlight_pitch_offset: f32,
}

impl Default for AppState {
    fn default() -> Self {
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
        }
    }
}

// ─────────────────────── Entry ───────────────────────────

fn main() {
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
    let grid = Grid::new(&display);
    let mut state = AppState::default();

    // ── Cameras ──
    let static_cam = StaticCamera::new(
        Point3::new(12.0, 10.0, 12.0),
        Point3::new(0.0, 0.0, 0.0),
    );
    let mut tracking_cam = TrackingCamera::new(Point3::new(0.0, 12.0, -14.0));
    let mut tpp_cam = TppCamera::new(Vector3::new(0.0, 2.0, 5.0), 3.0);
    let mut fpp_cam = FppCamera::new(Vector3::new(0.0, 1.2, 0.0));

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
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key,
                                ..
                            },
                        ..
                    } => {
                        handle_key(&logical_key, &mut state);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = (now - last_frame).as_secs_f32();
                        last_frame = now;

                        // ── Update ──
                        scene.moving_object.update(dt);

                        // Apply manual spotlight offset
                        for light in &mut scene.moving_object.spotlights {
                            light.direction.x += state.spotlight_yaw_offset * 0.1;
                            light.direction.y += state.spotlight_pitch_offset * 0.1;
                        }

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
                            if state.day_factor > 1.0 || state.day_factor < 0.0 {
                                state.day_night_speed = -state.day_night_speed;
                                state.day_factor = state.day_factor.clamp(0.0, 1.0);
                            }
                        }

                        // Modulate the sun (directional light) by day factor
                        if let Some(sun) = scene.lights.iter_mut().find(|l| {
                            l.kind == crate::light::LightKind::Directional
                        }) {
                            let f = state.day_factor;
                            sun.diffuse = [f * 1.0, f * 0.95, f * 0.85];
                            sun.ambient = [f * 0.15, f * 0.15, f * 0.15];
                        }

                        // ── Render ──
                        renderer.use_phong = state.use_phong;

                        let view: Matrix4<f32> = match state.active_camera {
                            0 => static_cam.view_matrix(),
                            1 => tracking_cam.view_matrix(),
                            2 => tpp_cam.view_matrix(),
                            3 => fpp_cam.view_matrix(),
                            _ => static_cam.view_matrix(),
                        };

                        let (width, height) = {
                            let size = window.inner_size();
                            (size.width as f32, size.height as f32)
                        };
                        let aspect = width / height;
                        let projection: Matrix4<f32> =
                            cgmath::perspective(Deg(55.0), aspect, 0.1, 200.0);

                        // Fog color adjusts with day/night
                        let fog_base = 0.55 * state.day_factor + 0.05;
                        let fog_color = [fog_base, fog_base, fog_base + 0.05];

                        // Collect all lights (scene + moving object headlights)
                        let mut all_lights: Vec<_> = scene.lights.clone();
                        all_lights.extend(scene.moving_object.spotlights.iter().cloned());

                        let ambient_strength = 0.15 + 0.85 * state.day_factor;

                        let params = glium::DrawParameters {
                            depth: glium::Depth {
                                test: glium::draw_parameters::DepthTest::IfLess,
                                write: true,
                                ..Default::default()
                            },
                            backface_culling:
                                glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                            ..Default::default()
                        };

                        let mut target = display.draw();

                        // Clear with sky colour based on day factor
                        let sky_r = 0.05 + 0.45 * state.day_factor;
                        let sky_g = 0.05 + 0.55 * state.day_factor;
                        let sky_b = 0.1 + 0.6 * state.day_factor;
                        target.clear_color_and_depth((sky_r, sky_g, sky_b, 1.0), 1.0);

                        // Draw grid floor
                        grid.draw(
                            &mut target,
                            &view,
                            &projection,
                            state.fog_enabled,
                            fog_color,
                            state.fog_density,
                            ambient_strength,
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

                        target.finish().expect("Failed to swap buffers");
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}

fn handle_key(key: &Key, state: &mut AppState) {
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
                state.fog_density = (state.fog_density + 0.005).min(0.2);
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
        Key::Named(NamedKey::ArrowLeft) => {
            state.spotlight_yaw_offset -= 0.05;
            println!("[Spotlight yaw] {:.2}", state.spotlight_yaw_offset);
        }
        Key::Named(NamedKey::ArrowRight) => {
            state.spotlight_yaw_offset += 0.05;
            println!("[Spotlight yaw] {:.2}", state.spotlight_yaw_offset);
        }
        Key::Named(NamedKey::ArrowUp) => {
            state.spotlight_pitch_offset += 0.05;
            println!("[Spotlight pitch] {:.2}", state.spotlight_pitch_offset);
        }
        Key::Named(NamedKey::ArrowDown) => {
            state.spotlight_pitch_offset -= 0.05;
            println!("[Spotlight pitch] {:.2}", state.spotlight_pitch_offset);
        }
        _ => {}
    }
}
