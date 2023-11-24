
use bevy::prelude::*;
use bevy_xpbd_3d::{
    math::*, prelude::*, PhysicsSchedule, PhysicsStepSet, SubstepSchedule, SubstepSet,
};

use super::CharacterControllerBundle;


#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Player;



pub fn setup_player_controller(
    mut commands: Commands,
    mut q_player: Query<(Entity, &mut Transform), Added<Player>>
) {
    for (entity, mut transform) in q_player.iter_mut(){
        commands.entity(entity)
            .insert(CharacterControllerBundle::new(
                Collider::capsule(1.0, 0.4), 
                Vector::NEG_Y * 9.81 * 2.0)
                .with_movement(
                    30.0, 
                    0.92, 
                    7.0,
                     (30.0 as Scalar).to_radians())
            );
        

        transform.translation = Vec3::new(2.,2.,2.);
    }
}







pub fn setup_player_controller2(
    mut commands: Commands,
    mut q_player: Query<(Entity, &mut Transform), Added<Player>>
) {
    for (entity, mut transform) in q_player.iter_mut(){
        commands.entity(entity)
            .insert(RigidBody::Kinematic,)
            .insert(Collider::capsule(1.0, 0.4))
            .insert(ShapeCaster::new(
                Collider::capsule(0.9, 0.35),
                Vector::ZERO,
                Quaternion::default(),
                Vector::NEG_Y,
            )
            .with_max_time_of_impact(0.11)
            .with_max_hits(1));

        transform.translation = Vec3::new(2.,2.,2.);
    }
}


pub fn character_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(&mut LinearVelocity, &ShapeHits), With<Player>>,
) {
    for (mut linear_velocity, ground_hits) in &mut players {
        // Reset vertical valocity if grounded, otherwise apply gravity
        if !ground_hits.is_empty() {
            linear_velocity.y = 0.0;
        } else {
            linear_velocity.y -= 0.4;
        }

        // Directional movement
        if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
            linear_velocity.z -= 1.2;
        }
        if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
            linear_velocity.x -= 1.2;
        }
        if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
            linear_velocity.z += 1.2;
        }
        if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
            linear_velocity.x += 1.2;
        }

        // Jump if space pressed and the player is close enough to the ground
        if keyboard_input.just_pressed(KeyCode::Space) && !ground_hits.is_empty() {
            linear_velocity.y += 10.0;
        }

        // Slow player down
        linear_velocity.x *= 0.8;
        linear_velocity.y *= 0.98;
        linear_velocity.z *= 0.8;
    }
}


pub fn kinematic_collision(
    collisions: Res<Collisions>,
    mut bodies: Query<(&RigidBody, &mut Position, &Rotation)>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }
        if let Ok([(rb1, mut position1, rotation1), (rb2, mut position2, _)]) =
            bodies.get_many_mut([contacts.entity1, contacts.entity2])
        {
            for manifold in contacts.manifolds.iter() {
                for contact in manifold.contacts.iter() {
                    if contact.penetration <= Scalar::EPSILON {
                        continue;
                    }
                    if rb1.is_kinematic() && !rb2.is_kinematic() {
                        position1.0 -= contact.global_normal1(rotation1) * contact.penetration;
                    } else if rb2.is_kinematic() && !rb1.is_kinematic() {
                        position2.0 += contact.global_normal1(rotation1) * contact.penetration;
                    }
                }
            }
        }
    }
}


// pub fn player_controller(
//     keycode: Res<Input<KeyCode>>,
//     mut controllers: Query<(&mut Transform, &mut KinematicCharacterController)>,
// ) {
//     let speed = 0.2;

//     let mut direction = Vec3::ZERO;

//     if let Ok((transform, mut controller)) = controllers.get_single_mut() {
//         if keycode.pressed(KeyCode::Left) || keycode.pressed(KeyCode::A) {
//             direction -= Vec3::X;
//         }
//         if keycode.pressed(KeyCode::Right) || keycode.pressed(KeyCode::D){
//             direction += Vec3::X;
//         }

//         if keycode.pressed(KeyCode::Up) || keycode.pressed(KeyCode::W){
//             direction -= Vec3::Z;
//         }
//         if keycode.pressed(KeyCode::Down) || keycode.pressed(KeyCode::S){
//             direction += Vec3::Z;
//         }

//         //let dir = transform.looking_at(direction, controller.up);// * speed;// = direction.normalize_or_zero();
//         //direction = transform.forward();
        

//         //println!("Movement::: {},::{}", direction, direction.normalize_or_zero());
//         //let mut forward = controller.translation.lookup_forward;
//         let mut dest = direction * speed;
//         dest.y -= 0.2;
//         controller.translation = Some(dest);
        
//         // controller.basis(TnuaBuiltinWalk {
//         //     // Move in the direction the player entered, at a speed of 10.0:
//         //     desired_velocity: direction * speed,

//         //     // Turn the character in the movement direction:
//         //     desired_forward: direction.normalize_or_zero(),
             
//         //     // Must be larger than the height of the entity's center from the bottom of its
//         //     // collider, or else the character will not float and Tnua will not work properly:
//         //     float_height: 1.0,

//         //     // TnuaBuiltinWalk has many other fields that can be configured:
//         //     ..Default::default()
//         // });

//     }
// }