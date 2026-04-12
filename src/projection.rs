use bevy::prelude::*;
use bevy_author::{AuthorPlugin, RowProjection, impl_project_row, sync_row};

use crate::model::*;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct NodeWorldPosition(pub Vec2);

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct EdgeEndpoints {
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct AttachmentWorldPosition(pub Vec2);

// --- Project functions (pure, testable without ECS) ---

pub fn project_node(
    _id: NetworkNodeId,
    node: &NetworkNode,
    _model: &CityModel,
) -> Option<NodeWorldPosition> {
    Some(NodeWorldPosition(node.position))
}

pub fn project_edge(
    _id: EdgeId,
    edge: &Edge,
    model: &CityModel,
) -> Option<EdgeEndpoints> {
    let start = model.nodes.get(edge.start)?.position;
    let end = model.nodes.get(edge.end)?.position;
    Some(EdgeEndpoints { start, end })
}

pub fn project_attachment(
    _id: AttachmentId,
    att: &Attachment,
    model: &CityModel,
    edge_proj: &RowProjection<EdgeId>,
    node_proj: &RowProjection<NetworkNodeId>,
) -> Option<(AttachmentWorldPosition, ChildOf)> {
    match att.placement {
        Placement::OnEdge { edge, t } => {
            let parent_entity = edge_proj.get(&edge)?;
            let e = model.edges.get(edge)?;
            let start = model.nodes.get(e.start)?.position;
            let end = model.nodes.get(e.end)?.position;
            Some((AttachmentWorldPosition(start.lerp(end, t)), ChildOf(parent_entity)))
        }
        Placement::AtNode { node } => {
            let parent_entity = node_proj.get(&node)?;
            let n = model.nodes.get(node)?;
            Some((AttachmentWorldPosition(n.position), ChildOf(parent_entity)))
        }
    }
}

// --- ProjectRow impls (wire functions into the trait) ---

impl_project_row!(CityModel, NetworkNode, NetworkNodeId, nodes {
    type Extra = NodeWorldPosition;
    fn project(id, node, model) { project_node(id, node, model) }
});

impl_project_row!(CityModel, Edge, EdgeId, edges {
    type Extra = EdgeEndpoints;
    fn project(id, edge, model) { project_edge(id, edge, model) }
});

impl_project_row!(CityModel, Attachment, AttachmentId, attachments {
    type Extra = (AttachmentWorldPosition, ChildOf);
    fn project(_id, att, model, world) {
        let edge_proj = world.resource::<RowProjection<EdgeId>>();
        let node_proj = world.resource::<RowProjection<NetworkNodeId>>();
        project_attachment(_id, att, model, edge_proj, node_proj)
    }
});

impl_project_row!(CityModel, Route, RouteId, routes {
    type Extra = ();
    fn project(_id, _route, _model) { Some(()) }
});

// --- Plugin ---

fn sync_projections(world: &mut World) {
    sync_row::<CityModel, NetworkNode>(world);
    sync_row::<CityModel, Edge>(world);
    sync_row::<CityModel, Attachment>(world);
    sync_row::<CityModel, Route>(world);
}

pub struct PlanrsCorePlugin;

impl Plugin for PlanrsCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AuthorPlugin::<CityModel>::new())
            .add_systems(Update, sync_projections.run_if(any_store_dirty));
    }
}

fn any_store_dirty(model: Res<CityModel>) -> bool {
    model.any_store_dirty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_author::History;
    use crate::commands;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(PlanrsCorePlugin);
        app
    }

    fn commit(app: &mut App, f: impl FnOnce(&mut CityModel)) {
        History::<CityModel>::commit(app.world_mut(), f);
    }

    #[test]
    fn creating_a_road_updates_document_and_projection() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model,
                Vec2::new(-4.0, 2.0),
                Vec2::new(8.0, 6.0),
                EdgeKind::Street,
            );
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        let edge = model.edges.get(edge_id).unwrap();
        let edge_proj = app.world().resource::<RowProjection<EdgeId>>();
        let node_proj = app.world().resource::<RowProjection<NetworkNodeId>>();
        let edge_entity = edge_proj.get(&edge_id).unwrap();
        let start_entity = node_proj.get(&edge.start).unwrap();
        let end_entity = node_proj.get(&edge.end).unwrap();

        let world = app.world();
        assert_eq!(
            world.entity(start_entity).get::<NodeWorldPosition>(),
            Some(&NodeWorldPosition(Vec2::new(-4.0, 2.0)))
        );
        assert_eq!(
            world.entity(end_entity).get::<NodeWorldPosition>(),
            Some(&NodeWorldPosition(Vec2::new(8.0, 6.0)))
        );
        assert_eq!(
            world.entity(edge_entity).get::<EdgeEndpoints>(),
            Some(&EdgeEndpoints {
                start: Vec2::new(-4.0, 2.0),
                end: Vec2::new(8.0, 6.0),
            })
        );
    }

    #[test]
    fn undo_redo_round_trip() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        app.update();

        assert!(app.world().resource::<CityModel>().edges.contains(edge_id));
        assert_eq!(app.world().resource::<History<CityModel>>().undo_len(), 1);

        History::<CityModel>::undo(app.world_mut());

        app.update();

        assert!(!app.world().resource::<CityModel>().edges.contains(edge_id));

        History::<CityModel>::redo(app.world_mut());

        app.update();

        assert!(app.world().resource::<CityModel>().edges.contains(edge_id));
    }

    #[test]
    fn cascade_delete_edge_removes_attachments_and_updates_routes() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        let mut stop_a = AttachmentId(0);
        let mut stop_b = AttachmentId(0);
        let mut route_id = RouteId(0);

        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(20.0, 0.0), EdgeKind::Street,
            );
        });
        commit(&mut app, |model| {
            stop_a = commands::place_on_edge(
                model, edge_id, 0.25, BusStop { name: Some("A".into()) },
            );
        });
        commit(&mut app, |model| {
            stop_b = commands::place_on_edge(
                model, edge_id, 0.75, BusStop { name: Some("B".into()) },
            );
        });
        commit(&mut app, |model| {
            route_id = commands::create_route(model, "Line 1", vec![stop_a, stop_b]);
        });
        app.update();

        commit(&mut app, |model| {
            commands::delete_edge(model, edge_id);
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.edges.get(edge_id).is_none());
        assert!(model.attachments.get(stop_a).is_none());
        assert!(model.attachments.get(stop_b).is_none());
        assert_eq!(model.routes.get(route_id).unwrap().stops, Vec::<AttachmentId>::new());

        // Undo restores everything
        History::<CityModel>::undo(app.world_mut());

        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.edges.get(edge_id).is_some());
        assert!(model.attachments.get(stop_a).is_some());
        assert!(model.attachments.get(stop_b).is_some());
        assert_eq!(model.routes.get(route_id).unwrap().stops, vec![stop_a, stop_b]);
    }

    #[test]
    fn moving_a_node_propagates_to_edge_endpoints_and_attachment_positions() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        let mut stop_id = AttachmentId(0);

        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        commit(&mut app, |model| {
            stop_id = commands::place_on_edge(
                model, edge_id, 0.5, BusStop { name: None },
            );
        });
        app.update();

        commit(&mut app, |model| {
            let edge = model.edges.get(edge_id).unwrap();
            let start_id = edge.start;
            model.nodes.insert(start_id, NetworkNode { position: Vec2::new(0.0, 10.0) });
        });
        app.update();

        let edge_proj = app.world().resource::<RowProjection<EdgeId>>();
        let att_proj = app.world().resource::<RowProjection<AttachmentId>>();
        let edge_entity = edge_proj.get(&edge_id).unwrap();
        let stop_entity = att_proj.get(&stop_id).unwrap();

        let world = app.world();
        assert_eq!(
            world.entity(edge_entity).get::<EdgeEndpoints>(),
            Some(&EdgeEndpoints {
                start: Vec2::new(0.0, 10.0),
                end: Vec2::new(10.0, 0.0),
            }),
        );
        assert_eq!(
            world.entity(stop_entity).get::<AttachmentWorldPosition>(),
            Some(&AttachmentWorldPosition(Vec2::new(5.0, 5.0))),
        );
    }

    #[test]
    fn multiple_undo_redo_steps() {
        let mut app = test_app();

        let mut edge_a = EdgeId(0);
        let mut edge_b = EdgeId(0);

        commit(&mut app, |model| {
            edge_a = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        commit(&mut app, |model| {
            edge_b = commands::create_edge_from_points(
                model, Vec2::new(10.0, 0.0), Vec2::new(20.0, 0.0), EdgeKind::Street,
            );
        });
        app.update();

        assert!(app.world().resource::<CityModel>().edges.contains(edge_a));
        assert!(app.world().resource::<CityModel>().edges.contains(edge_b));
        assert_eq!(app.world().resource::<History<CityModel>>().undo_len(), 2);

        History::<CityModel>::undo(app.world_mut());

        app.update();
        assert!(app.world().resource::<CityModel>().edges.contains(edge_a));
        assert!(!app.world().resource::<CityModel>().edges.contains(edge_b));

        History::<CityModel>::undo(app.world_mut());

        app.update();
        assert!(!app.world().resource::<CityModel>().edges.contains(edge_a));
        assert!(!app.world().resource::<CityModel>().edges.contains(edge_b));

        History::<CityModel>::redo(app.world_mut());

        app.update();
        assert!(app.world().resource::<CityModel>().edges.contains(edge_a));
        assert!(!app.world().resource::<CityModel>().edges.contains(edge_b));
    }

    // --- Pure function tests (no ECS needed) ---

    #[test]
    fn project_edge_computes_endpoints_from_nodes() {
        let mut model = CityModel::default();
        let eid = commands::create_edge_from_points(
            &mut model, Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0), EdgeKind::Street,
        );
        let edge = model.edges.get(eid).unwrap();

        assert_eq!(
            project_edge(eid, edge, &model),
            Some(EdgeEndpoints { start: Vec2::new(1.0, 2.0), end: Vec2::new(3.0, 4.0) })
        );
    }

    #[test]
    fn delete_node_cascades_through_entire_chain() {
        let mut app = test_app();

        let mut node_a = NetworkNodeId(0);
        let mut edge_id = EdgeId(0);
        let mut stop_id = AttachmentId(0);
        let mut route_id = RouteId(0);

        commit(&mut app, |model| {
            node_a = commands::create_node(model, Vec2::ZERO);
            let node_b = commands::create_node(model, Vec2::new(10.0, 0.0));
            edge_id = commands::create_edge(model, node_a, node_b, EdgeKind::Street);
        });
        commit(&mut app, |model| {
            stop_id = commands::place_on_edge(model, edge_id, 0.5, BusStop { name: None });
        });
        commit(&mut app, |model| {
            route_id = commands::create_route(model, "Line 1", vec![stop_id]);
        });
        app.update();

        // Delete node_a → edge orphaned → attachment orphaned → route cleaned
        commit(&mut app, |model| {
            model.nodes.remove(node_a);
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.nodes.get(node_a).is_none());
        assert!(model.edges.get(edge_id).is_none());
        assert!(model.attachments.get(stop_id).is_none());
        assert_eq!(model.routes.get(route_id).unwrap().stops, Vec::<AttachmentId>::new());
    }

    #[test]
    fn delete_attachment_cleans_route_references() {
        let mut app = test_app();

        let mut stop_a = AttachmentId(0);
        let mut stop_b = AttachmentId(0);
        let mut route_id = RouteId(0);

        commit(&mut app, |model| {
            let eid = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
            stop_a = commands::place_on_edge(model, eid, 0.25, BusStop { name: None });
            stop_b = commands::place_on_edge(model, eid, 0.75, BusStop { name: None });
            route_id = commands::create_route(model, "Line 1", vec![stop_a, stop_b]);
        });
        app.update();

        // Delete one stop — route keeps the other
        commit(&mut app, |model| {
            commands::delete_attachment(model, stop_a);
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.attachments.get(stop_a).is_none());
        assert!(model.attachments.get(stop_b).is_some());
        assert_eq!(model.routes.get(route_id).unwrap().stops, vec![stop_b]);
    }

    #[test]
    fn attachment_has_child_of_parent_entity() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        let mut stop_id = AttachmentId(0);

        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
            stop_id = commands::place_on_edge(model, edge_id, 0.5, BusStop { name: None });
        });
        app.update();

        let edge_proj = app.world().resource::<RowProjection<EdgeId>>();
        let att_proj = app.world().resource::<RowProjection<AttachmentId>>();
        let edge_entity = edge_proj.get(&edge_id).unwrap();
        let stop_entity = att_proj.get(&stop_id).unwrap();

        assert_eq!(
            app.world().entity(stop_entity).get::<ChildOf>(),
            Some(&ChildOf(edge_entity)),
        );
    }

    #[test]
    fn commit_merged_collapses_drag_into_one_undo() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        app.update();

        let start_id = app.world().resource::<CityModel>().edges.get(edge_id).unwrap().start;

        // Simulate drag: 5 merged commits moving the start node
        for i in 1..=5 {
            let pos = Vec2::new(i as f32, 0.0);
            History::<CityModel>::commit_merged(app.world_mut(), |model| {
                model.nodes.insert(start_id, NetworkNode { position: pos });
            });
        }
        app.update();

        // Node is at final drag position
        assert_eq!(
            app.world().resource::<CityModel>().nodes.get(start_id).unwrap().position,
            Vec2::new(5.0, 0.0),
        );

        // Only 2 undo entries: create + entire drag
        assert_eq!(app.world().resource::<History<CityModel>>().undo_len(), 2);

        // One undo reverts the entire drag
        History::<CityModel>::undo(app.world_mut());

        app.update();

        assert_eq!(
            app.world().resource::<CityModel>().nodes.get(start_id).unwrap().position,
            Vec2::ZERO,
        );
    }

    #[test]
    fn separate_drags_are_separate_undo_steps() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        app.update();

        let start_id = app.world().resource::<CityModel>().edges.get(edge_id).unwrap().start;

        // Drag 1
        for _ in 0..3 {
            History::<CityModel>::commit_merged(app.world_mut(), |model| {
                model.nodes.insert(start_id, NetworkNode { position: Vec2::new(5.0, 0.0) });
            });
        }
        app.world_mut().resource_mut::<History<CityModel>>().end_merge();

        // Drag 2
        for _ in 0..3 {
            History::<CityModel>::commit_merged(app.world_mut(), |model| {
                model.nodes.insert(start_id, NetworkNode { position: Vec2::new(9.0, 0.0) });
            });
        }

        // 3 undo entries: create + drag1 + drag2
        assert_eq!(app.world().resource::<History<CityModel>>().undo_len(), 3);

        History::<CityModel>::undo(app.world_mut());
        assert_eq!(
            app.world().resource::<CityModel>().nodes.get(start_id).unwrap().position,
            Vec2::new(5.0, 0.0),
        );

        History::<CityModel>::undo(app.world_mut());
        assert_eq!(
            app.world().resource::<CityModel>().nodes.get(start_id).unwrap().position,
            Vec2::ZERO,
        );
    }

    // --- Pure function tests (no ECS needed) ---

    #[test]
    fn project_attachment_computes_lerped_position() {
        let mut model = CityModel::default();
        let eid = commands::create_edge_from_points(
            &mut model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
        );
        let stop_id = commands::place_on_edge(&mut model, eid, 0.3, BusStop { name: None });
        let att = model.attachments.get(stop_id).unwrap();

        // Need RowProjections with fake entities for the test
        let mut edge_proj = RowProjection::<EdgeId>::default();
        let fake_entity = Entity::from_bits(42);
        edge_proj.insert(eid, fake_entity);
        let node_proj = RowProjection::<NetworkNodeId>::default();

        let result = project_attachment(stop_id, att, &model, &edge_proj, &node_proj);
        assert!(result.is_some());
        let (pos, parent) = result.unwrap();
        assert_eq!(pos, AttachmentWorldPosition(Vec2::new(3.0, 0.0)));
        assert_eq!(parent, ChildOf(fake_entity));
    }

    #[test]
    fn project_edge_returns_none_for_missing_node() {
        let mut model = CityModel::default();
        let bad_edge = Edge { start: NetworkNodeId(999), end: NetworkNodeId(998), kind: EdgeKind::Street };
        let eid = model.edges.allocate_id();
        model.edges.insert(eid, bad_edge.clone());

        assert_eq!(project_edge(eid, &bad_edge, &model), None);
    }

    // --- Edge case tests ---

    #[test]
    fn redo_cascade_delete_re_deletes_everything() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        let mut stop_id = AttachmentId(0);
        let mut route_id = RouteId(0);

        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        commit(&mut app, |model| {
            stop_id = commands::place_on_edge(model, edge_id, 0.5, BusStop { name: None });
        });
        commit(&mut app, |model| {
            route_id = commands::create_route(model, "Line 1", vec![stop_id]);
        });
        commit(&mut app, |model| {
            commands::delete_edge(model, edge_id);
        });
        app.update();

        // Undo restores everything
        History::<CityModel>::undo(app.world_mut());
        app.update();
        assert!(app.world().resource::<CityModel>().edges.get(edge_id).is_some());
        assert!(app.world().resource::<CityModel>().attachments.get(stop_id).is_some());

        // Redo re-deletes everything
        History::<CityModel>::redo(app.world_mut());
        app.update();
        let model = app.world().resource::<CityModel>();
        assert!(model.edges.get(edge_id).is_none());
        assert!(model.attachments.get(stop_id).is_none());
        assert_eq!(model.routes.get(route_id).unwrap().stops, Vec::<AttachmentId>::new());

        // Entities are despawned too
        assert!(app.world().resource::<RowProjection<EdgeId>>().get(&edge_id).is_none());
        assert!(app.world().resource::<RowProjection<AttachmentId>>().get(&stop_id).is_none());
    }

    #[test]
    fn redo_restores_entities_after_undo_despawn() {
        let mut app = test_app();

        let mut edge_id = EdgeId(0);
        commit(&mut app, |model| {
            edge_id = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
        });
        app.update();
        let edge_entity = app.world().resource::<RowProjection<EdgeId>>()
            .get(&edge_id).unwrap();

        // Undo despawns entity from world, not just from projection
        History::<CityModel>::undo(app.world_mut());
        app.update();
        assert!(app.world().get_entity(edge_entity).is_err());
        assert!(app.world().resource::<RowProjection<EdgeId>>().get(&edge_id).is_none());

        // Redo must re-create the entity
        History::<CityModel>::redo(app.world_mut());
        app.update();
        let entity = app.world().resource::<RowProjection<EdgeId>>().get(&edge_id);
        assert!(entity.is_some());
        assert_eq!(
            app.world().entity(entity.unwrap()).get::<EdgeEndpoints>(),
            Some(&EdgeEndpoints { start: Vec2::ZERO, end: Vec2::new(10.0, 0.0) }),
        );
    }

    #[test]
    fn two_routes_sharing_stop_both_cleaned_on_delete() {
        let mut app = test_app();

        let mut stop_id = AttachmentId(0);
        let mut route_a = RouteId(0);
        let mut route_b = RouteId(0);

        commit(&mut app, |model| {
            let eid = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
            stop_id = commands::place_on_edge(model, eid, 0.5, BusStop { name: None });
            route_a = commands::create_route(model, "Line A", vec![stop_id]);
            route_b = commands::create_route(model, "Line B", vec![stop_id]);
        });
        app.update();

        commit(&mut app, |model| {
            commands::delete_attachment(model, stop_id);
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.routes.get(route_a).unwrap().stops.is_empty());
        assert!(model.routes.get(route_b).unwrap().stops.is_empty());
    }

    #[test]
    fn create_and_delete_in_same_commit_leaves_no_trace() {
        let mut app = test_app();

        commit(&mut app, |model| {
            let eid = commands::create_edge_from_points(
                model, Vec2::ZERO, Vec2::new(10.0, 0.0), EdgeKind::Street,
            );
            commands::delete_edge(model, eid);
        });
        app.update();

        let model = app.world().resource::<CityModel>();
        assert!(model.edges.is_empty());
        // No entities should exist
        assert!(app.world().resource::<RowProjection<EdgeId>>().get(&EdgeId(0)).is_none());
    }

    #[test]
    fn undo_modified_value_restores_original() {
        let mut app = test_app();

        let mut route_id = RouteId(0);
        commit(&mut app, |model| {
            route_id = commands::create_route(model, "Original", vec![]);
        });
        app.update();

        commit(&mut app, |model| {
            model.routes.insert(route_id, Route { name: "Renamed".into(), stops: vec![] });
        });
        app.update();

        assert_eq!(
            app.world().resource::<CityModel>().routes.get(route_id).unwrap().name,
            "Renamed"
        );

        // Undo should restore the original name (diff detects value change)
        History::<CityModel>::undo(app.world_mut());

        assert_eq!(
            app.world().resource::<CityModel>().routes.get(route_id).unwrap().name,
            "Original"
        );
    }
}
