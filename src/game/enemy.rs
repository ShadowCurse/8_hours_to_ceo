use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_enemy_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EnemyResources {
    mesh_default: Handle<Mesh>,
    material_default: Handle<ColorMaterial>,
}

fn prepare_enemy_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.1, 0.0, 0.0));
    let mesh_default = meshes.add(Circle { radius: 10.0 });

    commands.insert_resource(EnemyResources {
        mesh_default,
        material_default,
    });
}
