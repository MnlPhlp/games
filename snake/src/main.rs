#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use std::collections::VecDeque;

use egui::{Color32, Key, Pos2, Vec2};
use egui_game::utils::random_usize;
use egui_game::{Anchor, EguiGame};
use egui_game::{DrawContext, Game, UpdateContext};

/// time per tick in s
const START_TICK: f32 = 0.5;

#[derive(Default)]
struct Snake {
    segments: VecDeque<Pos2>,
    apple: Pos2,
    direction: Vec2,
    tick: f32,
    score: u32,
    grid_size: usize,
    elapsed: f32,
    collision: bool,
    highscore: u32,
}

impl Game for Snake {
    fn new(storage: Option<&dyn eframe::Storage>) -> Self {
        let highscore = storage
            .and_then(|s| s.get_string("highscore"))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        Self {
            segments: [Pos2::new(0.0, 0.0)].into(),
            apple: random_pos(20, &[Pos2::new(0.0, 0.0)]),
            direction: Vec2::new(1.0, 0.0),
            tick: START_TICK,
            grid_size: 20,
            highscore,
            ..Default::default()
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("highscore", self.highscore.to_string());
    }

    fn update(&mut self, ctx: &mut UpdateContext<Self>, delta: f32, _size: Vec2) {
        if self.collision {
            if ctx.key_pressed(Key::R) {
                self.reset();
            }
            return;
        }
        // handle input
        if ctx.key_pressed(Key::ArrowLeft) {
            self.direction = Vec2::new(-1.0, 0.0);
        } else if ctx.key_pressed(Key::ArrowRight) {
            self.direction = Vec2::new(1.0, 0.0);
        } else if ctx.key_pressed(Key::ArrowUp) {
            self.direction = Vec2::new(0.0, -1.0);
        } else if ctx.key_pressed(Key::ArrowDown) {
            self.direction = Vec2::new(0.0, 1.0);
        }

        // update snake position
        self.elapsed += delta;
        if self.elapsed < self.tick {
            return;
        }
        self.elapsed = 0.0;
        // move snake
        let new_head = self.segments[0] + self.direction;
        // check for collision with apple
        if new_head == self.apple {
            self.segments.push_front(self.apple);
            self.apple = random_pos(self.grid_size, self.segments.make_contiguous());
            self.score += 1;
            self.tick *= 0.9;
        } else {
            // check for collision with walls or snake
            if new_head.x < 0.0
                || new_head.x >= self.grid_size as f32
                || new_head.y < 0.0
                || new_head.y >= self.grid_size as f32
                || self.segments.contains(&new_head)
            {
                // game over
                self.collision = true;
                self.highscore = self.score.max(self.highscore);
                return;
            }
            // move snake
            self.segments.push_front(new_head);
            self.segments.pop_back();
        }
    }

    fn draw(&mut self, ctx: &mut DrawContext<'_>, size: Vec2) {
        let w = size.x / self.grid_size as f32;
        let h = size.y / self.grid_size as f32;

        let head_color = if self.collision {
            Color32::RED
        } else {
            Color32::WHITE
        };
        ctx.rect_filled(
            (self.segments[0].x * w, self.segments[0].y * h),
            (w, h),
            head_color,
        );
        for segment in self.segments.iter().skip(1) {
            ctx.rect_filled((segment.x * w, segment.y * h), (w, h), Color32::WHITE);
        }
        ctx.rect_filled((self.apple.x * w, self.apple.y * h), (w, h), Color32::GREEN);
        if self.collision {
            let rect = ctx
                .text_centered_anchor(
                    (size.x / 2.0, size.y / 2.0),
                    "Game Over!\n Press R to restart",
                    30.,
                    Color32::WHITE,
                    Anchor::TopCenter,
                )
                .rect();
            ctx.text_centered_anchor(
                (size.x / 2.0, rect.min.y + rect.height() + 10.0),
                format!("Score: {}\nHighscore: {}", self.score, self.highscore),
                30.,
                Color32::GREEN,
                Anchor::TopCenter,
            );
        } else {
            ctx.text(
                (size.x - 100.0, 10.0),
                format!("Score: {}", self.score),
                20.,
                Color32::WHITE,
            );
        }
    }

    fn reset(&mut self) {
        let highscore = self.highscore;
        *self = Self::new(None);
        self.highscore = highscore;
    }
}

fn random_pos(max: usize, segments: &[Pos2]) -> Pos2 {
    let mut pos = Pos2::new(random_usize(0..max) as f32, random_usize(0..max) as f32);
    // check if pos is in segments
    while segments.contains(&pos) {
        pos = Pos2::new(random_usize(0..max) as f32, random_usize(0..max) as f32);
    }
    pos
}

fn main() {
    EguiGame::new().run::<Snake>("Snake");
}
