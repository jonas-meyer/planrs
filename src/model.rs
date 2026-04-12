use bevy::prelude::*;
use bevy_author::{AuthoredId, AuthoredModel, AuthoredStores, Store};

// --- IDs ---

#[derive(AuthoredId, Component, Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct NetworkNodeId(pub u64);

#[derive(AuthoredId, Component, Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct EdgeId(pub u64);

#[derive(AuthoredId, Component, Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct AttachmentId(pub u64);

#[derive(AuthoredId, Component, Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct RouteId(pub u64);

// --- Core types ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    Street,
    Arterial,
    Highway,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkNode {
    pub position: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub start: NetworkNodeId,
    pub end: NetworkNodeId,
    pub kind: EdgeKind,
}

// --- Placement ---

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    OnEdge { edge: EdgeId, t: f32 },
    AtNode { node: NetworkNodeId },
}

impl Placement {
    pub fn exists_in(
        &self,
        nodes: &Store<NetworkNodeId, NetworkNode>,
        edges: &Store<EdgeId, Edge>,
    ) -> bool {
        match self {
            Self::OnEdge { edge, .. } => edges.contains(*edge),
            Self::AtNode { node } => nodes.contains(*node),
        }
    }

    pub fn is_dirty_in(
        &self,
        nodes: &Store<NetworkNodeId, NetworkNode>,
        edges: &Store<EdgeId, Edge>,
    ) -> bool {
        match self {
            Self::OnEdge { edge, .. } => edges.is_id_dirty(*edge),
            Self::AtNode { node } => nodes.is_id_dirty(*node),
        }
    }
}

// --- Attachment data ---

#[derive(Clone, Debug, PartialEq)]
pub struct BusStop {
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttachmentKind {
    BusStop(BusStop),
}

impl From<BusStop> for AttachmentKind {
    fn from(data: BusStop) -> Self {
        Self::BusStop(data)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    pub placement: Placement,
    pub data: AttachmentKind,
}

// --- Category traits ---

pub trait EdgeAttachable: Into<AttachmentKind> {}

impl EdgeAttachable for BusStop {}

// --- Route ---

#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub name: String,
    pub stops: Vec<AttachmentId>,
}

// --- Model ---

#[derive(Resource, Default, Clone, AuthoredStores)]
pub struct CityModel {
    pub nodes: Store<NetworkNodeId, NetworkNode>,
    pub edges: Store<EdgeId, Edge>,
    pub attachments: Store<AttachmentId, Attachment>,
    pub routes: Store<RouteId, Route>,
}

impl CityModel {
    fn propagate_edges(&mut self) {
        if !self.nodes.is_dirty() { return; }
        let nodes = &self.nodes;
        self.edges.mark_dirty_where(|e| nodes.is_id_dirty(e.start) || nodes.is_id_dirty(e.end));
    }

    fn propagate_attachments(&mut self) {
        if !self.edges.is_dirty() && !self.nodes.is_dirty() { return; }
        let (nodes, edges) = (&self.nodes, &self.edges);
        self.attachments.mark_dirty_where(|a| a.placement.is_dirty_in(nodes, edges));
    }
}

impl AuthoredModel for CityModel {
    fn mark_all_dirty(&mut self) { self.mark_all_stores_dirty(); }
    fn mark_dirty_diff(&mut self, other: &Self) { self.diff_all_stores(other); }

    fn cascade_remove(&mut self) {
        let nodes = &self.nodes;
        self.edges
            .remove_where(|e| !nodes.contains(e.start) || !nodes.contains(e.end));
        let (nodes, edges) = (&self.nodes, &self.edges);
        self.attachments
            .remove_where(|a| !a.placement.exists_in(nodes, edges));
        let attachments = &self.attachments;
        self.routes.update_where(
            |r| r.stops.iter().any(|s| !attachments.contains(*s)),
            |r| r.stops.retain(|s| attachments.contains(*s)),
        );
    }

    fn propagate_dirty(&mut self) {
        self.propagate_edges();
        self.propagate_attachments();
    }

    fn validate(&self) {
        for (edge_id, edge) in self.edges.iter() {
            assert!(
                self.nodes.contains(edge.start),
                "Edge {edge_id:?} references missing node {:?}",
                edge.start
            );
            assert!(
                self.nodes.contains(edge.end),
                "Edge {edge_id:?} references missing node {:?}",
                edge.end
            );
        }
        for (att_id, att) in self.attachments.iter() {
            assert!(
                att.placement.exists_in(&self.nodes, &self.edges),
                "Attachment {att_id:?} has invalid placement {:?}",
                att.placement
            );
        }
        for (route_id, route) in self.routes.iter() {
            for att_id in &route.stops {
                assert!(
                    self.attachments.contains(*att_id),
                    "Route {route_id:?} references missing attachment {att_id:?}"
                );
            }
        }
    }
}
