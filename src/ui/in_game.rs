use bevy::prelude::*;

use crate::{game::GameImage, GlobalState};

use super::{spawn_button, UiState, UiStyle};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::InGame), in_game_setup);
        app.add_systems(Update, button_system.run_if(in_state(UiState::InGame)));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CyclesText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PauseText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InGameButton {
    Settings,
    MainMenu,
}

pub const UI_TOP_SIZE: f32 = 10.0;
pub const UI_RIGHT_SIZE: f32 = 30.0;

fn in_game_setup(mut commands: Commands, ui_style: Res<UiStyle>, game_image: Res<GameImage>) {
    // Root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..default()
        })
        .insert(StateScoped(UiState::InGame))
        .with_children(|builder| {
            // Top part
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(UI_TOP_SIZE),
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    // Cycles + pause part
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0 - UI_RIGHT_SIZE),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                justify_content: JustifyContent::SpaceAround,
                                ..Default::default()
                            },
                            background_color: Color::srgb(0.0, 0.0, 0.3).into(),
                            ..default()
                        })
                        .with_children(|builder| {
                            // Cycles text
                            builder.spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        format!("Cycles: {}", 5),
                                        ui_style.text_style.clone(),
                                    ),
                                    ..default()
                                },
                                CyclesText,
                            ));
                            // Pause state
                            builder.spawn((
                                TextBundle {
                                    text: Text::from_section("Paused", ui_style.text_style.clone()),
                                    ..default()
                                },
                                PauseText,
                            ));
                        });
                    // Settings + exit buttons
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(UI_RIGHT_SIZE),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                justify_content: JustifyContent::SpaceAround,
                                ..Default::default()
                            },
                            background_color: Color::srgb(0.0, 0.5, 0.3).into(),
                            ..default()
                        })
                        .with_children(|builder| {
                            spawn_button(builder, &ui_style, InGameButton::Settings);
                            spawn_button(builder, &ui_style, InGameButton::MainMenu);
                        });
                });

            // Main part
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0 - UI_TOP_SIZE),
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    // Game window + dynamic ui
                    builder.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0 - UI_RIGHT_SIZE),
                                ..Default::default()
                            },
                            background_color: Color::NONE.into(),
                            ..default()
                        },
                        UiImage::new(game_image.image.clone()),
                    ));
                    // Items and spells
                    builder.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(UI_RIGHT_SIZE),
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        background_color: Color::srgb(0.7, 0.1, 0.0).into(),
                        ..default()
                    });
                });
        });
}

fn button_system(
    ui_style: Res<UiStyle>,
    mut ui_state: ResMut<NextState<UiState>>,
    mut global_state: ResMut<NextState<GlobalState>>,
    mut interaction_query: Query<
        (&InGameButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (button, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                match button {
                    InGameButton::Settings => ui_state.set(UiState::Settings),
                    InGameButton::MainMenu => {
                        ui_state.set(UiState::MainMenu);
                        global_state.set(GlobalState::MainMenu);
                    }
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
