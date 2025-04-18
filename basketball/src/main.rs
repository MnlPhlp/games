use egui::{Color32, Key, Pos2, Rect, Vec2, emath::Float};
use egui_game::{Anchor, DrawContext, EguiGame, Game, ImageFit, UpdateContext};
use geo::{Intersects, Line};

// all positions are in fractions of the screen size
#[derive(Clone, Default)]
struct Basketball {
    score: usize,
    ball_pos: Pos2,
    ball_speed: Vec2,
    hit: Option<f64>,
    pad_pos: Pos2,
}

const FLOOR_HEIGHT: f32 = 0.1;

const BALL_SIZE: f32 = 0.05;

const HOOP_POS: Pos2 = Pos2::new(0.85, 0.5);
const HOOP_SIZE: f32 = 0.1;

const ACCELERATION: f32 = 1.5;
const ELASTICITY: f32 = 0.9;
const MIN_SPEED: f32 = 0.15;

const HIT_DELAY: f64 = 1.0;

impl Game for Basketball {
    fn new(_storage: Option<&dyn eframe::Storage>) -> Self {
        Self {
            ball_pos: Pos2::new(0.5, 0.5),
            pad_pos: Pos2::new(0.3, 1. - FLOOR_HEIGHT),
            ..Default::default()
        }
    }

    fn draw(&mut self, ctx: &mut DrawContext, size: Vec2) {
        ctx.background(
            egui::include_image!("../assets/background.png"),
            ImageFit::Cover,
        );
        ctx.image_centered(
            (self.pad_pos.x * size.x, self.pad_pos.y * size.y),
            Vec2::new(BALL_SIZE * 4. * size.x, 0.),
            egui::include_image!("../assets/trampolin.png"),
        );
        ctx.image_centered(
            (self.ball_pos.x * size.x, self.ball_pos.y * size.y),
            (BALL_SIZE * size.x, BALL_SIZE * size.y),
            egui::include_image!("../assets/ball.png"),
        );
        ctx.image_anchor(
            (HOOP_POS.x * size.x, HOOP_POS.y * size.y),
            (HOOP_SIZE * size.x, HOOP_SIZE * size.y),
            egui::include_image!("../assets/hoop.png"),
            Anchor::TopCenter,
        );
        ctx.text(
            (10., 10.),
            format!("Score: {}", self.score),
            30.,
            Color32::BLACK,
        );

        if self.hit.is_some() {
            ctx.rect_filled((0., 0.), size, Color32::from_black_alpha(200));
            ctx.text_centered((size / 2.).to_pos2(), "You scored!", 50.0, Color32::WHITE);
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext<Self>, delta: f32, size: Vec2) {
        if let Some(hit) = self.hit {
            if ctx.time() - hit > HIT_DELAY {
                self.hit = None;
                self.ball_pos = Pos2::new(0.5, 0.5);
                self.ball_speed = Vec2::new(0.0, 0.0);
            }
        }

        // update ball position
        self.ball_pos += self.ball_speed * delta;
        // update ball speed
        if self.ball_pos.y < 1. - BALL_SIZE / 2. - FLOOR_HEIGHT {
            self.ball_speed += Vec2::new(0.0, ACCELERATION * delta);
        }
        // collisions
        if self.ball_pos.x - BALL_SIZE / 2. < 0.0 || self.ball_pos.x + BALL_SIZE / 2. > 1.0 {
            self.ball_speed.x *= -ELASTICITY;
            if self.ball_pos.x < 0.5 {
                self.ball_pos.x = BALL_SIZE / 2.;
            } else {
                self.ball_pos.x = 1.0 - BALL_SIZE / 2.;
            }
        }
        if self.ball_pos.y - BALL_SIZE / 2. < 0.0 {
            self.ball_speed.y *= -ELASTICITY;
            self.ball_pos.y = BALL_SIZE / 2.;
        }
        if self.ball_pos.y + BALL_SIZE / 2. > 1.0 - FLOOR_HEIGHT {
            if self.ball_pos.x > self.pad_pos.x - BALL_SIZE * 2.
                && self.ball_pos.x < self.pad_pos.x + BALL_SIZE * 2.
            {
                self.ball_speed.y = -self.ball_speed.y * ELASTICITY - 0.3;
                if ctx.key_down(Key::ArrowLeft) {
                    self.ball_speed.x -= 0.3;
                } else if ctx.key_down(Key::ArrowRight) {
                    self.ball_speed.x += 0.3;
                }
            } else {
                self.ball_speed.y = -self.ball_speed.y * ELASTICITY;
            }
            self.ball_pos.y = 1. - BALL_SIZE / 2. - FLOOR_HEIGHT;
        }
        if self.hit.is_none() {
            // scoring
            // only check when ball is falling
            if self.ball_speed.y > 0.0 {
                let ball_path = Line::new(
                    (
                        self.ball_pos.x - self.ball_speed.x * delta,
                        self.ball_pos.y - self.ball_speed.y * delta,
                    ),
                    (self.ball_pos.x, self.ball_pos.y),
                );
                let rim_path = Line::new(
                    (HOOP_POS.x - HOOP_SIZE / 2., HOOP_POS.y),
                    (HOOP_POS.x + HOOP_SIZE / 2., HOOP_POS.y),
                );

                if ball_path.intersects(&rim_path) {
                    self.hit = Some(ctx.time());
                    self.ball_speed.x *= 0.5;
                    self.score += 1;
                }
            }
            // input
            if ctx.key_down(Key::ArrowLeft) && self.pad_pos.x > 0.0 {
                self.pad_pos.x -= 1.0 * delta;
            }
            if ctx.key_down(Key::ArrowRight) && self.pad_pos.x < 1.0 {
                self.pad_pos.x += 1.0 * delta;
            }
        }
        #[cfg(debug_assertions)]
        if ctx.mouse_button_down(egui::PointerButton::Primary) {
            let p = ctx.mouse_position();
            self.ball_pos = Pos2::new(p.x / size.x, p.y / size.y);
            self.ball_speed = Vec2::new(0.0, 0.0);
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
