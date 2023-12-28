use bevy::prelude::*;
use shared::{
    event::{client::Chat, server::SendChat, NetEntId, ERFE},
    netlib::{
        send_event_to_server, EventToClient, EventToServer, MainServerEndpoint, ServerResources,
    },
    AnyUnit,
};

use crate::{player::PlayerName, states::GameState};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ChatState {
    #[default]
    NotChatting,
    Chatting,
}

/// chat is this real?
pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ChatState>()
            .add_event::<Chat>()
            .add_event::<WeChat>()
            .insert_resource(ChatHistory::default())
            .insert_resource(ChatHistoryPtr::default())
            .add_systems(Startup, setup_panel)
            .add_systems(Update, on_chat)
            .add_systems(Update, on_chat_type.run_if(in_state(ChatState::Chatting)))
            .add_systems(
                Update,
                on_chat_toggle.run_if(shared::GameAction::Chat.just_pressed()),
            )
            .add_systems(
                Update,
                (receive_network_chats, on_local_chat_send)
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

#[derive(Component)]
struct ChatContainer;

/// This is the box that you type into
#[derive(Component)]
struct ChatTypeContainer;

trait EditableText {
    fn get_text(&mut self) -> &mut String;
}

impl EditableText for &mut Text {
    fn get_text(&mut self) -> &mut String {
        &mut self.sections[0].value
    }
}

#[derive(Resource, Debug, Default)]
struct ChatHistory(Vec<String>);

/// If you press the up arrow, we need to remember which chats you have seen.
#[derive(Resource, Debug, Default)]
struct ChatHistoryPtr(Option<usize>);

fn on_chat_toggle(
    cur_chat_state: Res<State<ChatState>>,
    mut chat_state: ResMut<NextState<ChatState>>,
    mut typed_text: Query<&mut Text, With<ChatTypeContainer>>,
    mut chat_bg_color: Query<&mut BackgroundColor, With<ChatContainer>>,
    mut chat_history: ResMut<ChatHistory>,
    mut chat_history_ptr: ResMut<ChatHistoryPtr>,
    mut ew: EventWriter<WeChat>,
) {
    let mut chatbox = typed_text.single_mut();
    let mut chatbox = chatbox.as_mut();
    let cur_text = chatbox.get_text();
    match cur_chat_state.get() {
        ChatState::Chatting => {
            let chat = std::mem::take(cur_text);
            // If this is a new chat, push it to the front
            if chat_history.0.last() != Some(&chat) {
                chat_history.0.push(chat.clone());
            }

            *chat_history_ptr = ChatHistoryPtr(None);
            ew.send(WeChat(chat));

            chat_state.set(ChatState::NotChatting);
            *chat_bg_color.single_mut() = Color::WHITE.with_a(0.00).into();
        }
        ChatState::NotChatting => {
            *cur_text = "".into();
            chat_state.set(ChatState::Chatting);
            *chat_bg_color.single_mut() = Color::WHITE.with_a(0.10).into();
        }
    }
}

#[derive(Event, Debug)]
struct WeChat(String);

fn on_chat_type(
    keyboard_input: Res<Input<KeyCode>>,
    mut typed_chars: EventReader<ReceivedCharacter>,
    mut typed_text: Query<&mut Text, With<ChatTypeContainer>>,
    mut chat_history_ptr: ResMut<ChatHistoryPtr>,
    chat_history: Res<ChatHistory>,
) {
    let mut chatbox = typed_text.single_mut();
    let mut chatbox = chatbox.as_mut();
    let cur_text = chatbox.get_text();

    if keyboard_input.just_pressed(KeyCode::Back) {
        cur_text.pop();
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        if chat_history.0.len() >= 1 {
            let new_ptr = chat_history_ptr.0
                .unwrap_or(chat_history.0.len())
                .saturating_sub(1);

            *chat_history_ptr = ChatHistoryPtr(Some(new_ptr));
            *cur_text = chat_history.0[new_ptr].clone();
        }
    }

    if keyboard_input.just_pressed(KeyCode::Down) {
        if let Some(cur_ptr) = chat_history_ptr.0 {
            let new_ptr = (cur_ptr + 1).min(chat_history.0.len() - 1);

            *chat_history_ptr = ChatHistoryPtr(Some(new_ptr));
            *cur_text = chat_history.0[new_ptr].clone();
        };
    }

    for typed_char in typed_chars.read() {
        if !typed_char.char.is_control() {
            cur_text.push(typed_char.char);
        }
    }
}

fn setup_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    // setup a flexbox container for notifications
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(50.0),
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    height: Val::Percent(50.0),

                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Relative,
                    ..default()
                },
                background_color: Color::WHITE.with_a(0.00).into(),
                ..default()
            },
            ChatContainer,
        ))
        .with_children(|parent| {
            // This is the box to type into
            parent.spawn((
                TextBundle::from_sections([TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                )])
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style { ..default() }),
                ChatTypeContainer,
            ));
        });
}

fn receive_network_chats(mut net_chats: ERFE<Chat>, mut ew: EventWriter<Chat>) {
    for c in net_chats.read() {
        ew.send(c.event.clone());
    }
}

fn on_local_chat_send(
    mut er: EventReader<WeChat>,
    //mut ew: EventWriter<Chat>,
    //our_id: Query<&NetEntId, With<Player>>,
    sr: Res<ServerResources<EventToClient>>,
    mse: Res<MainServerEndpoint>,
) {
    for e in er.read() {
        let event = EventToServer::SendChat(SendChat { text: e.0.clone() });
        send_event_to_server(&sr.handler, mse.0, &event);
    }
}

fn on_chat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    parent: Query<Entity, With<ChatContainer>>,
    players: Query<(&NetEntId, &PlayerName), With<AnyUnit>>,
    mut er: EventReader<Chat>,
    time: Res<Time>,
) {
    for chat in er.read() {
        let name = match chat.source {
            Some(ref eid) => {
                let matching = players.iter().find(|(ent_id, _)| eid == *ent_id);
                matching
                    .map(|(_, player_name)| player_name.0.as_ref())
                    .unwrap_or("Unknown Player")
            }
            None => "Server",
        };

        let parent = parent.single();
        commands.entity(parent).with_children(|p| {
            p.spawn((TextBundle::from_sections([
                TextSection::new(
                    &format!("{:03.3} ", time.elapsed_seconds()),
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 14.0,
                        color: Color::DARK_GRAY,
                    },
                ),
                TextSection::new(
                    name,
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::ORANGE_RED,
                    },
                ),
                TextSection::new(
                    &format!(": "),
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::GRAY,
                    },
                ),
                TextSection::new(
                    &chat.text,
                    TextStyle {
                        font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ),
            ])
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style { ..default() }),));
        });
    }
}
