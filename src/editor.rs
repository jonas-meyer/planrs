use bevy::prelude::*;

/// Current editing mode. Systems can run conditionally via
/// `.run_if(in_state(EditorMode::DrawRoad))`.
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditorMode {
    #[default]
    Select,
    DrawRoad,
    PlaceStop,
}
