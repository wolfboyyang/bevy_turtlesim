use bevy::log::LogPlugin;
use bevy::prelude::*;
use turtle_core::events::MoveEvent;
use turtlesim_plugin::TurtlesimPlugin;
use zenoh_plugin::ZenohPlugin;

const BOUNDS: Vec2 = Vec2::from_array([1200.0, 640.0]);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "TurtleSim".to_string(),
            width: BOUNDS.x,
            height: BOUNDS.y,
            ..Default::default()
        })
        .add_plugins_with(DefaultPlugins, |plugins| {
            plugins.disable::<bevy::log::LogPlugin>()
        })
        .add_startup_system(setup_camera)
        .add_plugin(TurtlesimPlugin)
        .add_plugin(ZenohPlugin)
        .add_plugin(LogPlugin)
        .add_system(input_system)
        //.add_startup_system(setup_asset)
        //.add_system(turtle_movement_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // 2D orthographic camera
    //commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // UI Camera
    //commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(Camera2dBundle::default());
}

/// applying rotation and movement based on keyboard input.
fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut move_event_writer: EventWriter<MoveEvent>,
) {
    let mut rotation = 0;
    let mut movement = 0;

    if keyboard_input.pressed(KeyCode::Left) {
        rotation += 1;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        rotation -= 1;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement += 1;
    }
    if rotation != 0 || movement != 0 {
        move_event_writer.send(MoveEvent { rotation, movement, teleop: false });
    }
}
