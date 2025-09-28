use bevy::prelude::{Component, Bundle};

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

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Movement {
    pub speed: f32,
    pub direction: f32,
}

// Group commonly used components for a character/entity into a bundle
#[derive(Bundle)]
pub struct CharacterBundle {
    pub name: Name,
    pub health: Health,
    pub hunger: Hunger,
    pub thirst: Thirst,
    pub sleep: Sleep,
    pub position: Position,
    pub movement: Movement,
}
