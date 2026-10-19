// Depth-only opaque shadows do not use material uniforms, colors or alpha tests.
// The vertex stage already computes the unclipped orthographic depth. Keep the
// fragment interface limited to that value; the result is identical to the full
// PBR prepass's depth output, without its unused material/varying dependencies.
@fragment
fn fragment(@location(6) unclipped_depth: f32) -> @builtin(frag_depth) f32 {
    return unclipped_depth;
}
