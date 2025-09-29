use bevy::prelude::*;

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

#[derive(Component)]
pub struct Destination(pub Vec2);

#[derive(Component)]
pub struct Food(pub f32);

#[derive(Bundle)]
pub struct CharacterBundle {
    pub name: Name,
    pub health: Health,
    pub hunger: Hunger,
    pub thirst: Thirst,
    pub sleep: Sleep,
    pub speed: Speed,
    pub destination: Destination,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
}

#[derive(Bundle)]
pub struct FoodBundle {
    pub name: Name,
    pub food: Food,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
}
