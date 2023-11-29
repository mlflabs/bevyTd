pub mod in_game;
use std::time::Duration;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle;
use bevy_gltf_blueprints::{AnimationPlayerLink, Animations};
use bevy_xpbd_3d::prelude::{
    Collision, CollisionEnded, CollisionStarted};

use bevy_xpbd_3d::{
    math::*, prelude::*, PhysicsSchedule, PhysicsStepSet, SubstepSchedule, SubstepSet,
};

use seldom_state::prelude::StateMachine;
use smooth_bevy_cameras::LookTransformPlugin;

pub use in_game::*;

pub mod in_main_menu;
pub use in_main_menu::*;

pub mod picking;
pub use picking::*;

use crate::{
    insert_dependant_component,
    state::{AppState, GameState},
};
use bevy::prelude::*;


// pub mod controller_player;
// pub use controller_player::*;


pub mod plugin_player;
pub use plugin_player::*;

// pub mod controller_character;
// pub use controller_character::*;

pub mod controller_camera;
pub use controller_camera::*;

// this file is just for demo purposes, contains various types of components, systems etc

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub enum SoundMaterial {
    Metal,
    Wood,
    Rock,
    Cloth,
    Squishy,
    #[default]
    None,
}



#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo component showing auto injection of components
pub struct ShouldBeWithPlayer;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Interactible;



// collision tests/debug
pub fn test_collision_events(
    mut collision_started_events: EventReader<CollisionStarted>,
    mut collision_ended_events: EventReader<CollisionEnded>,
) {
    for CollisionStarted(entity1, entity2) in collision_started_events.read() {
        println!("collision started")
    }

    for CollisionEnded(entity1, entity2) in collision_ended_events.read() {
        println!("collision ended")
    }
}


#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Fox;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Robot;


pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                PickingPlugin, 
                PlayerPlugin,
                //CharacterControllerPlugin, 
                LookTransformPlugin
            ))
            .register_type::<Interactible>()
            .register_type::<SoundMaterial>()
            .register_type::<Player>()
            .register_type::<Robot>()
            .register_type::<Fox>()
            // little helper utility, to automatically inject components that are dependant on an other component
            // ie, here an Entity with a Player component should also always have a ShouldBeWithPlayer component
            // you get a warning if you use this, as I consider this to be stop-gap solution (usually you should have either a bundle, or directly define all needed components)
            
            
            //.add_systems(PhysicsSchedule, character_movement.before(PhysicsStepSet::BroadPhase))
            // .add_systems(
            //     // Run collision handling in substep schedule
            //     SubstepSchedule,
            //     kinematic_collision.in_set(SubstepSet::SolveUserConstraints),
            // )
            .add_systems(Startup, (camera_setup, ))
            //.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    // insert_dependant_component::<Player, ShouldBeWithPlayer>,
                    //player_move_demo, //.run_if(in_state(AppState::Running)),
                    // setup_player_controller,
                    
                    fox_test,


                    move_camera_system,
                    //camera_setup, 
                    test_collision_events,
                    spawn_test,
                )
                    .run_if(in_state(GameState::InGame)), 
            )
            .add_systems(OnEnter(AppState::MenuRunning), setup_main_menu)
            .add_systems(OnExit(AppState::MenuRunning), teardown_main_menu)
            .add_systems(Update, (main_menu))
            .add_systems(OnEnter(AppState::AppRunning), setup_game);
    }
}
 


fn setup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 4.0, 12.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        })
        .insert(BloomSettings {
            intensity: 0.1,
            ..default()
        })
        .insert(TemporalAntiAliasBundle::default())
        .insert(Name::new("MainCamera"));
        //.insert(PlayerFollowingCamera);

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 0.2,
    // });
}


pub fn fox_test(
    animated_foxes: Query<(&AnimationPlayerLink, &Animations), With<Fox>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    keycode: Res<Input<KeyCode>>,
    // mut entities_with_animations : Query<(&mut AnimationPlayer, &mut Animations)>,
) {
    // robots
    if keycode.just_pressed(KeyCode::B) {
        println!("HIT BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB");
        for (link, animations) in animated_foxes.iter() {
            println!("------------------Hello Fox Animation");
            let mut animation_player = animation_players.get_mut(link.0).unwrap();
            let anim_name = "Survey";

            for key in animations.named_animations.keys() {
                println!("Animation: {}", key);
            }

                
            animation_player
                .play_with_transition(
                    animations
                        .named_animations
                        .get(anim_name)
                        .expect("animation name should be in the list")
                        .clone(),
                    Duration::from_secs(0),
                )
                .repeat();
        }
    
    }
}







