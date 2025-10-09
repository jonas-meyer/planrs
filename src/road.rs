use crate::segment::Segment;

#[derive(Debug, Clone)]
pub struct Road {
    pub id: u64,
    pub name: String,
    pub segments: Vec<Segment>,
}

impl Road {
    pub fn new(id: u64, name: String, segments: Vec<Segment>) -> Self {
        Self { id, name, segments }
    }

    pub fn draw(&self, painter: &egui::Painter, to_screen: &dyn Fn(glam::Vec2) -> egui::Pos2) {
        for segment in &self.segments {
            segment.draw(painter, to_screen);
        }
    }
}
