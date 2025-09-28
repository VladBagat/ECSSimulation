use bevy::prelude::Component;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Hunger(pub f32);

#[derive(Component)]
pub struct Thirst(pub f32);

#[derive(Component)]
pub struct Sleep(pub f32);

#[derive(Component)]
pub struct Name(pub String);
