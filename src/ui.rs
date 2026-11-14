use macroquad::prelude::*;

const BUTTON_ON: Color = Color::new(0.12, 0.51, 0.24, 1.0);
const BUTTON_ON_HOVER: Color = Color::new(0.16, 0.63, 0.31, 1.0);
const BUTTON_OFF: Color = Color::new(0.27, 0.27, 0.27, 1.0);
const BUTTON_OFF_HOVER: Color = Color::new(0.39, 0.39, 0.39, 1.0);

fn is_hovered(rect: Rect) -> bool {
    rect.contains(mouse_position().into())
}

fn draw_label(rect: Rect, text: &str) {
    draw_text(
        text,
        rect.x + 10.0,
        rect.y + rect.h / 2.0 + 5.0,
        14.0,
        WHITE,
    );
}

/// Draws a toggle button in screen space and returns `true` on the frame it's
/// clicked. Shows `label: on_text/off_text`, or just the state text if `label` is empty.
pub fn toggle_button(rect: Rect, label: &str, on_text: &str, off_text: &str, is_on: bool) -> bool {
    let hovered = is_hovered(rect);
    let colour = match (is_on, hovered) {
        (true, true) => BUTTON_ON_HOVER,
        (true, false) => BUTTON_ON,
        (false, true) => BUTTON_OFF_HOVER,
        (false, false) => BUTTON_OFF,
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, colour);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, DARKGRAY);

    let state = if is_on { on_text } else { off_text };
    let text = if label.is_empty() {
        state.to_string()
    } else {
        format!("{label}: {state}")
    };
    draw_label(rect, &text);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// A slider's drag state, owned by the caller so each slider tracks its own drag.
#[derive(Default)]
pub struct Slider {
    dragging: bool,
}

impl Slider {
    /// Draws the slider and updates `value` while it's dragged. Returns `true`
    /// while dragging, so the caller can ignore the same mouse drag elsewhere.
    pub fn draw(&mut self, rect: Rect, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
        if is_hovered(rect) && is_mouse_button_pressed(MouseButton::Left) {
            self.dragging = true;
        }
        if !is_mouse_button_down(MouseButton::Left) {
            self.dragging = false;
        }
        if self.dragging {
            let fraction = ((mouse_position().0 - rect.x) / rect.w).clamp(0.0, 1.0);
            *value = min + fraction * (max - min);
        }

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, BUTTON_OFF);
        let fill = (*value - min) / (max - min) * rect.w;
        draw_rectangle(rect.x, rect.y, fill, rect.h, BUTTON_ON_HOVER);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, DARKGRAY);
        draw_label(rect, &format!("{label}: {value:.1}x"));

        self.dragging
    }
}
