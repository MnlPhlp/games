#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use egui::{Color32, Key, PointerButton, Pos2, Rect, Vec2};
use egui_game::EguiGame;
use egui_game::{
    DrawContext, Game, UpdateContext,
    utils::{get_file_as_string, run_future, write_file_from_string},
};
use log::info;

enum GridMode {
    Lines,
    Shaded,
    None,
}

const START_SIZE: usize = 40;

struct GameOfLife {
    rows: usize,
    cols: usize,
    cells: Vec<bool>,
    next_cells: Vec<bool>,
    reset_cells: Vec<bool>,
    step_time: f32,
    last_step_time: f32,
    time_elapsed: f32,
    drawing_mode: bool,
    grid_mode: GridMode,
    paused: bool,
    grid_rect: Rect,
}
impl Default for GameOfLife {
    fn default() -> Self {
        let mut state = Self {
            rows: START_SIZE,
            cols: START_SIZE,
            cells: vec![false; START_SIZE * START_SIZE],
            next_cells: vec![false; START_SIZE * START_SIZE],
            reset_cells: vec![],
            step_time: 0.5,
            last_step_time: 0.5,
            time_elapsed: 0.0,
            drawing_mode: false,
            grid_mode: GridMode::Lines,
            paused: false,
            grid_rect: Rect::ZERO,
        };
        state.spawn_glider();
        state.reset_cells = state.cells.clone();
        state
    }
}

impl Game for GameOfLife {
    fn new(_storage: Option<&dyn eframe::Storage>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &mut UpdateContext<Self>, delta: f32, _size: Vec2) {
        self.handle_input(ctx);
        self.time_elapsed += delta;
        if self.drawing_mode || self.time_elapsed < self.step_time || self.paused {
            return;
        }
        self.last_step_time = self.time_elapsed;
        self.time_elapsed = 0.0;
        self.update_cells();
    }

    fn draw(&mut self, ctx: &mut DrawContext<'_>, _size: Vec2) {
        let line_1 = "Space: draw, R: reset,  Up/Down: delay, Left/Right: size, G: grid mode";
        let line_2 = if self.drawing_mode {
            "drawing mode. press Space to continue O: open file, S: save to file".to_string()
        } else if self.paused {
            "Paused, P to continue, S to step".to_string()
        } else {
            format!(
                "Delay Target: {:.1}s, Delay: {:.2}s press P to pause and step",
                self.step_time, self.last_step_time
            )
        };
        let text_rect = ctx.text((5., 5.), format!("{line_1}\n{line_2}"), 20., Color32::WHITE);

        let line_thickness = if matches!(self.grid_mode, GridMode::Lines) {
            2.0
        } else {
            0.0
        };
        self.grid_rect =
            ctx.sub_square_margin(text_rect.max.y + 10., Some(Color32::WHITE), |ctx, size| {
                let (w, h) = (size.x, size.y);
                let cw = w / self.cols as f32;
                let ch = h / self.rows as f32;
                let offset = line_thickness / 2.0;

                for row in 0..self.rows {
                    let y = row as f32 * ch;
                    if matches!(self.grid_mode, GridMode::Lines) && row > 0 {
                        ctx.line((0.0, y), (w, y), line_thickness, Color32::WHITE);
                    }
                    for col in 0..self.cols {
                        let x = col as f32 * cw;
                        if matches!(self.grid_mode, GridMode::Lines) && col > 0 && row == 0 {
                            ctx.line((x, 0.0), (x, h), line_thickness, Color32::WHITE);
                        }
                        let cell_color = if self.cells[self.get_index(col, row)] {
                            Color32::GREEN
                        } else if matches!(self.grid_mode, GridMode::Shaded) {
                            if row % 2 == col % 2 {
                                Color32::GRAY
                            } else {
                                Color32::DARK_GRAY
                            }
                        } else {
                            Color32::WHITE
                        };
                        if cell_color != Color32::WHITE {
                            ctx.rect_filled(
                                (x + offset, y + offset),
                                (cw - offset * 2.0, ch - offset * 2.0),
                                cell_color,
                            );
                        }
                    }
                }
            });
    }

    fn reset(&mut self) {
        self.cells.clone_from(&self.reset_cells);
        self.time_elapsed = 0.0;
    }
}

impl GameOfLife {
    fn update_cells(&mut self) {
        // Rules:
        // A cell keeps its state if it has two neighbors.
        // A cell becomes active if it has three neighbors.
        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut neighbors = 0;
                for n_row in row.saturating_sub(1)..=(row + 1).min(self.rows - 1) {
                    for n_col in col.saturating_sub(1)..=(col + 1).min(self.cols - 1) {
                        // skip self
                        if n_col == col && n_row == row {
                            continue;
                        }
                        // check neighbor
                        if self.cells[n_row * self.cols + n_col] {
                            neighbors += 1;
                        }
                    }
                }
                // apply rules
                if neighbors == 2 {
                    // A cell keeps its state if it has two neighbors.
                    self.next_cells[row * self.cols + col] = self.cells[row * self.cols + col];
                } else if neighbors == 3 {
                    // A cell becomes active if it has three neighbors.
                    self.next_cells[row * self.cols + col] = true;
                } else {
                    self.next_cells[row * self.cols + col] = false;
                }
            }
        }
        // swap cells
        std::mem::swap(&mut self.cells, &mut self.next_cells);
    }

    fn spawn_glider(&mut self) {
        // spawn glider in top left corner
        for (x, y) in [(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)] {
            let idx = self.get_index(x, y);
            if idx >= self.cells.len() {
                continue;
            }
            self.cells[idx] = true;
        }
    }

    fn resize(&mut self, rows: usize, cols: usize) {
        if rows < 1 || cols < 1 {
            return;
        }
        std::mem::swap(&mut self.cells, &mut self.next_cells);
        self.cells.resize(rows * cols, false);
        self.cells.fill(false);
        // map cells to new indices
        for row in 0..rows {
            for col in 0..cols {
                if col >= self.cols || row >= self.rows {
                    continue;
                }
                let old_index = row * self.cols + col;
                let new_index = row * cols + col;
                if new_index < self.cells.len() {
                    self.cells[new_index] = self.next_cells[old_index];
                }
            }
        }
        std::mem::swap(&mut self.reset_cells, &mut self.next_cells);
        self.reset_cells.resize(rows * cols, false);
        self.reset_cells.fill(false);
        // map cells to new indices
        for row in 0..rows {
            for col in 0..cols {
                if col >= self.cols || row >= self.rows {
                    continue;
                }
                let old_index = row * self.cols + col;
                let new_index = row * cols + col;
                if new_index < self.cells.len() {
                    self.reset_cells[new_index] = self.next_cells[old_index];
                }
            }
        }
        self.next_cells.resize(rows * cols, false);
        self.rows = rows;
        self.cols = cols;
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }

    fn handle_input(&mut self, ctx: &mut UpdateContext<'_, GameOfLife>) {
        if ctx.key_pressed(Key::Space) {
            if self.drawing_mode {
                // save drawing for reset
                self.reset_cells.clone_from(&self.cells);
                info!("Saved drawing");
            } else {
                self.cells.fill(false);
            }
            self.drawing_mode = !self.drawing_mode;
        }
        if ctx.key_pressed(Key::R) {
            self.reset();
        }
        if ctx.key_pressed(Key::ArrowUp) {
            self.step_time = (self.step_time + 0.1).min(2.0);
        }
        if ctx.key_pressed(Key::ArrowDown) {
            self.step_time = (self.step_time - 0.1).max(0.0);
        }
        if ctx.key_pressed(Key::ArrowLeft) {
            self.resize(self.rows - 1, self.cols - 1);
        }
        if ctx.key_pressed(Key::ArrowRight) {
            self.resize(self.rows + 1, self.cols + 1);
        }
        if ctx.key_pressed(Key::G) {
            self.grid_mode = match self.grid_mode {
                GridMode::Lines => GridMode::Shaded,
                GridMode::Shaded => GridMode::None,
                GridMode::None => GridMode::Lines,
            };
        }
        if !self.drawing_mode && ctx.key_pressed(Key::P) {
            self.paused = !self.paused;
        }
        if self.paused && ctx.key_pressed(Key::S) {
            // do a single step
            self.update_cells();
        }
        if self.drawing_mode {
            if ctx.key_pressed(Key::O) {
                ctx.launch_async_update(get_file_as_string(), |game, text| {
                    game.load_from_text(&text);
                });
            }
            if ctx.key_pressed(Key::S) {
                let text = self.save_to_text();
                run_future(write_file_from_string(text));
            }
            if ctx.mouse_button_pressed(PointerButton::Primary) {
                let mouse_pos = {
                    let Pos2 { x, y } = ctx.mouse_position();
                    // convert to grid coordinates
                    (x - self.grid_rect.min.x, y - self.grid_rect.min.y)
                };

                let (w, h) = (self.grid_rect.width(), self.grid_rect.height());
                let cw = w / self.cols as f32;
                let ch = h / self.rows as f32;
                let x = (mouse_pos.0 / cw).floor() as usize;
                let y = (mouse_pos.1 / ch).floor() as usize;
                let index = self.get_index(x, y);
                if index < self.cells.len() {
                    self.cells[index] = !self.cells[index];
                }
            }
        }
    }

    fn load_from_text(&mut self, text: &str) {
        for line in text.lines() {
            if line.starts_with("//") || line.is_empty() {
                continue;
            }
            let (Ok(x), Ok(y)) = ({
                let (x, y) = line.split_once(' ').unwrap();
                let x = x.parse();
                let y = y.parse();
                (x, y)
            }) else {
                println!("Invalid line: {line}");
                continue;
            };
            let index = self.get_index(x, y);
            if index < self.cells.len() {
                self.cells[index] = true;
            }
        }
    }

    fn save_to_text(&self) -> String {
        let mut text = String::new();
        for (i, cell) in self.cells.iter().enumerate() {
            if *cell {
                let x = i % self.cols;
                let y = i / self.cols;
                text.push_str(&format!("{x} {y}\n"));
            }
        }
        text
    }
}

fn main() {
    EguiGame::new().run::<GameOfLife>("Convay's Game of Life");
}
