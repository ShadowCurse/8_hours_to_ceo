use bevy::prelude::*;

use crate::{
    game::{
        inventory::{Inventory, Items, Spells},
        GameImage, GameState,
    },
    GlobalState,
};

use super::{spawn_button, UiState, UiStyle};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::InGame), in_game_setup)
            .add_systems(
                Update,
                (button_system, update_pause, update_inventory).run_if(in_state(UiState::InGame)),
            );
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CyclesText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PauseText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ActiveItemId(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BackpackItemId(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ActiveSpellId(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BackpackSpellId(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InGameButton {
    Settings,
    MainMenu,
}

pub const UI_TOP_SIZE: f32 = 10.0;
pub const UI_RIGHT_SIZE: f32 = 30.0;

fn spawn_inventory_button<C: Component>(builder: &mut ChildBuilder, c: C) {
    builder
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            border_radius: BorderRadius::all(Val::Percent(5.0)),
            background_color: Color::WHITE.into(),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section(
                    "--",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::srgb(0.2, 0.2, 0.2),
                        ..Default::default()
                    },
                ),
                c,
            ));
        });
}

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
                            ..default()
                        },
                        UiImage::new(game_image.image.clone()),
                    ));
                    // Items and spells
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(UI_RIGHT_SIZE),
                                flex_direction: FlexDirection::Column,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            // Items
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        height: Val::Percent(50.0),
                                        flex_direction: FlexDirection::Column,
                                        ..Default::default()
                                    },
                                    ..default()
                                })
                                .with_children(|builder| {
                                    // Active items
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                height: Val::Percent(30.0),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::SpaceAround,
                                                ..Default::default()
                                            },
                                            background_color: Color::srgb(0.8, 0.8, 0.0).into(),
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(builder, ActiveItemId(0));
                                            spawn_inventory_button(builder, ActiveItemId(1));
                                            spawn_inventory_button(builder, ActiveItemId(2));
                                            spawn_inventory_button(builder, ActiveItemId(3));
                                        });

                                    // Backpack items
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                height: Val::Percent(70.0),
                                                display: Display::Grid,
                                                align_items: AlignItems::Center,
                                                justify_items: JustifyItems::Center,
                                                grid_template_columns: RepeatedGridTrack::flex(
                                                    4, 1.0,
                                                ),
                                                ..Default::default()
                                            },
                                            background_color: Color::srgb(0.2, 0.8, 0.0).into(),
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(builder, BackpackItemId(0));
                                            spawn_inventory_button(builder, BackpackItemId(1));
                                            spawn_inventory_button(builder, BackpackItemId(2));
                                            spawn_inventory_button(builder, BackpackItemId(3));
                                            spawn_inventory_button(builder, BackpackItemId(4));
                                            spawn_inventory_button(builder, BackpackItemId(5));
                                            spawn_inventory_button(builder, BackpackItemId(6));
                                            spawn_inventory_button(builder, BackpackItemId(7));
                                        });
                                });

                            // Spells
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        height: Val::Percent(50.0),
                                        flex_direction: FlexDirection::Column,
                                        ..Default::default()
                                    },
                                    background_color: Color::srgb(0.4, 0.4, 0.8).into(),
                                    ..default()
                                })
                                .with_children(|builder| {
                                    // Active spells
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                height: Val::Percent(30.0),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::SpaceAround,
                                                ..Default::default()
                                            },
                                            background_color: Color::srgb(0.8, 0.8, 0.0).into(),
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(builder, ActiveSpellId(0));
                                            spawn_inventory_button(builder, ActiveSpellId(1));
                                            spawn_inventory_button(builder, ActiveSpellId(2));
                                            spawn_inventory_button(builder, ActiveSpellId(3));
                                        });

                                    // Backpack spells
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                height: Val::Percent(70.0),
                                                display: Display::Grid,
                                                align_items: AlignItems::Center,
                                                justify_items: JustifyItems::Center,
                                                grid_template_columns: RepeatedGridTrack::flex(
                                                    4, 1.0,
                                                ),
                                                ..Default::default()
                                            },
                                            background_color: Color::srgb(0.2, 0.8, 0.0).into(),
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(builder, BackpackSpellId(0));
                                            spawn_inventory_button(builder, BackpackSpellId(1));
                                            spawn_inventory_button(builder, BackpackSpellId(2));
                                            spawn_inventory_button(builder, BackpackSpellId(3));
                                            spawn_inventory_button(builder, BackpackSpellId(4));
                                            spawn_inventory_button(builder, BackpackSpellId(5));
                                            spawn_inventory_button(builder, BackpackSpellId(6));
                                            spawn_inventory_button(builder, BackpackSpellId(7));
                                        });
                                });
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

fn update_pause(
    game_state: Res<State<GameState>>,
    mut pause_text: Query<&mut Text, With<PauseText>>,
    mut local: Local<GameState>,
) {
    let Ok(mut pause_text) = pause_text.get_single_mut() else {
        return;
    };

    let current_state = *game_state.get();
    if *local != current_state {
        let text = if current_state == GameState::Paused {
            "Paused"
        } else {
            "Running"
        };

        pause_text.sections[0].value = text.into();
        *local = current_state;
    }
}

fn update_inventory(
    inventory: Res<Inventory>,
    items: Res<Items>,
    spells: Res<Spells>,
    mut active_items_text: Query<
        (&ActiveItemId, &mut Text),
        (
            Without<BackpackItemId>,
            Without<ActiveSpellId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut backpack_items_text: Query<
        (&BackpackItemId, &mut Text),
        (
            Without<ActiveItemId>,
            Without<ActiveSpellId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut active_spells_text: Query<
        (&ActiveSpellId, &mut Text),
        (
            Without<ActiveItemId>,
            Without<BackpackItemId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut backpack_spells_text: Query<
        (&BackpackSpellId, &mut Text),
        (
            Without<ActiveItemId>,
            Without<BackpackItemId>,
            Without<ActiveSpellId>,
        ),
    >,
) {
    for (i, item) in inventory.active_items.iter().enumerate() {
        for (id, mut text) in active_items_text.iter_mut() {
            if id.0 == i as u8 {
                text.sections[0].value = match item {
                    Some(idx) => items.0[idx.0].name.into(),
                    None => "NaN".into(),
                };
            }
        }
    }
    for (i, item) in inventory.backpack_items.iter().enumerate() {
        for (id, mut text) in backpack_items_text.iter_mut() {
            if id.0 == i as u8 {
                text.sections[0].value = match item {
                    Some(idx) => items.0[idx.0].name.into(),
                    None => "NaN".into(),
                };
            }
        }
    }
    for (i, spell) in inventory.active_spells.iter().enumerate() {
        for (id, mut text) in active_spells_text.iter_mut() {
            if id.0 == i as u8 {
                text.sections[0].value = match spell {
                    Some(idx) => spells.0[idx.0].name.into(),
                    None => "NaN".into(),
                };
            }
        }
    }
    for (i, spell) in inventory.backpack_spells.iter().enumerate() {
        for (id, mut text) in backpack_spells_text.iter_mut() {
            if id.0 == i as u8 {
                text.sections[0].value = match spell {
                    Some(idx) => spells.0[idx.0].name.into(),
                    None => "NaN".into(),
                };
            }
        }
    }
}
