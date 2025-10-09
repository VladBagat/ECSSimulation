mod components;
mod materials;
mod world_grid;
mod states;
mod building;
mod production;

use std::time::Duration;
use crate::{building::BuildingControlState, production::ProductionSystems};

use bevy::{input::mouse::{MouseMotion, MouseWheel}, math::ops::powf, platform::collections::HashSet, prelude::{Name, *}, render::view::RenderLayers};
use bevy_lunex::{*, prelude::*};
use components::{*, Velocity};
use materials::{CommonMaterials, setup_common_materials};
use world_grid::WorldGrid;
use rand::Rng;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode};
use bevy::ecs::schedule::common_conditions::on_event; // added

use crate::states::GameControlState;

pub struct Movement;
pub struct Visual;
pub struct CameraControls;

pub struct HumanPlugins;
pub struct GameBuildingPlugins;
pub struct GameDefaultPlugins;

#[derive(Resource, Default, Debug, Clone, Copy)]
struct DebugOptions {
    enabled: bool,
}

#[derive(Component)]
struct BuildingUi;

#[derive(Event, Debug, Clone, Copy)]
struct CursorWorldEvent {
    screen: Vec2,
    world: Vec2,
    grid: Vec2,
}

// Add a shared world tick event
#[derive(Event, Debug, Clone, Copy)]
struct WorldTick;

#[derive(Resource, Default)]
struct UiBlockHoverCount(pub usize);

#[derive(Resource)]
struct WorldTimer(Timer);

#[derive(Resource)]
struct LongBehaviourTimer(Timer);

fn main() {
    let dbg_enabled = std::env::args().any(|a| a == "--dbg" || a == "--debug" || a == "-d")
        || std::env::var("DBG").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false)
        || std::env::var("BEVY_DEBUG").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

    let mut app = App::new();
    app
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(WorldTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(LongBehaviourTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(WorldGrid::new(160, 160, 25))
        .insert_resource(DebugOptions { enabled: dbg_enabled })
        .add_plugins((DefaultPlugins, UiLunexPlugins))
        .add_plugins(Visual)
        .add_plugins(Movement)
        .add_plugins(CameraControls)
        .add_plugins((GameDefaultPlugins, GameBuildingPlugins))
        .add_plugins(ProductionSystems)
        //.add_plugins(HumanPlugins)
        .insert_state(GameControlState::Default);

    if dbg_enabled {
        // Enable Rapier's debug render if debug mode is on.
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.add_systems(Update, (draw_world_grid, draw_grid_enum));
        println!("[DBG] Debug mode enabled (RapierDebugRenderPlugin active)");
    }

    app.run();
}

impl Plugin for Movement {
    fn build(&self, app: &mut App) {
        app
            .add_event::<WorldTick>()
            .add_systems(Startup, (setup_common_materials).chain())
            .add_systems(Update, world_tick_emitter)
            .add_systems(
                Update,
                (
                    update_hunger.run_if(on_event::<WorldTick>),
                    update_thirst.run_if(on_event::<WorldTick>),
                    update_sleep.run_if(on_event::<WorldTick>),
                ),
            );
    }
}

impl Plugin for CameraControls {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiBlockHoverCount::default())
            .add_event::<CursorWorldEvent>()
            .add_systems(Update,
                (
                    track_mouse_world_position,
                    cursor_event_to_state, 
                    camera_controls,
                ),
            );
    }
}

impl Plugin for Visual {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup,  visual_setup);
    }
}

impl Plugin for HumanPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(AutomaticUpdate::<FoodTracking>::new()
            .with_frequency(Duration::from_secs_f32(1.))
            .with_transform(TransformMode::GlobalTransform)
            .with_spatial_ds(SpatialStructure::KDTree2))
        .add_systems(Startup, (add_animal, add_food))
        .add_systems(Update, (handle_food_collisions, food_search))
        .add_systems(FixedUpdate, update_movement);
    }
}

impl Plugin for GameDefaultPlugins {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, game_state_control_default.run_if(in_state(GameControlState::Default)));
    }
}

fn game_state_control_default(
    mut next_state: ResMut<NextState<GameControlState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        next_state.set(GameControlState::Building)
    }
}

fn visual_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d, UiSourceCamera::<0>,
        Transform::from_translation(Vec3::Z * 1000.0),
        RenderLayers::from_layers(&[0, 1]),
        Projection::from(OrthographicProjection {
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
    ));
}

fn food_search(
    tree: Res<KDTree2<FoodTracking>>,
    query: Query<(Entity, &Transform), (With<Speed>, With<Destination>)>,
    mut destination_query: Query<&mut Destination>,
    mut gizmos: Gizmos
) {
    let radius: f32 = 75.; //TODO: This should be some entity stat. preferable with variability
    let color = Color::srgba(0.75, 0.75, 0., 0.75); // Move all gizmos to a separate system?
    for (hero_entity, hero_pos) in query {
        let origin = hero_pos.translation.truncate();
        gizmos.circle_2d(origin, radius, color);
        let objects_prox = tree.within_distance(origin, radius);
        if objects_prox.len() != 0 {
            let mut items: Vec<(f32, Vec2, Option<Entity>)> = objects_prox
                .into_iter()
                .map(|(v, e)| (origin.distance(v), v, e))
                .collect();

            //Sorts all entities found by within_distance
            items.sort_by(|a, b| a.0.total_cmp(&b.0));
            
            //Picking direction for food
            for (_distance, position, _entity) in &items {
                if let Ok(mut dest) = destination_query.get_mut(hero_entity) {
                    dest.0 = *position;
                    break;
                }
            }
            //Debug
            /*for (_distance, position, _entity) in items {
                gizmos.line_2d(position, origin, color);
            }*/
        }
        else {
            //println!("Run out of food!")
        }
    } 
}

fn world_tick_emitter(
    time: Res<Time>,
    mut timer: ResMut<WorldTimer>,
    mut ev: EventWriter<WorldTick>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        ev.write(WorldTick);
    }
}

fn update_hunger(mut query: Query<&mut Hunger>) {
    for mut hunger in &mut query {
        hunger.value = update_parameter(&hunger.value, |x| (hunger.decay)(x));
    }
}

fn update_thirst(mut query: Query<&mut Thirst>) {
    for mut thirst in &mut query {
        thirst.value = update_parameter(&thirst.value, |x| (thirst.decay)(x));
    }
}

fn update_sleep(mut query: Query<&mut Sleep>) {
    for mut sleep in &mut query {
        sleep.value = update_parameter(&sleep.value, |x| (sleep.decay)(x));
    }
}

//TODO: This works fine but needs some tuning to be good
fn update_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Speed, &Destination, &mut Velocity)>
) {
    let slowing_distance = 75.0;

    for (mut transform, speed, destination, mut velocity) in &mut query {
        let delta = destination.0 - transform.translation.truncate();
        let distance = delta.length();
        if distance < 1.0 {
            **velocity = Vec2::ZERO;
            continue;
        }

        let ramped_speed = speed.0 * (distance / slowing_distance);
        let clipped_speed = ramped_speed.min(speed.0);

        let desired_velocity = (clipped_speed / distance) * delta;
        let steering = desired_velocity - **velocity;

        let damping = 1.0;
        **velocity = (**velocity + steering * time.delta_secs()) * damping;

        transform.translation += (**velocity * time.delta_secs()).extend(0.0);
    }
}

//TODO: Add wandering behaviour.
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
    common_materials: Res<CommonMaterials>,
) {
    let names = ["Vlad"];
    let mut rng = rand::rng();
    let mut bundles = Vec::with_capacity(names.len());
    let mesh_handle = meshes.add(Mesh::from(Circle::new(5.0)));
    for n in names {
        let x = rng.random_range(-400.0..=400.0);
        let y = rng.random_range(-400.0..=400.0);
        let character = CharacterBundle {
            name: EntityLabel(n.to_string()),
            health: Health(100.0),
            hunger: Hunger { value: 100.0, decay: |x| x - 1.0 },
            thirst: Thirst { value: 100.0, decay: |x| x - 1.0 },
            sleep: Sleep { value: 100.0, decay: |x| x - 1.0 },
            speed: Speed(50.0),
            velocity: Velocity(Vec2::ZERO),
            destination: Destination(Vec2 { x: 0.0, y: 0.0 }),
            tracked: TrackedByKDTree,
        };

        let material_handle = common_materials.hero.clone();
    
        let visuals = VisualBundle {
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle),
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
    common_materials: Res<CommonMaterials>,
) {
    let mut rng = rand::rng();
    let mut bundles = Vec::with_capacity(200);
    let mesh_handle = meshes.add(Mesh::from(Circle::new(5.0)));
    let material_handle = common_materials.food.clone();
    for i in 0..200 {
        let x = rng.random_range(-600.0..=600.0);
        let y = rng.random_range(-600.0..=600.0);
        let food = FoodBundle {
            name: EntityLabel(i.to_string()),
            food: Food(30.),
            tracked: FoodTracking,
        };
        let visuals = VisualBundle {
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle.clone()),
            transform: Transform::from_xyz(x, y, 0.0),
        };
        let collision = CollisionBundle::circle_sensor(
            5.0, RigidBody::Fixed, false);
        bundles.push((food, visuals, collision));
    }
    commands.spawn_batch(bundles);
}

fn handle_food_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    food_query: Query<(), With<Food>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _flags) = collision_event {
            for entity in [e1, e2] {
                if food_query.get(*entity).is_ok() {
                    commands.entity(*entity).despawn();
                }
            }
        }
    }
}
fn camera_controls(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut projection)) = query.single_mut() else { return; };

    if buttons.pressed(MouseButton::Left) {
        let mut drag_delta = Vec2::ZERO;
        for ev in mouse_motion_events.read() { drag_delta += ev.delta; }
        if drag_delta != Vec2::ZERO {
            transform.translation.x -= drag_delta.x;
            transform.translation.y += drag_delta.y;
        }
    } else {
        for _ in mouse_motion_events.read() {}
    }

    if let Projection::Orthographic(ortho) = &mut *projection {
        let dt = time.delta_secs();
        for ev in scroll_events.read() {
            if ev.y > 0.0 { // zoom in
                ortho.scale *= powf(0.0625, dt);
            } else if ev.y < 0.0 { // zoom out
                ortho.scale *= powf(16., dt);
            }
        }
        ortho.scale = ortho.scale.clamp(0.05, 50.0);
    } else {
        for _ in scroll_events.read() {}
    }
}

fn draw_grid_enum(grid: Res<WorldGrid>, mut commands: Commands){
    let tile_size = grid.scale() as f32;
    let tiles_wide = grid.width() as f32;
    let tiles_high = grid.height() as f32;
    let total_width = tiles_wide * tile_size;
    let total_height = tiles_high * tile_size;
    let origin = Vec2::new(-total_width / 2.0, -total_height / 2.0);

    for x in 0..=grid.width() {
        for y in 0..=grid.height() {
            let world_x = origin.x + (x as f32 * tile_size) + (tile_size / 2.0);
            let world_y = origin.y + (y as f32 * tile_size) + (tile_size / 2.0);
            
            commands.spawn((
                Text2d::new(format!("{x}|{y}")),
                TextFont {
                    font_size: 7.5,
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 1.0)
            ));
        }
    }
}

fn draw_world_grid(mut gizmos: Gizmos, grid: Res<WorldGrid>) {
    let tiles_wide = grid.width() as f32;
    let tiles_high = grid.height() as f32;
    let tile_size = grid.scale() as f32;

    if tiles_wide == 0.0 || tiles_high == 0.0 || tile_size == 0.0 {
        panic!("Recieved 0 size world grid")
    }

    let total_width = tiles_wide * tile_size;
    let total_height = tiles_high * tile_size;
    let origin = Vec2::new(-total_width / 2.0, -total_height / 2.0);
    let color = Color::srgba(0., 1., 0., 0.2);

    for x in 0..=grid.width() {
        let x_pos = origin.x + (x as f32 * tile_size);
        gizmos.line_2d(
            Vec2::new(x_pos, origin.y),
            Vec2::new(x_pos, origin.y + total_height),
            color,
        );
    }

    for y in 0..=grid.height() {
        let y_pos = origin.y + (y as f32 * tile_size);
        gizmos.line_2d(
            Vec2::new(origin.x, y_pos),
            Vec2::new(origin.x + total_width, y_pos),
            color,
        );
    }
}

fn track_mouse_world_position(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    grid: Res<WorldGrid>,
    mut ev_writer: EventWriter<CursorWorldEvent>,
) {
    let (camera, camera_transform) = camera_q.single().unwrap();
    let window = windows.single().unwrap();

    if let Some(screen_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            let grid_pos = grid.world_to_grid(world_pos);
            ev_writer.write(CursorWorldEvent { screen: screen_pos, world: world_pos, grid: grid_pos });
        }
    }
}

fn cursor_event_to_state(
    mut events: EventReader<CursorWorldEvent>,
    mut state: ResMut<BuildingControlState>,
) {
    // Use the most recent event this frame, if any
    if let Some(last) = events.read().last().copied() {
        state.cur_cel = last.grid;
    }
}


