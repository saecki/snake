use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

use eframe::{App, NativeOptions};
use egui::{CentralPanel, Color32, Frame, Key, Pos2, Rect, Stroke, Ui, Vec2};
use rand::seq::SliceRandom;
use rand::Rng;

const BOARD_SIZE: i16 = 30;

fn main() {
    eframe::run_native(
        "snake",
        NativeOptions::default(),
        Box::new(|_| Box::new(SnakeApp::new())),
    )
}

struct SnakeApp {
    paused: bool,
    direction: Direction,
    next_input: Option<Direction>,
    snake: VecDeque<Pos>,
    board: [[bool; BOARD_SIZE as usize]; BOARD_SIZE as usize],
    last_update: SystemTime,
    update_interval: Duration,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: i16,
    y: i16,
}

impl Pos {
    fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl App for SnakeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let now = SystemTime::now();
        let diff = now.duration_since(self.last_update).expect("Should be");

        if ctx.input().key_pressed(Key::Space) {
            self.paused = !self.paused;
        }

        if !self.paused {
            // arrow keys
            if ctx.input().key_pressed(Key::ArrowUp) {
                self.up();
            } else if ctx.input().key_pressed(Key::ArrowRight) {
                self.right();
            } else if ctx.input().key_pressed(Key::ArrowDown) {
                self.down();
            } else if ctx.input().key_pressed(Key::ArrowLeft) {
                self.left();
            }

            // wasd keys
            if ctx.input().key_pressed(Key::W) {
                self.up();
            } else if ctx.input().key_pressed(Key::D) {
                self.right();
            } else if ctx.input().key_pressed(Key::S) {
                self.down();
            } else if ctx.input().key_pressed(Key::A) {
                self.left();
            }

            // vim keys
            if ctx.input().key_pressed(Key::K) {
                self.up();
            } else if ctx.input().key_pressed(Key::L) {
                self.right();
            } else if ctx.input().key_pressed(Key::J) {
                self.down();
            } else if ctx.input().key_pressed(Key::H) {
                self.left();
            }

            if diff >= self.update_interval {
                self.last_update = now;
                self.update_state();
            }
        }

        CentralPanel::default()
            .frame(Frame::none().fill(Color32::from_rgb(20, 20, 20)))
            .show(ctx, |ui| {
                self.draw(ui);
            });
    }
}

impl SnakeApp {
    fn new() -> Self {
        Self {
            paused: true,
            direction: Direction::Right,
            next_input: None,
            snake: VecDeque::from([Pos::new(5, 3), Pos::new(4, 3), Pos::new(3, 3)]),
            board: [[false; BOARD_SIZE as usize]; BOARD_SIZE as usize],
            last_update: SystemTime::UNIX_EPOCH,
            update_interval: Duration::from_millis(100),
        }
    }

    fn up(&mut self) {
        if !(self.direction == Direction::Down) {
            self.next_input = Some(Direction::Up);
        }
    }

    fn right(&mut self) {
        if !(self.direction == Direction::Left) {
            self.next_input = Some(Direction::Right);
        }
    }

    fn down(&mut self) {
        if !(self.direction == Direction::Up) {
            self.next_input = Some(Direction::Down);
        }
    }

    fn left(&mut self) {
        if !(self.direction == Direction::Right) {
            self.next_input = Some(Direction::Left);
        }
    }

    fn update_state(&mut self) {
        if let Some(dir) = self.next_input {
            self.direction = dir;
        }

        let old_head = self.snake[0];
        let new_head = match self.direction {
            Direction::Up => Pos::new(old_head.x, old_head.y - 1),
            Direction::Right => Pos::new(old_head.x + 1, old_head.y),
            Direction::Down => Pos::new(old_head.x, old_head.y + 1),
            Direction::Left => Pos::new(old_head.x - 1, old_head.y),
        };

        if !(0..BOARD_SIZE).contains(&new_head.x) || !(0..BOARD_SIZE).contains(&new_head.y) {
            // lost
            *self = Self::new();
            return;
        }

        let eaten_apple = self.board[new_head.y as usize][new_head.x as usize];
        if eaten_apple {
            self.board[new_head.y as usize][new_head.x as usize] = false;
        } else {
            self.snake.pop_back();
        };

        if self.snake.contains(&new_head) {
            // lost
            *self = Self::new();
            return;
        }

        self.snake.push_front(new_head);

        // place apple
        let apple_count = self.board.iter().flatten().filter(|f| **f).count();
        let mut rng = rand::thread_rng();
        if rng.gen_bool(1.0 / 30.0) || apple_count == 0 {
            let mut options = Vec::new();
            for y in 0..BOARD_SIZE {
                for x in 0..BOARD_SIZE {
                    let pos = Pos::new(y, x);
                    if self.snake.contains(&pos) {
                        continue;
                    }
                    if self.board[y as usize][x as usize] {
                        continue;
                    }

                    options.push(pos);
                }
            }

            if let Some(apple) = options.choose(&mut rng) {
                self.board[apple.y as usize][apple.x as usize] = true;
            }
        }
    }

    fn draw(&mut self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let board_size = available_size.x.min(available_size.y);
        let board_x = (available_size.x - board_size) / 2.0;
        let board_y = (available_size.y - board_size) / 2.0;

        let field_width = board_size / BOARD_SIZE as f32;
        let field_height = board_size / BOARD_SIZE as f32;
        let field_size = Vec2::new(field_width, field_height);

        let board_pos = Pos2::new(board_x, board_y);
        let board_size = Vec2::new(board_size, board_size);
        let board_rect = Rect::from_min_size(board_pos, board_size);

        ui.allocate_ui_at_rect(board_rect, |ui| {
            let pos = ui.cursor().min;
            let board_rect = Rect::from_min_size(pos, board_size);
            let painter = ui.painter_at(board_rect);

            // board
            painter.rect_filled(board_rect, 0.0, Color32::from_rgb(35, 30, 40));

            // apples
            for y in 0..BOARD_SIZE {
                for x in 0..BOARD_SIZE {
                    if self.board[y as usize][x as usize] {
                        let apple_pos =
                            pos + Vec2::new(field_width * x as f32, field_height * y as f32);
                        let rect = Rect::from_min_size(apple_pos, field_size);
                        painter.rect_filled(rect, field_size.min_elem() / 2.0, Color32::RED);
                    }
                }
            }

            // snake
            for p in self.snake.iter() {
                let tile_pos = pos + Vec2::new(field_width * p.x as f32, field_height * p.y as f32);
                let rect = Rect::from_min_size(tile_pos, field_size);
                painter.rect(rect, 0.0, Color32::from_rgb(90, 80, 200), Stroke::none());
            }

            // paused
            if self.paused {
                let center_pos = pos + board_size / 2.0;
                let entire_pause_size = board_size / 8.0;

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
            }
        });
    }
}
