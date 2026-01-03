use bevy::prelude::*;
use std::f32::consts::TAU;

use avian3d::prelude::*;
use crate::game::prelude::*;

pub struct BallsPlugin;

impl Plugin for BallsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, spawn_balls)
        //    .add_systems(FixedUpdate, spin_ball);
    }
}

fn spin_ball(
    mut balls: Query<(&mut Transform, &Ball)>,
    timer: Res<Time>
) {
    for (mut transform, ball) in &mut balls {
        transform.rotate_x(ball.speed * TAU * timer.delta_secs());
    }
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(2.0));
    let mat = materials.add(Color::srgba(0.,0.,0.,0.1));

    for _ in 0..10 {
        commands.spawn((
            Ball { speed: 0.0 },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),

            // 衝突イベント
            CollisionEventsEnabled,
            // 連打抑制
            BounceSfxCooldown { last_played: -9999.0 },

            Transform::from_xyz(
                rand::random_range(-10.0..-5.0),
                rand::random_range(1.0..4.0),
                rand::random_range(-10.0..-5.0)
            ),

            RigidBody::Dynamic,
            Collider::sphere(2.0),
            SweptCcd::LINEAR, // 小さくて速い物体のすり抜け防止
            SpeculativeMargin::ZERO, // 推測

            Friction::new(1.0),
            Restitution::new(RESTITUTION),

        ));
    }
}
fn setup_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(2.0, 0.5, 0.5));
    let mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(0.1, 0.1, 0.1),
        ..default()
    });

    for z in 1..5 {
        for x in 1..5 {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(((x*10)-25) as f32, 5.0, ((z*10)-25) as f32)
            ));
        }
    }
}
