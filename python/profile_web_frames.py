"""Measure WebGL frames on the host GPU without OS input or software rendering.

Reports are comparative host measurements, not Android/Adreno emulation. GPU
times are available only when EXT_disjoint_timer_query_webgl2 is exposed.
The GPU query spans a whole callback and can include command-submission gaps;
it is not a measurement of GPU busy time alone. CPU callback time includes GL
driver work/waits. Run comparisons sequentially without competing GPU tests.
"""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
from threading import Thread

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url')
parser.add_argument('--seconds', type=float, default=12)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)

INSTRUMENT = r'''(() => {
  const state = {gl:null, ext:null, enabled:false, frames:[], pending:[], counts:{draw:0, blit:0, framebuffer:0, elements:0}};
  const getContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function(kind, ...options) {
    const gl = getContext.call(this, kind, ...options);
    if (kind === 'webgl2' && gl && this.id === 'game') {
      state.gl = gl; state.ext = gl.getExtension('EXT_disjoint_timer_query_webgl2');
    }
    return gl;
  };
  const proto = WebGL2RenderingContext.prototype;
  for (const [name, counter] of Object.entries({drawArrays:'draw', drawElements:'draw',
      drawArraysInstanced:'draw', drawElementsInstanced:'draw', blitFramebuffer:'blit', bindFramebuffer:'framebuffer'})) {
    const original = proto[name];
    proto[name] = function(...args) {
      if (this === state.gl) {
        state.counts[counter]++;
        if (counter === 'draw') {
          const arrays = name.startsWith('drawArrays');
          state.counts.elements += args[arrays ? 2 : 1] * (name.endsWith('Instanced') ? args[arrays ? 3 : 4] : 1);
        }
      }
      return original.apply(this,args);
    };
  }
  const raf = requestAnimationFrame.bind(window);
  window.requestAnimationFrame = callback => raf(timestamp => {
    if (!state.enabled || !state.gl || state.gl.isContextLost()) return callback(timestamp);
    const gl = state.gl, ext = state.ext, before = {...state.counts}, start = performance.now();
    let query = null;
    if (ext && !gl.getQuery(ext.TIME_ELAPSED_EXT, gl.CURRENT_QUERY)) {
      query = gl.createQuery(); gl.beginQuery(ext.TIME_ELAPSED_EXT,query);
    }
    try { callback(timestamp); }
    finally {
      if (query) gl.endQuery(ext.TIME_ELAPSED_EXT);
      const frame = {timestamp, cpu_ms:performance.now()-start, gpu_ms:null,
        draw:state.counts.draw-before.draw, blit:state.counts.blit-before.blit,
        elements:state.counts.elements-before.elements,
        framebuffer:state.counts.framebuffer-before.framebuffer};
      if (frame.draw) {
        state.frames.push(frame);
        if (query) state.pending.push({query,frame});
      } else if (query) gl.deleteQuery(query);
      if (ext) {
        const disjoint = gl.getParameter(ext.GPU_DISJOINT_EXT);
        state.pending = state.pending.filter(pending => {
          if (disjoint) { gl.deleteQuery(pending.query); return false; }
          if (!gl.getQueryParameter(pending.query,gl.QUERY_RESULT_AVAILABLE)) return true;
          pending.frame.gpu_ms = gl.getQueryParameter(pending.query,gl.QUERY_RESULT)/1e6;
          gl.deleteQuery(pending.query); return false;
        });
      }
    }
  });
  window.startFindcatProfile = () => { state.frames=[]; state.enabled=true; };
  window.stopFindcatProfile = () => {
    state.enabled=false;
    const summary = values => {
      const v=values.filter(v=>v!==null).sort((a,b)=>a-b);
      return v.length ? {samples:v.length, median:v[Math.floor(v.length*.5)], p95:v[Math.min(v.length-1,Math.floor(v.length*.95))], mean:v.reduce((a,b)=>a+b,0)/v.length} : null;
    };
    const gl=state.gl, debug=gl.getExtension('WEBGL_debug_renderer_info');
    const intervals=state.frames.slice(1).map((f,i)=>f.timestamp-state.frames[i].timestamp);
    return {renderer:gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER),
      physical:[gl.drawingBufferWidth,gl.drawingBufferHeight], gpu_timer:!!state.ext,
      frames:state.frames.length, interval_ms:summary(intervals),
      cpu_callback_ms:summary(state.frames.map(f=>f.cpu_ms)),gpu_ms:summary(state.frames.map(f=>f.gpu_ms)),
      draw_calls:summary(state.frames.map(f=>f.draw)),blits:summary(state.frames.map(f=>f.blit)),
      submitted_elements:summary(state.frames.map(f=>f.elements)),
      framebuffer_binds:summary(state.frames.map(f=>f.framebuffer))};
  };
})();'''


class Handler(SimpleHTTPRequestHandler):
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, '.mjs': 'text/javascript', '.wasm': 'application/wasm'}

    def do_GET(self):
        if self.path.startswith('/FindCat/'):
            self.path = self.path[len('/FindCat'):]
        super().do_GET()

    def log_message(self, *_):
        pass


server = None
if not args.url:
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(ROOT / 'dist/web')))
    Thread(target=server.serve_forever, daemon=True).start()
url = args.url or f'http://127.0.0.1:{server.server_port}/FindCat/'
reports = []
try:
    with sync_playwright() as playwright:
        for scene in ('title', 'play'):
            browser = playwright.chromium.launch(headless=True, args=[
                '--use-angle=gl', '--disable-software-rasterizer', '--enable-webgl',
            ])
            page = browser.new_page(viewport={'width': 1480, 'height': 924}, device_scale_factor=2)
            errors = []
            page.on('pageerror', lambda e: errors.append(str(e)))
            page.add_init_script(INSTRUMENT)
            page.goto(url, wait_until='domcontentloaded', timeout=120000)
            page.wait_for_function("getComputedStyle(document.getElementById('boot')).display === 'none'", timeout=180000)
            if scene == 'play':
                page.evaluate("document.documentElement.setAttribute('data-testbed','')")
                page.wait_for_function("document.documentElement.hasAttribute('data-findcat')")
                start = next(b for b in page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat')).buttons") if b['action'] == 'Start')
                page.mouse.click(start['x'], start['y'])
                page.wait_for_function("JSON.parse(document.documentElement.getAttribute('data-findcat')).case.phase.Story === 0")
                page.evaluate("document.documentElement.removeAttribute('data-testbed')")
            page.wait_for_timeout(4000)
            page.evaluate('window.startFindcatProfile()')
            page.wait_for_timeout(args.seconds * 1000)
            result = page.evaluate('window.stopFindcatProfile()')
            assert result['frames'] > 30 and not any(s in result['renderer'].lower() for s in ('swiftshader','llvmpipe','lavapipe'))
            assert not [e for e in errors if 'Using exceptions for control flow' not in e], errors
            result['scene'] = scene
            reports.append(result)
            print(json.dumps(result), flush=True)
            page.screenshot(path=str(args.output / f'{scene}.png'))
            browser.close()
finally:
    if server:
        server.shutdown()
(args.output / 'report.json').write_text(json.dumps(reports, indent=2), encoding='utf-8')
