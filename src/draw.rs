use glam::Vec2;

pub struct DrawCtx<'a, F>
where
    F: Fn(Vec2) -> egui::Pos2 + Copy,
{
    pub painter: &'a egui::Painter,
    pub to_screen: F,
}
