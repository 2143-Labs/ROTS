use std::time::Duration;

use bevy::prelude::*;
use shared::casting::DespawnTime;

#[derive(Event)]
pub struct Notification(pub String);

//type NotificationPointer = u64;

#[derive(Component)]
pub struct NotificationElement(pub String);

#[derive(Component)]
pub struct NotificationExpiresAt(pub f32);

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Notification>()
            //.insert_resource(NotificationResource::default())
            .add_systems(Startup, setup_panel)
            .add_systems(Update, notification_ui)
            .add_systems(Update, on_notification);
    }
}

//#[derive(Resource, Default)]
//pub struct NotificationResource {
//expired: Vec<Notification>,
//}

fn notification_ui() {}

#[derive(Component)]
struct NotificationContainer;

fn setup_panel(mut commands: Commands) {
    // setup a flexbox container for notifications
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(50.0),
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                height: Val::Percent(50.0),

                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..default()
            },
            //background_color: Color::WHITE.with_a(0.10).into(),
            ..default()
        },
        NotificationContainer,
    ));
}

fn on_notification(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    parent: Query<Entity, With<NotificationContainer>>,
    mut er: EventReader<Notification>,
    time: Res<Time>,
) {
    for e in er.read() {
        debug!("Got a notification... {}", e.0);
        let parent = parent.single();
        commands.entity(parent).with_children(|p| {
            p.spawn((
                TextBundle::from_section(
                    format!("{:03.3}: {}", time.elapsed_seconds(), e.0),
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_justify(JustifyText::Right)
                .with_style(Style { ..default() }),
                NotificationElement(e.0.clone()),
                DespawnTime(Timer::new(Duration::from_millis(10000), TimerMode::Once)),
            ));
        });
    }
}
