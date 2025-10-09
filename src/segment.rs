use crate::node::Node;

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: u64,
    pub start_node: Node,
    pub end_node: Node,
}

impl Segment {
    pub fn new(id: u64, start_node: Node, end_node: Node) -> Self {
        Self {
            id,
            start_node,
            end_node,
        }
    }

    pub fn draw(&self, painter: &egui::Painter, to_screen: &dyn Fn(glam::Vec2) -> egui::Pos2) {
        let start_pos = to_screen(self.start_node.position);
        let end_pos = to_screen(self.end_node.position);
        self.start_node.draw(painter, to_screen);
        self.end_node.draw(painter, to_screen);
        painter.line_segment([start_pos, end_pos], (2.0, egui::Color32::WHITE));
    }
}
