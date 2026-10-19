"""Compare instanced shadow depth on a real GPU, including nonzero base indices.

--programs is the programs.json saved by the WebGL shader capture helper.
Reference shaders are the original Bevy shadow fixtures in gpu-diagnostics.
Depth32Float is read back losslessly by encoding its bits into RGBA8UI.
"""
import argparse
import json
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--programs', type=Path, required=True)
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/shadow-depth')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
reference = json.loads((ROOT / 'web/gpu-diagnostics/programs.json').read_text(encoding='utf-8'))['programs']
reference = [p for p in reference if p['id'].startswith('previous-shadow')]
captured = json.loads(args.programs.read_text(encoding='utf-8'))
candidate = []
for program in captured:
    vertex = next(s['source'] for s in program['sources'] if s['stage'] == 35633)
    fragment = next(s['source'] for s in program['sources'] if s['stage'] == 35632)
    if 'prepass_io' in vertex and 'gl_FragDepth' in fragment:
        candidate.append({'vertex': vertex, 'fragment': fragment})
assert len(reference) == len(candidate) == 2, 'Expected textured and untextured shadow programs'

COMPARE = r'''({reference, candidate}) => {
  const gl = document.createElement('canvas').getContext('webgl2', {antialias: false});
  if (!gl) throw Error('GPU WebGL2 required');
  const debug = gl.getExtension('WEBGL_debug_renderer_info');
  const renderer = gl.getParameter(debug.UNMASKED_RENDERER_WEBGL);
  if (/swiftshader|llvmpipe|software rasterizer/i.test(renderer)) throw Error('Real GPU required');
  const compile = (stage, source) => {
    const shader = gl.createShader(stage); gl.shaderSource(shader, source); gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw Error(gl.getShaderInfoLog(shader));
    return shader;
  };
  const link = source => {
    const program = gl.createProgram();
    const vs = compile(gl.VERTEX_SHADER, source.vertex), fs = compile(gl.FRAGMENT_SHADER, source.fragment);
    gl.attachShader(program, vs); gl.attachShader(program, fs); gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw Error(gl.getProgramInfoLog(program));
    gl.deleteShader(vs); gl.deleteShader(fs); return program;
  };
  const size = 256;
  const decode = link({
    vertex: '#version 300 es\nvoid main(){vec2 p=vec2((gl_VertexID<<1)&2,gl_VertexID&2);gl_Position=vec4(p*2.0-1.0,0,1);}',
    fragment: '#version 300 es\nprecision highp float;precision highp int;uniform highp sampler2D depth_tex;layout(location=0) out uvec4 color;void main(){uint b=floatBitsToUint(texelFetch(depth_tex,ivec2(gl_FragCoord.xy),0).r);color=uvec4(b&255u,(b>>8u)&255u,(b>>16u)&255u,b>>24u);}'
  });
  function render(source, first, count, batch) {
    const program = link(source); gl.useProgram(program);
    const buffers = [];
    const blocks = gl.getProgramParameter(program, gl.ACTIVE_UNIFORM_BLOCKS);
    for (let i = 0; i < blocks; i++) {
      const bytes = gl.getActiveUniformBlockParameter(program, i, gl.UNIFORM_BLOCK_DATA_SIZE);
      const indices = Array.from(gl.getActiveUniformBlockParameter(program, i, gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES));
      const uniforms = indices.map(index => ({index, name: gl.getActiveUniform(program, index).name,
        offset: gl.getActiveUniforms(program, [index], gl.UNIFORM_OFFSET)[0]}));
      const mesh = uniforms.filter(u => /\[\d+\]\.world_from_local$/.test(u.name));
      const offset = mesh.length && batch ? Math.ceil(bytes / 256) * 256 : 0;
      const data = new Float32Array((bytes + offset) / 4);
      if (mesh.length) {
        // Fill all 23 entries, not just slot zero. A wrong constant index must
        // fail this comparison even if the resulting shader links successfully.
        for (const uniform of mesh) {
          const slot = Number(uniform.name.match(/\[(\d+)\]/)[1]);
          const a = slot * .13, sx = .32, sy = .39;
          const tx = -.83 + (slot % 6) * .32 + batch * .015;
          const ty = -.72 + Math.floor(slot / 6) * .46 - batch * .02;
          const tz = .05 + slot * .03 + batch * .015;
          data.set([Math.cos(a)*sx, -Math.sin(a)*sy, 0, tx,
                    Math.sin(a)*sx, Math.cos(a)*sy, 0, ty,
                    0, 0, .4 + (slot % 5)*.02, tz], (offset + uniform.offset) / 4);
        }
      }
      for (const uniform of uniforms.filter(u => /\.clip_from_world$/.test(u.name))) {
        for (let k = 0; k < 4; k++) data[(offset + uniform.offset) / 4 + k * 5] = 1;
      }
      const buffer = gl.createBuffer(); buffers.push(buffer);
      gl.bindBuffer(gl.UNIFORM_BUFFER, buffer); gl.bufferData(gl.UNIFORM_BUFFER, data, gl.STATIC_DRAW);
      gl.bindBufferRange(gl.UNIFORM_BUFFER, i, buffer, offset, bytes); gl.uniformBlockBinding(program, i, i);
    }
    const firstLocation = gl.getUniformLocation(program, 'naga_vs_first_instance');
    if (firstLocation !== null) gl.uniform1ui(firstLocation, first);
    const vao = gl.createVertexArray(), vertices = gl.createBuffer(); gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, vertices);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-.35,-.3,-.35, .35,-.3,.65, 0,.35,1.3]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    const depth = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, depth);
    gl.texStorage2D(gl.TEXTURE_2D, 1, gl.DEPTH_COMPONENT32F, size, size);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    const depthTarget = gl.createFramebuffer(); gl.bindFramebuffer(gl.FRAMEBUFFER, depthTarget);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, depth, 0);
    gl.drawBuffers([gl.NONE]); gl.readBuffer(gl.NONE);
    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) throw Error('Depth target incomplete');
    gl.viewport(0, 0, size, size); gl.enable(gl.DEPTH_TEST); gl.depthFunc(gl.GEQUAL);
    gl.clearDepth(0); gl.clear(gl.DEPTH_BUFFER_BIT); gl.drawArraysInstanced(gl.TRIANGLES, 0, 3, count);
    const color = gl.createTexture(); gl.bindTexture(gl.TEXTURE_2D, color);
    gl.texStorage2D(gl.TEXTURE_2D, 1, gl.RGBA8UI, size, size);
    const target = gl.createFramebuffer(); gl.bindFramebuffer(gl.FRAMEBUFFER, target);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, color, 0);
    gl.drawBuffers([gl.COLOR_ATTACHMENT0]); gl.readBuffer(gl.COLOR_ATTACHMENT0);
    gl.disable(gl.DEPTH_TEST); gl.useProgram(decode); gl.bindVertexArray(gl.createVertexArray());
    gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, depth);
    gl.uniform1i(gl.getUniformLocation(decode, 'depth_tex'), 0); gl.drawArrays(gl.TRIANGLES, 0, 3);
    const rgba = new Uint8Array(size * size * 4);
    gl.readPixels(0, 0, size, size, gl.RGBA_INTEGER, gl.UNSIGNED_BYTE, rgba);
    const error = gl.getError(); if (error) throw Error('WebGL error ' + error);
    gl.deleteProgram(program); buffers.forEach(b => gl.deleteBuffer(b)); gl.deleteBuffer(vertices);
    gl.deleteVertexArray(vao); gl.deleteTexture(depth); gl.deleteTexture(color);
    gl.deleteFramebuffer(depthTarget); gl.deleteFramebuffer(target);
    return new Uint32Array(rgba.buffer);
  }
  const cases = [];
  for (let variant = 0; variant < reference.length; variant++) {
    for (const [first, count, batch] of [[0,23,0], [7,16,1], [22,1,0], [22,1,1]]) {
      const a = render(reference[variant], first, count, batch), b = render(candidate[variant], first, count, batch);
      let changed = 0, maxUlp = 0, covered = 0;
      for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) changed++;
        maxUlp = Math.max(maxUlp, Math.abs(a[i] - b[i])); if (a[i]) covered++;
      }
      cases.push({variant, first, count, batch, changed, maxUlp, covered, distinctDepths: new Set(a).size});
    }
  }
  return {renderer, precision: 'Depth32Float bits via RGBA8UI', cases};
}'''

with sync_playwright() as playwright:
    browser = playwright.chromium.launch(headless=True, args=[
        '--use-angle=gl', '--disable-software-rasterizer', '--enable-webgl',
    ])
    page = browser.new_page()
    result = page.evaluate(COMPARE, {'reference': reference, 'candidate': candidate})
    browser.close()
(args.output / 'report.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
assert all(case['changed'] == 0 and case['covered'] > 20 and case['distinctDepths'] > 20 for case in result['cases']), result
print(json.dumps(result, indent=2))
