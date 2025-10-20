use glam::Vec2;

use crate::{
    draw::DrawCtx,
    node::Node,
    road::Road,
    segment::Segment,
    slotmap::{Key, SlotMap},
};

#[derive(Debug)]
pub struct World {
    pub nodes: SlotMap<Node>,
    pub segments: SlotMap<Segment>,
    pub roads: SlotMap<Road>,
}

pub trait WorldStore<T> {
    fn store(&self) -> &SlotMap<T>;
    fn store_mut(&mut self) -> &mut SlotMap<T>;
}

impl World {
    pub fn add<T>(&mut self, v: T) -> Key<T>
    where
        Self: WorldStore<T>,
    {
        self.store_mut().insert(v)
    }
    pub fn get<T>(&self, k: Key<T>) -> Option<&T>
    where
        Self: WorldStore<T>,
    {
        self.store().get(k)
    }
    pub fn get_mut<T>(&mut self, k: Key<T>) -> Option<&mut T>
    where
        Self: WorldStore<T>,
    {
        self.store_mut().get_mut(k)
    }
    pub fn remove<T>(&mut self, k: Key<T>) -> Option<T>
    where
        Self: WorldStore<T>,
    {
        self.store_mut().remove(k)
    }
    pub fn draw<F>(&self, ctx: &DrawCtx<'_, F>)
    where
        F: Fn(Vec2) -> egui::Pos2 + Copy,
    {
        for segment in self.segments.values() {
            segment.draw(self, ctx);
        }
        for node in self.nodes.values() {
            node.draw(ctx);
        }
    }
}

impl WorldStore<Node> for World {
    fn store(&self) -> &SlotMap<Node> {
        &self.nodes
    }
    fn store_mut(&mut self) -> &mut SlotMap<Node> {
        &mut self.nodes
    }
}

impl WorldStore<Segment> for World {
    fn store(&self) -> &SlotMap<Segment> {
        &self.segments
    }
    fn store_mut(&mut self) -> &mut SlotMap<Segment> {
        &mut self.segments
    }
}
impl WorldStore<Road> for World {
    fn store(&self) -> &SlotMap<Road> {
        &self.roads
    }
    fn store_mut(&mut self) -> &mut SlotMap<Road> {
        &mut self.roads
    }
}
