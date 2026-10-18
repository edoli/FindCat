"""Check desktop/tablet startup and failure recovery in isolated headless browsers.

No desktop input or existing browser sessions are used.
uv run --locked --project python python python/validate_web_startup.py
Add --url https://edoli.github.io/FindCat/ to check the deployment.
"""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import argparse
import json
import sys
import tarfile
import time

from PIL import Image, ImageStat
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--url')
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/web-startup')
parser.add_argument('--chromium-only', action='store_true')
parser.add_argument('--profile', choices=['desktop', 'android-tablet', 'android-tablet-desktop', 'ipad-webkit'])
parser.add_argument('--cached-archives', type=Path, help='Check old HTML against current deployment assets')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)


class Handler(SimpleHTTPRequestHandler):
    # Windows' registry can incorrectly associate ES modules with text/plain.
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map,
                      '.mjs': 'text/javascript', '.wasm': 'application/wasm'}
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
    with sync_playwright() as p:
        profiles = [
            ('desktop', p.chromium, {'viewport': {'width': 1440, 'height': 900}}),
            # S8 Ultra's 2960 x 1848 screen at representative DPR 2. This
            # exercises screen/input behavior on the host GPU, not Adreno emulation.
            ('android-tablet', p.chromium, {
                'viewport': {'width': 1480, 'height': 924},
                'screen': {'width': 1480, 'height': 924},
                'device_scale_factor': 2, 'is_mobile': True, 'has_touch': True,
                'user_agent': 'Mozilla/5.0 (Linux; Android 13; SM-X900) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36',
            }),
            ('android-tablet-desktop', p.chromium, {
                'viewport': {'width': 1480, 'height': 924},
                'screen': {'width': 1480, 'height': 924},
                'device_scale_factor': 2, 'is_mobile': False, 'has_touch': True,
                'user_agent': 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36',
            }),
        ]
        if not args.chromium_only:
            profiles.append(('ipad-webkit', p.webkit, dict(p.devices['iPad Pro 11'])))
        if args.profile:
            profiles = [profile for profile in profiles if profile[0] == args.profile]
        for name, engine, options in profiles:
            launch = {}
            if engine == p.chromium:
                launch['args'] = ['--enable-webgl', '--ignore-gpu-blocklist'] + (
                    ['--use-angle=d3d11'] if sys.platform == 'win32' else []) + ['--disable-software-rasterizer']
            browser = engine.launch(headless=True, **launch)
            page = browser.new_page(**options)
            page.set_default_timeout(90000)
            logs, errors = [], []
            def log(message):
                logs.append(message)
                (args.output / f'{name}.log').write_text('\n'.join(logs + errors), encoding='utf-8')
            page.on('console', lambda m: log(f'{m.type}: {m.text}'))
            page.on('pageerror', lambda e: (errors.append(str(e)), log(f'PAGE ERROR: {e}')))
            # Opt-in read-only telemetry also works on the unmodified public URL.
            page.add_init_script("document.documentElement?.setAttribute('data-testbed', ''); "
                                 "addEventListener('DOMContentLoaded', () => document.documentElement.setAttribute('data-testbed', ''));")
            started = time.monotonic()
            try:
                print(f'Checking {name}: {url}', flush=True)
                page.goto(url, wait_until='domcontentloaded', timeout=120000)
                page.wait_for_function("JSON.parse(document.documentElement.getAttribute('data-findcat') || '{}').ready", timeout=180000)
                page.wait_for_timeout(2500)
                state = lambda: page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")
                render = state()['render']
                renderer = page.evaluate("""() => {
                    const gl = document.getElementById('game').getContext('webgl2');
                    const ext = gl.getExtension('WEBGL_debug_renderer_info');
                    return gl.getParameter(ext ? ext.UNMASKED_RENDERER_WEBGL : gl.RENDERER);
                }""")
                assert not any(name in renderer.lower() for name in ['swiftshader', 'llvmpipe', 'lavapipe', 'software rasterizer']), renderer
                assert render['msaa'] == (4 if name == 'desktop' else 1), render
                assert render['text_antialiased'], render
                assert render['physical'] == [round(v * render['dpr']) for v in render['logical']], render
                assert render['viewport'] == render['physical'], render
                assert page.locator('#boot').is_hidden(), page.locator('#status').inner_text()
                page.screenshot(path=str(args.output / f'{name}-title.png'), timeout=90000)
                image = Image.open(args.output / f'{name}-title.png').convert('RGB')
                assert max(ImageStat.Stat(image).stddev) > 15, 'Blank title frame'
                start = next(b for b in state()['buttons'] if b['action'] == 'Start')
                # Playwright sends input only to its own headless page, never the OS.
                if options.get('has_touch'):
                    page.touchscreen.tap(start['x'], start['y'])
                else:
                    page.mouse.click(start['x'], start['y'], delay=100)
                page.wait_for_function("JSON.parse(document.documentElement.getAttribute('data-findcat')).case.phase.Story === 0", timeout=30000)
                if options.get('has_touch'):
                    # Chromium forbids a simulated window resize while fullscreen.
                    # Leave the first-touch fullscreen before emulating rotation.
                    page.evaluate("async () => { if (document.fullscreenElement) await document.exitFullscreen(); else if (document.webkitFullscreenElement) document.webkitExitFullscreen(); }")
                    rotation = {'width': 924, 'height': 1480} if name.startswith('android-tablet') else {'width': 1194, 'height': 834}
                    page.set_viewport_size(rotation)
                    page.wait_for_function("width => Math.abs(JSON.parse(document.documentElement.getAttribute('data-findcat')).render.logical[0] - width) < 1", arg=rotation['width'], timeout=30000)
                awaitable_errors = [e for e in errors if 'Using exceptions for control flow' not in e]
                assert not awaitable_errors, awaitable_errors
                report = {'profile': name, 'passed': True, 'render': render, 'gpu_renderer': renderer,
                          'start_input': 'touch' if options.get('has_touch') else 'mouse',
                          'elapsed_seconds': round(time.monotonic() - started, 2)}
                reports.append(report)
                print(json.dumps(report), flush=True)
                if name == 'android-tablet':
                    page.evaluate("document.getElementById('game').getContext('webgl2').getExtension('WEBGL_lose_context').loseContext()")
                    page.wait_for_function("document.getElementById('boot').dataset.failed === 'true'", timeout=15000)
                    assert page.locator('#boot-retry').is_visible()
                    reports.append({'profile': 'gpu-context-loss', 'passed': True})
                    print('PASS: GPU context loss has recovery instructions', flush=True)
            except Exception:
                print('FAILED state:', page.evaluate("({size:[innerWidth,innerHeight,devicePixelRatio], status:document.getElementById('status').textContent, game:JSON.parse(document.documentElement.getAttribute('data-findcat') || '{}')})"), flush=True)
                raise
            finally:
                (args.output / f'{name}.log').write_text('\n'.join(logs + errors), encoding='utf-8')
                browser.close()

        # A failed WASM fetch used to leave the loading screen without a useful reason.
        browser = p.chromium.launch(headless=True)
        try:
            page = browser.new_page()
            page.route('**/*.wasm', lambda route: route.abort('internetdisconnected'))
            page.goto(url, wait_until='domcontentloaded', timeout=120000)
            page.wait_for_function("document.getElementById('boot').dataset.failed === 'true'", timeout=30000)
            assert page.locator('#boot-retry').is_visible()
            assert page.locator('#boot-error-text').text_content().strip()
            reports.append({'profile': 'wasm-network-failure', 'passed': True})
            print('PASS: download failure has error details and retry', flush=True)
        finally:
            browser.close()
        if args.cached_archives:
            for archive in sorted(args.cached_archives.glob('*/artifact.tar')):
                with tarfile.open(archive) as bundle:
                    index = next(m for m in bundle.getmembers() if m.name.lstrip('./') == 'index.html')
                    with bundle.extractfile(index) as stream:
                        old_html = stream.read()
                # Reproduce the browser's cached document while all other requests use
                # the current server. No old JS/WASM is provided by this route.
                browser = p.chromium.launch(headless=True, args=['--disable-software-rasterizer'] + (['--use-angle=d3d11'] if sys.platform == 'win32' else []))
                try:
                    page = browser.new_page()
                    page.route(url, lambda route: route.fulfill(body=old_html, content_type='text/html'))
                    page.goto(url, wait_until='domcontentloaded', timeout=120000)
                    page.wait_for_function("document.getElementById('boot').style.display === 'none'", timeout=180000)
                    reports.append({'profile': f'cached-{archive.parent.name}', 'passed': True})
                    print(f'PASS: cached HTML from {archive.parent.name} still starts', flush=True)
                finally:
                    browser.close()
    (args.output / 'report.json').write_text(json.dumps({'url': url, 'checks': reports}, indent=2), encoding='utf-8')
finally:
    if server:
        server.shutdown()
