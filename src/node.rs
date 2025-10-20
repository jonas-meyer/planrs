use egui::Color32;
use glam::Vec2;

use crate::{
    draw::DrawCtx,
    slotmap::Key,
    world::{World, WorldStore},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub position: Vec2,
    refs: u32,
}

impl Node {
    pub fn draw<F>(&self, ctx: &DrawCtx<'_, F>)
    where
        F: Fn(Vec2) -> egui::Pos2 + Copy,
    {
        ctx.painter
            .circle_filled((ctx.to_screen)(self.position), 4.0, Color32::LIGHT_BLUE);
    }
}

pub enum Endpoint {
    Key(Key<Node>),
    Pos(Vec2),
}
pub trait NodeExt {
    fn ensure_point(&mut self, ep: Endpoint) -> Option<Key<Node>>;
}

impl NodeExt for World
where
    World: WorldStore<Node>,
{
    fn ensure_point(&mut self, ep: Endpoint) -> Option<Key<Node>> {
        match ep {
            Endpoint::Key(k) => self.get(k).is_some().then_some(k),
            Endpoint::Pos(p) => Some(self.add(Node {
                position: p,
                refs: 0,
            })),
        }
    }
}
