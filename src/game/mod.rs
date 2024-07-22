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

mod circle_sectors;
mod enemy;
mod items;

use circle_sectors::SectorsPlugin;
use enemy::EnemyPlugin;
use items::ItemsPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SectorsPlugin, EnemyPlugin, ItemsPlugin))
            .add_sub_state::<GameState>()
            .add_systems(Startup, setup_game)
            .add_systems(OnEnter(GameState::Preparing), spawn_base_game)
            .add_systems(
                Update,
                (player_run, camera_follow_player).run_if(in_state(GameState::Running)),
            )
            .add_systems(
                OnTransition {
                    exited: GameState::Running,
                    entered: GameState::Paused,
                },
                move_camera_default,
            )
            .add_systems(Update, game_pause.run_if(state_exists::<GameState>));
    }
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InGame)]
pub enum GameState {
    #[default]
    Preparing,
    Running,
    Battle,
    Paused,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameImage {
    pub image: Handle<Image>,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GameRenderLayer {
    layer: RenderLayers,
}

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
    game_render_layer: Res<GameRenderLayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Circle { radius: 20.0 });
    let material = materials.add(Color::srgb(0.1, 0.9, 0.2));

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_xyz(0.0, 220.0, 1.0),
            ..default()
        },
        Player,
        PlayerSpeed(0.1),
        Wireframe2d,
        game_render_layer.layer.clone(),
        StateScoped(GlobalState::InGame),
    ));

    game_state.set(GameState::Running);
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

fn game_pause(
    key_input: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut game_state_next: ResMut<NextState<GameState>>,
    mut local: Local<GameState>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        if game_state.get() == &GameState::Paused {
            game_state_next.set(*local);
        } else {
            *local = *game_state.get();
            game_state_next.set(GameState::Paused);
        }
    }
}

fn camera_follow_player(
    player: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };

    let mut t = *player_transform;
    t.scale = Vec3::new(0.5, 0.5, 0.5);
    *camera_transform = t;
}

fn move_camera_default(mut camera: Query<&mut Transform, With<GameCamera>>) {
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };
    *camera_transform = Transform::default();
}
