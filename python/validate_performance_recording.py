"""Exercise opt-in device traces and JSON downloads on real GPU, without OS input.

Checks normal play has no instrumentation, a recorded slow main-thread task is
visible, game phase transitions are present, and recording can be restarted.
"""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
from threading import Thread
from urllib.parse import parse_qsl, urlencode, urlsplit, urlunsplit

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url')
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/performance-recording')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)


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
parts = urlsplit(url)
record_url = urlunsplit(parts._replace(query=urlencode([*parse_qsl(parts.query), ('performance', '1')])))
try:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True, args=[
            '--use-angle=gl', '--disable-software-rasterizer', '--enable-webgl',
        ])
        context = browser.new_context(viewport={'width': 1480, 'height': 924}, device_scale_factor=2)
        context.add_init_script("""window.originalRaf = requestAnimationFrame;
            window.originalDraw = WebGL2RenderingContext.prototype.drawElements;
            localStorage.setItem('unrelated-recorder-sentinel','preserved');""")
        errors = []
        page = context.new_page()
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.goto(url, wait_until='domcontentloaded', timeout=120000)
        page.wait_for_function("document.getElementById('boot').style.display === 'none'", timeout=180000)
        assert page.evaluate("requestAnimationFrame === originalRaf && WebGL2RenderingContext.prototype.drawElements === originalDraw")
        assert page.locator('#performance-panel').count() == 0
        page.goto(record_url, wait_until='domcontentloaded', timeout=120000)
        page.wait_for_function("document.getElementById('boot').style.display === 'none'", timeout=180000)
        assert page.evaluate("requestAnimationFrame !== originalRaf")
        page.locator('#performance-start').click()
        page.wait_for_timeout(1000)
        # A controlled timer task tests the trace's ability to distinguish a long
        # main-thread stall from a slow GPU frame. This changes no rendering state.
        page.evaluate("setTimeout(() => {const end=performance.now()+85; while(performance.now()<end) {}}, 0)")
        page.wait_for_timeout(500)
        page.evaluate("document.documentElement.setAttribute('data-testbed','')")
        page.wait_for_function("document.documentElement.hasAttribute('data-findcat')")
        state = page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")
        start = next(b for b in state['buttons'] if b['action'] == 'Start')
        page.evaluate("document.documentElement.removeAttribute('data-testbed')")
        page.mouse.click(start['x'], start['y'])
        page.wait_for_timeout(1500)
        page.locator('#performance-start').click()
        with page.expect_download() as pending:
            page.locator('#performance-download').click()
        pending.value.save_as(args.output / 'findcat-performance.json')
        report = json.loads((args.output / 'findcat-performance.json').read_text(encoding='utf-8'))
        assert report['schema'] == 'findcat.performance.v1'
        assert report['device']['physical'] == [2960, 1848], report['device']
        assert not any(s in report['device']['renderer'].lower() for s in ('swiftshader', 'llvmpipe', 'lavapipe'))
        assert report['summary']['frames'] > 30, report['summary']
        assert all(f['width'] == 2960 and f['height'] == 1848 and f['draws'] > 0 for f in report['frames'])
        assert any(t['duration_ms'] >= 75 for t in report['long_tasks']), report['long_tasks']
        assert report['summary']['intervals_over_50ms'] > 0, report['summary']
        phases = [e['detail']['phase'] for e in report['events'] if e['type'] == 'game']
        assert 'Title' in phases and {'Story': 0} in phases, phases
        assert page.evaluate("!document.documentElement.hasAttribute('data-performance')")
        assert page.evaluate("localStorage.getItem('unrelated-recorder-sentinel')") == 'preserved'
        first_duration = report['duration_ms']
        page.locator('#performance-start').click()
        assert page.locator('#performance-download').is_hidden()
        page.wait_for_timeout(600)
        page.locator('#performance-start').click()
        with page.expect_download() as pending:
            page.locator('#performance-download').click()
        pending.value.save_as(args.output / 'restarted.json')
        restarted = json.loads((args.output / 'restarted.json').read_text(encoding='utf-8'))
        assert restarted['duration_ms'] < first_duration
        assert restarted['frames'][0]['at_ms'] < 200
        assert any(e['type'] == 'game' for e in restarted['events']), restarted['events']
        assert not [e for e in errors if 'Using exceptions for control flow' not in e], errors
        print(json.dumps({'normal_play_uninstrumented': True, 'physical': report['device']['physical'],
            'frames': report['summary']['frames'], 'controlled_stall_recorded': True,
            'phase_transitions': phases, 'restart': True}), flush=True)
        browser.close()
finally:
    if server:
        server.shutdown()
