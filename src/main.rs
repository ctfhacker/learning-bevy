use bevy::prelude::*;

#[derive(Resource)]
struct TickTimer(Timer);

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TickTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Update, tick)
        .run()
}

fn tick(time: Res<Time>, mut timer: ResMut<TickTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        info!("tick 1234");
    }
}
