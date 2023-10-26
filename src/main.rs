//In percent
pub const MARGIN_LEFT: f32 = 10.0;
pub const MARGIN_RIGHT: f32 = 20.0;
pub const MARGIN_BOTTOM: f32 = 10.0;
pub const MARGIN_TOP: f32 = 10.0;

use bevy::prelude::*;
use bevy::window::WindowResized;
use crate::chess::{ChessPluginClient,BoardEntity};

pub mod chess;
pub mod drag;
fn main() {
    App::new()
    .add_plugins((DefaultPlugins,ChessPluginClient))
    .add_systems(Update, window_resized_event)
    .run();
}

fn window_resized_event(mut events: EventReader<WindowResized>, mut boardents: Query<&mut Transform, With<BoardEntity>>) {
    let event: Option<&WindowResized> = events.iter().last(); // might be able to unwrap here idk
    let resized: &WindowResized = match event {
        Some(x) => x,
        None => return
    };
    //println!("{resized:?}");
    let hori_mult = (MARGIN_LEFT - MARGIN_RIGHT) / 200.0;
    let vert_mult = (MARGIN_BOTTOM - MARGIN_TOP) / 200.0;
    let center = Vec3::new(hori_mult * resized.width, vert_mult * resized.height,0.0);
    let hori_percent = (100.0 - (MARGIN_RIGHT + MARGIN_LEFT)) / 100.0;
    let vert_percent = (100.0 - (MARGIN_TOP + MARGIN_BOTTOM)) / 100.0;
    let space = (hori_percent * resized.width).min(vert_percent * resized.height);
    for mut boardent in boardents.iter_mut(){
        boardent.translation = center;
        boardent.scale = Vec3::new(space,space,1.0);
    }

    // event.width, event.height
}
