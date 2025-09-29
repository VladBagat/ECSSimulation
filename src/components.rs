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
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Movement {
    pub speed: f32,
    pub direction: f32,
}

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

#[derive(Bundle)]
pub struct VisualBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
}
