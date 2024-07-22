use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prepare_items_resources);
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ItemsResources {
    pub material_default: Handle<ColorMaterial>,
    pub mesh_default: Handle<Mesh>,
}

fn prepare_items_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default = materials.add(Color::srgb(0.7, 0.3, 0.8));
    let mesh_default = meshes.add(Rectangle::new(20.0, 10.0));

    commands.insert_resource(ItemsResources {
        material_default,
        mesh_default,
    });
}
