use bevy::window::PresentMode;
use bevy::{input::mouse::MouseButtonInput, prelude::*};

use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, Intersection, RayCastMesh, RayCastMethod,
    RayCastSource, RaycastSystem,
};

const GRID_SIZE: u64 = 5;

#[derive(Component)]
struct BlockPosition {
    x: i64,
    y: i64,
    z: i64,
}

impl BlockPosition {
    pub fn into_transform(&self) -> Transform {
        Transform::from_xyz(self.x as f32, self.y as f32, self.z as f32)
    }
}

struct MyRaycastSet;

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<MyRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
    }
}

fn new_cube_from_raycast(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mouse_button_input: EventReader<MouseButtonInput>,
    query: Query<&Intersection<MyRaycastSet>>,
) {
    let intersection = query.get_single().ok();

    if let Some((position, normal)) = intersection.and_then(|i| Some((i.position()?, i.normal()?)))
    {
        if let Some(mouse_button_input) = mouse_button_input.iter().next() {
            if mouse_button_input.button != MouseButton::Left {
                return;
            }
        } else {
            return;
        }

        mouse_button_input.clear();

        println!("Clicked and got an intersection");

        let mut offset_x = 0.0;
        let mut offset_y = 0.0;
        let mut offset_z = 0.0;

        if normal.x > 0.0 {
            offset_x = 0.5;
        } else if normal.x < 0.0 {
            offset_x = -0.5;
        }

        if normal.y > 0.0 {
            offset_y = 0.5;
        } else if normal.y < 0.0 {
            offset_y = -0.5;
        }

        if normal.z > 0.0 {
            offset_z = 0.5;
        } else if normal.z < 0.0 {
            offset_z = -0.5;
        }

        let rough_cube_position = *position + Vec3::new(offset_x, offset_y, offset_z);

        // Rounding takes care of the good positionning of the cube
        let cube_position = BlockPosition {
            x: rough_cube_position.x as i64,
            y: rough_cube_position.y as i64,
            z: rough_cube_position.z as i64,
        };

        let cube_transform = cube_position.into_transform();

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
                transform: cube_transform,
                ..default()
            })
            .insert(cube_position)
            .insert(RayCastMesh::<MyRaycastSet>::default());
    }
}

fn setup_base_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(DefaultPluginState::<MyRaycastSet>::default().with_debug_cursor());

    let floor_tile = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(Color::rgb(0.1, 0.8, 0.1).into()),
        ..default()
    };

    for x in 0..GRID_SIZE {
        for z in 0..GRID_SIZE {
            let position = BlockPosition {
                x: x as i64,
                y: 0,
                z: z as i64,
            };

            let floor_tile = PbrBundle {
                transform: position.into_transform(),
                ..floor_tile.clone()
            };

            commands
                .spawn_bundle(floor_tile.clone())
                .insert(position)
                .insert(RayCastMesh::<MyRaycastSet>::default());
        }
    }

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });

    commands
        .spawn_bundle(Camera3dBundle {
            projection: bevy::render::camera::Projection::Orthographic(OrthographicProjection {
                scale: 0.01,
                ..default()
            }),
            transform: Transform::from_xyz(-5.0, 10.0, -5.0)
                .looking_at(Vec3::new(2.5, 0.0, 2.5), Vec3::Y),
            ..default()
        })
        .insert(RayCastSource::<MyRaycastSet>::new()); // Designate the camera as our source
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
        )
        .add_startup_system(setup_base_grid)
        .add_system(new_cube_from_raycast)
        .run();
}
