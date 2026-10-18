//! Compatibility for NVIDIA 570-series drivers with Bevy 0.20's PBR shader.
//! https://github.com/bevyengine/bevy/issues/26078
use bevy::{prelude::*, shader::Source};

/// Some Vulkan drivers miscompile boolean equality involving `front_facing`.
/// `select` expresses the same winding correction without flipping front-face
/// normals. Keep double-sided materials, mirrored transforms and shadows intact.
pub fn correct_front_facing(mut shaders: ResMut<Assets<Shader>>, mut checked: Local<bool>) {
    if *checked {
        return;
    }
    let candidate = shaders.iter().find_map(|(id, shader)| {
        if !shader.path.ends_with("render/pbr_functions.wesl") {
            return None;
        }
        let Source::Wesl(source) = &shader.source else {
            return None;
        };
        Some((
            id,
            source.replace(
                "return is_front == positive_determinant;",
                "return select(!is_front, is_front, positive_determinant);",
            ),
        ))
    });
    if let Some((id, source)) = candidate {
        if let Some(mut shader) = shaders.get_mut(id) {
            shader.source = Source::Wesl(source.into());
            *checked = true;
            info!("Applied front-facing shader compatibility correction");
        }
    }
}
