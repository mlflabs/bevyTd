use bevy::prelude::*;
use smooth_bevy_cameras::{LookTransformBundle, LookTransform, Smoother};

use super::Player;



#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct MainCamera;






pub fn camera_setup(mut commands: Commands) {
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform {
                eye: Vec3::new(-2.0, 2.5, 5.0),
                target: Vec3::new(0.0, 0.5, 0.0),
                up: Vec3::Y,
            },
            smoother: Smoother::new(0.5),
        })
        .insert(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        });
}



pub fn camera_setup2(
    mut q_camera: Query<Entity, Added<Camera3d>>,
    mut commands: Commands) {






    for entity in q_camera.iter_mut(){
        commands.entity(entity)
            .insert(MainCamera)
            .insert(LookTransformBundle {
                transform: LookTransform {
                    eye: Vec3::new(-2.0, 2.5, 5.0),
                    target: Vec3::new(0.0, 0.5, 0.0),
                    up: Vec3::Y,
                },
                smoother: Smoother::new(0.5),
            });
    }
}



pub fn move_camera_system(
    mut q_camera: Query<&mut LookTransform>,
    q_player: Query<&Transform, With<Player>>) 
{

    if let Ok(pp) = q_player.get_single() {
        if let Ok(mut cc) = q_camera.get_single_mut(){
            cc.target = pp.translation;
            cc.eye = pp.translation + Vec3::new(0.0, 10.0, 16.0)
            
        }
        
    }
    
    // if let Ok(playerTransfrom) = player.get_single() {
    //     if let OK(mut lookTransform) = camera.get_single_mut(){

    //     }
    // }
    // Later, another system will update the `Transform` and apply smoothing automatically.
    //for mut c in cameras.iter_mut() { c.target += Vec3::new(1.0, 1.0, 1.0); }
}
