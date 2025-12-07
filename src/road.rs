use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::Deserialize;

use crate::camera::CursorWorldPosition;
use crate::input::{InputBinding, load_config};

pub struct RoadToolPlugin;

impl Plugin for RoadToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<RoadToolState>()
            .init_resource::<CurrentRoad>()
            .add_input_context::<RoadBuilder>()
            .add_systems(Startup, setup_input_context)
            .add_systems(Update, (draw_road_preview, draw_finalized_road))
            .add_systems(OnEnter(RoadToolState::Building), on_enter_building)
            .add_systems(OnExit(RoadToolState::Building), on_exit_building)
            .add_observer(on_toggle_build)
            .add_observer(on_place_node)
            .add_observer(on_confirm_road)
            .add_observer(on_cancel_building);
    }
}

#[derive(Deserialize)]
struct RoadInputConfig {
    toggle_build: InputBinding,
    place_node: InputBinding,
    confirm_road: InputBinding,
    cancel_building: InputBinding,
}

impl Default for RoadInputConfig {
    fn default() -> Self {
        RoadInputConfig {
            toggle_build: InputBinding::Key(KeyCode::KeyR),
            place_node: InputBinding::Mouse(MouseButton::Left),
            confirm_road: InputBinding::Key(KeyCode::Enter),
            cancel_building: InputBinding::Key(KeyCode::Escape),
        }
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleBuild;

#[derive(InputAction)]
#[action_output(bool)]
struct PlaceNode;

#[derive(InputAction)]
#[action_output(bool)]
struct ConfirmRoad;

#[derive(InputAction)]
#[action_output(bool)]
struct CancelBuilding;

#[derive(Component)]
struct RoadBuilder;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoadToolState {
    #[default]
    Idle,
    Building,
}

#[derive(Component)]
pub struct Road;

#[derive(Component)]
pub struct RoadNode;

#[derive(Resource, Default)]
pub struct CurrentRoad {
    pub nodes: Vec<Vec2>,
}

impl CurrentRoad {
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

fn setup_input_context(mut commands: Commands) {
    let config: RoadInputConfig = load_config("settings.ron");

    commands.spawn((
        RoadBuilder,
        actions!(RoadBuilder[
            (Action::<ToggleBuild>::new(), bindings![config.toggle_build]),
            (Action::<PlaceNode>::new(), bindings![config.place_node]),
            (Action::<ConfirmRoad>::new(), bindings![config.confirm_road]),
            (
                Action::<CancelBuilding>::new(),
                bindings![config.cancel_building],
            ),]
        ),
    ));
}

fn on_toggle_build(
    _trigger: On<Start<ToggleBuild>>,
    current_state: Res<State<RoadToolState>>,
    mut next_state: ResMut<NextState<RoadToolState>>,
) {
    match current_state.get() {
        RoadToolState::Idle => next_state.set(RoadToolState::Building),
        RoadToolState::Building => next_state.set(RoadToolState::Idle),
    }
}

fn on_place_node(
    _trigger: On<Start<PlaceNode>>,
    current_state: Res<State<RoadToolState>>,
    cursor_pos: Res<CursorWorldPosition>,
    mut current_road: ResMut<CurrentRoad>,
) {
    if *current_state.get() != RoadToolState::Building {
        return;
    }

    current_road.nodes.push(cursor_pos.0);
}

fn on_confirm_road(
    _trigger: On<Start<ConfirmRoad>>,
    current_state: Res<State<RoadToolState>>,
    current_road: Res<CurrentRoad>,
    mut next_state: ResMut<NextState<RoadToolState>>,
    mut commands: Commands,
) {
    if *current_state.get() != RoadToolState::Building {
        return;
    }

    if current_road.nodes.len() < 2 {
        return;
    }

    info!("Created road with {} nodes", current_road.nodes.len());

    commands
        .spawn((Road, Transform::default()))
        .with_children(|parent| {
            for &node in current_road.nodes.iter() {
                parent.spawn((RoadNode, Transform::from_translation(node.extend(0.0))));
            }
        });
    next_state.set(RoadToolState::Idle)
}

fn on_cancel_building(
    _trigger: On<Start<CancelBuilding>>,
    current_state: Res<State<RoadToolState>>,
    mut next_state: ResMut<NextState<RoadToolState>>,
) {
    if *current_state.get() != RoadToolState::Building {
        return;
    }

    next_state.set(RoadToolState::Idle);
}

fn on_enter_building(mut current_road: ResMut<CurrentRoad>) {
    current_road.clear();
    info!("Entered road building mode. Click to place nodes, Enter to confirm, Escape to cancel.");
}

fn on_exit_building(mut current_road: ResMut<CurrentRoad>) {
    current_road.clear();
    info!("Exited road building mode.");
}

fn draw_road_preview(
    mut gizmos: Gizmos,
    current_road: Res<CurrentRoad>,
    cursor_position: Res<CursorWorldPosition>,
) {
    let nodes = &current_road.nodes;

    if nodes.len() >= 2 {
        gizmos.linestrip_2d(nodes.iter().copied(), Color::srgb(0.8, 0.8, 0.8))
    }

    if let Some(&last) = nodes.last() {
        gizmos.line_2d(last, cursor_position.0, Color::srgba(1.0, 1.0, 0.0, 0.5));
    }

    for (i, &node) in nodes.iter().enumerate() {
        let color = if i == 0 {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.8, 0.2)
        };
        gizmos.circle_2d(node, 6.0, color);
    }

    gizmos.circle_2d(cursor_position.0, 4.0, Color::srgba(0.2, 1.0, 0.2, 0.7));
}

fn draw_finalized_road(
    mut gizmos: Gizmos,
    roads: Query<&Children, With<Road>>,
    nodes: Query<&Transform, With<RoadNode>>,
) {
    for children in &roads {
        let nodes: Vec<Vec2> = nodes
            .iter_many(children)
            .map(|t| t.translation.truncate())
            .collect();

        if nodes.len() >= 2 {
            gizmos.linestrip_2d(nodes.iter().copied(), Color::srgb(0.3, 0.5, 0.9));
        }

        for &node in &nodes {
            gizmos.circle_2d(node, 5.0, Color::srgb(0.4, 0.6, 1.0));
        }
    }
}
