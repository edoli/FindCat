"""Check camera/UI integration, asynchronous character binding and fades on GPU.

Uses the normal saved-game Continue path in an isolated headless page. No OS
input or production save is touched. --url also checks a published release.
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
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/render-optimizations')
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
checks = []
try:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True, args=['--use-angle=gl', '--disable-software-rasterizer', '--enable-webgl'])
        page = browser.new_page(viewport={'width': 1440, 'height': 900})
        page.add_init_script("addEventListener('DOMContentLoaded',()=>document.documentElement.setAttribute('data-testbed',''))")
        errors = []
        page.on('pageerror', lambda e: errors.append(str(e)))
        def wait(expression, timeout=30000):
            page.wait_for_function("s => {const raw=document.documentElement.getAttribute('data-findcat');return raw && (new Function('s','return ('+s+')'))(JSON.parse(raw));}", arg=expression, timeout=timeout)
        def state():
            return page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")
        def click(action):
            wait(f"s.buttons.some(b=>b.action==={json.dumps(action)})")
            button = next(b for b in state()['buttons'] if b['action'] == action)
            page.mouse.click(button['x'], button['y'])
        def check(label):
            render = state()['render']
            assert render['cameras'] == 1 and render['msaa'] == 4 and render['text_antialiased'], render
            assert render['bound_characters'] == 2 and render['non_shadow_casters'] >= 48, render
            assert 0 < render['visible_ground_depth'] < render['shadow_distance'] == 64, render
            assert len(render['fading_materials']) == material_count, 'Character materials were lost or rebound'
            checks.append({'scene': label, 'ground_depth': render['visible_ground_depth'], 'bound_characters': render['bound_characters'], 'fading_materials': len(render['fading_materials'])})
            page.screenshot(path=str(args.output / f'{label}.png'))
        page.goto(url, wait_until='domcontentloaded', timeout=120000)
        wait('s.ready && s.render.bound_characters===2', 180000)
        page.wait_for_timeout(1500)
        base = state()['case']
        material_count = len(state()['render']['fading_materials'])
        assert material_count > 0
        check('title')
        def load(phase):
            saved = {**base, 'phase': phase, 'muted': True}
            page.evaluate("saved=>localStorage.setItem('findcat-save-v1',JSON.stringify(saved))", saved)
            page.reload(wait_until='domcontentloaded')
            wait('s.ready && s.render.bound_characters===2', 180000)
            click('Continue')
            wait(f's.case.phase.Story==={phase["Story"]}' if isinstance(phase, dict) else f's.case.phase==={json.dumps(phase)}')
        load({'Story': 1})
        wait("s.render.fading_materials.some(m=>!m.fairy && m.alpha>.01 && m.alpha<.99)")
        check('cat-mid-fade')
        wait("s.render.fading_materials.filter(m=>!m.fairy).every(m=>m.alpha<.001)")
        check('cat-hidden')
        load({'Story': 3})
        wait("s.render.fading_materials.some(m=>m.fairy && m.alpha>.01 && m.alpha<.99)")
        check('fairy-mid-fade')
        wait("s.render.fading_materials.filter(m=>m.fairy).every(m=>m.alpha>.99)")
        assert all(m['alpha'] < .001 for m in state()['render']['fading_materials'] if not m['fairy'])
        check('fairy-visible')
        load('Search')
        page.wait_for_timeout(2000)
        check('search')
        page.mouse.wheel(0, 1600)
        page.wait_for_timeout(2000)
        assert state()['camera_height'] > 20
        check('search-wide')
        page.keyboard.press('Tab')
        wait('s.overview')
        page.wait_for_timeout(2000)
        check('search-overview')
        assert not [e for e in errors if 'Using exceptions for control flow' not in e], errors
        browser.close()
finally:
    if server:
        server.shutdown()
(args.output / 'report.json').write_text(json.dumps({'passed': True, 'checks': checks}, indent=2), encoding='utf-8')
print(f'PASS: {len(checks)} GPU-rendered scenes; UI camera, character fades, saved-game rebinding, shadow coverage')
