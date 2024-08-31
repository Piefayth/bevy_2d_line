use bevy::{
    color::palettes::css::{BLUE, GREEN, PURPLE, RED, YELLOW},
    math::VectorSpace,
    prelude::*,
};
use bevy_2d_line::{Line, LineRenderingPlugin};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LineRenderingPlugin)
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, animate_lines)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Define colors using LinearRgba directly
    let colors = [RED, GREEN, BLUE, YELLOW, PURPLE];

    // Generate multiple curved lines
    for i in 0..1000 {
        let base_x = i as f32 * 15.0 - 30.0;
        let base_y = 0.0;
        let curve_height = 10.0 + i as f32 * 4.0;

        let points = generate_bezier_curve(
            Vec2::new(base_x, base_y),
            Vec2::new(base_x - 5.0, base_y + curve_height),
            Vec2::new(base_x + 5.0, base_y - curve_height),
            Vec2::new(base_x, base_y),
            30,
        );

        let color_start = colors[i % colors.len()];
        let color_end = colors[(i + 1) % colors.len()];

        let colors = generate_gradient(color_start.into(), color_end.into(), points.len());

        commands.spawn(Line {
            points,
            colors,
            thickness: 2.0,
        });
    }
}

fn generate_bezier_curve(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, segments: usize) -> Vec<Vec2> {
    (0..segments)
        .map(|i| {
            let t = i as f32 / (segments - 1) as f32;
            cubic_bezier(p0, p1, p2, p3, t)
        })
        .collect()
}

fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

fn generate_gradient(
    start_color: LinearRgba,
    end_color: LinearRgba,
    steps: usize,
) -> Vec<LinearRgba> {
    (0..steps)
        .map(|i| {
            let t = i as f32 / (steps - 1) as f32;
            start_color.lerp(end_color, t).into()
        })
        .collect()
}

fn animate_lines(time: Res<Time>, mut query: Query<&mut Line>) {
    let time_seconds = time.elapsed_seconds();
    let movement_factor = 50.0 * time_seconds.sin(); // Simple oscillation amplitude

    for mut line in query.iter_mut() {
        line.points.iter_mut().for_each(|point| {
            point.y += movement_factor; // Apply vertical oscillation
        });
    }
}
