use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy_mod_raycast::Intersection;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, RigidBody};
use rand::{thread_rng, Rng};

use crate::networking::client_bullet_receiver::NetworkPlayer;
use crate::{player::Player, networking::client_bullet_receiver::MainServerEndpoint};
use crate::setup::MyRaycastSet;
use shared::{BulletPhysics, BulletAI, EventToClient, ServerResources, EventToServer};

pub fn init(app: &mut App) -> &mut App {
    app
        .add_system(lifetime_despawn)
        .add_system(lifetime_event)
        .add_system(update_all_bullets)
        .add_system(spawn_bullet)
        .add_system(spawn_animations)
        // .add_system(tower_shooting)
        .add_system(camera_aim)
        //.add_system(update_collisions)
        .register_type::<Tower>()
        .add_startup_system(spawn_tower)
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component)]
pub struct LifetimeWithEvent {
    pub timer: Timer,
}

fn lifetime_despawn(
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

fn lifetime_event(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut LifetimeWithEvent)>,
    time: Res<Time>,

    player: Query<&Transform, With<Player>>,
    intersect: Query<&Intersection<MyRaycastSet>>,
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();

            let target: &Intersection<_> = match intersect.iter().next() {
                Some(s) => s,
                None => {
                    info!("No intersection with ground");
                    return;
                }
            };

            debug!(?target);

            let isect = match target.position() {
                Some(s) => s,
                None => {
                    error!("No intersect position?");
                    return;
                }
            };

            debug!(?isect);

            let player_transform: &Transform = player.single();
            //let _tower_transform: &Transform = towers.single();
            //let spawn_transform = Transform::from_xyz(0.0, -100., 0.0);

            let v: Vec<EventToServer> = (0..100).map(|_i| {
                let mut rng = thread_rng();
                let x = rng.gen_range(-1.0..1.0);
                let y = rng.gen_range(-1.0..1.0);
                let spd = rng.gen_range(1.0..20.0);
                let rand_offset = Vec2::new(x, y);
                let from = Vec2 {
                    x: player_transform.translation.x,
                    y: player_transform.translation.z,
                };

                let phys = BulletPhysics {
                    fired_target: from + rand_offset,
                    fired_from: from,
                    speed: spd,
                    ai: BulletAI::Wavy,
                };

                let ev = EventToServer::ShootBullet(phys);
                ev
            }).collect();

            let data = serde_json::to_string(&v).unwrap();
            event_list_res.handler.network().send(mse.0, data.as_bytes());
        }
    }
}

fn update_all_bullets(
    mut bullets: Query<(&Lifetime, &BulletPhysics, &mut Transform)>,
    _time: Res<Time>,
) {
    for (lifetime, phys, mut transform) in bullets.iter_mut() {
        let nanos: f64 = lifetime.timer.elapsed().as_nanos() as f64;
        let secs = nanos / 1_000_000_000.0;
        let distance = (secs as f32) * phys.speed;

        let dir: Vec2 = (phys.fired_target - phys.fired_from).normalize();

        // Bullet positions are deterministic, based purely on time elapsed
        let offset: Vec2 = match phys.ai {
            BulletAI::Direct => distance * dir,
            BulletAI::Wavy => {
                let rotate_right = Vec2::new(dir.y, -dir.x);
                let wavy_offset = rotate_right * distance.sin();
                distance * dir + wavy_offset * 0.5
            }
            BulletAI::Wavy2 => {
                let rotate_right = Vec2::new(dir.x, dir.y);
                let wavy_offset = rotate_right * distance.sin();
                distance * dir + wavy_offset * 2.0
            }
        };

        // Bullets float 0.5 above the ground
        let nl: Vec2 = phys.fired_from + offset;
        *transform = Transform::from_xyz(nl.x, 0.5, nl.y);
    }
}

fn update_collisions(
    bullets: Query<(Entity, &Transform), With<BulletPhysics>>,
    players: Query<&Transform, (Without<Player>, With<NetworkPlayer>)>,
    mut commands: Commands,
) {
    for (bullet_ent, bullet_transform) in bullets.iter() {
        for player_transform in &players {
            let dist = (
                player_transform.translation - bullet_transform.translation
            ).length_squared();

            if dist < 1.0 {
                commands
                    .entity(bullet_ent)
                    .despawn_recursive();

                warn!("hit");
            }
        }
    }
}

fn camera_aim(
    intersect: Query<&Intersection<MyRaycastSet>>,
    mut aim_target_cube: Query<&mut Transform, With<AimVectorTarget>>,
) {
    for i in &intersect {
        if let Ok(mut s) = aim_target_cube.get_single_mut() {
            match i.position() {
                Some(pos) => s.translation = *pos,
                None => s.translation = Vec3::ZERO,
            }
        }
    }
}

#[derive(Component, Reflect)]
struct AimVectorTarget;


fn spawn_animations(
    _buttons: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    //player: Query<&Transform, With<Player>>,
    //intersect: Query<&Intersection<MyRaycastSet>>,
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    if keyboard_input.just_pressed(KeyCode::B) {
        let ev = EventToServer::BeginAnimation(shared::event::AnimationThing::Waterball);
        let data = serde_json::to_string(&ev).unwrap();
        event_list_res.handler.network().send(mse.0, data.as_bytes());
    }
}

fn spawn_bullet(
    _buttons: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    intersect: Query<&Intersection<MyRaycastSet>>,
    event_list_res: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    // Right click, red wavy, left click, blue direct
    let (_color, ai) = if keyboard_input.just_pressed(KeyCode::E) {
        (Color::PINK, BulletAI::Wavy2)
    } else if keyboard_input.just_pressed(KeyCode::R) {
        (Color::RED, BulletAI::Wavy)
    } else if keyboard_input.just_pressed(KeyCode::T) {
        (Color::OLIVE, BulletAI::Direct)
    } else {
        return;
    };

    let target: &Intersection<_> = match intersect.iter().next() {
        Some(s) => s,
        None => {
            info!("No intersection with ground");
            return;
        }
    };

    debug!(?target);

    let isect = match target.position() {
        Some(s) => s,
        None => {
            error!("No intersect position?");
            return;
        }
    };

    debug!(?isect);

    let player_transform: &Transform = player.single();
    //let _tower_transform: &Transform = towers.single();
    //let spawn_transform = Transform::from_xyz(0.0, -100., 0.0);

    let phys = BulletPhysics {
        fired_target: Vec2 {
            x: isect.x,
            y: isect.z,
        },
        fired_from: Vec2 {
            x: player_transform.translation.x,
            y: player_transform.translation.z,
        },
        speed: 10.0,
        ai,
    };

    let ev = EventToServer::ShootBullet(phys);
    let data = serde_json::to_string(&ev).unwrap();
    event_list_res.handler.network().send(mse.0, data.as_bytes());

}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
}

fn _tower_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut towers: Query<(&mut Tower, &Transform)>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for (mut tower, tower_transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if !tower.shooting_timer.just_finished() {
            continue;
        }

        let color = Color::OLIVE;

        if let Some(player_transform) = player.iter().next() {
            let size = 0.5;
            let spawn_transform = Transform::from_xyz(
                tower_transform.translation.x,
                0.5,
                tower_transform.translation.z,
            );
            commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube::new(size * 2.))),
                        material: materials.add(color.into()),
                        transform: spawn_transform,
                        ..default()
                    },
                    RigidBody::Dynamic,
                ))
                .insert(Collider::cuboid(size, size, size))
                .insert(Lifetime {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                })
                .insert(BulletPhysics {
                    // make this player position
                    fired_from: Vec2 {
                        x: tower_transform.translation.x,
                        y: tower_transform.translation.z,
                    },
                    // randomize these
                    fired_target: Vec2 {
                        x: player_transform.translation.x,
                        y: player_transform.translation.z,
                    },
                    speed: 10.0,
                    ai: BulletAI::Direct,
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Name::new("Bullet"));
        }
    }
}

fn spawn_tower(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.5, 4., 0.5))),
            material: materials.add(Color::hex("#FF0000").unwrap().into()),
            transform: Transform::from_xyz(-4., 2., 4.),
            ..default()
        })
        .insert(Tower {
            shooting_timer: Timer::from_seconds(2., TimerMode::Repeating),
        })
        .insert(Name::new("Tower"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(0.1))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::default(),
            ..default()
        })
        .insert(AimVectorTarget)
        .insert(Name::new("AimVector"));
}
