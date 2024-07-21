use bevy::prelude::*;

use crate::GlobalState;

use super::{spawn_button, UiState, UiStyle};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::MainMenu), main_menu_setup);
        app.add_systems(Update, button_system.run_if(in_state(UiState::MainMenu)));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MainMenuButton {
    Start,
    Settings,
    Exit,
}

fn main_menu_setup(mut commands: Commands, ui_style: Res<UiStyle>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                // center verticaly
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            ..default()
        })
        .insert(StateScoped(UiState::MainMenu))
        .with_children(|builder| {
            spawn_button(builder, &ui_style, MainMenuButton::Start);
            spawn_button(builder, &ui_style, MainMenuButton::Settings);
            spawn_button(builder, &ui_style, MainMenuButton::Exit);
        });
}

fn button_system(
    ui_style: Res<UiStyle>,
    mut ui_state: ResMut<NextState<UiState>>,
    mut global_state: ResMut<NextState<GlobalState>>,
    mut interaction_query: Query<
        (&MainMenuButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut exit: EventWriter<AppExit>,
) {
    for (button, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                match button {
                    MainMenuButton::Start => {
                        ui_state.set(UiState::InGame);
                        global_state.set(GlobalState::InGame);
                    }
                    MainMenuButton::Settings => ui_state.set(UiState::Settings),
                    MainMenuButton::Exit => _ = exit.send(AppExit::Success),
                }
            }
            Interaction::Hovered => {
                *color = ui_style.btn_color_hover.into();
            }
            Interaction::None => {
                *color = ui_style.btn_color_normal.into();
            }
        }
    }
}
