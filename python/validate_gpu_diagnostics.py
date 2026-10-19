"""Verify the shader diagnostic and failure recorder with real GPU WebGL2.

uv run --locked --project python python python/validate_gpu_diagnostics.py
Add --url https://edoli.github.io/FindCat/gpu-diagnostics/ for the deployment.
No OS input, existing browser sessions or software rasterizer are used.
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
parser.add_argument('--angle', default='gl', choices=['gl', 'd3d11'])
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/gpu-diagnostic-review')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)


class Handler(SimpleHTTPRequestHandler):
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, '.mjs': 'text/javascript'}

    def log_message(self, *_):
        pass


server = None
if not args.url:
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(ROOT / 'dist/web')))
    Thread(target=server.serve_forever, daemon=True).start()
url = args.url or f'http://127.0.0.1:{server.server_port}/gpu-diagnostics/'

try:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True, args=[
            '--use-angle=' + args.angle, '--disable-software-rasterizer', '--enable-webgl',
        ])
        page = browser.new_page(viewport={'width': 1000, 'height': 900})
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(url)
        page.wait_for_function('window.findcatGpuReport !== undefined', timeout=120000)
        report = page.evaluate('window.findcatGpuReport')
        (args.output / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
        page.screenshot(path=str(args.output / 'diagnostic.png'), full_page=True)
        assert not report.get('error') and not errors, (report.get('error'), errors)
        assert len(report['cases']) >= 30
        assert all(case.get('linked') for case in report['cases']), 'Shader comparison failed; see report.json'
        assert not report['captured'], 'The isolated page should have no previous failed shaders'
        print(f"GPU: {report['gpu']}\nLinked comparisons: {len(report['cases'])}")

        # A link failure with two successfully compiled stages exercises the
        # actual error class from the tablet, without depending on a driver bug.
        page.add_script_tag(url=url + 'capture.js')
        captured = page.evaluate('''() => {
            localStorage.setItem('findcat.diagnostic-test-save', 'unchanged');
            const gl = document.createElement('canvas').getContext('webgl2');
            const sources = [
                `#version 300 es\nprecision highp float;\nout vec3 diagnostic_v;\nvoid main(){gl_Position=vec4(1.0);diagnostic_v=vec3(1.0);}`,
                `#version 300 es\nprecision highp float;\nin vec4 diagnostic_v;\nout vec4 color;\nvoid main(){color=diagnostic_v;}`
            ];
            const program = gl.createProgram();
            sources.forEach((source, i) => {
                const shader = gl.createShader(i === 0 ? gl.VERTEX_SHADER : gl.FRAGMENT_SHADER);
                gl.shaderSource(shader, source); gl.compileShader(shader); gl.attachShader(program, shader);
            });
            gl.linkProgram(program);
            const before = gl.getProgramParameter(program, gl.LINK_STATUS);
            const data = JSON.parse(localStorage.getItem('findcat.failed-shaders.v1'));
            const after = gl.getProgramParameter(program, gl.LINK_STATUS);
            gl.deleteProgram(program);
            return {before, after, data, saved: localStorage.getItem('findcat.diagnostic-test-save')};
        }''')
        assert captured['before'] is False and captured['after'] is False
        assert captured['saved'] == 'unchanged'
        assert len(captured['data']) == 1 and all(s['compiled'] for s in captured['data'][0]['shaders'])
        page.reload()
        page.wait_for_function('window.findcatGpuReport !== undefined', timeout=120000)
        replay = page.evaluate('window.findcatGpuReport')
        assert len(replay['captured']) == 1
        case = next(c for c in replay['cases'] if c['id'] == 'device-failure-1')
        assert case['vertexCompiled'] and case['fragmentCompiled'] and not case['linked']
        assert not replay.get('error') and not errors
        with page.expect_download() as pending:
            page.locator('#download').click()
        pending.value.save_as(str(args.output / 'captured-failure.json'))
        downloaded = json.loads((args.output / 'captured-failure.json').read_text(encoding='utf-8'))
        assert downloaded['captured'] == replay['captured']
        print('PASS: passive capture, exact failed source replay, report download, unrelated storage preserved')
        browser.close()
finally:
    if server:
        server.shutdown()
