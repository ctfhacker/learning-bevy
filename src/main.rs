use bevy::{ecs::schedule::common_conditions::run_once, prelude::*};
use bevy_hotpatching_experiments::prelude::*;

#[derive(Component)]
struct WorldRoot;

#[derive(Component)]
struct Table;

// Message used to trigger re-running startup
#[derive(Message, Debug)]
struct RebuildWorld;

#[derive(Component, Debug, Clone)]
struct Card {
    name: &'static str,
    value: u8,
    base_fame: u8,
}

#[derive(Component, Debug, Clone)]
struct CardVisual {
    size: Vec2,
}

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_message::<RebuildWorld>()
        .add_systems(Startup, boot)
        .add_systems(
            Update,
            (reset_world, check_hotpatch.run_if(run_once), click_to_print),
        )
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
    let cards = [
        Card {
            name: "Ostrich",
            value: 1,
            base_fame: 1,
        },
        Card {
            name: "Eagle",
            value: 4,
            base_fame: 2,
        },
        Card {
            name: "Dog",
            value: 5,
            base_fame: 0,
        },
        Card {
            name: "Camel",
            value: 8,
            base_fame: 2,
        },
        Card {
            name: "Rabbit",
            value: 9,
            base_fame: 3,
        },
    ];

    // Create a table
    commands.spawn((
        Table,
        Sprite::from_color(Color::srgb(0.12, 0.18, 0.12), Vec2::new(1200.0, 700.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Draw all of the cards on the table
    let card_size = Vec2::new(120.0, 220.0);
    for (i, card) in cards.into_iter().enumerate() {
        let x = -360.0 + i as f32 * 180.0;
        let y = 0.0;

        commands.spawn((
            card,
            CardVisual { size: card_size },
            Sprite::from_color(
                Color::srgb(0.2 + 0.2 * i as f32, 1.0 - 0.1 * i as f32, 0.88),
                card_size,
            ),
            Transform::from_xyz(x, y, 0.0),
        ));
    }
}

/// Click on a card and print the card name
#[hot]
fn click_to_print(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    cards: Query<(&Card, &CardVisual, &Transform)>,
) {
    // Only react to a completed left-click to avoid repeated processing.
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // Abort if the cursor is outside the window.
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    // Convert the screen-space cursor position into world-space.
    let (camera, camera_transform) = *camera_query;

    let Ok(click_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    // Check each card's bounds to see if the click landed on it.
    for (card, visual, transform) in &cards {
        if point_in_aabb(click_pos, transform.translation.truncate(), visual.size) {
            info!("clicked card {}", card.name);
            return;
        }
    }

    // Fall back when no card matched the click.
    info!("Clicked no card");
}

/// Returns true if the given point is within the rectangle with `center` and `size`
#[hot]
fn point_in_aabb(point: Vec2, center: Vec2, size: Vec2) -> bool {
    let half = size * 0.5;

    point.x >= center.x - half.x
        && point.x <= center.x + half.x
        && point.y >= center.y - half.y
        && point.y <= center.y + half.y
}
