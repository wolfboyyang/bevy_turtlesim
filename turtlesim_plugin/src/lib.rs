use bevy::app::App;
use bevy::time::FixedTimestep;
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use turtle_core::events::MoveEvent;

const TIME_STEP: f32 = 1.0 / 60.0;
const DURATION: f32 = 10.0;
const MAX_ACTIVE_TIME: f32 = 1.0;

#[derive(Component)]
struct Turtle {
    /// linear speed in meters per second
    movement_speed: f32,
    /// rotation speed in radians per second
    rotation_speed: f32,
    /// movement or rotaion duration
    active_time: f32,

    movement_factor: f32,
    rotation_factor: f32,
}

pub struct TurtlesimPlugin;

impl Plugin for TurtlesimPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_plugin(DebugLinesPlugin::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                    .with_system(turtle_movement_system),
            )
            .add_event::<MoveEvent>();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let turtle_handle = asset_server.load("sprites/turtle.png");

    commands
        .spawn_bundle(SpriteBundle {
            texture: turtle_handle,
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(Turtle {
            movement_speed: 50.0,                  // metres per second
            rotation_speed: f32::to_radians(36.0), // degrees per second
            active_time: 0.0,
            movement_factor: 0.0,
            rotation_factor: 0.0,
        });
}

fn turtle_movement_system(
    mut query: Query<(&mut Turtle, &mut Transform)>,
    mut move_event_reader: EventReader<MoveEvent>,
    mut lines: ResMut<DebugLines>,
) {
    let (mut turtle, mut transform) = query.single_mut();

    for event in move_event_reader.iter() {
        turtle.rotation_factor = event.rotation as f32;
        turtle.movement_factor = event.movement as f32;
        turtle.active_time = MAX_ACTIVE_TIME;
        info!("move event: rotate {} move {}", event.rotation, event.movement);
    }

    if turtle.active_time > f32::EPSILON {
        let start = transform.translation;

        let angular = turtle.rotation_factor * turtle.rotation_speed * TIME_STEP;

        // create the change in rotation around the Z axis (perpendicular to the 2D plane of the screen)
        let rotation_delta = Quat::from_rotation_z(angular);
        // update the ship rotation with our rotation delta
        transform.rotation *= rotation_delta;

        // get the ship's forward vector by applying the current rotation to the ships initial facing vector
        let movement_direction = transform.rotation * Vec3::X;
        // get the distance the ship will move based on direction, the ship's movement speed and delta time
        let movement_distance = turtle.movement_factor * turtle.movement_speed * TIME_STEP;
        // create the change in translation using the new movement direction and distance
        let translation_delta = movement_direction * movement_distance;
        // update the ship translation with our new translation delta
        transform.translation += translation_delta;

        let end = transform.translation;
        lines.line(start, end, DURATION);

        turtle.active_time -= TIME_STEP;
    }
}
