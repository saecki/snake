use eframe::{App, NativeOptions};
use egui::{CentralPanel, Color32, Frame};
use serde::{Deserialize, Serialize};
use snake::Snake;

fn main() {
    eframe::run_native(
        "snake",
        NativeOptions::default(),
        Box::new(|c| {
            Box::new(
                c.storage
                    .and_then(|s| eframe::get_value::<SnakeApp>(s, eframe::APP_KEY))
                    .unwrap_or_default(),
            )
        }),
    )
}

#[derive(Default, Serialize, Deserialize)]
struct SnakeApp {
    snake: Snake,
}

impl App for SnakeApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::none().fill(Color32::from_rgb(20, 20, 20)))
            .show(ctx, |ui| {
                snake::snake_game(ui, &mut self.snake);
            });
    }
}
