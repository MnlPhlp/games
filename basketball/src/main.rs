use egui::{Color32, Key, Pos2, Rect, Vec2, emath::Float};
use egui_game::{Anchor, DrawContext, EguiGame, Game, ImageFit, UpdateContext};

// all positions are in fractions of the screen size
#[derive(Clone, Default)]
struct Basketball {
    score: usize,
    ball_pos: Pos2,
    ball_speed: Vec2,
    hit: Option<f64>,
    show_hit: bool,
}

const FLOOR_HEIGHT: f32 = 0.1;

const BALL_SIZE: f32 = 0.05;

const HOOP_POS: Pos2 = Pos2::new(0.85, 0.5);
const HOOP_WIDTH: f32 = 0.1;

const ACCELERATION: f32 = 0.6;
const ELASTICITY: f32 = 0.8;
const MIN_SPEED: f32 = 0.05;

const HIT_DELAY: f64 = 0.5;

impl Game for Basketball {
    fn new(_storage: Option<&dyn eframe::Storage>) -> Self {
        Self {
            ball_pos: Pos2::new(0.5, 0.5),
            ..Default::default()
        }
    }

    fn draw(&mut self, ctx: &mut DrawContext, size: Vec2) {
        ctx.background(
            egui::include_image!("../assets/background.png"),
            ImageFit::Cover,
        );
        ctx.image_centered(
            (self.ball_pos.x * size.x, self.ball_pos.y * size.y),
            (BALL_SIZE * size.y, BALL_SIZE * size.y),
            egui::include_image!("../assets/ball.png"),
        );
        ctx.image_anchor(
            (HOOP_POS.x * size.x, HOOP_POS.y * size.y),
            (HOOP_WIDTH * size.y, HOOP_WIDTH * size.y),
            egui::include_image!("../assets/hoop.png"),
            Anchor::TopCenter,
        );

        if self.show_hit {
            ctx.rect_filled((0., 0.), size, Color32::from_black_alpha(200));
            ctx.text_centered(
                (size / 2.).to_pos2(),
                "You scored!\nSpace to continue",
                50.0,
                Color32::WHITE,
            );
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext<Self>, delta: f32) {
        if let Some(hit) = self.hit {
            if ctx.time() - hit > HIT_DELAY {
                self.hit = None;
                self.show_hit = true;
                self.score += 1;
            }
        }

        if self.show_hit {
            if ctx.key_pressed(Key::Space) {
                self.show_hit = false;
                self.ball_pos = Pos2::new(0.5, 0.5);
                self.ball_speed = Vec2::new(0.0, 0.0);
            }
            return;
        }

        // update ball position
        self.ball_pos += self.ball_speed * delta;
        // update ball speed
        if self.ball_pos.y < 1. - BALL_SIZE / 2. - FLOOR_HEIGHT {
            self.ball_speed += Vec2::new(0.0, ACCELERATION * delta);
        }
        // collisions
        if self.ball_pos.x - BALL_SIZE / 2. < 0.0 || self.ball_pos.x + BALL_SIZE / 2. > 1.0 {
            self.ball_speed.x = if self.ball_speed.x.abs() > MIN_SPEED {
                -self.ball_speed.x * ELASTICITY
            } else {
                0.0
            };
            if self.ball_pos.x < 0.5 {
                self.ball_pos.x = BALL_SIZE / 2.;
            } else {
                self.ball_pos.x = 1.0 - BALL_SIZE / 2.;
            }
        }
        if self.ball_pos.y - BALL_SIZE / 2. < 0.0
            || self.ball_pos.y + BALL_SIZE / 2. > 1.0 - FLOOR_HEIGHT
        {
            self.ball_speed.y = if self.ball_speed.y.abs() > MIN_SPEED {
                -self.ball_speed.y * ELASTICITY
            } else {
                0.0
            };
            if self.ball_pos.y < 0.5 {
                self.ball_pos.y = BALL_SIZE / 2.;
            } else {
                self.ball_pos.y = 1.0 - BALL_SIZE / 2. - FLOOR_HEIGHT;
            }
        }
        // scoring
        if Rect::from_center_size(self.ball_pos, Vec2::new(0.01, 0.01)).intersects(
            Rect::from_center_size(
                HOOP_POS + Vec2::new(0.01, 0.),
                Vec2::new(HOOP_WIDTH - 0.02, 0.0),
            ),
        ) {
            self.hit = Some(ctx.time());
        }
        // input
        if ctx.key_pressed(Key::S) {
            self.ball_pos = Pos2::new(0.85, 0.3);
        }
    }

    fn reset(&mut self) {
        // Reset your game state here
        *self = Self::new(None);
    }
}

fn main() {
    EguiGame::new().run::<Basketball>("basketball");
}
