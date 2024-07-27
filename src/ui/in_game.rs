use bevy::prelude::*;

use crate::{
    game::{
        circle_sectors::{FullCycles, SectorPlacedEvent, Sectors},
        inventory::{Inventory, InventoryUpdateEvent},
        items::Items,
        spells::{CastSpellEvent, Spells},
        GameState,
    },
    GlobalState,
};

use super::{UiState, UiStyle};

const BUTTON_IMAGE_TINT_DEFAULT: Color = Color::srgb(0.8, 0.8, 0.8);
const BUTTON_IMAGE_TINT_HOVER: Color = Color::srgb(0.7, 0.7, 0.7);
const BUTTON_IMAGE_TINT_PRESSED: Color = Color::srgb(0.6, 0.6, 0.6);
const BUTTON_IMAGE_TINT_DISABLED: Color = Color::srgb(0.2, 0.2, 0.2);

const TOOLTIP_TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const TOOLTIP_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UiState::InGame), in_game_setup)
            .add_systems(
                Update,
                (
                    button_system,
                    active_items_button_system,
                    backpack_items_button_system,
                    active_spells_update_state,
                    active_spells_button_system,
                    backpack_spells_button_system,
                    backpack_sectors_button_system,
                    backpack_sectors_deselect,
                    backpack_sectors_on_sector_placed,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum UiItemId {
    ActiveItemId(ActiveItemId),
    BackpackItemId(BackpackItemId),
}
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemsTooltipContainer(Option<UiItemId>);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemsTooltipText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum UiSpellId {
    ActiveSpellId(ActiveSpellId),
    BackpackSpellId(BackpackSpellId),
}
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellsTooltipContainer(Option<UiSpellId>);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpellsTooltipText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorsTooltipContainer(Option<BackpackSectorId>);
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorsTooltipText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InGameButton {
    MainMenu,
}

pub const UI_TOP_SIZE: f32 = 10.0;
pub const UI_MIDDLE_SIZE: f32 = 70.0;
pub const UI_BOTTOM_SIZE: f32 = 20.0;

fn spawn_system_button<B>(child_builder: &mut ChildBuilder, style: &UiStyle, button: B)
where
    B: Component + std::fmt::Debug,
{
    child_builder
        .spawn(ButtonBundle {
            style: Style {
                margin: UiRect::all(Val::Percent(1.0)),
                padding: UiRect::all(Val::Percent(1.0)),
                border: UiRect::all(Val::Percent(1.0)),
                // make text in the middle
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            border_radius: BorderRadius::all(Val::Percent(2.0)),
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

fn spawn_inventory_button<C: Component + Copy>(
    builder: &mut ChildBuilder,
    visibility: Visibility,
    c: C,
) {
    builder.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                margin: UiRect::all(Val::Percent(1.0)),
                ..Default::default()
            },
            visibility,
            background_color: BUTTON_IMAGE_TINT_DEFAULT.into(),
            ..Default::default()
        },
        Interaction::default(),
        c,
    ));
}

fn in_game_setup(mut commands: Commands, ui_style: Res<UiStyle>) {
    commands.insert_resource(SelectedSectionButton(None));

    // Tooltip root node
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
            // Top
            builder.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(UI_TOP_SIZE),
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                ..default()
            });
            // Middle
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(UI_MIDDLE_SIZE),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    // Top (unused)
                    builder.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(25.0),
                            ..Default::default()
                        },
                        ..default()
                    });
                    // Zone card tooltip box
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(50.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::End,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            // Actual sector tooltip zone
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(15.0),
                                        height: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_items: JustifyItems::Center,
                                        ..Default::default()
                                    },
                                    ..default()
                                })
                                .with_children(|builder| {
                                    // Tooltip box
                                    builder
                                        .spawn((NodeBundle {
                                            style: Style {
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(50.0),
                                                border: UiRect::all(Val::Percent(1.0)),
                                                align_items: AlignItems::Center,
                                                justify_items: JustifyItems::Center,
                                                ..Default::default()
                                            },
                                            border_color: BorderColor(Color::BLACK),
                                            border_radius: BorderRadius::all(Val::Percent(5.0)),
                                            background_color: TOOLTIP_BACKGROUND_COLOR.into(),
                                            visibility: Visibility::Hidden,
                                            ..Default::default()
                                        },
                                                SectorsTooltipContainer(None),
                                        ))
                                        .with_children(|builder| {
                                            builder.spawn((
                                                TextBundle {
                                                    text: Text::from_section(
                                                        "Some very long and interesting explanation for the sectors",
                                                        TextStyle {
                                                            font_size: 25.0,
                                                            color: TOOLTIP_TEXT_COLOR,
                                                            ..Default::default()
                                                        },
                                                ),
                                                ..Default::default()
                                                },
                                                SectorsTooltipText,
                                            ));
                                        });
                                });
                            // Empty block
                            builder.spawn(NodeBundle {
                                style: Style {
                                    width: Val::Percent(10.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    ..Default::default()
                                },
                                ..default()
                            });
                        });
                    // Items + spells tooltip
                    builder.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(25.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        ..default()
                    }).with_children(|builder|{
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
                                // Items tooltip
                                builder
                                    .spawn((NodeBundle {
                                        style: Style {
                                            width: Val::Percent(50.0),
                                                border: UiRect::all(Val::Percent(1.0)),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..Default::default()
                                        },
                                        border_color: BorderColor(Color::BLACK),
                                        border_radius: BorderRadius::all(Val::Percent(5.0)),
                                        background_color: TOOLTIP_BACKGROUND_COLOR.into(),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    },
                                            ItemsTooltipContainer(None))
                                    )
                                    .with_children(|builder| {
                                        builder.spawn((
                                            TextBundle {
                                                text: Text::from_section(
                                                    "Some very long and interesting explanation for the items",
                                                    TextStyle {
                                                        font_size: 25.0,
                                                        color: TOOLTIP_TEXT_COLOR,
                                                        ..Default::default()
                                                    },
                                            ),
                                            ..Default::default()
                                            },
                                            ItemsTooltipText
                                        ));
                                    });

                                // Spells
                                builder
                                    .spawn((NodeBundle {
                                        style: Style {
                                            width: Val::Percent(50.0),
                                                border: UiRect::all(Val::Percent(1.0)),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..Default::default()
                                        },
                                        border_color: BorderColor(Color::BLACK),
                                        border_radius: BorderRadius::all(Val::Percent(5.0)),
                                        background_color: TOOLTIP_BACKGROUND_COLOR.into(),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    }, SpellsTooltipContainer(None)))
                                    .with_children(|builder| {
                                        builder.spawn((
                                            TextBundle {
                                                text: Text::from_section(
                                                    "Some very long and interesting explanation for the spells",
                                                    TextStyle {
                                                        font_size: 25.0,
                                                        color: TOOLTIP_TEXT_COLOR,
                                                        ..Default::default()
                                                    },
                                            ),
                                            ..Default::default()
                                            },
                                            SpellsTooltipText
                                        ));
                                    });
                            });
                        });
                });

            // Bottom
            builder.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(UI_BOTTOM_SIZE),
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                ..default()
            });
        });

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
                                // align_items: AlignItems::Center,
                                // justify_items: JustifyItems::Start,
                                // justify_content: JustifyContent::Start,
                                ..Default::default()
                            },
                            ..default()
                        })
                        .with_children(|builder| {
                            spawn_system_button(builder, &ui_style, InGameButton::MainMenu);
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
                                width: Val::Percent(10.0),
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
                                        margin: UiRect::all(Val::Percent(1.0)),
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
                                                margin: UiRect::all(Val::Percent(1.0)),
                                                padding: UiRect::all(Val::Percent(5.0)),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                border: UiRect::all(Val::Percent(1.0)),
                                                ..Default::default()
                                            },
                                            border_color: BorderColor(Color::BLACK),
                                            border_radius: BorderRadius::all(Val::Percent(10.0)),
                                            background_color: Color::srgb(0.85, 0.6, 0.15).into(),
                                            ..Default::default()
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
                                                margin: UiRect::all(Val::Percent(1.0)),
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

                            // Vertical line
                            builder.spawn(NodeBundle {
                                style: Style {
                                    width: Val::Percent(1.0),
                                    height: Val::Percent(50.0),
                                    margin: UiRect::all(Val::Percent(1.0)),
                                    align_self: AlignSelf::Center,
                                    ..Default::default()
                                },
                                background_color: Color::BLACK.into(),
                                ..default()
                            });

                            // Spells
                            builder
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(50.0),
                                        margin: UiRect::all(Val::Percent(1.0)),
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
                                                margin: UiRect::all(Val::Percent(1.0)),
                                                padding: UiRect::all(Val::Percent(5.0)),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                border: UiRect::all(Val::Percent(1.0)),
                                                ..Default::default()
                                            },
                                            border_color: BorderColor(Color::BLACK),
                                            border_radius: BorderRadius::all(Val::Percent(10.0)),
                                            background_color: Color::srgb(0.3, 0.6, 1.0).into(),
                                            ..Default::default()
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
                                                margin: UiRect::all(Val::Percent(1.0)),
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

fn active_items_button_system(
    items: Res<Items>,
    inventory: Res<Inventory>,
    mut interaction_query: Query<(&ActiveItemId, &Interaction), Changed<Interaction>>,
    mut tooltip_text: Query<&mut Text, With<ItemsTooltipText>>,
    mut tooltip_container: Query<(&mut Visibility, &mut ItemsTooltipContainer)>,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for (item_id, interaction) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                event_writer.send(InventoryUpdateEvent);
            }
            Interaction::Hovered => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_item_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Ok(mut tooltip_container_text) = tooltip_text.get_single_mut() else {
                    return;
                };

                let Some(item_idx) = inventory.active_items[item_id.0 as usize] else {
                    return;
                };

                let item_info = &items[item_idx];

                *tooltip_container_visibility = Visibility::Visible;
                tooltip_container_item_id.0 = Some(UiItemId::ActiveItemId(*item_id));
                tooltip_container_text.sections[0].value = item_info.description.into();
            }
            Interaction::None => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_item_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Some(tooltip_item_id) = tooltip_container_item_id.0 else {
                    return;
                };
                if tooltip_item_id == UiItemId::ActiveItemId(*item_id) {
                    tooltip_container_item_id.0 = None;
                    *tooltip_container_visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn backpack_items_button_system(
    items: Res<Items>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<(&BackpackItemId, &Interaction), Changed<Interaction>>,
    mut tooltip_text: Query<&mut Text, With<ItemsTooltipText>>,
    mut tooltip_container: Query<(&mut Visibility, &mut ItemsTooltipContainer)>,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for (item_id, interaction) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                inventory.equip_item(item_id.0 as usize);
                event_writer.send(InventoryUpdateEvent);
            }
            Interaction::Hovered => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_item_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Ok(mut tooltip_container_text) = tooltip_text.get_single_mut() else {
                    return;
                };

                let Some(item_idx) = inventory.backpack_items[item_id.0 as usize] else {
                    return;
                };

                let item_info = &items[item_idx];

                *tooltip_container_visibility = Visibility::Visible;
                tooltip_container_item_id.0 = Some(UiItemId::BackpackItemId(*item_id));
                tooltip_container_text.sections[0].value = item_info.description.into();
            }
            Interaction::None => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_item_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Some(tooltip_item_id) = tooltip_container_item_id.0 else {
                    return;
                };
                if tooltip_item_id == UiItemId::BackpackItemId(*item_id) {
                    tooltip_container_item_id.0 = None;
                    *tooltip_container_visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn active_spells_update_state(
    spells: Res<Spells>,
    inventory: Res<Inventory>,
    game_state: Res<State<GameState>>,
    mut interaction_query: Query<(&ActiveSpellId, &mut UiImage)>,
) {
    for (spell_id, mut ui_image) in interaction_query.iter_mut() {
        let on_cooldown = || {
            if let Some(spell_idx) = inventory.get_spell_idx(spell_id.0 as usize) {
                let spell = &spells[spell_idx];
                return !spell.cooldown.finished();
            }
            true
        };

        if on_cooldown() || game_state.get() != &GameState::Battle {
            ui_image.color = BUTTON_IMAGE_TINT_DISABLED;
        } else {
            ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
        }
    }
}

fn active_spells_button_system(
    spells: Res<Spells>,
    inventory: Res<Inventory>,
    mut interaction_query: Query<(&ActiveSpellId, &Interaction), Changed<Interaction>>,
    mut tooltip_text: Query<&mut Text, With<SpellsTooltipText>>,
    mut tooltip_container: Query<(&mut Visibility, &mut SpellsTooltipContainer)>,
    mut event_writer: EventWriter<CastSpellEvent>,
) {
    for (spell_id, interaction) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Some(spell_idx) = inventory.get_spell_idx(spell_id.0 as usize) {
                    event_writer.send(CastSpellEvent(spell_idx));
                }
            }
            Interaction::Hovered => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_spell_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Ok(mut tooltip_container_text) = tooltip_text.get_single_mut() else {
                    return;
                };

                let Some(spell_idx) = inventory.active_spells[spell_id.0 as usize] else {
                    return;
                };

                let spell_info = &spells[spell_idx];

                *tooltip_container_visibility = Visibility::Visible;
                tooltip_container_spell_id.0 = Some(UiSpellId::ActiveSpellId(*spell_id));
                tooltip_container_text.sections[0].value = spell_info.description.into();
            }
            Interaction::None => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_spell_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Some(tooltip_spell_id) = tooltip_container_spell_id.0 else {
                    return;
                };
                if tooltip_spell_id == UiSpellId::ActiveSpellId(*spell_id) {
                    tooltip_container_spell_id.0 = None;
                    *tooltip_container_visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn backpack_spells_button_system(
    spells: Res<Spells>,
    mut inventory: ResMut<Inventory>,
    mut interaction_query: Query<(&BackpackSpellId, &Interaction), Changed<Interaction>>,
    mut tooltip_text: Query<&mut Text, With<SpellsTooltipText>>,
    mut tooltip_container: Query<(&mut Visibility, &mut SpellsTooltipContainer)>,
    mut event_writer: EventWriter<InventoryUpdateEvent>,
) {
    for (spell_id, interaction) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                inventory.equip_spell(spell_id.0 as usize);
                event_writer.send(InventoryUpdateEvent);
            }
            Interaction::Hovered => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_spell_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Ok(mut tooltip_container_text) = tooltip_text.get_single_mut() else {
                    return;
                };

                let Some(spell_idx) = inventory.backpack_spells[spell_id.0 as usize] else {
                    return;
                };

                let spell_info = &spells[spell_idx];

                *tooltip_container_visibility = Visibility::Visible;
                tooltip_container_spell_id.0 = Some(UiSpellId::BackpackSpellId(*spell_id));
                tooltip_container_text.sections[0].value = spell_info.description.into();
            }
            Interaction::None => {
                let Ok((mut tooltip_container_visibility, mut tooltip_container_spell_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Some(tooltip_spell_id) = tooltip_container_spell_id.0 else {
                    return;
                };
                if tooltip_spell_id == UiSpellId::BackpackSpellId(*spell_id) {
                    tooltip_container_spell_id.0 = None;
                    *tooltip_container_visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn backpack_sectors_button_system(
    sectors: Res<Sectors>,
    inventory: Res<Inventory>,
    mut selected_section_button: ResMut<SelectedSectionButton>,
    mut interaction_query: Query<
        (Entity, &BackpackSectorId, &Interaction, &mut UiImage),
        Changed<Interaction>,
    >,
    mut tooltip_text: Query<&mut Text, With<SectorsTooltipText>>,
    mut tooltip_container: Query<(&mut Visibility, &mut SectorsTooltipContainer)>,
) {
    for (entity, sector_id, interaction, mut ui_image) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                selected_section_button.0 = Some(entity);
            }
            Interaction::Hovered => {
                ui_image.color = BUTTON_IMAGE_TINT_HOVER;

                let Ok((mut tooltip_container_visibility, mut tooltip_container_sector_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Ok(mut tooltip_container_text) = tooltip_text.get_single_mut() else {
                    return;
                };

                let Some(sector_idx) = inventory.backpack_sectors[sector_id.0 as usize] else {
                    return;
                };

                let sector_info = &sectors[sector_idx];

                *tooltip_container_visibility = Visibility::Visible;
                tooltip_container_sector_id.0 = Some(*sector_id);
                tooltip_container_text.sections[0].value = sector_info.description.into();
            }
            Interaction::None => {
                ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;

                let Ok((mut tooltip_container_visibility, mut tooltip_container_sector_id)) =
                    tooltip_container.get_single_mut()
                else {
                    return;
                };
                let Some(tooltip_sector_id) = tooltip_container_sector_id.0 else {
                    return;
                };
                if tooltip_sector_id == *sector_id {
                    tooltip_container_sector_id.0 = None;
                    *tooltip_container_visibility = Visibility::Hidden;
                }
            }
        }

        if let Some(e) = selected_section_button.0 {
            if e == entity {
                ui_image.color = BUTTON_IMAGE_TINT_PRESSED;
            }
        }
    }
}

fn backpack_sectors_deselect(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut selected_section_button: ResMut<SelectedSectionButton>,
    mut section_image: Query<&mut UiImage, With<BackpackSectorId>>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        if let Some(e) = selected_section_button.0 {
            let Ok(mut ui_image) = section_image.get_mut(e) else {
                return;
            };

            ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
        }
        selected_section_button.0 = None;
    }
}

fn backpack_sectors_on_sector_placed(
    mut event_reader: EventReader<SectorPlacedEvent>,
    mut tooltip_container: Query<(&mut Visibility, &mut SectorsTooltipContainer)>,
) {
    for _ in event_reader.read() {
        let Ok((mut tooltip_container_visibility, mut tooltip_container_sector_id)) =
            tooltip_container.get_single_mut()
        else {
            return;
        };
        tooltip_container_sector_id.0 = None;
        *tooltip_container_visibility = Visibility::Hidden;
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
        (&ActiveItemId, &mut UiImage),
        (
            Without<BackpackItemId>,
            Without<ActiveSpellId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut backpack_items_text: Query<
        (&BackpackItemId, &mut UiImage),
        (
            Without<ActiveItemId>,
            Without<ActiveSpellId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut active_spells_text: Query<
        (&ActiveSpellId, &mut UiImage),
        (
            Without<ActiveItemId>,
            Without<BackpackItemId>,
            Without<BackpackSpellId>,
        ),
    >,
    mut backpack_spells_text: Query<
        (&BackpackSpellId, &mut UiImage),
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
            for (id, mut ui_image) in active_items_text.iter_mut() {
                if id.0 == i as u8 {
                    *ui_image = match item {
                        Some(idx) => items[*idx].image.clone().into(),
                        None => Default::default(),
                    };
                    ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
                }
            }
        }
        for (i, item) in inventory.backpack_items.iter().enumerate() {
            for (id, mut ui_image) in backpack_items_text.iter_mut() {
                if id.0 == i as u8 {
                    *ui_image = match item {
                        Some(idx) => items[*idx].image.clone().into(),
                        None => Default::default(),
                    };
                    ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
                }
            }
        }
        for (i, spell) in inventory.active_spells.iter().enumerate() {
            for (id, mut ui_image) in active_spells_text.iter_mut() {
                if id.0 == i as u8 {
                    *ui_image = match spell {
                        Some(idx) => spells[*idx].image.clone().into(),
                        None => Default::default(),
                    };
                    ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
                }
            }
        }
        for (i, spell) in inventory.backpack_spells.iter().enumerate() {
            for (id, mut ui_image) in backpack_spells_text.iter_mut() {
                if id.0 == i as u8 {
                    *ui_image = match spell {
                        Some(idx) => spells[*idx].image.clone().into(),
                        None => Default::default(),
                    };
                    ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
                }
            }
        }
    }
}

fn update_sectors(
    sectors: Res<Sectors>,
    inventory: Res<Inventory>,
    mut selected_section_button: ResMut<SelectedSectionButton>,
    mut tooltip_container: Query<(&mut Visibility, &mut SectorsTooltipContainer), Without<UiImage>>,
    mut sectors_buttons: Query<(&BackpackSectorId, &mut Visibility, &mut UiImage)>,
    mut event_reader: EventReader<InventoryUpdateEvent>,
) {
    for _ in event_reader.read() {
        for (button_sector_id, mut button_visibility, mut ui_image) in sectors_buttons.iter_mut() {
            if let Some(sector_idx) = inventory.backpack_sectors[button_sector_id.0 as usize] {
                let sector_info = &sectors[sector_idx];

                *button_visibility = Visibility::Visible;
                *ui_image = sector_info.card.clone().into();
                ui_image.color = BUTTON_IMAGE_TINT_DEFAULT;
            } else {
                *button_visibility = Visibility::Hidden;
            }
        }

        selected_section_button.0 = None;

        let Ok((mut tooltip_container_visibility, mut tooltip_container_sector_id)) =
            tooltip_container.get_single_mut()
        else {
            return;
        };
        tooltip_container_sector_id.0 = None;
        *tooltip_container_visibility = Visibility::Hidden;
    }
}
