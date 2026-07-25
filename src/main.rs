use macroquad::prelude::*;
use tree_simulator::{BranchState, TreeSimulation, Vector2D};

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
    let trunk_thickness = 12.0;

    let mut sim = TreeSimulation::new(trunk_start, trunk_length, trunk_thickness);

    // Initialize Camera State
    // Target starts focused on the middle of the screen
    let mut camera_target = default_camera_target;
    let mut zoom_level: f32 = 1.0;
    let mut invert_pan = true;

    let mut show_leaves = false;

    loop {
        let delta_time = get_frame_time();

        // --- INPUT HANDLING ---

        // Reset Simulation
        if is_key_pressed(KeyCode::R) {
            sim.reset(trunk_start, trunk_length, trunk_thickness);
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
        if is_mouse_button_down(MouseButton::Left) {
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

        // --- UPDATE SIMULATION ---
        sim.update(delta_time);

        // --- RENDER LAYER ---
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

        // Render world elements (branches)
        for branch in &sim.branches {
            let start = branch.start_point;
            let end = branch.end_point();

            draw_line(start.x, start.y, end.x, end.y, branch.thickness, DARKBROWN);
        }

        if sim.is_finished() && show_leaves {
            for branch in &sim.branches {
                if let BranchState::FullyGrown { ref child_indices } = branch.state {
                    if child_indices.is_empty() {
                        let end = branch.end_point();
                        let rx = branch.thickness * 5.0;
                        let ry = branch.thickness * 3.0;
                        let angle = branch.angle.to_degrees();
                        let outline_thickness = 0.2;

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

        // Switch back to default screen space camera for UI overlay
        set_default_camera();

        // --- UI OVERLAY ---
        draw_text(
            "Controls: [Scroll / +/-] Zoom | [Click + Drag] Pan | [R] Reset Tree | [C] Center View",
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
            640.0, 15.0, 140.0, 35.0, "Pan Drag", "INVERTED", "REGULAR", invert_pan,
        ) {
            invert_pan = !invert_pan;
        }

        if ui::draw_toggle_button(
            640.0,
            55.0,
            140.0,
            35.0,
            "Show Leaves",
            "TRUE",
            "FALSE",
            show_leaves,
        ) {
            show_leaves = !show_leaves;
        }

        next_frame().await;
    }
}
