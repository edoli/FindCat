//! Portable shader specializations and Bevy 0.20 PBR compatibility.
//! https://github.com/bevyengine/bevy/issues/26078
use bevy::{prelude::*, shader::Source};

// This is a pipeline specialization, independent of device names or user agents.
// Keep all material-dependent, dithered and normal/motion/deferred passes intact.
const OPAQUE_DEPTH: &str = "PREPASS_FRAGMENT && UNCLIPPED_DEPTH_ORTHO_EMULATION && !NORMAL_PREPASS && !MOTION_VECTOR_PREPASS && !DEFERRED_PREPASS && !MAY_DISCARD && !VISIBILITY_RANGE_DITHER && !MESHLET_MESH_MATERIAL_PASS";

/// Give the depth-only shadow pass its actual scalar interface instead of the
/// full PBR fragment interface and unused StandardMaterial uniform reads.
pub fn specialize_opaque_depth(mut shaders: ResMut<Assets<Shader>>, mut checked: Local<bool>) {
    if *checked {
        return;
    }
    let candidate = shaders.iter().find_map(|(id, shader)| {
        if !shader.path.ends_with("render/pbr_prepass.wesl") {
            return None;
        }
        let Source::Wesl(source) = &shader.source else {
            return None;
        };
        let full_fragment = "@if(PREPASS_FRAGMENT)\n@fragment\nfn fragment(";
        let fallback_fragment = "@else\n@fragment\nfn fragment(in: prepass_io::VertexOutput)";
        // Fail visibly if a future engine update changes this shader's structure.
        assert_eq!(source.matches(full_fragment).count(), 1);
        assert_eq!(source.matches(fallback_fragment).count(), 1);
        let source = source
            .replacen(
                full_fragment,
                &format!("@if(PREPASS_FRAGMENT && !({OPAQUE_DEPTH}))\n@fragment\nfn fragment("),
                1,
            )
            .replacen(
                fallback_fragment,
                "@elif(!PREPASS_FRAGMENT)\n@fragment\nfn fragment(in: prepass_io::VertexOutput)",
                1,
            );
        Some((
            id,
            format!(
                "{source}\n@if({OPAQUE_DEPTH})\n{}",
                include_str!("../assets/shaders/opaque_depth.wesl")
            ),
        ))
    });
    if let Some((id, source)) = candidate {
        if let Some(mut shader) = shaders.get_mut(id) {
            shader.source = Source::Wesl(source.into());
            *checked = true;
            info!("Specialized opaque shadow fragment interface");
        }
    }
}

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
