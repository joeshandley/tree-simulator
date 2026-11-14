use macroquad::prelude::*;
use tree_simulator::params::HOURS_PER_YEAR;
use tree_simulator::simulation::{SIM_DT, plan_steps};
use tree_simulator::{GrowthSectionState, TreeSimulation, Vector2D};

mod ui;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

/// Screen pixels per simulated metre at zoom 1×. The simulation never sees
/// pixels; only drawing and camera input convert (plan 0003, D14).
const PX_PER_M: f32 = 200.0;

/// Simulated hours per real second at the slider's 1×: about a year per 3 s.
/// Higher slider settings hit `MAX_STEPS_PER_FRAME` and run as fast as the
/// machine allows.
const SPEED_SCALE: f32 = 3000.0;

/// Leaves unfold over a simulated week.
const LEAF_GROW_DURATION: f32 = 7.0 * 24.0;

const ZOOM_RANGE: (f32, f32) = (0.2, 10.0);
const GROUND: Color = Color::new(0.47, 0.78, 0.31, 1.0);

/// World coordinates are metres with the trunk base at the origin (+Y down).
const TRUNK_START: Vector2D = Vector2D { x: 0.0, y: 0.0 };

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

/// Camera target that puts the ground 50 px above the bottom edge at zoom 1×.
fn default_camera_target() -> Vec2 {
    vec2(0.0, -(SCREEN_HEIGHT / 2.0 - 50.0) / PX_PER_M)
}

/// How the tree is viewed and driven, as opposed to the tree itself.
struct View {
    camera_target: Vec2,
    zoom: f32,
    invert_pan: bool,
    show_leaves: bool,
    speed: f32,
    paused: bool,
    speed_slider: ui::Slider,
}

impl View {
    fn new() -> Self {
        View {
            camera_target: default_camera_target(),
            zoom: 1.0,
            invert_pan: true,
            show_leaves: true,
            speed: 1.0,
            paused: false,
            speed_slider: ui::Slider::default(),
        }
    }

    fn camera(&self) -> Camera2D {
        Camera2D {
            target: self.camera_target,
            zoom: vec2(
                PX_PER_M / (screen_width() / 2.0),
                PX_PER_M / (screen_height() / 2.0),
            ) * self.zoom,
            ..Default::default()
        }
    }
}

/// Ages (simulated hours) of each tip's leaf, for the unfold animation.
/// Drawing-only state, so it lives here rather than in the simulation.
fn update_leaf_ages(leaf_ages: &mut Vec<f32>, sim: &TreeSimulation, hours: f32, show: bool) {
    leaf_ages.resize(sim.growth_sections.len(), 0.0);
    for (age, section) in leaf_ages.iter_mut().zip(&sim.growth_sections) {
        let is_tip = matches!(&section.state, GrowthSectionState::FullyGrown { child_indices } if child_indices.is_empty());
        *age = if show && is_tip {
            (*age + hours).min(LEAF_GROW_DURATION)
        } else {
            0.0
        };
    }
}

fn draw_world(sim: &TreeSimulation, leaf_ages: &[f32], view: &View) {
    clear_background(SKYBLUE);
    set_camera(&view.camera());

    // Ground, far wider and deeper than any pan or zoom can reach (m).
    draw_rectangle(-1000.0, TRUNK_START.y, 2000.0, 1000.0, GROUND);

    for section in &sim.growth_sections {
        let (start, end) = (section.start_point, section.end_point());
        draw_line(start.x, start.y, end.x, end.y, section.thickness, DARKBROWN);
    }

    for (section, &age) in sim.growth_sections.iter().zip(leaf_ages) {
        if age <= 0.0 {
            continue;
        }
        // Quadratic ease-out: 1 - (1 - t)² for a smooth unfold.
        let progress = age / LEAF_GROW_DURATION;
        let scale = 1.0 - (1.0 - progress) * (1.0 - progress);
        let rx = sim.params.leaf_length / 2.0 * scale;
        let ry = sim.params.leaf_width / 2.0 * scale;
        let outline = rx / 20.0;
        let end = section.end_point();
        let angle = section.angle.to_degrees();
        draw_ellipse(end.x, end.y, rx + outline, ry + outline, angle, BLACK);
        draw_ellipse(end.x, end.y, rx, ry, angle, DARKGREEN);
    }

    set_default_camera();
}

/// Draws the overlay and returns whether the speed slider is being dragged.
fn draw_ui(sim: &TreeSimulation, view: &mut View) -> bool {
    draw_text(
        "Controls: [Scroll / +/-] Zoom | [Space] Play/Pause | [Click + Drag] Pan | [R] New Tree | [Shift+R] Regrow | [C] Center View",
        10.0,
        25.0,
        16.0,
        BLACK,
    );
    let years = (sim.hours as f32 / HOURS_PER_YEAR) as u32;
    let days = ((sim.hours as f32 % HOURS_PER_YEAR) / 24.0) as u32;
    draw_text(
        format!(
            "Age: {years} y {days} d | Sections: {} | Zoom: {:.1}x | Seed: {}",
            sim.growth_sections.len(),
            view.zoom,
            sim.seed
        ),
        10.0,
        50.0,
        16.0,
        BLACK,
    );

    if ui::toggle_button(
        Rect::new(640.0, 40.0, 140.0, 35.0),
        "Pan Drag",
        "INVERTED",
        "REGULAR",
        view.invert_pan,
    ) {
        view.invert_pan = !view.invert_pan;
    }
    if ui::toggle_button(
        Rect::new(640.0, 80.0, 140.0, 35.0),
        "Show Leaves",
        "TRUE",
        "FALSE",
        view.show_leaves,
    ) {
        view.show_leaves = !view.show_leaves;
    }
    let dragging = view.speed_slider.draw(
        Rect::new(640.0, 120.0, 140.0, 35.0),
        "Speed",
        &mut view.speed,
        0.1,
        5.0,
    );
    if ui::toggle_button(
        Rect::new(600.0, 120.0, 35.0, 35.0),
        "",
        ">>",
        "||",
        view.paused,
    ) {
        view.paused = !view.paused;
    }
    dragging
}

/// Keyboard and mouse. Returns `true` when the tree was reset.
fn handle_input(sim: &mut TreeSimulation, view: &mut View, slider_dragging: bool) -> bool {
    let mut reset = false;
    // R grows a new tree; Shift+R regrows the same seed.
    if is_key_pressed(KeyCode::R) {
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let seed = if shift { sim.seed } else { new_seed() };
        sim.reset(TRUNK_START, seed);
        reset = true;
    }
    if is_key_pressed(KeyCode::Space) {
        view.paused = !view.paused;
    }

    let mut zoom_by =
        |factor: f32| view.zoom = (view.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        zoom_by(1.2);
    }
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        zoom_by(1.0 / 1.2);
    }
    let (_, scroll_y) = mouse_wheel();
    if scroll_y != 0.0 {
        zoom_by(if scroll_y > 0.0 { 1.08 } else { 1.0 / 1.08 });
    }

    // Drag to pan. Mouse deltas are normalised to the window (2 units across each
    // axis), so each axis converts with its own screen dimension.
    if is_mouse_button_down(MouseButton::Left) && !slider_dragging {
        let delta = mouse_delta_position();
        let direction = if view.invert_pan { 1.0 } else { -1.0 };
        let metres_per_unit = |screen_px: f32| screen_px / 2.0 / view.zoom / PX_PER_M;
        view.camera_target.x += delta.x * metres_per_unit(screen_width()) * direction;
        view.camera_target.y += delta.y * metres_per_unit(screen_height()) * direction;
    }

    if is_key_pressed(KeyCode::C) {
        view.camera_target = default_camera_target();
        view.zoom = 1.0;
    }
    reset
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sim = TreeSimulation::new(TRUNK_START, new_seed());
    let mut view = View::new();
    let mut accumulator: f32 = 0.0;
    let mut leaf_ages: Vec<f32> = Vec::new();

    loop {
        // Fixed steps (see plan_steps): same speed on any display.
        let speed = if view.paused {
            0.0
        } else {
            view.speed * SPEED_SCALE
        };
        let (steps, next_accumulator) = plan_steps(accumulator, get_frame_time(), speed);
        accumulator = next_accumulator;
        for _ in 0..steps {
            sim.step();
        }
        update_leaf_ages(
            &mut leaf_ages,
            &sim,
            steps as f32 * SIM_DT,
            view.show_leaves,
        );

        draw_world(&sim, &leaf_ages, &view);
        let slider_dragging = draw_ui(&sim, &mut view);
        if handle_input(&mut sim, &mut view, slider_dragging) {
            accumulator = 0.0;
            leaf_ages.clear();
        }

        next_frame().await;
    }
}
