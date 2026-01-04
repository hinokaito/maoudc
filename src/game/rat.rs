use bevy::prelude::*;
use avian3d::prelude::*;
use crate::game::prelude::*;
use rand::Rng;
use crate::game::audio::Sfx;

pub struct RatPlugin;

impl Plugin for RatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_rat)
            .add_systems(FixedUpdate, patrol_move_system);
    }
}

fn spawn_rat (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    
    let mesh: Handle<Mesh> = asset_server.load(
    GltfAssetLabel::Primitive { mesh: 0, primitive: 0 }
        .from_asset("object/rat.glb"),
    );

    let mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });

    for _ in 0..100 {
        let idx = rng.random_range(0..100);

        commands.spawn((
            Rat,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(rng.random_range(-500.0..500.0), -18.5, rng.random_range(-500.0..500.0)).with_scale(Vec3::splat(10.0)),
            RigidBody::Kinematic,
            LinearVelocity::ZERO,
            Collider::sphere(0.05),
            RatPatrol {
                dir: Vec3::ZERO,
                speed: rng.random_range(15.0..30.0),
                turn_lerp: rng.random_range(10.0..20.0),
                jitter: rng.random_range(0.4..0.9),
                look_ahead: rng.random_range(2.8..3.6),
                avoid_strength: rng.random_range(1.5..3.0),
            },
            RatFootstepAudio {
                sample_index: idx,
                child_emitter: None,
                is_active: false,
            },
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_z(),
                // .lock_rotation_y(),
            AngularVelocity::ZERO, 
        ));
    }
}

fn patrol_move_system(
    spatial: SpatialQuery,
    mut q: Query<(Entity, &Transform, &mut LinearVelocity, &mut RatPatrol), With<Rat>>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();
    
    for (me, tf, mut lv, mut w) in &mut q {
        let yaw = (rng.random::<f32>() - 0.5) * 2.0 * w.jitter * dt;
        let rot = Quat::from_rotation_y(yaw);
        let mut desired = (rot * w.dir).normalize_or_zero();
        desired.y = 0.0;
        if desired.length_squared() < 1e-6 {
            desired = Vec3::X;
        }

        let origin = tf.translation + Vec3::Y * 0.15;
        let dir3 = Dir3::new(desired).unwrap_or(Dir3::X);

        let filter = SpatialQueryFilter::from_excluded_entities([me]);

        if let Some(hit) = spatial.cast_ray(origin, dir3, w.look_ahead, true, &filter) {
            let t = (1.0 - (hit.distance / w.look_ahead)).clamp(0.0, 1.0);
            let avoid = hit.normal * (w.avoid_strength * t);

            desired = (desired + avoid).normalize_or_zero();
            desired.y = 0.0;
        }

        let alpha = (w.turn_lerp * dt).clamp(0.0, 1.0);
        w.dir = (w.dir.lerp(desired, alpha)).normalize_or_zero();
        w.dir.y = 0.0;

        lv.0 = w.dir * w.speed;
    }
}