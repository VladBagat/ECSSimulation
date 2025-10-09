pub struct ProductionSystems;
use crate::*;

impl Plugin for ProductionSystems {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TempStorage {food:0, money:0})
            .add_systems(Startup, test_setup_production)
            .add_systems(Update, produce_resource.run_if(on_event::<WorldTick>));
    }
}

#[derive(Resource)]
struct TempStorage {
    food: i32,
    money: i32
}

#[derive(Component)]
struct Workstation {
    current_work: f32,
    total_work: f32 // move out to product once it exists
}

fn test_setup_production(
    mut commands : Commands
){
    commands.spawn((EntityLabel("Work50".to_string()), Workstation { current_work: 0.0, total_work: 50.0}));
    commands.spawn((EntityLabel("Work25".to_string()), Workstation { current_work: 0.0, total_work: 25.0}));
}

fn produce_resource(
    mut workstation_query: Query<(&mut Workstation, &EntityLabel)>,
    mut storage: ResMut<TempStorage>
) {
    let given_produce_per_worker = 2.0;
    let workers = 5.0;
    let product_to_produce = 1.0;

    for (mut workstation, _name) in &mut workstation_query {
        let new_current_work = workstation.current_work + (given_produce_per_worker * workers);
        if new_current_work >= workstation.total_work {
            let items_produced = (new_current_work / workstation.total_work).floor();
            workstation.current_work = new_current_work - (workstation.total_work * items_produced);
            storage.food += (product_to_produce * items_produced) as i32;
        } else {
            workstation.current_work = new_current_work;
        }
    }
}