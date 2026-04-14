use std::io::{self, BufWriter, Stdout, Write};

use crossterm::{
    cursor,
    execute, queue,
    style::{Color as CtColor, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use nobiscuit_engine::framebuffer::{Color, Framebuffer};

#[derive(Default, PartialEq)]
pub struct HalfBlockCell {
    pub top: Color,
    pub bottom: Color,
}

pub struct TerminalRenderer {
    front: Vec<HalfBlockCell>,
    back: Vec<HalfBlockCell>,
    cols: usize,
    rows: usize,
    writer: BufWriter<Stdout>,
}

impl TerminalRenderer {
    pub fn new() -> Self {
        let mut writer = BufWriter::new(io::stdout());
        terminal::enable_raw_mode().expect("Failed to enable raw mode");
        execute!(writer, EnterAlternateScreen, cursor::Hide).expect("Failed to setup terminal");

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let cols = cols as usize;
        let rows = rows as usize;
        let size = cols * rows;

        Self {
            front: (0..size).map(|_| HalfBlockCell::default()).collect(),
            back: (0..size).map(|_| HalfBlockCell::default()).collect(),
            cols,
            rows,
            writer,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    pub fn resize(&mut self) {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let cols = cols as usize;
        let rows = rows as usize;

        if cols != self.cols || rows != self.rows {
            self.cols = cols;
            self.rows = rows;
            let size = cols * rows;
            self.front = (0..size).map(|_| HalfBlockCell::default()).collect();
            self.back = (0..size).map(|_| HalfBlockCell::default()).collect();
        }
    }

    pub fn present(&mut self, fb: &Framebuffer) {
        for r in 0..self.rows {
            let pixel_y_top = r * 2;
            let pixel_y_bottom = r * 2 + 1;

            for c in 0..self.cols {
                let top_color = fb.get_pixel(c, pixel_y_top);
                let bottom_color = fb.get_pixel(c, pixel_y_bottom);

                let cell = HalfBlockCell {
                    top: top_color,
                    bottom: bottom_color,
                };

                let idx = r * self.cols + c;
                if cell != self.front[idx] {
                    let _ = queue!(
                        self.writer,
                        cursor::MoveTo(c as u16, r as u16),
                        SetForegroundColor(CtColor::Rgb {
                            r: top_color.r,
                            g: top_color.g,
                            b: top_color.b,
                        }),
                        SetBackgroundColor(CtColor::Rgb {
                            r: bottom_color.r,
                            g: bottom_color.g,
                            b: bottom_color.b,
                        }),
                        Print("▀")
                    );
                }

                self.back[idx] = cell;
            }
        }

        let _ = self.writer.flush();
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn cleanup(&mut self) {
        let _ = execute!(
            self.writer,
            cursor::Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
