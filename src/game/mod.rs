use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{MaterialMesh2dBundle, Wireframe2d},
    window::PrimaryWindow,
};

use crate::{
    ui::in_game::{UI_RIGHT_SIZE, UI_TOP_SIZE},
    GlobalState,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>()
            .add_systems(Startup, setup_game)
            .add_systems(OnEnter(GameState::Running), spawn_base_game)
            .add_systems(
                Update,
                (player_run, section_detect_player).run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(SubStates, Debug, Default, Clone, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InGame)]
pub enum GameState {
    #[default]
    Running,
    Battle,
    Paused {
        in_settings: bool,
    },
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameImage {
    pub image: Handle<Image>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GameRenderLayer {
    layer: RenderLayers,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SectorMaterials {
    default: Handle<ColorMaterial>,
    with_player: Handle<ColorMaterial>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircleSector(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerSpeed(f32);

fn setup_game(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(primary_window) = windows.get_single() else {
        return;
    };

    let size = Extent3d {
        width: (primary_window.resolution.width() * (100.0 - UI_RIGHT_SIZE) / 100.0) as u32,
        height: (primary_window.resolution.height() * (100.0 - UI_TOP_SIZE) / 100.0) as u32,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: image_handle.clone().into(),
                clear_color: Color::BLACK.into(),
                ..default()
            },
            ..default()
        },
        GameCamera,
        first_pass_layer.clone(),
    ));

    commands.insert_resource(GameRenderLayer {
        layer: first_pass_layer,
    });
    commands.insert_resource(GameImage {
        image: image_handle,
    });
}

fn spawn_base_game(
    mut commands: Commands,
    game_render_layer: Res<GameRenderLayer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let section_default_material = materials.add(Color::srgb(0.7, 0.7, 0.7));
    let section_with_player_material = materials.add(Color::srgb(0.2, 0.8, 0.2));

    let mesh = meshes.add(CircularSector::new(200.0, std::f32::consts::FRAC_PI_8));

    // Sectors
    for i in 0..8 {
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        // Rotate PI/8 more to start section at 0/12
        transform
            .rotate_local_z(std::f32::consts::FRAC_PI_8 + std::f32::consts::FRAC_PI_4 * i as f32);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: section_default_material.clone(),
                transform,
                ..default()
            },
            CircleSector(i),
            game_render_layer.layer.clone(),
            StateScoped(GlobalState::InGame),
        ));
    }

    // Center
    let mesh = meshes.add(Circle { radius: 180.0 });
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material: section_default_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));

    commands.insert_resource(SectorMaterials {
        default: section_default_material,
        with_player: section_with_player_material,
    });

    let mesh = meshes.add(Circle { radius: 20.0 });
    let material = materials.add(Color::srgb(0.1, 0.9, 0.2));

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_xyz(0.0, 220.0, 0.0),
            ..default()
        },
        Player,
        PlayerSpeed(0.5),
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));
}

fn player_run(time: Res<Time>, mut player: Query<(&PlayerSpeed, &mut Transform)>) {
    let Ok((speed, mut transform)) = player.get_single_mut() else {
        return;
    };

    let to_center = transform.translation;
    let rotation = Quat::from_rotation_z(-speed.0 * time.delta_seconds());
    let rotated = rotation * to_center;

    transform.translation = rotated;
    transform.rotation *= rotation;
}

fn section_detect_player(
    section_materials: Res<SectorMaterials>,
    player: Query<&Transform, With<Player>>,
    mut sections: Query<(&CircleSector, &mut Handle<ColorMaterial>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let to_center = player_transform.translation.normalize();
    let angle = to_center.angle_between(Vec3::Y);

    let mut section_id = (angle / std::f32::consts::FRAC_PI_4).floor() as u8;

    if 0.0 < player_transform.translation.x {
        section_id = 7 - section_id;
    }

    let previous_section_id = if section_id == 7 { 0 } else { section_id + 1 };

    for (section, mut material_handle) in sections.iter_mut() {
        if section.0 == section_id {
            *material_handle = section_materials.with_player.clone();
        }
        if section.0 == previous_section_id {
            *material_handle = section_materials.default.clone();
        }
    }
}
