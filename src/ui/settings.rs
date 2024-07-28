use bevy::{audio::Volume, prelude::*};

use crate::game::sound::{BackgroundMusic, SoundResources};

use super::{spawn_button, UiState, UiStyle};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::Settings), settings_setup);
        app.add_systems(Update, button_system.run_if(in_state(UiState::Settings)));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SettingsButton {
    VolumeUp,
    VolumeDown,
    Exit,
}

fn settings_setup(mut commands: Commands, ui_style: Res<UiStyle>) {
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
        .insert(StateScoped(UiState::Settings))
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
                .insert(StateScoped(UiState::MainMenu))
                .with_children(|builder| {
                    spawn_button(builder, &ui_style, SettingsButton::VolumeUp);
                    spawn_button(builder, &ui_style, SettingsButton::VolumeDown);
                    spawn_button(builder, &ui_style, SettingsButton::Exit);
                });
        });
}

fn button_system(
    ui_style: Res<UiStyle>,
    music_controller: Query<&AudioSink, With<BackgroundMusic>>,
    mut sounds: ResMut<SoundResources>,
    mut ui_state: ResMut<NextState<UiState>>,
    mut interaction_query: Query<
        (&SettingsButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (button, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                match button {
                    SettingsButton::VolumeUp => {
                        if let Ok(sink) = music_controller.get_single() {
                            sink.set_volume(sink.volume() + 0.1);
                            sounds.volume = Volume::new(sounds.volume.get() + 0.1);
                        }
                    }
                    SettingsButton::VolumeDown => {
                        if let Ok(sink) = music_controller.get_single() {
                            sink.set_volume(sink.volume() - 0.1);
                            sounds.volume = Volume::new(sounds.volume.get() - 0.1);
                        }
                    }
                    SettingsButton::Exit => {
                        ui_state.set(UiState::MainMenu);
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
