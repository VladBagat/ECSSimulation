use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Hunger{
    pub value: f32,
    pub decay: fn(&f32) -> f32,
}

#[derive(Component)]
pub struct Thirst{
    pub value: f32,
    pub decay: fn(&f32) -> f32,
}

#[derive(Component)]
pub struct Sleep{
    pub value: f32,
    pub decay: fn(&f32) -> f32,
}

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component, DerefMut, Deref)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Destination(pub Vec2);

#[derive(Component)]
pub struct Food(pub f32);

#[derive(Component)]
pub struct TrackedByKDTree;

#[derive(Bundle)]
pub struct CharacterBundle {
    pub name: Name,
    pub health: Health,
    pub hunger: Hunger,
    pub thirst: Thirst,
    pub sleep: Sleep,
    pub speed: Speed,
    pub velocity: Velocity,
    pub destination: Destination,
    pub tracked: TrackedByKDTree,
}

#[derive(Bundle)]
pub struct FoodBundle {
    pub name: Name,
    pub food: Food,
    pub tracked: TrackedByKDTree,
}

#[derive(Bundle)]
pub struct VisualBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
}

#[derive(Bundle)]
pub struct CollisionBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub collision_types: ActiveCollisionTypes,
    pub rigidbody: RigidBody,
    pub active_events: ActiveEvents,
}

impl CollisionBundle {
    pub fn circle_sensor(radius: f32, rb_type: RigidBody, events: bool) -> Self {
        Self {
            collider: Collider::ball(radius),
            sensor: Sensor,
            collision_types: ActiveCollisionTypes::all(),
            active_events: if events {ActiveEvents::COLLISION_EVENTS } else { ActiveEvents::empty() },
            rigidbody: rb_type
        }
    }
}
