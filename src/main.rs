use macroquad::prelude::*;
use tree_simulator::simulation::{SIM_DT, plan_steps};
use tree_simulator::{GrowthSectionState, TreeSimulation, Vector2D};

mod ui;

// Default screen dimensions
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

/// A fresh seed from the clock, so each new tree differs unless regrown on purpose.
fn new_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Tree Growth Simulator".to_string(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let trunk_start = Vector2D {
        x: SCREEN_WIDTH / 2.0,
        y: SCREEN_HEIGHT - 50.0,
    };
    let default_camera_target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);

    let trunk_thickness = 20.0;

    let mut sim = TreeSimulation::new(trunk_start, trunk_thickness, new_seed());
    let mut sim_accumulator: f32 = 0.0;

    // Initialize Camera State
    // Target starts focused on the middle of the screen
    let mut camera_target = default_camera_target;
    let mut zoom_level: f32 = 1.0;
    let mut invert_pan = true;
    let mut show_leaves = true;
    let mut sim_speed: f32 = 1.0;
    let mut is_paused = false;
    const LEAF_GROW_DURATION: f32 = 1.0; // 1.0 second growth animation
    // Sim-time per real second at the slider's 1x. Raised from an implicit 1.0 because the old 5x
    // cap still felt slow; keeps the slider labels while running every setting 4x faster.
    const SPEED_SCALE: f32 = 4.0;

    loop {
        let raw_delta_time = get_frame_time();

        // --- UPDATE SIMULATION (fixed steps; see plan_steps) ---
        let speed = if is_paused { 0.0 } else { sim_speed * SPEED_SCALE };
        let (steps, next_accumulator) = plan_steps(sim_accumulator, raw_delta_time, speed);
        sim_accumulator = next_accumulator;
        for _ in 0..steps {
            sim.step();
        }
        // Simulated time actually stepped, so leaves pause and slow down with the tree.
        let delta_time = steps as f32 * SIM_DT;

        // Update Leaf Growth Timers
        if show_leaves {
            for section in &mut sim.growth_sections {
                // If it is a terminal section, grow its leaf
                if let GrowthSectionState::FullyGrown { ref child_indices } = section.state {
                    if child_indices.is_empty() {
                        section.leaf_timer =
                            (section.leaf_timer + delta_time).min(LEAF_GROW_DURATION);
                    }
                }
            }
        } else {
            // Reset all timers if toggle is turned off
            for section in &mut sim.growth_sections {
                section.leaf_timer = 0.0;
            }
        }

        // --- DRAW FIXED ELEMENTS ---
        clear_background(SKYBLUE);

        // Setup Camera
        let aspect_ratio_correction =
            vec2(1.0 / (screen_width() / 2.0), 1.0 / (screen_height() / 2.0));

        let camera = Camera2D {
            target: camera_target,
            zoom: aspect_ratio_correction * zoom_level,
            ..Default::default()
        };

        // Activate world space camera
        set_camera(&camera);

        // --- DRAW NON-FIXED ELEMENTS ---
        // Draw ground
        draw_rectangle(
            -10000.0,      // Huge left offset for panning
            trunk_start.y, // Starts at the trunk's base
            20000.0,       // Huge width
            10000.0,       // Huge height downward
            Color::from_rgba(120, 200, 80, 255),
        );

        // Draw branches
        for section in &sim.growth_sections {
            let start = section.start_point;
            let end = section.end_point();
            draw_line(start.x, start.y, end.x, end.y, section.thickness, DARKBROWN);
        }

        // Draw leaves
        if show_leaves {
            for section in &sim.growth_sections {
                if let GrowthSectionState::FullyGrown { ref child_indices } = section.state {
                    if child_indices.is_empty() && section.leaf_timer > 0.0 {
                        // Progress normalized between 0.0 and 1.0
                        let progress = (section.leaf_timer / LEAF_GROW_DURATION).clamp(0.0, 1.0);

                        // Quadratic Ease-Out: 1 - (1 - t)^2 for smooth popping effect
                        let scale = 1.0 - (1.0 - progress) * (1.0 - progress);

                        let end = section.end_point();

                        // Multiply final radius by animated scale
                        let rx = 10.0 * scale;
                        let ry = 6.0 * scale;
                        let angle = section.angle.to_degrees();
                        let outline_thickness = rx / 20.0;

                        // Only render if scale is noticeably greater than zero
                        if rx > 0.01 {
                            draw_ellipse(
                                end.x,
                                end.y,
                                rx + outline_thickness,
                                ry + outline_thickness,
                                angle,
                                BLACK,
                            );

                            draw_ellipse(end.x, end.y, rx, ry, angle, DARKGREEN);
                        }
                    }
                }
            }
        }

        // Switch back to default screen space camera for UI overlay
        set_default_camera();

        // --- UI OVERLAY ---
        draw_text(
            "Controls: [Scroll / +/-] Zoom | [Space] Play/Pause | [Click + Drag] Pan | [R] New Tree | [Shift+R] Regrow | [C] Center View",
            10.0,
            25.0,
            16.0,
            BLACK,
        );
        draw_text(
            &format!(
                "Number of Growth Sections: {} | Zoom: {:.1}x | Seed: {}",
                sim.growth_sections.len(),
                zoom_level,
                sim.seed
            ),
            10.0,
            50.0,
            16.0,
            BLACK,
        );

        // Call our function cleanly via `ui::draw_toggle_button`
        if ui::draw_toggle_button(
            640.0, 40.0, 140.0, 35.0, "Pan Drag", "INVERTED", "REGULAR", invert_pan,
        ) {
            invert_pan = !invert_pan;
        }

        if ui::draw_toggle_button(
            640.0,
            80.0,
            140.0,
            35.0,
            "Show Leaves",
            "TRUE",
            "FALSE",
            show_leaves,
        ) {
            show_leaves = !show_leaves;
        }

        let is_slider_dragging =
            ui::draw_slider(640.0, 120.0, 140.0, 35.0, "Speed", &mut sim_speed, 0.1, 5.0);

        if ui::draw_toggle_button(600.0, 120.0, 35.0, 35.0, "", ">>", "||", is_paused) {
            is_paused = !is_paused;
        }

        // --- INPUT HANDLING ---

        // Reset Simulation
        // R grows a new tree; Shift+R regrows the same seed.
        if is_key_pressed(KeyCode::R) {
            let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            let seed = if shift { sim.seed } else { new_seed() };
            sim.reset(trunk_start, trunk_thickness, seed);
            sim_accumulator = 0.0;
        }

        // Pause Simulation
        if is_key_pressed(KeyCode::Space) {
            is_paused = !is_paused;
        }

        // Keyboard Zoom Shortcuts (+ / - or = / -)
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            zoom_level = (zoom_level * 1.2).clamp(0.2, 10.0);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            zoom_level = (zoom_level / 1.2).clamp(0.2, 10.0);
        }

        // Scrolling -> Zoom In / Zoom Out
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            // Scroll UP zooms IN, Scroll DOWN zooms OUT
            let zoom_factor = if scroll_y > 0.0 { 1.08 } else { 1.0 / 1.08 };
            zoom_level = (zoom_level * zoom_factor).clamp(0.2, 10.0);
        }

        // Mouse / Trackpad Dragging for Panning (With inversion toggle)
        if is_mouse_button_down(MouseButton::Left) && !is_slider_dragging {
            let mouse_delta = mouse_delta_position();
            let sensitivity = (screen_width() / 2.0) / zoom_level;
            let direction_multiplier = if invert_pan { 1.0 } else { -1.0 };

            camera_target.x += mouse_delta.x * sensitivity * direction_multiplier;
            camera_target.y += mouse_delta.y * sensitivity * direction_multiplier;
        }

        // Reset View Center
        if is_key_pressed(KeyCode::C) {
            camera_target = vec2(400.0, 300.0);
            zoom_level = 1.0;
        }

        next_frame().await;
    }
}
