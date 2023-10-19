use bevy::prelude::*;
use kayak_ui::{
    prelude::{widgets::*, *},
    CameraUIKayak,
};

use crate::states::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((KayakContextPlugin, KayakWidgets))
            .add_systems(
                Update,
                menu_startup.run_if(in_state(GameState::Menu).and_then(run_once())),
            );
    }
}

pub fn menu_startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((
            Camera2dBundle::default(),
            CameraUIKayak,
            Name::new("UI Camera"),
        ))
        .id();

    font_mapping.set_default(asset_server.load("roboto.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    dbg!("Hello World");
    rsx! {
       <KayakAppBundle>
            <TextWidgetBundle
                text={TextProps {
                    content: "Hello World".into(),
                    size: 20.0,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

#[derive(Component, Clone, PartialEq, Default)]
pub struct MyButtonProps {}
impl Widget for MyButtonProps {}
#[derive(Bundle, Default)]
pub struct MyButtonBundle {
    pub props: MyButtonProps,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    // This allows us to hook into on click events!
    pub on_event: OnEvent,
    // Widget name is required by Kayak UI!
    pub widget_name: WidgetName,
}

pub fn my_button_render(
    // This is a bevy feature which allows custom parameters to be passed into a system.
    // In this case Kayak UI gives the system an `Entity`.
    In(entity): In<Entity>,
    //    This struct allows us to make changes to the widget tree.
    widget_context: ResMut<KayakWidgetContext>,
    mut commands: Commands,
    query: Query<&KChildren>,
) -> bool {
    // Grab our children for our button widget:
    if let Ok(children) = query.get(entity) {
        let background_styles = KStyle {
            // Lets use red for our button background!
            background_color: StyleProp::Value(Color::RED),
            // 50 pixel border radius.
            border_radius: Corner::all(50.0).into(),
            ..Default::default()
        };

        let parent_id = Some(entity);

        rsx! {
            <BackgroundBundle
                styles={background_styles}
                // We pass the children to the background bundle!
                children={children.clone()}
            />
        };
    }

    // The boolean returned here tells kayak UI to update the tree. You can avoid tree updates by
    // returning false, but in practice this should be done rarely. As kayak diff's the tree and
    // will avoid tree updates if nothing has changed!
    true
}
