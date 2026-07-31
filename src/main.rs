use macroquad::prelude::*;
use tree_simulator::{GrowthSectionState, TreeSimulation, Vector2D};

mod ui;

// Default screen dimensions
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

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

    let trunk_length = 100.0;
    let trunk_thickness = 20.0;

    let mut sim = TreeSimulation::new(trunk_start, trunk_length, trunk_thickness);

    // Initialize Camera State
    // Target starts focused on the middle of the screen
    let mut camera_target = default_camera_target;
    let mut zoom_level: f32 = 1.0;
    let mut invert_pan = true;
    let mut show_leaves = true;
    let mut sim_speed: f32 = 1.0;
    let mut is_paused = false;
    const LEAF_GROW_DURATION: f32 = 1.0; // 1.0 second growth animation

    loop {
        let raw_delta_time = get_frame_time();

        // --- APPLY SPEED MULTIPLIER ---
        let delta_time = if is_paused {
            0.0
        } else {
            raw_delta_time * sim_speed
        };

        // --- UPDATE SIMULATION ---
        sim.update(delta_time);

        // Update Leaf Growth Timers
        if show_leaves {
            for branch in &mut sim.branches {
                // If it is a terminal branch, grow its leaf
                if let GrowthSectionState::FullyGrown { ref child_indices } = branch.state {
                    if child_indices.is_empty() {
                        branch.leaf_timer =
                            (branch.leaf_timer + delta_time).min(LEAF_GROW_DURATION);
                    }
                }
            }
        } else {
            // Reset all timers if toggle is turned off
            for branch in &mut sim.branches {
                branch.leaf_timer = 0.0;
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
        for branch in &sim.branches {
            let start = branch.start_point;
            let end = branch.end_point();
            draw_line(start.x, start.y, end.x, end.y, branch.thickness, DARKBROWN);
        }

        // Draw leaves
        if show_leaves {
            for branch in &sim.branches {
                if let GrowthSectionState::FullyGrown { ref child_indices } = branch.state {
                    if child_indices.is_empty() && branch.leaf_timer > 0.0 {
                        // Progress normalized between 0.0 and 1.0
                        let progress = (branch.leaf_timer / LEAF_GROW_DURATION).clamp(0.0, 1.0);

                        // Quadratic Ease-Out: 1 - (1 - t)^2 for smooth popping effect
                        let scale = 1.0 - (1.0 - progress) * (1.0 - progress);

                        let end = branch.end_point();

                        // Multiply final radius by animated scale
                        let rx = 10.0 * scale;
                        let ry = 6.0 * scale;
                        let angle = branch.angle.to_degrees();
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
            "Controls: [Scroll / +/-] Zoom | [Space] Play/Pause | [Click + Drag] Pan | [R] Reset Tree | [C] Center View",
            10.0,
            25.0,
            16.0,
            BLACK,
        );
        draw_text(
            &format!(
                "Active Branches: {} | Zoom: {:.1}x",
                sim.branches.len(),
                zoom_level
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
        if is_key_pressed(KeyCode::R) {
            sim.reset(trunk_start, trunk_length, trunk_thickness);
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
