use rand::Rng;
use termray::{Color, Framebuffer};

const BAR_WIDTH: usize = 20;
const BAR_HEIGHT: usize = 4;
const BAR_MARGIN: usize = 4;

/// Render hunger bar at top-left of framebuffer
pub fn render_hunger_bar(fb: &mut Framebuffer, hunger: f64) {
    let x0 = BAR_MARGIN;
    let y0 = BAR_MARGIN;

    // Background (dark gray)
    let bg = Color::rgb(40, 40, 40);
    for y in y0..y0 + BAR_HEIGHT {
        for x in x0..x0 + BAR_WIDTH {
            fb.set_pixel(x, y, bg);
        }
    }

    // Fill (green → yellow → red based on hunger level)
    let fill_width = (hunger.clamp(0.0, 1.0) * BAR_WIDTH as f64) as usize;
    let fill_color = if hunger > 0.6 {
        Color::rgb(80, 200, 80) // green
    } else if hunger > 0.3 {
        Color::rgb(220, 200, 40) // yellow
    } else {
        Color::rgb(220, 50, 50) // red
    };

    for y in y0..y0 + BAR_HEIGHT {
        for x in x0..x0 + fill_width {
            fb.set_pixel(x, y, fill_color);
        }
    }

    // Border (white outline)
    let border = Color::rgb(180, 180, 180);
    for x in x0..x0 + BAR_WIDTH {
        fb.set_pixel(x, y0, border);
        fb.set_pixel(x, y0 + BAR_HEIGHT - 1, border);
    }
    for y in y0..y0 + BAR_HEIGHT {
        fb.set_pixel(x0, y, border);
        fb.set_pixel(x0 + BAR_WIDTH - 1, y, border);
    }
}

/// Render floor indicator below the hunger bar (e.g. "2F")
pub fn render_floor_indicator(fb: &mut Framebuffer, current_floor: usize, total_floors: usize) {
    let text = format!("{}F", current_floor);
    let char_w = 4;
    let x0 = BAR_MARGIN;
    let y0 = BAR_MARGIN + BAR_HEIGHT + 2;

    // Dim color for floor number, brighter if not ground floor
    let color = if current_floor > 1 {
        Color::rgb(200, 180, 100)
    } else {
        Color::rgb(140, 140, 140)
    };

    for (ci, ch) in text.chars().enumerate() {
        let bitmap = char_bitmap(ch);
        let cx = x0 + ci * char_w;
        for (row, bits) in bitmap.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    let px = cx + col;
                    let py = y0 + row;
                    if px < fb.width() && py < fb.height() {
                        fb.set_pixel(px, py, color);
                    }
                }
            }
        }
    }

    // Small dots showing total floors (bottom = 1F, top = top floor, like elevator)
    let dot_x = x0 + text.len() * char_w + 2;
    for f in 0..total_floors {
        let dot_y = y0 + (total_floors - 1 - f) * 3;
        let dot_color = if f + 1 == current_floor {
            Color::rgb(255, 200, 50)
        } else {
            Color::rgb(80, 80, 80)
        };
        if dot_x < fb.width() && dot_y < fb.height() {
            fb.set_pixel(dot_x, dot_y, dot_color);
            if dot_x + 1 < fb.width() {
                fb.set_pixel(dot_x + 1, dot_y, dot_color);
            }
        }
    }
}

/// Render a text message centered near bottom of framebuffer
/// Each character is rendered as a 3x5 pixel block
pub fn render_message(fb: &mut Framebuffer, text: &str, color: Color) {
    let char_w = 4; // 3 pixels + 1 gap
    let char_h = 6; // 5 pixels + 1 gap
    let total_w = text.len() * char_w;
    let x0 = fb.width().saturating_sub(total_w) / 2;
    let y0 = fb.height().saturating_sub(char_h + 8);

    // Background strip
    let bg = Color::rgb(0, 0, 0);
    for y in y0.saturating_sub(2)..=(y0 + char_h).min(fb.height() - 1) {
        for x in x0.saturating_sub(4)..(x0 + total_w + 4).min(fb.width()) {
            fb.set_pixel(x, y, bg);
        }
    }

    // Simple 3x5 bitmap font for basic ASCII
    for (ci, ch) in text.chars().enumerate() {
        let bitmap = char_bitmap(ch);
        let cx = x0 + ci * char_w;
        for (row, bits) in bitmap.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    fb.set_pixel(cx + col, y0 + row, color);
                }
            }
        }
    }
}

/// Render game over result screen
pub fn render_game_over_screen(fb: &mut Framebuffer, timer: f64) {
    let color = Color::rgb(255, 80, 80);
    render_centered_text(fb, "You can no longer move...", color, fb.height() / 2 - 4);
    if timer >= 2.0 {
        render_retry_prompt(fb);
    }
}

/// Render clear (escape) result screen with staged title reveal
pub fn render_clear_screen(
    fb: &mut Framebuffer,
    timer: f64,
    biscuits_eaten: u32,
    elapsed_time: f64,
    floors_visited: usize,
) {
    let text_color = Color::rgb(200, 255, 200);
    let center_y = fb.height() / 2 - 10;

    if timer < 1.5 {
        render_centered_text(fb, "no biscuit...", text_color, center_y);
    } else if timer < 3.0 {
        render_centered_text(fb, "...nobiscuit...", text_color, center_y);
    } else if timer < 4.5 {
        render_centered_text(fb, "...nobisuke.", Color::rgb(255, 255, 200), center_y);
    } else {
        render_centered_text(fb, "...nobisuke.", Color::rgb(255, 255, 200), center_y);
        // Score display
        let score_y = center_y + 10;
        let score_color = Color::rgb(180, 180, 180);
        let biscuit_text = format!("Biscuits  {}", biscuits_eaten);
        render_centered_text(fb, &biscuit_text, score_color, score_y);

        let mins = elapsed_time as u32 / 60;
        let secs = elapsed_time as u32 % 60;
        let time_text = format!("Survived  {}m {}s", mins, secs);
        render_centered_text(fb, &time_text, score_color, score_y + 8);

        let floor_text = format!("Floors  {}", floors_visited);
        render_centered_text(fb, &floor_text, score_color, score_y + 16);
    }

    if timer >= 6.0 {
        render_retry_prompt(fb);
    }
}

/// Render "[Y] Retry  [N] Quit" prompt
fn render_retry_prompt(fb: &mut Framebuffer) {
    let color = Color::rgb(160, 160, 160);
    let y = fb.height() - 12;
    render_centered_text(fb, "[Y] Retry  [N] Quit", color, y);
}

/// Helper: render text centered horizontally at a given y position
fn render_centered_text(fb: &mut Framebuffer, text: &str, color: Color, y0: usize) {
    let char_w = 4;
    let total_w = text.len() * char_w;
    let x0 = fb.width().saturating_sub(total_w) / 2;

    for (ci, ch) in text.chars().enumerate() {
        let bitmap = char_bitmap(ch);
        let cx = x0 + ci * char_w;
        for (row, bits) in bitmap.iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    let px = cx + col;
                    let py = y0 + row;
                    if px < fb.width() && py < fb.height() {
                        fb.set_pixel(px, py, color);
                    }
                }
            }
        }
    }
}

/// Render the galagala (lottery machine) opening screen
pub fn render_garagara_screen(fb: &mut Framebuffer, spins: u32, shake_timer: f64) {
    fb.clear(Color::rgb(0, 0, 0));

    // Draw spin count as large 3x-scaled digits in the center
    let text = format!("{}", spins);
    let scale = 3_usize;
    let char_w = 4 * scale; // 3px * scale + scale gap
    let char_h = 5 * scale;
    let total_w = text.len() * char_w;
    let x0 = fb.width().saturating_sub(total_w) / 2;
    let y0 = fb.height().saturating_sub(char_h) / 2;

    let digit_color = if spins == 0 {
        Color::rgb(120, 120, 120)
    } else if spins <= 4 {
        Color::rgb(200, 200, 200)
    } else if spins <= 10 {
        Color::rgb(255, 200, 50)
    } else {
        Color::rgb(255, 80, 80)
    };

    for (ci, ch) in text.chars().enumerate() {
        let bitmap = char_bitmap(ch);
        let cx = x0 + ci * char_w;
        for (row, bits) in bitmap.iter().enumerate() {
            for col in 0..3_usize {
                if bits & (1 << (2 - col)) != 0 {
                    // Draw a scale x scale block for each pixel
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + col * scale + sx;
                            let py = y0 + row * scale + sy;
                            if px < fb.width() && py < fb.height() {
                                fb.set_pixel(px, py, digit_color);
                            }
                        }
                    }
                }
            }
        }
    }

    // Instruction text at the bottom
    let prompt = if spins == 0 {
        "Press any key to spin"
    } else {
        "Press any key to spin / Enter to start"
    };
    let prompt_color = Color::rgb(160, 160, 160);
    let prompt_y = fb.height().saturating_sub(12);
    render_centered_text(fb, prompt, prompt_color, prompt_y);

    // Camera shake: shift pixels randomly by ±2 when shake_timer > 0
    if shake_timer > 0.0 {
        let mut rng = rand::thread_rng();
        let dx: i32 = rng.gen_range(-2..=2);
        let dy: i32 = rng.gen_range(-2..=2);
        shift_framebuffer(fb, dx, dy);
    }
}

/// Shift all pixels in the framebuffer by (dx, dy), filling gaps with black.
// TODO: reuse temp buffer instead of allocating every frame (called ~9 times per shake)
fn shift_framebuffer(fb: &mut Framebuffer, dx: i32, dy: i32) {
    let w = fb.width();
    let h = fb.height();
    // Read all pixels into a temp buffer
    let mut tmp = vec![Color::rgb(0, 0, 0); w * h];
    for y in 0..h {
        for x in 0..w {
            let sx = x as i32 - dx;
            let sy = y as i32 - dy;
            if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                tmp[y * w + x] = fb.get_pixel(sx as usize, sy as usize);
            }
        }
    }
    // Write back
    for y in 0..h {
        for x in 0..w {
            fb.set_pixel(x, y, tmp[y * w + x]);
        }
    }
}

/// 3x5 bitmap font — returns 5 rows, each row is 3 bits (MSB = left)
fn char_bitmap(c: char) -> [u8; 5] {
    match c.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b110, 0b100, 0b110, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'V' => [0b101, 0b101, 0b101, 0b010, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b010, 0b101, 0b101, 0b101, 0b010],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b010, 0b101, 0b010, 0b101, 0b010],
        '9' => [0b010, 0b101, 0b011, 0b001, 0b110],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => [0b010, 0b101, 0b010, 0b000, 0b010],
        '*' => [0b000, 0b101, 0b010, 0b101, 0b000],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '[' => [0b110, 0b100, 0b100, 0b100, 0b110],
        ']' => [0b011, 0b001, 0b001, 0b001, 0b011],
        ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        _ => [0b111, 0b111, 0b111, 0b111, 0b111], // unknown = filled block
    }
}
