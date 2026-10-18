"""Real GPU resolution, text smoothing, resize/DPI and canvas input checks."""
from pathlib import Path
import argparse, base64, json, sys
from urllib.parse import urlsplit, urlunsplit
from PIL import Image
from playwright.sync_api import sync_playwright

parser=argparse.ArgumentParser()
parser.add_argument('--url',default='http://127.0.0.1:8080/')
parser.add_argument('--output',type=Path,default=Path('artifacts/resolution'))
args=parser.parse_args()
args.output.mkdir(parents=True,exist_ok=True)
url=urlsplit(args.url)
url=urlunsplit((url.scheme,url.netloc,url.path,'testbed',''))
records,errors=[],[]
with sync_playwright() as p:
    browser=p.chromium.launch(headless=True,args=['--enable-webgl','--ignore-gpu-blocklist']+
        ['--disable-software-rasterizer'] + (['--use-angle=d3d11'] if sys.platform=='win32' else []))
    def state():return page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")
    def wait(expression):
        page.wait_for_function("""expression=>{const s=JSON.parse(document.documentElement.getAttribute('data-findcat')||'{}');
            return(new Function('s','return ('+expression+')'))(s)}""",arg=expression,timeout=180000)
    def click(action):
        wait(f"s.buttons?.some(b=>b.action==={json.dumps(action)})")
        button=next(b for b in state()['buttons'] if b['action']==action)
        page.mouse.move(button['x'],button['y']);page.mouse.down()
        page.wait_for_timeout(100);page.mouse.up();page.wait_for_timeout(300)
    def check(tag):
        page.wait_for_function("""()=>{const s=JSON.parse(document.documentElement.getAttribute('data-findcat')||'{}');
            const c=document.querySelector('canvas');return s.ready&&c.width===Math.round(innerWidth*devicePixelRatio)
            &&c.height===Math.round(innerHeight*devicePixelRatio)&&s.render.physical[0]===c.width
            &&s.render.physical[1]===c.height&&s.render.viewport[0]===c.width&&s.render.viewport[1]===c.height} """,timeout=45000)
        page.wait_for_timeout(1800)
        metrics=page.evaluate("""()=>{const c=document.querySelector('canvas'),r=c.getBoundingClientRect(),gl=c.getContext('webgl2');return{
            window:[innerWidth,innerHeight],dpr:devicePixelRatio,canvas:[c.width,c.height],display:[r.width,r.height],
            drawing_buffer:[gl.drawingBufferWidth,gl.drawingBufferHeight],
            transform:getComputedStyle(document.querySelector('#surface')).transform}}""")
        s=state()
        assert metrics['transform']=='none' and metrics['display']==metrics['window'],metrics
        assert metrics['drawing_buffer']==metrics['canvas'],metrics
        assert s['render']['text_antialiased'] and s['render']['msaa']==4,s['render']
        for panel in s['panels']:
            assert min(panel['min'])>=-2 and not panel['overflow'],panel
            assert all(v<=d+2 for v,d in zip(panel['max'],metrics['canvas'])),panel
        path=args.output/f'{tag}.png'
        if tag=='runtime-dpr1.5':
            # Playwright's screenshot helper restores the context's original DPI;
            # capture directly to preserve the live emulated monitor change.
            path.write_bytes(base64.b64decode(session.send('Page.captureScreenshot',{'format':'png'})['data']))
        else:page.screenshot(path=str(path))
        if s['case']['phase']=='Title':
            title=next(t for t in s['texts'] if t['text']=='오늘도,\n너에게')
            image=Image.open(path).convert('RGB').crop(tuple(round(v) for v in title['min']+title['max']))
            shades={c for n,c in image.getcolors(image.width*image.height) if 35<c[0]<225 and c[1]<235}
            assert len(shades)>20,f'Binary/pixelated glyph edges: {len(shades)} shades'
            metrics['text_edge_shades']=len(shades);image.save(args.output/f'{tag}-text.png')
        metrics.update(render=s['render'],tag=tag);records.append(metrics)
        print(json.dumps(metrics,ensure_ascii=False),flush=True)
    try:
        for w,h,dpr in [(1920,1080,1),(1440,900,2),(1536,960,1.25)]:
            page=browser.new_page(viewport={'width':w,'height':h},device_scale_factor=dpr)
            page.on('pageerror',lambda e:errors.append(str(e)))
            page.goto(url,wait_until='domcontentloaded');wait('s.ready')
            tag=f'{w}x{h}-dpr{dpr}';check(tag)
            click('Help');wait("s.modal==='Help'");check(f'{tag}-help');click('Close')
            if dpr==1:
                for rw,rh in [(2560,1440),(1100,760),(1280,720)]:
                    page.set_viewport_size({'width':rw,'height':rh});check(f'resize-{rw}x{rh}')
                session=page.context.new_cdp_session(page)
                session.send('Emulation.setDeviceMetricsOverride',{'width':1280,'height':720,'deviceScaleFactor':1.5,'mobile':False})
                check('runtime-dpr1.5')
            click('Start');wait('s.case.phase.Story===0')
            assert any('율' in t['text'] for t in state()['texts']);check(f'{tag}-story')
            for _ in range(5):click('Next')
            wait("s.case.phase==='Search'");click('Inspect(0)');wait("s.modal==='Clue(0)'")
            assert any('율의 털' in t['text'] for t in state()['texts']);check(f'{tag}-clue')
            click('Close');click('Inspect(1)');wait("s.modal==='Clue(1)'")
            assert state()['case']['found'][1]
            page.close()
        assert not errors,errors
        (args.output/'report.json').write_text(json.dumps({'passed':True,'url':args.url,'checks':records,'page_errors':errors},ensure_ascii=False,indent=2),encoding='utf-8')
        print('PASS: native resolution, resize/DPI changes, smooth text, UI layout and marker input.',flush=True)
    finally:browser.close()
