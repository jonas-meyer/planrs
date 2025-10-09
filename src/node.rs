use glam::Vec2;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    pub position: Vec2,
    pub connected_segments: Vec<usize>,
}

impl Node {
    pub fn new(id: u64, position: Vec2) -> Self {
        Self {
            id,
            position,
            connected_segments: Vec::new(),
        }
    }

    pub fn draw(&self, painter: &egui::Painter, to_screen: &dyn Fn(Vec2) -> egui::Pos2) {
        let screen_pos = to_screen(self.position);
        painter.circle_filled(screen_pos, 5.0, egui::Color32::RED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(1, Vec2::new(10.0, 20.0));
        assert_eq!(node.id, 1);
        assert_eq!(node.position, Vec2::new(10.0, 20.0));
        assert!(node.connected_segments.is_empty());
    }
}
