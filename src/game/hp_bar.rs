use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::{GameState, Health};

const HP_BAR_WIDTH: f32 = 10.0;
const HP_BAR_HEIGHT: f32 = 2.0;

pub struct HpBarPlugin;

impl Plugin for HpBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_hp_bar_resources)
            .add_systems(Update, update_hp_bar.run_if(in_state(GameState::Battle)));
    }
}

#[derive(Resource, Debug, Clone)]
pub struct HpBarResources {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component, Debug)]
pub struct HpBar {
    parent_entity: Entity,
}

fn prepare_hp_bar_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(HP_BAR_WIDTH, HP_BAR_HEIGHT));
    let material = materials.add(Color::srgb(1.0, 0.0, 0.0));
    commands.insert_resource(HpBarResources { mesh, material })
}

pub fn hp_bar_bundle(hp_bar_resources: &HpBarResources, parent_entity: Entity) -> impl Bundle {
    (
        MaterialMesh2dBundle {
            mesh: hp_bar_resources.mesh.clone().into(),
            material: hp_bar_resources.material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
        HpBar { parent_entity },
    )
}

fn update_hp_bar(with_hp: Query<&Health>, mut hp_bars: Query<(&HpBar, &mut Transform)>) {
    for (hp_bar, mut hp_bar_transform) in hp_bars.iter_mut() {
        let Ok(health) = with_hp.get(hp_bar.parent_entity) else {
            continue;
        };
        let percent = health.current / health.max;

        hp_bar_transform.scale.x = percent;
        let offset = HP_BAR_WIDTH / 2.0 * (1.0 - percent);
        hp_bar_transform.translation.x = -offset;
    }
}
