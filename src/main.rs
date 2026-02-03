use bevy::{ecs::schedule::common_conditions::run_once, prelude::*};
use bevy_hotpatching_experiments::prelude::*;

#[derive(Component)]
struct WorldRoot;

#[derive(Component)]
struct Table;

// Message used to trigger re-running startup
#[derive(Message, Debug)]
struct RebuildWorld;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_message::<RebuildWorld>()
        .add_systems(Startup, boot)
        .add_systems(Update, (reset_world, check_hotpatch.run_if(run_once)))
        .run()
}

fn boot(mut commands: Commands) {
    // Create a camera singleton
    commands.spawn(Camera2d);

    // Create the root entity that all, despawnable entities are derived
    commands.spawn((
        WorldRoot,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("world_root"),
    ));
}

/// Called during hot patch to force a reload of the entire world.
///
/// This works by despawning all entities that are children of the WorldRoot and then calling the setup
/// function once more.
#[hot(rerun_on_hot_patch = true)]
fn check_hotpatch(mut event_writer: MessageWriter<RebuildWorld>) {
    event_writer.write(RebuildWorld);
}

/// Reset all of the child entities from the world root node
#[hot]
fn reset_world(
    mut commands: Commands,
    mut event_reader: MessageReader<RebuildWorld>,
    root: Single<Entity, With<WorldRoot>>,
) {
    // If no event is here, ignore it
    if event_reader.read().next().is_none() {
        return;
    }

    commands.entity(*root).despawn_children();
    info!("World cleared!");

    // Ensure all actual game code is a child of the root node for easier hot patching
    commands.entity(*root).with_children(|parent| {
        setup(parent);
    });
}

/// Setup the world with initial entities
#[hot]
fn setup(commands: &mut ChildSpawnerCommands<'_>) {
    let card_size = Vec2::new(220.0, 320.0);

    // Create a table
    commands.spawn((
        Table,
        Sprite::from_color(Color::srgb(0.12, 0.18, 0.12), Vec2::new(1200.0, 700.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Create a card zone
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.8, 0.85), card_size),
        Transform::from_xyz(-300.0, 0.0, 0.0),
    ));

    // Create a card zone
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.8, 0.85), card_size),
        Transform::from_xyz(0.0, 100.0, 0.0),
    ));

    // Create a card zone
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.8, 0.85), card_size),
        Transform::from_xyz(300.0, 0.0, 0.0),
    ));
}
