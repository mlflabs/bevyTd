use bevy::prelude::*;

use crate::{
    assets::GameAssets,
    state::{GameState, InAppRunning},
};
use bevy_gltf_blueprints::{BluePrintBundle, BlueprintName, GameWorldTag};

// use bevy_rapier3d::prelude::Velocity;
use bevy_xpbd_3d::prelude::*;

use rand::Rng;

pub fn setup_game(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    models: Res<Assets<bevy::gltf::Gltf>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    println!("setting up all stuff");
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
    // here we actually spawn our game world/level

    commands.spawn((
        SceneBundle {
            // note: because of this issue https://github.com/bevyengine/bevy/issues/10436, "world" is now a gltf file instead of a scene
            scene: models
                .get(game_assets.world.id())
                .expect("main level should have been loaded")
                .scenes[0]
                .clone(),
            ..default()
        },
        bevy::prelude::Name::from("world"),
        GameWorldTag,
        InAppRunning,
    ));

    next_game_state.set(GameState::InGame)
}

pub fn spawn_test(
    keycode: Res<Input<KeyCode>>,
    mut commands: Commands,

    mut game_world: Query<(Entity, &Children), With<GameWorldTag>>,
) {
    if keycode.just_pressed(KeyCode::T) {
        let world = game_world.single_mut();
        let world = world.1[0];

        let mut rng = rand::thread_rng();
        let range = 5.5;
        let x: f32 = rng.gen_range(-range..range);
        let y: f32 = rng.gen_range(-range..range);

        let mut rng = rand::thread_rng();
        let range = 0.8;
        let vel_x: f32 = rng.gen_range(-range..range);
        let vel_y: f32 = rng.gen_range(2.0..2.5);
        let vel_z: f32 = rng.gen_range(-range..range);

        let name_index: u64 = rng.gen();

        let new_entity = commands
            .spawn((
                BluePrintBundle {
                    blueprint: BlueprintName("Health_Pickup".to_string()),
                    transform: TransformBundle::from_transform(Transform::from_xyz(x, 2.0, y)),
                    ..Default::default()
                },
                bevy::prelude::Name::from(format!("test{}", name_index)),
                // BlueprintName("Health_Pickup".to_string()),
                // SpawnHere,
                // TransformBundle::from_transform(Transform::from_xyz(x, 2.0, y)),
                LinearVelocity(Vec3::new(vel_x, vel_y, vel_z)),
                AngularVelocity::ZERO,
            ))
            .id();
        commands.entity(world).add_child(new_entity);
    }
}
