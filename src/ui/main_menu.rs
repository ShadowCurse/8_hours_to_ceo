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
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..default()
        })
        .insert(StateScoped(UiState::MainMenu))
        .with_children(|builder| {
            // Title
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        border: UiRect::all(Val::Percent(1.0)),
                        align_self: AlignSelf::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::all(Val::Percent(5.0)),
                    background_color: ui_style.btn_color_normal.into(),
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn(TextBundle {
                        text: Text::from_section(
                            "8 hours to CEO",
                            TextStyle {
                                font: ui_style.text_style.font.clone(),
                                font_size: 50.0,
                                color: ui_style.text_style.color,
                            },
                        ),
                        ..default()
                    });
                });

            // Buttons
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(20.0),
                        height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        align_self: AlignSelf::Start,
                        justify_items: JustifyItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    spawn_button(builder, &ui_style, MainMenuButton::Start);
                    spawn_button(builder, &ui_style, MainMenuButton::Settings);
                    spawn_button(builder, &ui_style, MainMenuButton::Exit);
                });
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
