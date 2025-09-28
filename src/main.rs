mod components;

use bevy::prelude::*;
use components::{Health, Hunger, Name, Sleep, Thirst, CharacterBundle};

use crate::components::{Movement, Position};

pub struct HelloPlugin;

#[derive(Resource)]
struct GreetTimer(Timer);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_animal);
        app.add_systems(Update, (update_hunger, update_thirst, update_sleep));
    }
}

fn update_hunger(time: Res<Time>, mut timer: ResMut<GreetTimer>, mut query: Query<&mut Hunger>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut hunger in &mut query {
            hunger.0 = update_parameter(&hunger.0, |x: &f32| x - 1.0);
        }
    }
}
fn update_thirst(time: Res<Time>, mut timer: ResMut<GreetTimer>, mut query: Query<&mut Thirst>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut thirst in &mut query {
            thirst.0 = update_parameter(&thirst.0, |x: &f32| x - 1.0);
        }
    }
}
fn update_sleep(time: Res<Time>, mut timer: ResMut<GreetTimer>, mut query: Query<&mut Sleep>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut sleep in &mut query {
            sleep.0 = update_parameter(&sleep.0, |x: &f32| x - 1.0);
        }
    }
}

fn update_parameter<F>(value: &f32, f: F) -> f32
where
    F: Fn(&f32) -> f32,
{
    f(value)
}

fn add_animal(mut commands: Commands) {
    let names = ["Vlad", "Sanya", "Miha", "Lexa"];

    commands.spawn_batch(names.into_iter().map(|n| CharacterBundle {
        name: Name(n.to_string()),
        health: Health(100.0),
        hunger: Hunger(100.0),
        thirst: Thirst(100.0),
        sleep: Sleep(100.0),
        position: Position { x: 0.0, y: 0.0 },
        movement: Movement { speed: 1.0, direction: 0.0 },
    }));
}