use macroquad::prelude::*;

/// Draws an interactive toggle button in screen space.
/// Returns `true` on the exact frame the button is clicked.
pub fn draw_toggle_button(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: &str,
    on_text: &str,
    off_text: &str,
    is_on: bool,
) -> bool {
    let (mx, my) = mouse_position();
    let is_hovered = mx >= x && mx <= x + width && my >= y && my <= y + height;

    let bg_color = if is_hovered {
        if is_on {
            Color::from_rgba(40, 160, 80, 255)
        } else {
            Color::from_rgba(100, 100, 100, 255)
        }
    } else {
        if is_on {
            Color::from_rgba(30, 130, 60, 255)
        } else {
            Color::from_rgba(70, 70, 70, 255)
        }
    };

    draw_rectangle(x, y, width, height, bg_color);
    draw_rectangle_lines(x, y, width, height, 2.0, DARKGRAY);

    let display_text = if label.is_empty() {
        if is_on {
            on_text.to_string()
        } else {
            off_text.to_string()
        }
    } else {
        format!("{}: {}", label, if is_on { on_text } else { off_text })
    };

    draw_text(&display_text, x + 10.0, y + height / 2.0 + 5.0, 14.0, WHITE);

    is_hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Draws an interactive slider.
/// Modifies the `value` directly when dragged.
pub fn draw_slider(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: &str,
    value: &mut f32,
    min_val: f32,
    max_val: f32,
) -> bool {
    let (mx, my) = mouse_position();
    let is_hovered = mx >= x && mx <= x + width && my >= y && my <= y + height;

    // Check if mouse button is held down this frame
    let is_down = is_mouse_button_down(MouseButton::Left);

    // To handle dragging smoothly even if the mouse moves outside the slider bounds:
    // We track interaction if it started here, or if it is currently hovered while down.
    // However, a simple immediate-mode check: if hovered or actively dragging.
    // Macroquad maps touches to mouse events by default, making this unified.
    static mut SLIDER_ACTIVE: bool = false;

    unsafe {
        if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
            SLIDER_ACTIVE = true;
        }
        if !is_down {
            SLIDER_ACTIVE = false;
        }

        if SLIDER_ACTIVE {
            let normalized_x = ((mx - x) / width).clamp(0.0, 1.0);
            *value = min_val + normalized_x * (max_val - min_val);
        }

        // Draw track background
        draw_rectangle(x, y, width, height, Color::from_rgba(70, 70, 70, 255));

        // Draw filled portion
        let fill_width = ((*value - min_val) / (max_val - min_val)) * width;
        draw_rectangle(x, y, fill_width, height, Color::from_rgba(40, 160, 80, 255));

        // Draw outline
        draw_rectangle_lines(x, y, width, height, 2.0, DARKGRAY);

        // Draw label and value
        let display_text = format!("{}: {:.1}x", label, value);
        draw_text(&display_text, x + 10.0, y + height / 2.0 + 5.0, 14.0, WHITE);

        SLIDER_ACTIVE
    }
}
