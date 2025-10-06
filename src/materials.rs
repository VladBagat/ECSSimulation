use bevy::prelude::*;

// Shared material handles to reuse across systems
#[derive(Resource, Clone)]
pub struct CommonMaterials {
    pub hero: Handle<ColorMaterial>,
    pub food: Handle<ColorMaterial>,
    pub building: Handle<ColorMaterial>,
    pub green_half: Handle<ColorMaterial>,
    pub red_half: Handle<ColorMaterial>,
}

// Initialize and register the shared materials
pub fn setup_common_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hero = materials.add(ColorMaterial::from(Color::hsl(200., 0.95, 0.5)));
    let food = materials.add(ColorMaterial::from(Color::hsl(21., 1., 0.356)));
    // Keeping the original building color choice from the code
    let building = materials.add(ColorMaterial::from(Color::srgb(50., 50., 0.)));

    // Common semi-transparent helpers
    let green_half = materials.add(ColorMaterial::from(Color::srgba(0., 1., 0., 0.5)));
    let red_half = materials.add(ColorMaterial::from(Color::srgba(1., 0., 0., 0.5)));

    commands.insert_resource(CommonMaterials {
        hero,
        food,
        building,
        green_half,
        red_half,
    });
}
