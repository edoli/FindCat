"""Play the served WASM build through real canvas input, without touching the desktop.

uv run --project python python python/validate_web.py --url https://edoli.github.io/FindCat/
"""
from pathlib import Path
import argparse
import json
import time
import sys
from urllib.parse import urlsplit, urlunsplit
from playwright.sync_api import sync_playwright
from PIL import Image, ImageStat

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--url', default='http://127.0.0.1:8080/')
parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/web')
parser.add_argument('--smoke', action='store_true')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
url = urlsplit(args.url)
test_url = urlunsplit((url.scheme, url.netloc, url.path, 'testbed', ''))
errors, failed, messages, shots = [], [], [], []

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True, args=[
        '--enable-webgl', '--ignore-gpu-blocklist',
        '--disable-background-timer-throttling', '--disable-renderer-backgrounding',
    ] + (['--use-angle=d3d11'] if sys.platform == 'win32' else ['--enable-unsafe-swiftshader']))
    page = browser.new_page(viewport={'width':1440, 'height':900}, device_scale_factor=1)
    page.on('pageerror', lambda e: errors.append(str(e)))
    def log_message(m):
        messages.append(f'{m.type}: {m.text}')
        (args.output / 'browser.log').write_text('\n'.join(messages), encoding='utf-8')
    page.on('console', log_message)
    page.on('response', lambda r: failed.append(f'{r.status} {r.url}') if r.status >= 400 else None)
    page.on('requestfailed', lambda r: failed.append(f'{r.failure} {r.url}'))

    def state():
        return page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")

    def wait(condition, timeout=45000):
        page.wait_for_function("""condition => {
            const raw = document.documentElement.getAttribute('data-findcat');
            if (!raw) return false;
            return (new Function('s', 'return (' + condition + ')'))(JSON.parse(raw));
        }""", arg=condition, timeout=timeout)

    def click(action):
        if action.startswith('Inspect(') and not any(b['action'] == action for b in state()['buttons']):
            page.keyboard.press('Tab')
            page.wait_for_timeout(1500)
        wait(f"s.buttons.some(b => b.action === {json.dumps(action)})")
        button = next(b for b in state()['buttons'] if b['action'] == action)
        rect = page.locator('#game').bounding_box()
        x = rect['x'] + button['x'] * rect['width'] / 1440
        y = rect['y'] + button['y'] * rect['height'] / 900
        page.mouse.move(x, y)
        page.mouse.down()
        page.wait_for_timeout(100)
        page.mouse.up()
        page.wait_for_timeout(250)
        print(f'Clicked {action}', flush=True)

    def shot(name):
        page.wait_for_timeout(1800)
        for panel in state().get('panels', []):
            assert min(panel['min']) >= -2 and panel['max'][0] <= 1442 and panel['max'][1] <= 902, panel
            assert not panel['overflow'], panel
        path = args.output / f'{name}.png'
        page.screenshot(path=str(path))
        im = Image.open(path).convert('RGB')
        assert max(ImageStat.Stat(im).stddev) > 15, f'Blank render: {name}'
        shots.append(str(path))

    try:
        response = page.goto(test_url, wait_until='domcontentloaded', timeout=60000)
        assert response.status == 200
        wait('s.ready', 240000)
        assert state()['case']['phase'] == 'Title'
        assert max(state()['cat_scale']) < .29
        shot('01-title')
        click('Start')
        wait("s.case.phase.Story === 0")
        click('Play')
        assert page.evaluate("contexts.some(c => c.state === 'running')"), 'Audio did not unlock on input'
        shot('02-play')
        for n in range(5):
            click('Next')
            if n == 0:
                shot('03-escape')
        wait("s.case.phase === 'Search'")
        shot('04-search')
        page.keyboard.down('KeyD')
        page.wait_for_timeout(250)
        page.keyboard.up('KeyD')
        click('Inspect(0)')
        wait("s.modal === 'Clue(0)'")
        shot('05-clue')
        click('Close')
        assert page.evaluate("localStorage.getItem('findcat-save-v1')")

        if not args.smoke:
            def inspect(i):
                click(f'Inspect({i})')
                wait(f"s.modal === 'Clue({i})'", 90000)
                assert state()['case']['found'][i]
                if i == 5:
                    shot('08-found-cat')
                click('Close')

            def deduce(chapter, pair, answer):
                click('Deduce')
                wait("s.modal === 'Deduction'")
                for clue in pair:
                    click(f'Select({clue})')
                shot(f'06-deduction-{chapter}')
                click(f'Theory({answer})')
                wait(f's.case.chapter === {chapter+1}')

            for i in [6,1,2]: inspect(i)
            deduce(0,[0,1],1)
            for i in [3,4,7]: inspect(i)
            deduce(1,[4,7],2)
            inspect(5)
            deduce(2,[5,6],0)
            click('Journal')
            wait("s.modal === 'Journal'")
            shot('09-journal')
            click('Close')
            click('Inspect(5)')
            wait("s.modal === 'Rescue'", 90000)
            shot('10-rescue')
            click('Rescue(0)')
            assert state()['case']['mistakes'] == 1
            for trust, action in enumerate([1,0,2], 1):
                click(f'Rescue({action})')
                wait(f's.case.trust === {trust}')
            wait("s.case.phase === 'Reunion'")
            page.wait_for_timeout(3500)
            shot('11-reunion')
            assert sum(state()['case']['found']) == 8

        saved = state()['case']
        assert max(state()['cat_scale']) < .29
        page.set_viewport_size({'width':1100, 'height':760})
        shot('12-small-window')
        page.reload(wait_until='domcontentloaded')
        wait('s.ready', 240000)
        click('Continue')
        wait(f"s.case.phase === {json.dumps(saved['phase'])}")
        assert state()['case']['found'] == saved['found']
        assert state()['case']['trust'] == saved['trust']
        shot('13-restored')
        real_errors = [e for e in errors if 'Using exceptions for control flow' not in e]
        assert not real_errors, real_errors
        assert not failed, failed
        report = {'passed':True, 'url':args.url, 'case':state()['case'],
                  'screenshots':shots, 'page_errors':real_errors, 'failed_requests':failed}
        (args.output / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
        print('PASS: live WASM rendering, canvas input, clues, deductions, rescue, browser save/reload.', flush=True)
    except Exception:
        page.screenshot(path=str(args.output / 'failure.png'))
        print('STATE:', state())
        print('ERRORS:', errors, failed)
        raise
    finally:
        (args.output / 'browser.log').write_text('\n'.join(messages + errors + failed), encoding='utf-8')
        browser.close()
