pub mod node;
pub mod road;
pub mod segment;

use eframe::egui;
use glam::Vec2;

use crate::{node::Node, road::Road};
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PlanRS",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    roads: Vec<Road>,
}

impl Default for MyApp {
    fn default() -> Self {
        let start_node = Node::new(1, Vec2::new(100.0, 100.0));
        let end_node = Node::new(2, Vec2::new(300.0, 300.0));
        let segment = segment::Segment::new(1, start_node, end_node);
        let mut segments = Vec::new();
        segments.push(segment);
        let mut roads = Vec::new();
        roads.push(Road::new(1, "Main St".to_string(), segments));
        Self { roads }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, PlanRS!");

            // Get the painter and rect for drawing
            let (_, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            // Simple coordinate transform: world coords to screen coords
            let to_screen =
                |world_pos: Vec2| -> egui::Pos2 { egui::pos2(world_pos.x, world_pos.y) };

            for road in &self.roads {
                road.draw(&painter, &to_screen);
            }
        });
    }
}
