use bevy::prelude::*;

pub mod in_game;
pub mod main_menu;
pub mod settings;

use in_game::InGamePlugin;
use main_menu::MainMenuPlugin;
use settings::SettingsPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<UiState>()
            .enable_state_scoped_entities::<UiState>()
            .add_systems(PreStartup, setup_ui_style);

        app.add_plugins((MainMenuPlugin, SettingsPlugin, InGamePlugin));
    }
}

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum UiState {
    #[default]
    MainMenu,
    InGame,
    Settings,
}

#[derive(Resource, Debug, Clone)]
pub struct UiStyle {
    pub btn_style: Style,
    pub btn_color_normal: Color,
    pub btn_color_hover: Color,
    pub btn_color_pressed: Color,
    pub text_style: TextStyle,
}

fn setup_ui_style(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiStyle {
        btn_style: Style {
            margin: UiRect::all(Val::Percent(10.0)),
            padding: UiRect::all(Val::Percent(10.0)),
            border: UiRect::all(Val::Percent(1.0)),
            // make text in the middle
            justify_content: JustifyContent::Center,
            ..default()
        },
        btn_color_normal: Color::srgb(0.15, 0.15, 0.15),
        btn_color_hover: Color::srgb(0.25, 0.25, 0.25),
        btn_color_pressed: Color::srgb(0.35, 0.75, 0.35),
        text_style: TextStyle {
            font: asset_server.load("fonts/8bitOperatorPlus8-Regular.ttf"),
            font_size: 20.0,
            color: Color::srgb_u8(0xfa, 0xfa, 0xfa),
        },
    });
}

fn spawn_button<B>(child_builder: &mut ChildBuilder, style: &UiStyle, button: B)
where
    B: Component + std::fmt::Debug,
{
    child_builder
        .spawn(ButtonBundle {
            style: style.btn_style.clone(),
            border_color: BorderColor(Color::BLACK),
            border_radius: BorderRadius::all(Val::Percent(5.0)),
            background_color: style.btn_color_normal.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(format!("{:?}", button), style.text_style.clone()),
                ..default()
            });
        })
        .insert(button);
}
