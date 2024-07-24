use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

use crate::ui::in_game::{SelectedSectionButton, UI_RIGHT_SIZE, UI_TOP_SIZE};

use super::{
    circle_sectors::{position_to_sector_position, SectorPosition},
    GameCamera, GameRenderLayer, GameState,
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Running), prepare_cursor_resources)
            .add_systems(Update, update_cursor.run_if(in_state(GameState::Paused)));
    }
}

#[derive(Resource, Debug)]
pub struct CursorSector(pub Option<SectorPosition>);

#[derive(Component)]
pub struct Cursor;

fn prepare_cursor_resources(
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(CursorSector(None));

    let mesh = meshes.add(Circle { radius: 2.0 });
    let material = materials.add(Color::srgb(0.9, 0.7, 0.7));
    let transform = Transform::from_xyz(0.0, 0.0, 10.0);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform,
            ..default()
        },
        Cursor,
        game_render_layer.layer.clone(),
    ));
}

fn update_cursor(
    selected_section_button: Res<SelectedSectionButton>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), Without<GameCamera>>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
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

    let Ok(mut ct) = cursor.get_single_mut() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(mut world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // adjust to account for texture is moved to the side
    world_pos.x += window.width() * (UI_RIGHT_SIZE / 100.0) / 2.0;
    world_pos.y += window.height() * (UI_TOP_SIZE / 100.0) / 2.0;

    ct.translation.x = world_pos.x;
    ct.translation.y = world_pos.y;

    let sector_position = position_to_sector_position(world_pos.extend(0.0));
    cursor_sector.0 = Some(SectorPosition(sector_position));
}
