use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Default)]
pub struct CursorWorldPosition(pub Vec2);

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, update_cursor_world_position);
    }
}

pub fn update_cursor_world_position(
    mut cursor_pos: ResMut<CursorWorldPosition>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = *camera;

    if let Some(screen_position) = window.cursor_position()
        && let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, screen_position)
    {
        cursor_pos.0 = world_position;
    }
}
