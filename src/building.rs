use crate::*;


#[derive(Event)]
struct RequestSpawnBuildingTemplate {
    size: Vec2,
    pos: Vec2,
}

#[derive(Resource)]
pub struct BuildingControlState{
    pub cur_cel: Vec2,
    pub cur_building: Option<Entity>,
    pub cur_size: Vec2,
    pub overlaps: HashSet<Entity>,
}

impl Plugin for GameBuildingPlugins {
    fn build(&self, app: &mut App) {
        app
        .add_event::<RequestSpawnBuildingTemplate>()
        .insert_resource(BuildingControlState {
            cur_cel: Vec2::default(),
            cur_building: None,
            overlaps: HashSet::default(),
            cur_size: Vec2::default()
        })
        .add_systems(Update,
             (select_building, handle_building_collisions, game_state_control_building, building_prototype, create_building_template)
            .run_if(in_state(GameControlState::Building)))
        .add_systems(OnExit(GameControlState::Building), state_cleanup_building)
        .add_systems(OnEnter(GameControlState::Building), state_ui_startup_building);
    }
}

fn building_prototype(
    mut state: ResMut<BuildingControlState>,
    m_buttons: Res<ButtonInput<MouseButton>>,
    mut grid: ResMut<WorldGrid>,
    common_materials: Res<CommonMaterials>,
    mut query: Query<&mut Transform>,
    mut material_query: Query<&mut MeshMaterial2d<ColorMaterial>>,
    over_ui: Res<UiBlockHoverCount>
){
    // this function will eventually be stripped out because none of its behaviour is desired
    let origin = state.cur_cel;
    let pos = grid.grid_to_world(origin, state.cur_size);
    if let Some(building) = state.cur_building {
        if let Ok(mut transform) = query.get_mut(building) {
            transform.translation = vec3(pos.x, pos.y, 0.0);
        }
        
        if m_buttons.just_pressed(MouseButton::Left) && state.overlaps.is_empty() && over_ui.0 <= 0 {
            //also triggers when trying to drag camera. 
            if let Ok(mut material) = material_query.get_mut(building){
                material.0 = common_materials.building.clone();
                state.cur_building = None;
                grid.modify_rectangle(origin, state.cur_size);
            }
        }
    }
}

fn create_building_template (
    mut events: EventReader<RequestSpawnBuildingTemplate>,
    grid: Res<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    common_materials: Res<CommonMaterials>,
    mut commands: Commands,
    mut state: ResMut<BuildingControlState>
) {
    for ev in events.read() {
        let mesh_handle = meshes.add(Mesh::from(
                Rectangle::new(ev.size.x * grid.scale() as f32,
                    ev.size.y * grid.scale() as f32)));
        let material_handle = common_materials.green_half.clone();
        let visual = VisualBundle{
            mesh: Mesh2d(mesh_handle.clone()),
            material: MeshMaterial2d(material_handle.clone()),
            transform: Transform::from_xyz(ev.pos.x, ev.pos.y, 0.0)
        };
        let collision = CollisionBundle::rect_sensor(
            (ev.size - 0.01 )* grid.scale() as f32, RigidBody::Fixed, true);
        let ent = commands.spawn(BuildingBundle { visual, collision, building: Building });
        state.cur_building = Some(ent.id());
    }
    events.clear();
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
        }
        else {
            return;
        }
        let rapier_context = rapier_context.single().unwrap();
        let ray_dir = Vec2::new(0.,0.);
        let max_toi = 99999.;
        let solid = true;
        let filter = QueryFilter::default();

        if let Some((_entity, toi)) = rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
            let _hit_point = ray_pos + ray_dir * toi;
            //for now let any object get the context menu
            // TODO: Repurpose this for event emmiter and create separate building context menu event subscriber.
            
        }
    }
}

fn game_state_control_building(
    mut next_state: ResMut<NextState<GameControlState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        next_state.set(GameControlState::Default)
    }
}

fn state_cleanup_building(
    mut state: ResMut<BuildingControlState>,
    mut commands: Commands,
    ui_query: Query<Entity, With<BuildingUi>>,
){
    if let Some(building) = state.cur_building {
        commands.entity(building).despawn();
        state.cur_building = None;
    }

    for e in &ui_query {
        commands.entity(e).despawn();
    }
}


// TODO: UI doesn't scale with camera 
fn state_ui_startup_building(
    camera_q: Query<Entity, With<MainCamera>>,
    mut commands: Commands,
) {
    if let Ok(camera) = camera_q.single() {
        commands.entity(camera).with_children(|cam| {
            cam.spawn((
                Name::new("UI Root"),
                UiLayoutRoot::new_2d(),
                UiFetchFromCamera::<0>,
                BuildingUi,
            ))
            .with_children(|ui| {
                ui.spawn((
                    Name::new("Background"),
                    UiLayout::window()
                        .anchor(Anchor::BottomCenter)
                        .pos(Rl((50.0, 100.0)))
                        .size((600.0, 75.0))
                        .pack(),
                    Sprite::from_color(
                        Color::srgba(0.5, 0.5, 0.5, 1.0),
                        Vec2::new(50.0, 50.0),
                    ),
                ))
                .observe(|_: Trigger<Pointer<Over>>, mut cnt: ResMut<UiBlockHoverCount>| {
                    cnt.0 += 1;
                })
                .observe(|_: Trigger<Pointer<Out>>, mut cnt: ResMut<UiBlockHoverCount>| {
                    if cnt.0 > 0 { cnt.0 -= 1; }
                })
                .with_children(|ui| {
                    ui.spawn((
                        Name::new("Stuff"),
                        UiLayout::window()
                            .anchor(Anchor::Center)
                            .pos(Rl((20.0, 50.0)))
                            .size((50.0, 50.0))
                            .pack(),
                        Sprite::from_color(
                            Color::srgba(1.0, 0.0, 0.0, 1.0),
                            Vec2::new(50.0, 50.0),
                        ),
                        OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                    ))
                    .observe(|_: Trigger<Pointer<Click>>,
                        mut spawn_ev: EventWriter<RequestSpawnBuildingTemplate>,
                        mut state: ResMut<BuildingControlState>,
                        grid: Res<WorldGrid>| {
                        let origin = state.cur_cel;
                        state.cur_size = Vec2::new(4.0, 2.0);
                        let pos = grid.grid_to_world(origin, state.cur_size);
                        spawn_ev.write(RequestSpawnBuildingTemplate { size: state.cur_size, pos: pos });
                    });

                    ui.spawn((
                        Name::new("Stuff2"),
                        UiLayout::window()
                            .anchor(Anchor::Center)
                            .pos(Rl((30.0, 50.0)))
                            .size((50.0, 50.0))
                            .pack(),
                        Sprite::from_color(
                            Color::srgba(0.0, 1.0, 0.0, 1.0),
                            Vec2::new(50.0, 50.0),
                        ),
                        OnHoverSetCursor::new(SystemCursorIcon::Pointer),
                    ))
                    .observe(|_: Trigger<Pointer<Click>>,
                        mut spawn_ev: EventWriter<RequestSpawnBuildingTemplate>,
                        mut state: ResMut<BuildingControlState>,
                        grid: Res<WorldGrid>| {
                        let origin = state.cur_cel;
                        state.cur_size = Vec2::new(4.0, 5.0);
                        let pos = grid.grid_to_world(origin, state.cur_size);
                        spawn_ev.write(RequestSpawnBuildingTemplate { size: state.cur_size, pos: pos });
                    });
                });
            });
        });
    }
}


fn handle_building_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    building_query: Query<(), With<Building>>,
    common_materials: Res<CommonMaterials>,
    mut state: ResMut<BuildingControlState>,
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
