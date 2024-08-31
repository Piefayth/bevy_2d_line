use bevy::{color::palettes::css::{BLUE, GREEN, RED}, prelude::*};
use bevy_2d_line::{Line, LineRenderingPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LineRenderingPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let points = vec![
        Vec2::new(-200.0, 0.0),
        Vec2::new(0.0, 200.0),
        Vec2::new(200.0, 0.0),
    ];

    let colors = vec![
        RED.into(),
        GREEN.into(),
        BLUE.into(),
    ];

    commands.spawn(Line {
        points,
        colors,
        thickness: 5.0,
    });
}