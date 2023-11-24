use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

mod core;
use crate::core::*;

pub mod assets;
use assets::*;

pub mod state;
use state::*;

mod game;
use game::*;

mod test_components;
use test_components::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // resolution: (800., 460.).into(), //2850, 0
                        position: WindowPosition::At((5,5).into()),
                        resolution: (1000., 600.).into(), //2850, 0
                        // position: WindowPosition::At((2550,0).into()),
    
                        //resolution: (1280., 720.).into(),
                        //position: (0, 0).into(),
                        // fill the entire browser window
                        fit_canvas_to_parent: true,
                        // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    
                    ..default()
                })
                .set(AssetPlugin::default()),
            // editor
            EditorPlugin::default(),
            // our custom plugins
            StatePlugin,
            AssetsPlugin,
            CorePlugin,           // reusable plugins
            GamePlugin,           // specific to our game
            ComponentsTestPlugin, // Showcases different type of components /structs
        ))
        .run();
}
