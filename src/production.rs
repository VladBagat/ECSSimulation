pub struct ProductionSystems;
use crate::*;
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_event;

impl Plugin for ProductionSystems {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, test_setup_production)
            .add_systems(Update, produce_resource.run_if(on_event::<WorldTick>))
            .add_systems(Update, show_business_ui);
    }
}

#[derive(Component, Default)]
struct Storage {
    food: i32,
    money: i32
}

#[derive(Component)]
struct Workstation {
    current_work: f32,
    total_work: f32,
}

#[derive(Component)]
struct Bussiness;

#[derive(Component)]
struct PlayerOwned;

// === UI components for Business HUD ===
#[derive(Component)]
struct BusinessUiRoot;

#[derive(Component)]
struct BusinessUiEntry {
    target: Entity,
}

fn test_setup_production(
    mut commands : Commands
){
    let business_id = commands.spawn((EntityLabel("Bussiness".to_string()), Bussiness, PlayerOwned, Storage {food:0, money:0})).id();
    let ws1 = commands.spawn((EntityLabel("Work50".to_string()), Workstation { current_work: 0.0, total_work: 50.0})).id();
    let ws2 = commands.spawn((EntityLabel("Work25".to_string()), Workstation { current_work: 0.0, total_work: 25.0})).id();
    commands.entity(business_id).add_children(&[ws1, ws2]);
    let business_id = commands.spawn((EntityLabel("Bussiness2".to_string()), Bussiness, PlayerOwned, Storage {food:0, money:0})).id();
    let ws1 = commands.spawn((EntityLabel("Work50".to_string()), Workstation { current_work: 0.0, total_work: 50.0})).id();
    let ws2 = commands.spawn((EntityLabel("Work25".to_string()), Workstation { current_work: 0.0, total_work: 25.0})).id();
    commands.entity(business_id).add_children(&[ws1, ws2]);
}

fn produce_resource(
    mut workstation_query: Query<(&mut Workstation, &ChildOf)>,
    mut storages_query: Query<&mut Storage>,
) {
    let given_produce_per_worker = 2.0;
    let workers = 5.0;
    let product_to_produce = 1.0;

    for (mut workstation, business) in &mut workstation_query {
        let new_current_work = workstation.current_work + (given_produce_per_worker * workers);
        if new_current_work >= workstation.total_work {
            let items_produced = (new_current_work / workstation.total_work).floor();
            workstation.current_work = new_current_work - (workstation.total_work * items_produced);
            if let Ok(mut storage) = storages_query.get_mut(business.parent()) {
                storage.food += (product_to_produce * items_produced) as i32;
            }
        } else {
            workstation.current_work = new_current_work;
        }
    }
}

fn show_business_ui(
    mut commands: Commands,
    player_businesses: Query<(Entity, Option<&EntityLabel>, &Storage), (With<PlayerOwned>, With<Bussiness>)>,
    root_query: Query<Entity, With<BusinessUiRoot>>,
    entry_query: Query<(Entity, &BusinessUiEntry)>,
    mut text_query: Query<&mut Text>,
) {
    // Ensure there is a UI root anchored to the top-right
    let root_entity = match root_query.single() {
        Ok(e) => e,
        Err(_) => {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        right: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(4.0),
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BusinessUiRoot,
                ))
                .id()
        }
    };

    // Build a lookup of existing UI entries by target entity
    use std::collections::HashMap;
    let mut existing_entries: HashMap<Entity, Entity> = HashMap::new();
    for (entry_entity, entry) in &entry_query {
        existing_entries.insert(entry.target, entry_entity);
    }

    // Track which targets we saw this frame to remove stale entries
    let mut seen_targets: Vec<Entity> = Vec::new();

    // For each player-owned business, spawn/update an entry
    for (biz_entity, label_opt, storage) in &player_businesses {
        seen_targets.push(biz_entity);
        let name = label_opt.map(|l| l.0.clone()).unwrap_or_else(|| format!("Business {:?}", biz_entity));
        let line = format!("{}  |  Food: {}   Money: {}", name, storage.food, storage.money);

        if let Some(entry_entity) = existing_entries.get(&biz_entity).copied() {
            // Update existing text
            if let Ok(mut text) = text_query.get_mut(entry_entity) {
                // In Bevy 0.16, Text is a tuple struct with the content at index 0
                text.0 = line;
            }
        } else {
            // Create a new entry as a child of the root
            commands.entity(root_entity).with_children(|parent| {
                parent
                    .spawn((
                        Text::new(line),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                        BusinessUiEntry { target: biz_entity },
                    ));
            });
        }
    }

    // Remove UI entries for businesses that no longer exist
    for (entry_entity, entry) in &entry_query {
        if !seen_targets.contains(&entry.target) {
            commands.entity(entry_entity).despawn();
        }
    }
}