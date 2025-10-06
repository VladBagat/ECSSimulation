mod components;
mod materials;
mod world_grid;

use core::f32;
use std::time::Duration;

use bevy::{input::mouse::{MouseMotion, MouseWheel}, math::ops::powf, platform::collections::HashSet, prelude::*};
use components::{*, Velocity};
use materials::{CommonMaterials, setup_common_materials};
use world_grid::WorldGrid;
use rand::Rng;
use bevy_rapier2d::prelude::*;
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode};

pub struct Movement;

pub struct CameraControls;

// Global cursor event so multiple systems can subscribe without relying on shared state
#[derive(Event, Debug, Clone, Copy)]
struct CursorWorldEvent {
    /// Cursor position in window/screen coordinates (pixels)
    screen: Vec2,
    /// Cursor position in world coordinates
    world: Vec2,
    /// Cursor position snapped to grid cells (in grid coordinates)
    grid: Vec2,
}

#[derive(Resource)]
struct WorldTimer(Timer);

#[derive(Resource)]
struct TempState{
    building_mode: bool,
    cur_cel: Vec2,
    cur_building: Option<Entity>,
    overlaps: HashSet<Entity>,
}

#[derive(Resource)]
struct LongBehaviourTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Movement)
        .add_plugins(CameraControls)
        .run();
}

impl Plugin for Movement {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(AutomaticUpdate::<FoodTracking>::new()
            .with_frequency(Duration::from_secs_f32(1.))
            .with_transform(TransformMode::GlobalTransform)
            .with_spatial_ds(SpatialStructure::KDTree2))
        .insert_resource(WorldTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(LongBehaviourTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .insert_resource(WorldGrid::new(160, 160, 25))
        .insert_resource(TempState {
            building_mode: false,
            cur_cel: Vec2::default(),
            cur_building: None,
            overlaps: HashSet::default()
        })
        .add_systems(Startup, ((setup_common_materials, visual_setup, add_animal, add_food, draw_grid_enum)).chain())
        .add_systems(Update, (update_hunger, update_thirst, update_sleep))
        .add_systems(Update, (handle_food_collisions, handle_building_collisions, test_tree, draw_world_grid, select_building))
        .add_systems(FixedUpdate, (update_movement).chain());
    }
}

impl Plugin for CameraControls {
    fn build(&self, app: &mut App) {
        app
            // Register a global event for cursor world/grid position
            .add_event::<CursorWorldEvent>()
            // Systems
            .add_systems(
                Update,
                (
                    track_mouse_world_position, // emit cursor event
                    cursor_event_to_state,       // keep TempState.cur_cel in sync for legacy users
                    camera_controls,
                    building_prototype,
                ),
            );
    }
}

fn test_tree(
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
            for (_distance, position, _entity) in items {
                gizmos.line_2d(position, origin, color);
            }
        }
        else {
            //println!("Run out of food!")
        }
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
            name: Name(n.to_string()),
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
            name: Name(i.to_string()),
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
fn handle_building_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    building_query: Query<(), With<Building>>,
    common_materials: Res<CommonMaterials>,
    mut state: ResMut<TempState>,
    mut material_query: Query<&mut MeshMaterial2d<ColorMaterial>>
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _flags) = collision_event {
            if building_query.get(*e1).is_ok() && building_query.get(*e2).is_ok() {
                material_query.get_mut(state.cur_building.unwrap()).unwrap().0 = common_materials.red_half.clone();
                for entity in [e1, e2] {
                    if state.cur_building.is_none() || *entity == state.cur_building.unwrap() {continue;}
                    state.overlaps.insert(*entity);
                    break;
                }  
            }
        }   
        else if let CollisionEvent::Stopped(e1, e2, _flags) = collision_event {
            if building_query.get(*e1).is_ok() && building_query.get(*e2).is_ok() {
                for entity in [e1, e2] {
                    if let Some(building) = state.cur_building{
                        if *entity != building {
                            state.overlaps.remove(entity);
                            break;
                        }
                    } 
                }  
                if state.overlaps.is_empty() {
                    material_query.get_mut(state.cur_building.unwrap()).unwrap().0 = common_materials.green_half.clone();
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

fn visual_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection {
            ..OrthographicProjection::default_2d()
        }),
    ));
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
    mut state: ResMut<TempState>,
) {
    // Use the most recent event this frame, if any
    if let Some(last) = events.read().last().copied() {
        state.cur_cel = last.grid;
    }
}

fn building_prototype(
    mut state: ResMut<TempState>,
    keys: Res<ButtonInput<KeyCode>>,
    m_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    common_materials: Res<CommonMaterials>,
    mut query: Query<&mut Transform>,
    mut material_query: Query<&mut MeshMaterial2d<ColorMaterial>>
){
    if keys.just_pressed(KeyCode::KeyB) {
        state.building_mode = !state.building_mode;
        println!("Building mode: {}", state.building_mode);
    }

    if state.building_mode == true {
        let origin = state.cur_cel;
        let building_size = Vec2::new(3.0, 3.0);
        let pos = grid.grid_to_world(origin, building_size);
        if let None = state.cur_building {
            let mesh_handle = meshes.add(Mesh::from(
                Rectangle::new(building_size.x * grid.scale() as f32,
                 building_size.y * grid.scale() as f32)));
            let material_handle = common_materials.green_half.clone();
            let visual = VisualBundle{
                mesh: Mesh2d(mesh_handle.clone()),
                material: MeshMaterial2d(material_handle.clone()),
                transform: Transform::from_xyz(pos.x, pos.y, 0.0)
            };
            let collision = CollisionBundle::rect_sensor(
            (building_size - 0.01 )* grid.scale() as f32, RigidBody::Fixed, true);
            let ent = commands.spawn((visual, collision, Building));
            state.cur_building = Some(ent.id());
        }
        else if let Some(building) = state.cur_building {
            let mut transform = query.get_mut(building).unwrap();
            transform.translation = vec3(pos.x, pos.y, 0.0);

            if m_buttons.just_pressed(MouseButton::Left) && state.overlaps.is_empty(){
                //also triggers when trying to drag camera. 
                let mut material = material_query.get_mut(building).unwrap();
                material.0 = common_materials.building.clone();
                state.cur_building = None;
                grid.modify_rectangle(origin, building_size);
            }
        }
    }
    else {
        if let Some(building) = state.cur_building {
            commands.entity(building).despawn();
            state.cur_building = None;
        }
    }
}

fn select_building(
    rapier_context: ReadRapierContext,
    m_buttons: Res<ButtonInput<MouseButton>>,
    mut events: EventReader<CursorWorldEvent>,
) {
    if m_buttons.just_pressed(MouseButton::Left) {
        let ray_pos ;
        if let Some(last) = events.read().last().copied() {
            ray_pos = last.world;
            println!("{:?}", last.world)
        }
        else {
            return;
        }
        let rapier_context = rapier_context.single().unwrap();
        let ray_dir = Vec2::new(0.,0.);
        let max_toi = 99999.;
        let solid = true;
        let filter = QueryFilter::default();

        if let Some((entity, toi)) = rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
            let hit_point = ray_pos + ray_dir * toi;
            println!("Entity {:?} hit at point {}", entity, hit_point);
        }
    }
}
