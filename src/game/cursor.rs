use bevy::{prelude::*, window::PrimaryWindow};

use crate::ui::in_game::SelectedSectionButton;

use super::{
    circle_sectors::{
        position_to_sector_position, SectorPosition, CIRCLE_INNER_RADIUS, CIRCLE_RADIUS,
    },
    GameCamera, GameState,
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_cursor_resources)
            .add_systems(Update, update_cursor.run_if(in_state(GameState::Paused)));
    }
}

#[derive(Resource, Debug)]
pub struct CursorSector(pub Option<SectorPosition>);

fn prepare_cursor_resources(mut commands: Commands) {
    commands.insert_resource(CursorSector(None));
}

fn update_cursor(
    selected_section_button: Res<SelectedSectionButton>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut cursor_sector: ResMut<CursorSector>,
) {
    if selected_section_button.0.is_none() {
        cursor_sector.0 = None;
        return;
    }
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if world_pos.length() < CIRCLE_INNER_RADIUS || CIRCLE_RADIUS < world_pos.length() {
        cursor_sector.0 = None;
        return;
    }

    let sector_position = position_to_sector_position(world_pos.extend(0.0));
    cursor_sector.0 = Some(SectorPosition(sector_position));
}
