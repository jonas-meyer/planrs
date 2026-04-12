use bevy::prelude::*;
use planrs::PlanrsCorePlugin;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(PlanrsCorePlugin)
        .run();
}
