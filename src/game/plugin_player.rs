use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_gltf_blueprints::{AnimationPlayerLink, Animations};
use bevy_tnua::{builtins::TnuaBuiltinWalk, controller::TnuaController, TnuaUserControlsSystemSet};
use bevy_tnua::{control_helpers::TnuaCrouchEnforcerPlugin, prelude::*};
use bevy_tnua_xpbd3d::*;
use bevy_xpbd_3d::prelude::*;
use leafwing_input_manager::prelude::*;
use seldom_state::prelude::*;
use seldom_state::trigger::{AndTrigger, OrTrigger};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};


const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_WIDTH: f32 = 1.0;

const MOVEMENT_SPEED: f32 = 16.0;
const JUMP_HEIGHT: f32 = 5.0;
const FLOATING_HEIGHT: f32 = 0.1;
const INTERACT_RAY_TIME: f32 = 1.0; //how long ray can travel, longer time equals longer distance

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Startup, startup);
        build_player_add(app);
        build_movement(app);
        //build_player_camera(app);

        app.add_plugins(StateMachinePlugin::default());
        app.register_type::<CurrentState>();
        //app.add_event::<LadderInteractionBeginEvent>()
       //     .add_event::<LadderInteractionEndEvent>();
        // Required to apply LinearVelocity
        app.add_systems(
            Update,
            apply_deferred
                .after(seldom_state::set::StateSet::Transition)
                .before(TnuaPipelineStages::Motors),
        );
        app.add_systems(Update, player_animation);
    }
} 

fn build_player_add(app: &mut App) {
    app.add_systems(Update, add_player);
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Player;

#[derive(Debug, Clone, Eq, PartialEq, Reflect, Hash)]
enum PlayerStates {
    Idle,
    Move,
    Jump,
}

impl Default for PlayerStates {
    fn default() -> Self {
        Self::Idle
    }
}


#[derive(Component, Clone, Default, Reflect, Debug)]
pub struct CurrentState {
    state: PlayerStates,
}

impl CurrentState {
    pub fn new(state: PlayerStates) -> Self { Self { state } }
}




#[derive(Component, Clone, Debug)]
struct PlayerGrounded;

#[derive(Component, Clone, Debug)]
struct PlayerJumping;

#[derive(Component, Clone, Debug)]
struct PlayerDoubleJumping;


#[derive(Component, Clone, Debug)]
struct PlayerMoving;

#[derive(Component, Clone, Debug)]
struct PlayerRunning;

#[derive(Component)]
struct InteractionRayCaster;






struct IsJumping;

impl Trigger for IsJumping {
    type Param<'w, 's> = Query<'w, 's, &'static TnuaController>;

    type Ok = ();

    type Err = ();

    fn trigger(
        &self,
        entity: Entity,
        query: <<Self as Trigger>::Param<'_, '_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Result<Self::Ok, Self::Err> {
        query
            .get(entity)
            .unwrap()
            .concrete_action::<TnuaBuiltinJump>()
            .map(|_| ())
            .ok_or(())
    }
}



fn add_player(
    mut commands: Commands,
    player: Query<Entity, Added<Player>>,
) {
    for entity in player.iter() {
        commands
            .entity(entity)
            .insert(Name::new("Player"))
            .insert(CurrentState::new(PlayerStates::Idle))
            .insert((
                RigidBody::Dynamic,
            ))
            .insert(LockedAxes::new().lock_rotation_x().lock_rotation_z())
            .insert(TnuaControllerBundle::default())
            .insert(player_state_machine(entity))
            .with_children(|children| {
                // Spawn the child colliders positioned relative to the rigid body
                children.spawn((Collider::capsule(PLAYER_HEIGHT / 4., PLAYER_WIDTH / 4.), Transform::from_xyz(0.0, 0.5, 0.0)));
            })
            .with_children(|builder| {
                // RayCaster for interaction
                builder.spawn((
                    Name::new("InteractionRayCaster"),
                    InteractionRayCaster,
                    RayCaster::new(Vec3::ZERO, Vec3::Z)
                        .with_max_time_of_impact(INTERACT_RAY_TIME)
                        .with_query_filter(
                            SpatialQueryFilter::new().without_entities([builder.parent_entity()]),
                        ),
                    SpatialBundle::default(),
                ));
            });
        add_action_state(commands.entity(entity));
    }
}




fn player_state_machine(entity: Entity) -> impl Bundle {
    let initial = PlayerGrounded;
    let state_machine = StateMachine::default()
        
        //.trans::<PlayerGrounded>(PressedTrigger(Action::Move),PlayerMoving)
        .trans::<PlayerGrounded>(PressedTrigger(Action::Up),PlayerMoving)
        .trans::<PlayerGrounded>(PressedTrigger(Action::Down),PlayerMoving)
        .trans::<PlayerGrounded>(PressedTrigger(Action::Right),PlayerMoving)
        .trans::<PlayerGrounded>(PressedTrigger(Action::Left),PlayerMoving)
        .trans::<PlayerMoving>(PressedTrigger(Action::Move).not(), PlayerGrounded)
        // .trans::<PlayerMoving>(
        //     PressedTrigger(Action::Up).not()
        //     .and(PressedTrigger(Action::Down)).not()
        //     .and(PressedTrigger(Action::Right)).not()
        //     .and(PressedTrigger(Action::Left)).not(),PlayerGrounded)
        
        
        
        .trans::<PlayerGrounded>(JustPressedTrigger(Action::Jump), PlayerJumping)
        .trans::<PlayerMoving>(JustPressedTrigger(Action::Jump), PlayerJumping)
        .trans::<PlayerJumping>(
            AndTrigger(IsJumping, PressedTrigger(Action::Jump)),
            PlayerJumping,
        )
        .trans::<PlayerJumping>(
            AndTrigger(IsJumping.not(), PressedTrigger(Action::Jump).not()),
            PlayerGrounded,
        )
        .on_enter::<PlayerGrounded>(move| entity| {
            entity.insert(CurrentState::new(PlayerStates::Idle));
        })
        .on_enter::<PlayerMoving>(move| entity| {
            entity.insert(CurrentState::new(PlayerStates::Move));
        })
        .on_enter::<PlayerJumping>(move| entity| {
            entity.insert(CurrentState::new(PlayerStates::Jump));
        })
        
        .set_trans_logging(true);
    (initial, state_machine)
}



// Player movement

fn build_movement(app: &mut App) {
    app.add_plugins((
        TnuaXpbd3dPlugin,
        TnuaControllerPlugin,
        TnuaCrouchEnforcerPlugin,
    ))
    .add_plugins(InputManagerPlugin::<Action>::default())
    .add_systems(
        FixedUpdate,
        (player_jumping, player_movement_walk)
            .in_set(TnuaUserControlsSystemSet),
    );
    //.add_systems(Update, player_animation);
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Up,
    Down,
    Left,
    Right,
    Jump,
    Interact,
    Move,
}

fn add_action_state(mut entity: EntityCommands) {
    entity.insert(InputManagerBundle::<Action> {
        action_state: default(),
        input_map: InputMap::new([
            // WASD
            (KeyCode::W, Action::Up),
            (KeyCode::S, Action::Down),
            (KeyCode::A, Action::Left),
            (KeyCode::D, Action::Right),
            (KeyCode::W, Action::Move),
            (KeyCode::S, Action::Move),
            (KeyCode::A, Action::Move),
            (KeyCode::D, Action::Move),
            // Cursor keys
            (KeyCode::Up, Action::Up),
            (KeyCode::Down, Action::Down),
            (KeyCode::Left, Action::Left),
            (KeyCode::Right, Action::Right),
            (KeyCode::Up, Action::Move),
            (KeyCode::Down, Action::Move),
            (KeyCode::Left, Action::Move),
            (KeyCode::Right, Action::Move),
            // Space
            (KeyCode::Space, Action::Jump),
            // E
            (KeyCode::E, Action::Interact),
        ]),
    });
}

fn player_jumping(
    mut player: Query<
        (
            Ref<PlayerJumping>,
            &ActionState<Action>,
            &mut TnuaController,
        ),
        (With<Player>, With<PlayerJumping>),
    >,
) {
    for (jumping_state, input, mut controller) in player.iter_mut() {
        if jumping_state.is_added() || input.pressed(Action::Jump) {
            controller.action(TnuaBuiltinJump {
                height: JUMP_HEIGHT,
                ..default()
            });
        }
    }
}

fn insert_or_modify<T: Component>(
    commands: &mut Commands,
    entity: Entity,
    component: &mut Option<Mut<T>>,
    insert: impl Fn() -> T,
    modify: impl FnOnce(&mut T),
) {
    if let Some(mut c) = component.as_mut() {
        modify(&mut c);
    } else {
        let mut c = insert();
        modify(&mut c);
        commands.entity(entity).insert(c);
    }
}

fn player_movement_walk(
    mut commands: Commands,
    mut player: Query<
        (Entity, &ActionState<Action>, Option<&mut TnuaController>),With<Player>>   
        //, Or<(With<PlayerGrounded>, With<PlayerJumping>)>),
){
    

    for (entity, input, mut controller) in player.iter_mut() {
        let mut movement = Vec3::ZERO;

        if input.pressed(Action::Up) {
            movement.z -= MOVEMENT_SPEED;
        }
        if input.pressed(Action::Down) {
            movement.z += MOVEMENT_SPEED;
        }
        if input.pressed(Action::Left) {
            movement.x -= MOVEMENT_SPEED;
        }
        if input.pressed(Action::Right) {
            movement.x += MOVEMENT_SPEED;
        }

        movement = movement.clamp_length_max(MOVEMENT_SPEED);

        insert_or_modify(
            &mut commands,
            entity,
            &mut controller,
            || TnuaController::default(),
            |c| {
                c.basis(TnuaBuiltinWalk {
                    desired_velocity: movement,
                    desired_forward: -movement.normalize_or_zero(),
                    float_height: FLOATING_HEIGHT,
                    ..default()
                });
            },
        );
    }
}

fn player_animation(
    mut player: Query<(&AnimationPlayerLink, &Animations, &TnuaController, &CurrentState), 
        (With<Player>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (link, animations, controller, state) in player.iter_mut() {
        
         
        let mut anim_name = "Idle";
        if state.state == PlayerStates::Jump {
            anim_name = "Jump";
        }
        else if state.state == PlayerStates::Move{
            anim_name = "Move";
        }

        println!("Chaning animation: {:?}", anim_name);
        let mut animation_player = animation_players.get_mut(link.0).unwrap();
        animation_player
            .play_with_transition(
                animations
                    .named_animations
                    .get(anim_name)
                    .expect("animation name should be in the list")
                    .clone(),
                Duration::from_secs(3),
            )
            .repeat();
        
    }
}

// Player camera





// fn player_interaction(
//     ray: Query<(&RayCaster, &RayHits, &Parent), With<InteractionRayCaster>>,
//     //ladders: Query<(Entity, &Ladder, &Position, &Rotation, &Collider), Without<Player>>,
//     mut player: Query<(&ActionState<Action>, Has<PlayerGrounded>, &mut Transform), With<Player>>,
//     //mut ladder_begin: EventWriter<LadderInteractionBeginEvent>,
//     //mut ladder_end: EventWriter<LadderInteractionEndEvent>,
// ) {
//     return;
//     // for (ray, hits, parent) in &ray {
//     //     screen_print!("hit: {:?}", hits.as_slice());

//     //     let player_entity = parent.get();

//     //     let Ok((action, walking, mut transform)) = player.get_mut(player_entity) else {
//     //         error!("Player missing");
//     //         continue;
//     //     };

//     //     if action.just_pressed(Action::Interact) {
//     //         if walking {
//     //             for hit in hits.iter() {
//     //                 if let Some((ladder_entity, ladder, ladder_pos, ladder_rot, col)) =
//     //                     ladders.get(hit.entity).ok()
//     //                 {
//     //                     // align with the center of the ladder
//     //                     let hit_pos =
//     //                         ray.global_origin() + ray.global_direction() * hit.time_of_impact;
//     //                     let ladder_center = (hit_pos - ladder_pos.0).dot(ladder.face_normal)
//     //                         * ladder.face_normal
//     //                         + ladder_pos.0;
//     //                     let player_pos =
//     //                         Vec3::new(ladder_center.x, transform.translation.y, ladder_center.z);
//     //                     transform.translation = player_pos;
//     //                     transform.rotation =
//     //                         Quat::from_rotation_y(ladder.face_normal.xz().angle_between(Vec2::Y));

//     //                     let aabb = col.compute_aabb(ladder_pos.0, ladder_rot.0);
//     //                     let half_height = aabb.half_extents().y;
//     //                     let center = aabb.center().y;
//     //                     let (top, bottom) = (
//     //                         center + half_height,
//     //                         center - half_height + PLAYER_HEIGHT / 2.0,
//     //                     );

//     //                     ladder_begin.send(LadderInteractionBeginEvent {
//     //                         entity: player_entity,
//     //                         face_normal: ladder.face_normal,
//     //                         top: Vec3::new(player_pos.x, top, player_pos.z),
//     //                         bottom: Vec3::new(player_pos.x, bottom, player_pos.z),
//     //                     });

//     //                     screen_print!("begin moving on ladder {ladder_entity:?}");
//     //                     break;
//     //                 }
//     //             }
//     //         } else {
//     //             ladder_end.send(LadderInteractionEndEvent(player_entity));
//     //             screen_print!("end moving on ladder");
//     //         }
//     //     }
//     // }
// }












