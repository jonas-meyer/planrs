pub mod draw;
pub mod node;
pub mod road;
pub mod segment;
pub mod slotmap;
pub mod world;

use eframe::egui;
use glam::Vec2;

use crate::{draw::DrawCtx, road::RoadExt, segment::SegmentExt, world::World};
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PlanRS",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    world: World,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut world = World {
            nodes: slotmap::SlotMap::new(),
            segments: slotmap::SlotMap::new(),
            roads: slotmap::SlotMap::new(),
        };
        let road = world.add_road("Main Street");
        world.add_segment(
            Some(road),
            node::Endpoint::Pos(Vec2::new(100.0, 100.0)),
            node::Endpoint::Pos(Vec2::new(300.0, 300.0)),
        );
        world.add_segment(
            Some(road),
            node::Endpoint::Pos(Vec2::new(300.0, 300.0)),
            node::Endpoint::Pos(Vec2::new(500.0, 100.0)),
        );
        Self { world }
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

            self.world.draw(&DrawCtx {
                painter: &painter,
                to_screen,
            });
        });
    }
}
