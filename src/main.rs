mod components;

use core::f32;

use bevy::prelude::*;
use components::{Health, Hunger, Name, Sleep, Thirst, CharacterBundle, Speed, Destination};
use rand::Rng;

pub struct Movement;

#[derive(Resource)]
struct WorldTimer(Timer);

#[derive(Resource)]
struct LongBehaviourTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Movement)
        .run();
}

impl Plugin for Movement {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.insert_resource(LongBehaviourTimer(Timer::from_seconds(5.0, TimerMode::Repeating)));
        app.add_systems(Startup, (visual_setup, add_animal));
        app.add_systems(Update, ((update_destination, update_movement).chain(), update_hunger, update_thirst, update_sleep));
    }
}

fn update_hunger(time: Res<Time>, mut timer: ResMut<WorldTimer>, mut query: Query<&mut Hunger>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut hunger in &mut query {
            hunger.value = update_parameter(&hunger.value, |x| (hunger.decay)(x));
        }
    }
}
fn update_thirst(time: Res<Time>, mut timer: ResMut<WorldTimer>, mut query: Query<&mut Thirst>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut thirst in &mut query {
            thirst.value = update_parameter(&thirst.value, |x| (thirst.decay)(x));
        }
    }
}
fn update_sleep(time: Res<Time>, mut timer: ResMut<WorldTimer>, mut query: Query<&mut Sleep>) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut sleep in &mut query {
            sleep.value = update_parameter(&sleep.value, |x| (sleep.decay)(x));
        }
    }
}

fn update_movement(time: Res<Time>, mut query: Query<(&mut Transform, &Speed, &Destination)>) {
    for (mut transform, speed, destination) in &mut query {
        let direction = (destination.0 - transform.translation.truncate()).normalize_or_zero();
        transform.translation += (direction * speed.0 * time.delta_secs()).extend(0.0);
    }
}

fn update_destination(time: Res<Time>, mut timer: ResMut<LongBehaviourTimer>, mut query: Query<(&mut Destination, &Transform)>) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::rng();
        for (mut destination, transform) in &mut query {
            let current_pos = transform.translation.truncate();
            let x = rng.random_range(current_pos.x-50.0..=current_pos.x+50.0);
            let y = rng.random_range(current_pos.y-50.0..=current_pos.y+50.0);
            destination.0 = Vec2::new(x, y);
        }
    }
}

fn update_parameter<F>(value: &f32, f: F) -> f32
where
    F: Fn(&f32) -> f32,
{
    f(value)
}

fn add_animal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let names = ["Vlad", "Sanya", "Miha", "Lexa", "Vlad", "Sanya", "Miha", "Lexa", "Vlad", "Sanya", "Miha", "Lexa", "Vlad", "Sanya", "Miha", "Lexa"];
    let mut rng = rand::rng();
    let mut bundles = Vec::with_capacity(names.len());
    let mesh_handle = meshes.add(Mesh::from(Circle::new(5.0)));
    let material_handle = materials.add(ColorMaterial::from(Color::hsl(200., 0.95, 0.5)));
    for n in names {
        let x = rng.random_range(-400.0..=400.0);
        let y = rng.random_range(-400.0..=400.0);
        bundles.push(CharacterBundle {
            name: Name(n.to_string()),
            health: Health(100.0),
            hunger: Hunger { value: 100.0, decay: |x| x - 1.0 },
            thirst: Thirst { value: 100.0, decay: |x| x - 1.0 },
            sleep: Sleep { value: 100.0, decay: |x| x - 1.0 },
            speed: Speed(15.0),
            destination: Destination(Vec2 { x: 0.0, y: 0.0 }),
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle.clone()),
            transform: Transform::from_xyz(x, y, 0.0),
        });
    }
    commands.spawn_batch(bundles);
}

fn visual_setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
