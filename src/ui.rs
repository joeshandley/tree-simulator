use macroquad::prelude::*;

/// Draws an interactive toggle button in screen space.
/// Returns `true` on the exact frame the button is clicked.
pub fn draw_toggle_button(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: &str,
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

    let display_text = format!("{}: {}", label, if is_on { "INVERTED" } else { "REGULAR" });
    draw_text(&display_text, x + 10.0, y + height / 2.0 + 5.0, 14.0, WHITE);

    is_hovered && is_mouse_button_pressed(MouseButton::Left)
}
