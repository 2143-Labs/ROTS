use bevy::prelude::*;
use bevy::reflect::Reflect;

pub fn init(app: &mut App) -> &mut App {
    app.add_system(lifetime_despawn).add_system(update_all_bullets).add_system(spawn_bullet)
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

pub fn lifetime_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

enum BulletAI {
    /// Bullet directly travels from point to point
    Direct,
    Wavy,
}

#[derive(Component)]
struct BulletPhysics {
    fired_from: Vec2,
    fired_target: Vec2,
    speed: f32,
    ai: BulletAI,
}

fn update_all_bullets(
    mut bullets: Query<(&Lifetime, &BulletPhysics, &mut Transform)>,
) {
    for (lifetime, phys, mut transform) in bullets.iter_mut() {
        let nanos: f64 = lifetime.timer.elapsed().as_nanos() as f64;
        let secs = nanos / 1_000_000.0;
        let distance = (secs as f32) * phys.speed;

        let dir: Vec2 = (phys.fired_target - phys.fired_from).normalize();

        // Bullet positions are deterministic, based purely on time elapsed
        let offset: Vec2 = match phys.ai {
            BulletAI::Direct => {
                distance * dir
            }
            BulletAI::Wavy => {
                let rotate_right = Vec2::new(dir.y, -dir.x);
                let wavy_offset = rotate_right * distance.sin();
                distance * dir + wavy_offset * 0.5
            }
        };

        // Bullets float 0.5 above the ground
        let nl: Vec2 = phys.fired_from + offset;
        *transform = Transform::from_xyz(nl.x, 0.5, nl.y);
    }
}

fn spawn_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    buttons: Res<Input<MouseButton>>,
) {
    // Right click, red wavy, left click, blue direct
    let (color, ai) = if buttons.just_pressed(MouseButton::Left) {
        (Color::BLUE, BulletAI::Direct)
    } else if buttons.just_pressed(MouseButton::Right) {
        (Color::RED, BulletAI::Wavy)
    } else {
        return
    };

    let spawn_transform = Transform::from_xyz(0.0, 0.5, 0.0);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(0.4))),
            material: materials.add(color.into()),
            transform: spawn_transform,
            ..default()
        })
        .insert(Lifetime {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        })
        .insert(BulletPhysics {
            // make this player position
            fired_from: Vec2 { x: 0.0, y: 0.0 },
            // randomize these
            fired_target: Vec2 { x: 1.0, y: 1.0 },
            speed: 0.02,
            ai,
        })
        .insert(Name::new("Bullet"));
}
