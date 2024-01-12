use std::path::{Path, PathBuf};

use bevy::{
    asset::Handle,
    prelude::{Assets, Mesh, Plugin},
};
use bevy_svg::prelude::Svg;

pub const EXPAND_LESS_HANDLE: Handle<Svg> = Handle::weak_from_u128(4238701051302568451);

pub const EXPAND_MORE_HANDLE: Handle<Svg> = Handle::weak_from_u128(9116091369991258337);

pub struct IconsPlugin;
impl Plugin for IconsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let expand_less_bytes = include_bytes!("expand_less.svg");
        let expand_more_bytes = include_bytes!("expand_more.svg");
        let mut expand_less =
            Svg::from_bytes(expand_less_bytes, Path::new(""), None::<PathBuf>).unwrap();
        let mut expand_more =
            Svg::from_bytes(expand_more_bytes, Path::new(""), None::<PathBuf>).unwrap();

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        expand_less.mesh = meshes.add(expand_less.tessellate());
        expand_more.mesh = meshes.add(expand_more.tessellate());

        let mut svgs = app.world.get_resource_mut::<Assets<Svg>>().unwrap();
        svgs.insert(EXPAND_LESS_HANDLE, expand_less);
        svgs.insert(EXPAND_MORE_HANDLE, expand_more);
    }
}
