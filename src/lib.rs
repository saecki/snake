use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

use egui::epaint::Hsva;
use egui::{Align2, Color32, FontFamily, FontId, Id, Key, Rect, Stroke, Ui, Vec2};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

const START_LENGTH: usize = 3;
const BOARD_WIDTH: i16 = 64;
const BOARD_HEIGHT: i16 = 20;
const SCORE_COLOR: [(usize, Color32); 5] = [
    (5, Color32::from_rgb(110, 90, 220)),
    (10, Color32::from_rgb(90, 200, 120)),
    (20, Color32::from_rgb(250, 180, 80)),
    (30, Color32::from_rgb(220, 40, 40)),
    (50, Color32::from_rgb(240, 90, 200)),
];

#[derive(Default, Serialize, Deserialize)]
pub struct Snake {
    pub scores: Vec<usize>,
    #[serde(skip)]
    state: State,
}

pub fn snake_game(ui: &mut Ui, snake: &mut Snake) {
    ui.ctx().request_repaint();

    let now = SystemTime::now();
    let diff = now
        .duration_since(snake.state.last_update)
        .expect("Should be");

    ui.input(|i| {
        if i.key_pressed(Key::Space) {
            snake.state.paused = !snake.state.paused;
        }

        if !snake.state.paused {
            // arrow keys
            if i.key_pressed(Key::ArrowUp) {
                snake.steer(Direction::Up);
            } else if i.key_pressed(Key::ArrowRight) {
                snake.steer(Direction::Right);
            } else if i.key_pressed(Key::ArrowDown) {
                snake.steer(Direction::Down);
            } else if i.key_pressed(Key::ArrowLeft) {
                snake.steer(Direction::Left);
            }

            // wasd keys
            if i.key_pressed(Key::W) {
                snake.steer(Direction::Up);
            } else if i.key_pressed(Key::D) {
                snake.steer(Direction::Right);
            } else if i.key_pressed(Key::S) {
                snake.steer(Direction::Down);
            } else if i.key_pressed(Key::A) {
                snake.steer(Direction::Left);
            }

            // vim keys
            if i.key_pressed(Key::K) {
                snake.steer(Direction::Up);
            } else if i.key_pressed(Key::L) {
                snake.steer(Direction::Right);
            } else if i.key_pressed(Key::J) {
                snake.steer(Direction::Down);
            } else if i.key_pressed(Key::H) {
                snake.steer(Direction::Left);
            }
        }
    });

    if !snake.state.paused && diff >= snake.state.update_interval {
        snake.state.last_update = now;
        snake.update_state(ui);
    }

    snake.draw(ui);
}

struct State {
    paused: bool,
    direction: Direction,
    next_input: Option<(Direction, Option<Direction>)>,
    snake: VecDeque<Pos>,
    last_tail_pos: Pos,
    board: [[bool; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    last_update: SystemTime,
    update_interval: Duration,
    last_score: Option<usize>,
    tick: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            paused: true,
            direction: Direction::Right,
            next_input: None,
            snake: VecDeque::from_iter((0..START_LENGTH).rev().map(|i| Pos::new(2 + i as i16, 3))),
            last_tail_pos: Pos::new(1, 3),
            board: [[false; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
            last_update: SystemTime::UNIX_EPOCH,
            update_interval: Duration::from_millis(100),
            last_score: None,
            tick: 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Pos {
    x: i16,
    y: i16,
}

impl Pos {
    fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl Snake {
    fn steer(&mut self, new_dir: Direction) {
        match &mut self.state.next_input {
            None => {
                if self.state.direction != new_dir && self.state.direction != new_dir.opposite() {
                    self.state.next_input = Some((new_dir, None))
                }
            }
            Some((first, second)) => {
                if second.is_none() && *first != new_dir.opposite() {
                    *second = Some(new_dir);
                }
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        self.state.paused = !self.state.paused;
    }

    fn score(&self) -> usize {
        self.state.snake.len() - START_LENGTH
    }

    fn lost(&mut self, ui: &mut Ui) {
        let score = self.score();
        if score > 0 {
            self.scores.push(score);
            self.scores.sort_by(|a, b| b.cmp(a));
            self.scores.truncate(10);
        }
        self.state = State::default();
        self.state.last_score = Some(score);

        ui.ctx().clear_animations();
    }

    fn update_state(&mut self, ui: &mut Ui) {
        let score = self.score() as f32;
        let state = &mut self.state;

        if let Some((first, second)) = state.next_input {
            state.direction = first;
            state.next_input = second.map(|d| (d, None));
        }

        state.update_interval = Duration::from_millis((200.0 * (20.0 / (score + 20.0))) as u64);

        let old_head = state.snake[0];
        let new_head = match state.direction {
            Direction::Up => Pos::new(old_head.x, old_head.y - 1),
            Direction::Right => Pos::new(old_head.x + 1, old_head.y),
            Direction::Down => Pos::new(old_head.x, old_head.y + 1),
            Direction::Left => Pos::new(old_head.x - 1, old_head.y),
        };

        if !(0..BOARD_WIDTH).contains(&new_head.x) || !(0..BOARD_HEIGHT).contains(&new_head.y) {
            self.lost(ui);
            return;
        }

        state.last_tail_pos = *state.snake.back().unwrap();

        let eaten_apple = state.board[new_head.y as usize][new_head.x as usize];
        if eaten_apple {
            state.board[new_head.y as usize][new_head.x as usize] = false;
        } else {
            state.snake.pop_back();
        };

        if state.snake.contains(&new_head) {
            self.lost(ui);
            return;
        }

        state.snake.push_front(new_head);

        // place apple
        let apple_count = state.board.iter().flatten().filter(|f| **f).count();
        let mut rng = rand::thread_rng();
        if apple_count == 0
            || apple_count < 10 && rng.gen_bool(state.update_interval.as_secs_f64() / 3.0)
        {
            let mut options = Vec::new();
            for (y, row) in state.board.iter().enumerate() {
                for (x, &f) in row.iter().enumerate() {
                    if f {
                        continue;
                    }

                    let pos = Pos::new(x as i16, y as i16);
                    if !state.snake.contains(&pos) {
                        options.push(pos);
                    }
                }
            }

            if let Some(apple) = options.choose(&mut rng) {
                state.board[apple.y as usize][apple.x as usize] = true;
            }
        }

        state.tick += 1;
    }

    fn draw(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let field_size = {
            let field_width = available_size.x / BOARD_WIDTH as f32;
            let field_height = available_size.x / BOARD_HEIGHT as f32;
            field_width.min(field_height)
        };

        let board_size = Vec2::new(
            field_size * BOARD_WIDTH as f32,
            field_size * BOARD_HEIGHT as f32,
        );
        let board_pos = ((available_size - board_size) / 2.0).to_pos2();
        let board_rect = Rect::from_min_size(board_pos, board_size);

        ui.allocate_ui_at_rect(board_rect, |ui| {
            let score = self.score();
            let frac = {
                let time = SystemTime::now();
                let duration = time.duration_since(SystemTime::UNIX_EPOCH).expect("what");
                duration.subsec_millis() as f32 / 1000.0
            };

            let pos = ui.cursor().min;
            let board_rect = Rect::from_min_size(pos, board_size);
            let painter = ui.painter_at(board_rect);

            // board
            if score < 70 {
                painter.rect_filled(board_rect, 0.0, Color32::from_gray(0x12));
            } else {
                const TILE_FRACTIONS: i16 = 4;
                let tile_size = Vec2::splat(field_size / TILE_FRACTIONS as f32);
                for x in 0..BOARD_WIDTH * TILE_FRACTIONS {
                    for y in 0..BOARD_HEIGHT * TILE_FRACTIONS {
                        let hue = (2.0 * frac
                            + 0.001
                                * f32::sqrt((x as f32 + 10.0).powi(2) + (y as f32 + 10.0).powi(2)))
                            % 1.0;
                        let color = Hsva::new(hue, 0.9, 0.8, 1.0);

                        let tile_pos = pos + tile_size * Vec2::new(x as f32, y as f32);
                        let overlap = Vec2::splat(field_size / (TILE_FRACTIONS * 10) as f32);
                        let tile_rect =
                            Rect::from_min_size(tile_pos - overlap, tile_size + 2.0 * overlap);
                        painter.rect_filled(tile_rect, 0.0, color);
                    }
                }
            }

            // apples
            for (y, row) in self.state.board.iter().enumerate() {
                for (x, &f) in row.iter().enumerate() {
                    if f {
                        let apple_pos =
                            pos + field_size * Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        painter.circle_filled(
                            apple_pos,
                            0.4 * field_size,
                            Color32::from_rgb(0xff, 0x77, 0x33),
                        )
                    }
                }
            }

            // snake
            let interp = ui.ctx().animate_value_with_time(
                Id::new("snake"),
                self.state.tick as f32,
                self.state.update_interval.as_secs_f32(),
            ) - self.state.tick.saturating_sub(1) as f32;
            let color = SCORE_COLOR
                .iter()
                .find_map(|(s, color)| (score < *s).then_some(color));

            for (i, p) in self.state.snake.iter().enumerate() {
                let color = match color {
                    Some(c) => *c,
                    None => {
                        let hue = (frac + 0.01 * i as f32) % 1.0;
                        Hsva::new(hue, 0.9, 0.8, 1.0).into()
                    }
                };
                let new_pos = Vec2::new(p.x as f32, p.y as f32);
                if i == 0 {
                    // animated head
                    let p = self.state.snake[i + 1];
                    let last_pos = Vec2::new(p.x as f32, p.y as f32);
                    let tile_pos =
                        pos + field_size * (interp * new_pos + (1.0 - interp) * last_pos);
                    let tile_rect = Rect::from_min_size(tile_pos, Vec2::splat(field_size));
                    painter.rect(tile_rect, 0.0, color, Stroke::new(1.0, color));
                } else if i == self.state.snake.len() - 1 {
                    // animated tail
                    let p = self.state.last_tail_pos;
                    let last_pos = Vec2::new(p.x as f32, p.y as f32);
                    let tile_pos =
                        pos + field_size * (interp * new_pos + (1.0 - interp) * last_pos);
                    let tile_rect = Rect::from_min_size(tile_pos, Vec2::splat(field_size));
                    painter.rect(tile_rect, 0.0, color, Stroke::new(1.0, color));

                    // draw tail in new position so there is no gap
                    let tile_pos = pos + field_size * new_pos;
                    let tile_rect = Rect::from_min_size(tile_pos, Vec2::splat(field_size));
                    painter.rect(tile_rect, 0.0, color, Stroke::new(1.0, color));
                } else {
                    let tile_pos = pos + field_size * new_pos;
                    let tile_rect = Rect::from_min_size(tile_pos, Vec2::splat(field_size));
                    painter.rect(tile_rect, 0.0, color, Stroke::new(1.0, color));
                }
            }

            if self.state.paused {
                // pause
                let center_pos = pos + board_size / 2.0;
                let entire_pause_size = field_size * Vec2::new(2.4, 3.0);

                let pause_rect_width = entire_pause_size.x / 3.0;
                let pause_rect_size = Vec2::new(pause_rect_width, entire_pause_size.y);
                let left_rect_pos = center_pos - entire_pause_size / 2.0;
                let right_rect_pos = left_rect_pos + Vec2::new(2.0 * pause_rect_width, 0.0);

                painter.rect_filled(
                    Rect::from_min_size(left_rect_pos, pause_rect_size),
                    0.0,
                    Color32::from_rgba_unmultiplied(200, 200, 200, 40),
                );
                painter.rect_filled(
                    Rect::from_min_size(right_rect_pos, pause_rect_size),
                    0.0,
                    Color32::from_rgba_unmultiplied(200, 200, 200, 40),
                );

                // high scores
                if let Some(last) = self.state.last_score {
                    painter.text(
                        pos + Vec2::new((BOARD_WIDTH / 2) as f32 * field_size, 5.0 * field_size),
                        Align2::CENTER_CENTER,
                        format!("You scored {last}"),
                        FontId::new(1.4 * field_size, FontFamily::Proportional),
                        Color32::LIGHT_GRAY,
                    );
                }

                painter.text(
                    pos + Vec2::new(6.0 * field_size, 2.0 * field_size),
                    Align2::LEFT_TOP,
                    "High scores",
                    FontId::new(1.4 * field_size, FontFamily::Proportional),
                    Color32::LIGHT_GRAY,
                );
                for (i, score) in self.scores.iter().enumerate().take(10) {
                    painter.text(
                        pos + Vec2::new(
                            6.0 * field_size,
                            (i + 3) as f32 * 1.5 * field_size,
                        ),
                        Align2::LEFT_TOP,
                        score.to_string(),
                        FontId::new(1.4 * field_size, FontFamily::Proportional),
                        Color32::LIGHT_GRAY,
                    );
                }
            }

            // score
            painter.text(
                pos + Vec2::new((BOARD_WIDTH as f32 - 0.5) * field_size, 0.5 * field_size),
                Align2::RIGHT_TOP,
                self.score().to_string(),
                FontId::new(1.4 * field_size, FontFamily::Proportional),
                Color32::LIGHT_GRAY,
            );
        });
    }
}
