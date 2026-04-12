use crate::model::*;

pub fn create_node(model: &mut CityModel, position: bevy::math::Vec2) -> NetworkNodeId {
    let id = model.nodes.allocate_id();
    model.nodes.insert(id, NetworkNode { position });
    id
}

pub fn create_edge(
    model: &mut CityModel,
    start: NetworkNodeId,
    end: NetworkNodeId,
    kind: EdgeKind,
) -> EdgeId {
    let id = model.edges.allocate_id();
    model.edges.insert(id, Edge { start, end, kind });
    id
}

pub fn create_edge_from_points(
    model: &mut CityModel,
    start: bevy::math::Vec2,
    end: bevy::math::Vec2,
    kind: EdgeKind,
) -> EdgeId {
    let start_id = create_node(model, start);
    let end_id = create_node(model, end);
    create_edge(model, start_id, end_id, kind)
}

pub fn place_on_edge<T: EdgeAttachable>(
    model: &mut CityModel,
    edge: EdgeId,
    t: f32,
    data: T,
) -> AttachmentId {
    let id = model.attachments.allocate_id();
    model.attachments.insert(
        id,
        Attachment {
            placement: Placement::OnEdge { edge, t },
            data: data.into(),
        },
    );
    id
}

pub fn create_route(
    model: &mut CityModel,
    name: impl Into<String>,
    stops: Vec<AttachmentId>,
) -> RouteId {
    let id = model.routes.allocate_id();
    model.routes.insert(id, Route { name: name.into(), stops });
    id
}

pub fn delete_edge(model: &mut CityModel, id: EdgeId) {
    model.edges.remove(id);
}

pub fn delete_attachment(model: &mut CityModel, id: AttachmentId) {
    model.attachments.remove(id);
}
