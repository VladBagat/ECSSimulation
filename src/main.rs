mod components;
mod world_grid;

use core::f32;

use bevy::{input::mouse::{MouseMotion, MouseWheel}, math::ops::powf, prelude::*};
use components::*;
use world_grid::WorldGrid;
use rand::Rng;
use bevy_rapier2d::prelude::*;

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
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.insert_resource(WorldGrid::new(640, 640));
        app.add_systems(Startup, (visual_setup, add_animal, add_food));
        app.add_systems(Update, (update_hunger, update_thirst, update_sleep, pan_camera_on_drag, camera_zoom, display_events, draw_world_grid));
        app.add_systems(FixedUpdate, (update_destination, update_movement).chain(),);
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
        let delta = destination.0 - transform.translation.truncate();
        let distance = delta.length();
        if distance < 0.5 {
            continue;
        }
        let direction = delta / distance;
        transform.translation += (direction * speed.0 * time.delta_secs()).extend(0.0);
    }
}

fn update_destination(time: Res<Time>, mut timer: ResMut<LongBehaviourTimer>, mut query: Query<(&mut Destination, &Transform)>) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::rng();
        for (mut destination, transform) in &mut query {
            let current_pos = transform.translation.truncate();
            let x = rng.random_range(current_pos.x-150.0..=current_pos.x+150.0);
            let y = rng.random_range(current_pos.y-150.0..=current_pos.y+150.0);
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
        let character = CharacterBundle {
            name: Name(n.to_string()),
            health: Health(100.0),
            hunger: Hunger { value: 100.0, decay: |x| x - 1.0 },
            thirst: Thirst { value: 100.0, decay: |x| x - 1.0 },
            sleep: Sleep { value: 100.0, decay: |x| x - 1.0 },
            speed: Speed(35.0),
            destination: Destination(Vec2 { x: 0.0, y: 0.0 }),
        };
        let visuals = VisualBundle {
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle.clone()),
            transform: Transform::from_xyz(x, y, 0.0),
        };
        let collision = CollisionBundle::circle_sensor(
            5.0, RigidBody::KinematicPositionBased, true);
        bundles.push((character, visuals, collision));
    }
    commands.spawn_batch(bundles);
}

fn add_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    let mut bundles = Vec::with_capacity(200);
    let mesh_handle = meshes.add(Mesh::from(Circle::new(5.0)));
    let material_handle = materials.add(ColorMaterial::from(Color::hsl(21., 1., 0.356)));
    for i in 0..200 {
        let x = rng.random_range(-600.0..=600.0);
        let y = rng.random_range(-600.0..=600.0);
        let food = FoodBundle {
            name: Name(i.to_string()),
            food: Food(30.),
        };
        let visuals = VisualBundle {
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle.clone()),
            transform: Transform::from_xyz(x, y, 0.0),
        };
        let collision = CollisionBundle::circle_sensor(
            15.0, RigidBody::Fixed, false);
        bundles.push((food, visuals, collision));
    }
    commands.spawn_batch(bundles);
}

fn display_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(_entity1, entity2, _flags) => {
                commands.entity(*entity2).despawn();
            }
            CollisionEvent::Stopped(_entity1, _entity2, _flags) => {
                
            }
        }
    }

    for contact_force_event in contact_force_events.read() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

fn pan_camera_on_drag(
    mut mouse_motion_events: EventReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        delta += ev.delta;
    }

    if delta != Vec2::ZERO {
        let mut transform = query.single_mut().unwrap();
        transform.translation.x -= delta.x;
        transform.translation.y += delta.y;
    }
}

fn camera_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut Projection, With<Camera2d>>,
    time: Res<Time<Fixed>>
) {
    let Ok(mut projection) = query.single_mut() else {
        return;
    };

    if let Projection::Orthographic(projection2d) = &mut *projection {
        for ev in scroll_evr.read() {
            if ev.y > 0. {
                projection2d.scale *= powf(0.25f32, time.delta_secs());
            }
            else if ev.y < 0. {
                projection2d.scale *= powf(4.0f32, time.delta_secs());
            }
        }
    }
}

fn visual_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection {
            ..OrthographicProjection::default_2d()
        }),
    ));
}

// this works but the scale is fucked up
fn draw_world_grid(mut gizmos: Gizmos, grid: Res<WorldGrid>) {
    let width = grid.width() as f32;
    let height = grid.height() as f32;
    if width == 0.0 || height == 0.0 {
        return;
    }

    let origin = Vec2::new(-width / 2.0, -height / 2.0);
    let color = Color::srgba(0., 1., 0., 0.75);

    for x in 0..=grid.width() {
        let x_pos = origin.x + x as f32;
        gizmos.line_2d(
            Vec2::new(x_pos, origin.y),
            Vec2::new(x_pos, origin.y + height),
            color,
        );
    }

    for y in 0..=grid.height() {
        let y_pos = origin.y + y as f32;
        gizmos.line_2d(
            Vec2::new(origin.x, y_pos),
            Vec2::new(origin.x + width, y_pos),
            color,
        );
    }
}
