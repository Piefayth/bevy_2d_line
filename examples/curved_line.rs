use bevy::{color::palettes::css::{BLUE, RED}, math::VectorSpace, prelude::*};
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

    let points = generate_bezier_curve(
        Vec2::new(-200.0, 0.0),
        Vec2::new(-100.0, 200.0),
        Vec2::new(100.0, -200.0),
        Vec2::new(200.0, 0.0),
        50,
    );

    let colors = generate_gradient(RED.into(), BLUE.into(), points.len());

    commands.spawn(Line {
        points,
        colors,
        thickness: 5.0,
    });
}

fn generate_bezier_curve(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, segments: usize) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(segments);
    for i in 0..segments {
        let t = i as f32 / (segments - 1) as f32;
        let point = cubic_bezier(p0, p1, p2, p3, t);
        points.push(point);
    }
    points
}

fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    return p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3;
}

fn generate_gradient(start_color: LinearRgba, end_color: LinearRgba, steps: usize) -> Vec<LinearRgba> {
    let mut colors = Vec::with_capacity(steps);
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        colors.push(start_color.lerp(end_color, t));
    }
    colors
}
