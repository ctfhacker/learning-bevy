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
struct CardVisual {
    size: Vec2,
}

#[derive(Resource, Default)]
struct GameState {
    state: number_crunch::State,
}

#[derive(Component, Debug)]
struct Card {
    card: number_crunch::Card,
}

impl From<number_crunch::Card> for Card {
    fn from(card: number_crunch::Card) -> Card {
        Card { card }
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            state: number_crunch::State::new(),
        }
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleSubsecondPlugin::default())
        .insert_resource(GameState::new())
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
    state: Res<GameState>,
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
        setup(parent, state);
    });
}

/// Setup the world with initial entities
#[hot]
fn setup(commands: &mut ChildSpawnerCommands<'_>, state: Res<GameState>) {
    // Fixed board size in world units (taller than wide).
    let board_size = Vec2::new(480.0, 720.0);
    let grid_cols = 3.0;
    let grid_rows = 3.0;
    let board_inset = 48.0;
    let inner_size = board_size - Vec2::splat(board_inset * 2.0);
    let cell_size = Vec2::new(inner_size.x / grid_cols, inner_size.y / grid_rows);

    // ^
    // |
    // x
    //   y---->
    // Place board on (0,0) in lower left of the screen.
    let board_origin = Vec2::new(
        -board_size.x * 0.5 + board_inset,
        -board_size.y * 0.5 + board_inset,
    );

    // Table: wood base + felt surface to suggest depth.
    commands.spawn((
        Table,
        Sprite::from_color(
            Color::srgb(0.28, 0.18, 0.10),
            board_size + Vec2::new(40.0, 40.0),
        ),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));
    commands.spawn((
        Table,
        Sprite::from_color(Color::srgb(0.12, 0.24, 0.14), board_size),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    println!("{:?}", state.state.board);

    // Draw all of the cards on the table (lower-left origin mapping).
    let card_size = cell_size * 0.78;
    for (i, card) in state.state.board.into_iter().enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        let center = board_origin + Vec2::new(cell_size.x * (col + 0.5), cell_size.y * (row + 0.5));

        // Card shadow for depth.
        commands.spawn((
            Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.25), card_size * 1.02),
            Transform::from_xyz(center.x + 6.0, center.y - 6.0, 0.0),
        ));

        // Spwan the actual card
        commands.spawn((
            Card { card },
            CardVisual { size: card_size },
            Sprite::from_color(
                Color::srgb(0.2 + 0.2 * i as f32, 1.0 - 0.1 * i as f32, 0.88),
                card_size,
            ),
            Transform::from_xyz(center.x, center.y, 1.0),
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
            info!("clicked card {:?}", card,);
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
