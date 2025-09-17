//! [Bevy TrenchBroom](https://github.com/Noxmore/bevy_trenchbroom) is the integration layer between Bevy and [TrenchBroom](https://trenchbroom.github.io/).
//! We use TrenchBroom to edit our levels.

use bevy::{image::ImageSampler, prelude::*};
use bevy_trenchbroom::prelude::*;

pub(crate) use util::*;

use crate::asset_processing::default_image_sampler_descriptor;

mod util;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TrenchBroomPlugins(
        TrenchBroomConfig::new("foxtrot")
            .texture_extensions(to_string_vec(&["png", "jpg", "jpeg"]))
            .texture_exclusions(to_string_vec(&[
                "*_disp_*",
                "*_arm_*",
                "*_nor_*",
                "*_local",
                "*_normal",
                "*_roughness",
            ]))
            .texture_sampler(texture_sampler())
            .default_solid_spawn_hooks(|| {
                SpawnHooks::new()
                    .convex_collider()
                    .smooth_by_default_angle()
            }),
    ));
    app.add_plugins(util::plugin);
}

fn texture_sampler() -> ImageSampler {
    let mut sampler = ImageSampler::linear();
    *sampler.get_or_init_descriptor() = default_image_sampler_descriptor();
    sampler
}

fn to_string_vec(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| s.to_string()).collect()
}
