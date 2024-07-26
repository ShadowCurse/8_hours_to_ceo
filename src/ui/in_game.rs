use bevy::prelude::*;

use crate::{
    game::{
        circle_sectors::{FullCycles, Sectors},
        inventory::{Inventory, InventoryUpdateEvent},
        items::Items,
        spells::{CastSpellEvent, Spells},
        GameState,
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
                (
                    button_system,
                    backpack_items_button_system,
                    active_spells_button_system,
                    backpack_spells_button_system,
                    backpack_sectors_button_system,
                    update_cycles,
                    update_pause,
                    update_inventory,
                    update_sectors,
                )
                    .run_if(in_state(UiState::InGame)),
            );
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectedSectionButton(pub Option<Entity>);

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
pub struct BackpackSectorId(pub u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InGameButton {
    Settings,
    MainMenu,
}

pub const UI_TOP_SIZE: f32 = 10.0;
pub const UI_MIDDLE_SIZE: f32 = 70.0;
pub const UI_BOTTOM_SIZE: f32 = 20.0;

fn spawn_inventory_button<C: Component + Copy>(
    builder: &mut ChildBuilder,
    visibility: Visibility,
    c: C,
) {
    builder
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Percent(1.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                visibility,
                border_color: BorderColor(Color::BLACK),
                border_radius: BorderRadius::all(Val::Percent(5.0)),
                background_color: Color::WHITE.into(),
                ..default()
            },
            c,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section(
                    "NaN",
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

fn in_game_setup(mut commands: Commands, ui_style: Res<UiStyle>) {
    commands.insert_resource(SelectedSectionButton(None));

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
                    // Cycles
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0 / 3.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
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
                        });

                    // Pause state
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0 / 3.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            // Pause state text
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
                                width: Val::Percent(100.0 / 3.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                justify_content: JustifyContent::SpaceAround,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            spawn_button(builder, &ui_style, InGameButton::Settings);
                            spawn_button(builder, &ui_style, InGameButton::MainMenu);
                        });
                });

            // Zones cards on the right
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(UI_MIDDLE_SIZE),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::End,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    // Inner container for zone buttons
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(5.0),
                                height: Val::Percent(80.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            spawn_inventory_button(
                                builder,
                                Visibility::Hidden,
                                BackpackSectorId(0),
                            );
                            spawn_inventory_button(
                                builder,
                                Visibility::Hidden,
                                BackpackSectorId(1),
                            );
                            spawn_inventory_button(
                                builder,
                                Visibility::Hidden,
                                BackpackSectorId(2),
                            );
                            spawn_inventory_button(
                                builder,
                                Visibility::Hidden,
                                BackpackSectorId(3),
                            );
                        });
                });

            // Items and spells
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(UI_BOTTOM_SIZE),
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    // Center container
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(40.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceEvenly,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            // Items
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(50.0),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        ..Default::default()
                                    },
                                    ..default()
                                })
                                .with_children(|builder| {
                                    // Active items
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..Default::default()
                                            },
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveItemId(0),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveItemId(1),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveItemId(2),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveItemId(3),
                                            );
                                        });

                                    // Backpack items
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..Default::default()
                                            },
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackItemId(0),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackItemId(1),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackItemId(2),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackItemId(3),
                                            );
                                        });
                                });

                            // Spells
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(50.0),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        ..Default::default()
                                    },
                                    ..default()
                                })
                                .with_children(|builder| {
                                    // Active spells
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..Default::default()
                                            },
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveSpellId(0),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveSpellId(1),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveSpellId(2),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                ActiveSpellId(3),
                                            );
                                        });

                                    // Backpack spells
                                    builder
                                        .spawn(NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..Default::default()
                                            },
                                            ..default()
                                        })
                                        .with_children(|builder| {
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackSpellId(0),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackSpellId(1),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackSpellId(2),
                                            );
                                            spawn_inventory_button(
                                                builder,
                                                Visibility::default(),
                                                BackpackSpellId(3),
                                            );
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

fn backpack_items_button_system(
    ui_style: Res<UiStyle>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<
        (&BackpackItemId, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for (item_id, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                inventory.equip_item(item_id.0 as usize);
                event_writer.send(InventoryUpdateEvent);
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

fn active_spells_button_system(
    spells: Res<Spells>,
    ui_style: Res<UiStyle>,
    inventory: Res<Inventory>,
    game_state: Res<State<GameState>>,
    mut interaction_query: Query<
        (&ActiveSpellId, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<CastSpellEvent>,
) {
    for (spell_id, interaction, mut color) in interaction_query.iter_mut() {
        let on_cooldown = || {
            if let Some(spell_idx) = inventory.get_spell_idx(spell_id.0 as usize) {
                let spell = &spells[spell_idx];
                return !spell.cooldown.finished();
            }
            true
        };

        if on_cooldown() || game_state.get() != &GameState::Battle {
            *color = ui_style.btn_color_disabled.into();
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *color = ui_style.btn_color_pressed.into();
                    if let Some(spell_idx) = inventory.get_spell_idx(spell_id.0 as usize) {
                        event_writer.send(CastSpellEvent(spell_idx));
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
}

fn backpack_spells_button_system(
    ui_style: Res<UiStyle>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<
        (&BackpackSpellId, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for (spell_id, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                inventory.equip_spell(spell_id.0 as usize);
                event_writer.send(InventoryUpdateEvent);
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

fn backpack_sectors_button_system(
    ui_style: Res<UiStyle>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_section_button: ResMut<SelectedSectionButton>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        With<BackpackSectorId>,
    >,
) {
    for (entity, interaction, mut color) in interaction_query.iter_mut() {
        if let Some(e) = selected_section_button.0 {
            if e == entity {
                *color = ui_style.btn_color_disabled.into();
                continue;
            }
        }
        match *interaction {
            Interaction::Pressed => {
                *color = ui_style.btn_color_pressed.into();
                selected_section_button.0 = Some(entity);
            }
            Interaction::Hovered => {
                *color = ui_style.btn_color_hover.into();
            }
            Interaction::None => {
                *color = ui_style.btn_color_normal.into();
            }
        }
    }
    if mouse_input.just_pressed(MouseButton::Right) {
        selected_section_button.0 = None;
    }
}

fn update_cycles(full_cycles: Res<FullCycles>, mut pause_text: Query<&mut Text, With<CyclesText>>) {
    let Ok(mut pause_text) = pause_text.get_single_mut() else {
        return;
    };

    pause_text.sections[0].value = format!("Cycles: {}", full_cycles.0);
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
    mut event_reader: EventReader<InventoryUpdateEvent>,
) {
    for _ in event_reader.read() {
        for (i, item) in inventory.active_items.iter().enumerate() {
            for (id, mut text) in active_items_text.iter_mut() {
                if id.0 == i as u8 {
                    text.sections[0].value = match item {
                        Some(idx) => items[*idx].name.into(),
                        None => "NaN".into(),
                    };
                }
            }
        }
        for (i, item) in inventory.backpack_items.iter().enumerate() {
            for (id, mut text) in backpack_items_text.iter_mut() {
                if id.0 == i as u8 {
                    text.sections[0].value = match item {
                        Some(idx) => items[*idx].name.into(),
                        None => "NaN".into(),
                    };
                }
            }
        }
        for (i, spell) in inventory.active_spells.iter().enumerate() {
            for (id, mut text) in active_spells_text.iter_mut() {
                if id.0 == i as u8 {
                    text.sections[0].value = match spell {
                        Some(idx) => spells[*idx].name.into(),
                        None => "NaN".into(),
                    };
                }
            }
        }
        for (i, spell) in inventory.backpack_spells.iter().enumerate() {
            for (id, mut text) in backpack_spells_text.iter_mut() {
                if id.0 == i as u8 {
                    text.sections[0].value = match spell {
                        Some(idx) => spells[*idx].name.into(),
                        None => "NaN".into(),
                    };
                }
            }
        }
    }
}

fn update_sectors(
    sectors: Res<Sectors>,
    inventory: Res<Inventory>,
    mut sectors_buttons: Query<(&BackpackSectorId, &mut Visibility), With<Button>>,
    mut sectors_texts: Query<(&BackpackSectorId, &mut Text)>,
    mut event_reader: EventReader<InventoryUpdateEvent>,
) {
    for _ in event_reader.read() {
        for (button_sector_id, mut button_visibility) in sectors_buttons.iter_mut() {
            let Some((_, mut text)) = sectors_texts
                .iter_mut()
                .find(|(text_sector_id, _)| **text_sector_id == *button_sector_id)
            else {
                continue;
            };

            if let Some(sector_idx) = inventory.backpack_sectors[button_sector_id.0 as usize] {
                let sector_info = &sectors[sector_idx];

                *button_visibility = Visibility::Visible;
                text.sections[0].value = sector_info.name.into();
            } else {
                *button_visibility = Visibility::Hidden;
            }
        }
    }
}
