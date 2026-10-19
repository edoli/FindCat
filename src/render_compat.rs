//! Portable shader specializations and Bevy 0.20 PBR compatibility.
//! https://github.com/bevyengine/bevy/issues/26078
use bevy::{
    prelude::*,
    shader::{ShaderDefVal, Source},
};

const LOAD_WORLD: &str = "@if(!MESHLET_MESH_MATERIAL_PASS)\nfn get_world_from_local(instance_index: u32) -> mat4x4<f32> {\n    return affine3_to_square(mesh[instance_index].world_from_local);\n}";
const STATIC_SHADOW_LOAD: &str =
    "WEBGL2 && UNCLIPPED_DEPTH_ORTHO_EMULATION && !MESHLET_MESH_MATERIAL_PASS";

/// Adreno 730 fails to link depth-writing programs whose interpolated depth
/// depends on dynamically indexed mesh uniforms. Each stage compiles, and both
/// the same vertex with constant depth and the same depth with a constant mesh
/// index link successfully (the on-device diagnostic isolates this combination).
///
/// Select the actual instance through constant-address buffer loads. Preserve
/// batching, every instance's transform, the uniform layout and depth arithmetic.
/// The slot count comes from Bevy's binding definition, never from a device name
/// or a hard-coded limit. Other passes keep their original lookup.
pub fn specialize_shadow_mesh_lookup(
    mut shaders: ResMut<Assets<Shader>>,
    mut checked: Local<bool>,
) {
    if *checked {
        return;
    }
    let Some(bindings) = shaders
        .iter()
        .map(|(_, shader)| shader)
        .find(|shader| shader.path.ends_with("render/mesh_bindings.wesl"))
    else {
        return;
    };
    let batch_size = bindings.shader_defs.iter().find_map(|def| match def {
        ShaderDefVal::UInt(name, count) if name == "PER_OBJECT_BUFFER_BATCH_SIZE" => Some(*count),
        _ => None,
    });
    // Storage-buffer renderers do not use this uniform-array path.
    let Some(batch_size) = batch_size else {
        *checked = true;
        return;
    };
    let candidate = shaders.iter().find_map(|(id, shader)| {
        if !shader.path.ends_with("render/mesh_functions.wesl") {
            return None;
        }
        let Source::Wesl(source) = &shader.source else {
            return None;
        };
        assert_eq!(source.matches(LOAD_WORLD).count(), 1);
        assert!(batch_size > 0);
        let original = LOAD_WORLD.replacen(
            "@if(!MESHLET_MESH_MATERIAL_PASS)",
            &format!("@if(!MESHLET_MESH_MATERIAL_PASS && !({STATIC_SHADOW_LOAD}))"),
            1,
        );
        let mut lookup = format!(
            "\n@if({STATIC_SHADOW_LOAD})\nfn get_world_from_local(instance_index: u32) -> mat4x4<f32> {{\n    switch instance_index {{\n"
        );
        for slot in 0..batch_size - 1 {
            lookup.push_str(&format!(
                "        case {slot}u: {{ return affine3_to_square(mesh[{slot}u].world_from_local); }}\n"
            ));
        }
        // Keep any other unsigned index inside the buffer, matching the captured
        // WebGL translation's clamp. Every valid index retains its exact slot.
        lookup.push_str(&format!(
            "        default: {{ return affine3_to_square(mesh[{}u].world_from_local); }}\n    }}\n}}\n",
            batch_size - 1
        ));
        Some((id, source.replacen(LOAD_WORLD, &original, 1) + &lookup))
    });
    if let Some((id, source)) = candidate {
        if let Some(mut shader) = shaders.get_mut(id) {
            shader.source = Source::Wesl(source.into());
            *checked = true;
            info!("Specialized shadow mesh lookup with {batch_size} constant-address slots");
        }
    }
}

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
