use egui::{Color32, Stroke};
use glam::Vec2;

use crate::{
    draw::DrawCtx,
    node::{Endpoint, Node, NodeExt},
    road::Road,
    slotmap::Key,
    world::{World, WorldStore},
};

#[derive(Debug, Clone)]
pub struct Segment {
    pub a: Key<Node>,
    pub b: Key<Node>,
    pub road: Option<Key<Road>>,
}

impl Segment {
    pub fn draw<F>(&self, world: &World, ctx: &DrawCtx<'_, F>)
    where
        F: Fn(Vec2) -> egui::Pos2 + Copy,
    {
        let (Some(a), Some(b)) = (world.nodes.get(self.a), world.nodes.get(self.b)) else {
            return;
        };
        ctx.painter.line_segment(
            [(ctx.to_screen)(a.position), (ctx.to_screen)(b.position)],
            Stroke::new(2.0, Color32::DARK_GRAY),
        );
    }
}

pub trait SegmentExt {
    fn add_segment(
        &mut self,
        road: Option<Key<Road>>,
        a: Endpoint,
        b: Endpoint,
    ) -> Option<Key<Segment>>;
}

impl SegmentExt for World
where
    World: WorldStore<Segment> + WorldStore<Road> + WorldStore<Node> + NodeExt,
{
    fn add_segment(
        &mut self,
        road: Option<Key<Road>>,
        a: Endpoint,
        b: Endpoint,
    ) -> Option<Key<Segment>> {
        let a = self.ensure_point(a)?;
        let b = self.ensure_point(b)?;
        if a == b {
            return None;
        }

        let seg = self.add(Segment { a, b, road });

        if let Some(rk) = road {
            self.get_mut(rk)?.segments.push(seg);
        }
        Some(seg)
    }
}
