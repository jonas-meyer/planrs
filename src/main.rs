mod camera;
mod input;
mod road;

use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use camera::CameraPlugin;
use road::RoadToolPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(EnhancedInputPlugin)
        .add_plugins(RoadToolPlugin)
        .run();
}
