const status = document.getElementById('status');
const output = document.getElementById('results');
const report = {version: 2, date: new Date().toISOString(), userAgent: navigator.userAgent, cases: []};
const lines = [];
const write = line => { lines.push(line); output.textContent = lines.join('\n'); };
const pause = () => new Promise(resolve => setTimeout(resolve, 20));
const fingerprint = text => {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 16777619);
  return (h >>> 0).toString(16).padStart(8, '0');
};

// Controlled comparisons only: these transformations never enter the game.
export function variants(vertex, fragment) {
  const cases = [];
  const add = (id, vs, fs) => { if (vs !== vertex || fs !== fragment) cases.push({id, vertex: vs, fragment: fs}); };
  add('no-point-size', vertex.replace(/^\s*gl_PointSize\s*=.*;\s*$/gm, ''), fragment);
  // Remove only unconsumed output declarations and their final assignments.
  // Do not alter internal structs, position/depth arithmetic or uniform layout.
  let pruned = vertex;
  for (const match of vertex.matchAll(/^(?:smooth|flat) out \w+ (_vs2fs_location\d+);$/gm)) {
    if (!new RegExp(`\\b${match[1]}\\b`).test(fragment)) {
      pruned = pruned.replace(match[0], '').replace(new RegExp(`^\\s*${match[1]} = [^;]+;`, 'gm'), '');
    }
  }
  add('matching-varyings', pruned, fragment);
  add('matching-varyings-no-point-size', pruned.replace(/^\s*gl_PointSize\s*=.*;\s*$/gm, ''), fragment);
  // Same depth written through a local before assigning the builtin.
  add('depth-local', vertex, fragment.replace(/gl_FragDepth = ([^;]+);/g, 'float diagnostic_depth = $1; gl_FragDepth = diagnostic_depth;'));
  const minimal = '#version 300 es\nprecision highp float;\nprecision highp int;\n';
  add('fixed-depth', vertex, minimal + 'void main() { gl_FragDepth = 0.5; }');
  add('implicit-depth', vertex, minimal + 'void main() {}');
  // Adding a color attachment output isolates depth-only program linkage. The
  // diagnostic links this program; it does not draw into the game's framebuffer.
  if (fragment.includes('gl_FragDepth') && !/\bout vec4\b/.test(fragment)) {
    const color = fragment.replace('void main()', 'layout(location=0) out vec4 diagnostic_color;\nvoid main()')
      .replace(/(void main\(\)\s*\{)/, '$1\n diagnostic_color = vec4(1.0);');
    add('depth-and-color', vertex, color);
  }
  // Captured depth vertices can contain an unused mesh metadata load. Remove
  // that load only, leaving the buffer type/bindings and transforms intact.
  const metadataPruned = vertex.replace(/\w+ \w*decompress_vertex\([^)]*\) \{[^}]*\}/g, body =>
    body.replace(/^\s*\w+ (_e\d+) = \w*get_metadata\([^;]*;\s*$/gm, (line, name) =>
      (body.match(new RegExp(`\\b${name}\\b`, 'g')) || []).length === 1 ? '' : line));
  add('no-unused-metadata-read', metadataPruned, fragment);
  // Exercise the affine transpose and instance indexing independently.
  add('constant-instance-zero', vertex.replace(/uint\(gl_InstanceID\) \+ naga_vs_first_instance/g, '0u'), fragment);
  return cases;
}

function compile(gl, stage, source) {
  const shader = gl.createShader(stage);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  return shader;
}

async function run(gl, id, vertex, fragment) {
  await pause();
  const start = performance.now();
  let vs, fs, program;
  const result = {id, vertexHash: fingerprint(vertex), fragmentHash: fingerprint(fragment)};
  try {
    vs = compile(gl, gl.VERTEX_SHADER, vertex); fs = compile(gl, gl.FRAGMENT_SHADER, fragment);
    program = gl.createProgram(); gl.attachShader(program, vs); gl.attachShader(program, fs); gl.linkProgram(program);
    result.vertexCompiled = gl.getShaderParameter(vs, gl.COMPILE_STATUS);
    result.fragmentCompiled = gl.getShaderParameter(fs, gl.COMPILE_STATUS);
    result.linked = gl.getProgramParameter(program, gl.LINK_STATUS);
    result.vertexLog = gl.getShaderInfoLog(vs); result.fragmentLog = gl.getShaderInfoLog(fs);
    result.programLog = gl.getProgramInfoLog(program); result.contextLost = gl.isContextLost();
    if (!result.linked) {
      result.vertex = vertex; result.fragment = fragment;
      const debug = gl.getExtension('WEBGL_debug_shaders');
      if (debug) {
        result.translatedVertex = debug.getTranslatedShaderSource(vs);
        result.translatedFragment = debug.getTranslatedShaderSource(fs);
      }
    }
  } catch (error) { result.error = String(error); }
  finally {
    if (program) gl.deleteProgram(program);
    if (vs) gl.deleteShader(vs); if (fs) gl.deleteShader(fs);
  }
  result.ms = Math.round(performance.now() - start);
  report.cases.push(result);
  write(`${result.linked ? '통과' : '실패'} ${id}${result.linked ? '' : ` (정점 ${result.vertexCompiled ? 'OK' : '실패'} / 픽셀 ${result.fragmentCompiled ? 'OK' : '실패'})`}`);
  if (result.contextLost) throw new Error('검사 도중 WebGL 컨텍스트 연결이 끊겼습니다.');
  return result;
}

try {
  const canvas = document.createElement('canvas'); canvas.width = canvas.height = 16;
  const gl = canvas.getContext('webgl2', {antialias: false});
  if (!gl) throw new Error('WebGL2 컨텍스트를 만들지 못했습니다.');
  const debug = gl.getExtension('WEBGL_debug_renderer_info');
  report.gpu = debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER);
  report.vendor = debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR);
  report.glVersion = gl.getParameter(gl.VERSION); report.extensions = gl.getSupportedExtensions();
  report.limits = {};
  for (const name of ['MAX_VERTEX_UNIFORM_BLOCKS', 'MAX_FRAGMENT_UNIFORM_BLOCKS', 'MAX_UNIFORM_BLOCK_SIZE', 'MAX_VARYING_VECTORS', 'MAX_VERTEX_OUTPUT_COMPONENTS', 'MAX_FRAGMENT_INPUT_COMPONENTS']) {
    report.limits[name] = gl.getParameter(gl[name]);
  }
  write(`진단 v2\nGPU: ${report.gpu}\n`);
  if (/swiftshader|llvmpipe|software rasterizer/i.test(report.gpu)) throw new Error('현재 브라우저가 소프트웨어 렌더러를 사용하고 있습니다. GPU 검사를 진행하지 않았습니다.');
  const response = await fetch('programs.json?v=2');
  if (!response.ok) throw new Error(`셰이더 자료를 불러오지 못했습니다: HTTP ${response.status}`);
  const fixtures = await response.json();
  let captured = [];
  try { captured = JSON.parse(localStorage.getItem('findcat.failed-shaders.v1') || '[]'); } catch {}
  report.captured = captured;
  write(`저장된 실패 셰이더: ${captured.length}개 (이전 실행 기록 포함)`);
  status.textContent = '기본 GPU 검사 중…';
  const simpleVertex = '#version 300 es\nprecision highp float;\nout float diagnostic_depth;\nvoid main() { gl_Position = vec4(0.0,0.0,0.5,1.0); diagnostic_depth = 0.5; }';
  await run(gl, 'control-color', simpleVertex, '#version 300 es\nprecision highp float;\nout vec4 color;\nvoid main() { color = vec4(1.0); }');
  await run(gl, 'control-depth', simpleVertex, '#version 300 es\nprecision highp float;\nin float diagnostic_depth;\nvoid main() { gl_FragDepth = diagnostic_depth; }');
  const originals = [...captured.map(program => ({
    id: program.id,
    vertex: program.shaders.find(s => s.stage === gl.VERTEX_SHADER).source,
    fragment: program.shaders.find(s => s.stage === gl.FRAGMENT_SHADER).source,
  })), ...fixtures.programs];
  status.textContent = '원본 셰이더 검사 중…';
  const failed = [], shadows = [];
  for (const program of originals) {
    const result = await run(gl, program.id, program.vertex, program.fragment);
    if (!result.linked) failed.push(program);
    if (program.id.startsWith('current-shadow')) shadows.push(program);
  }
  // Prefer exact failing device shaders. If the standalone original passes,
  // retain the device capture and compare the two actual game shadow variants.
  const comparisons = [...captured.map(p => originals.find(o => o.id === p.id)), ...failed, ...shadows];
  const compared = new Set();
  status.textContent = '항목별 비교 검사 중…';
  for (const program of comparisons) {
    const hash = fingerprint(program.vertex + program.fragment);
    if (compared.has(hash)) continue;
    compared.add(hash);
    for (const variant of variants(program.vertex, program.fragment)) {
      await run(gl, `${program.id}/${variant.id}`, variant.vertex, variant.fragment);
    }
  }
  const current = report.cases.filter(c => c.id.startsWith('current-') && !c.id.includes('/'));
  const currentFailed = current.filter(c => !c.linked).length;
  report.currentGame = {total: current.length, failed: currentFailed};
  const comparisonFailures = report.cases.filter(c => !c.linked).length - currentFailed;
  status.textContent = `현재 게임 셰이더: ${current.length}개 중 ${currentFailed}개 실패 · 이전/비교 검사: ${comparisonFailures}개 실패`;
  write(`\n현재 게임 셰이더: ${current.length}개 중 ${currentFailed}개 실패`);
  write('\n위 결과를 복사하거나 이 화면을 캡처해 주세요.');
  gl.getExtension('WEBGL_lose_context')?.loseContext();
} catch (error) {
  report.error = String(error); status.textContent = '검사를 완료하지 못했습니다.'; write(report.error);
} finally {
  window.findcatGpuReport = report;
  document.getElementById('copy').disabled = false;
  document.getElementById('download').disabled = false;
}

document.getElementById('copy').onclick = async () => {
  try { await navigator.clipboard.writeText(lines.join('\n')); document.getElementById('notice').textContent = '결과를 복사했습니다.'; }
  catch { document.getElementById('notice').textContent = '복사 권한이 없습니다. 화면을 캡처하거나 상세 결과를 저장해 주세요.'; }
};
document.getElementById('download').onclick = () => {
  const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2)], {type: 'application/json'}));
  const link = document.createElement('a'); link.href = url; link.download = 'findcat-gpu-diagnostic.json'; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
