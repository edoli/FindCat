"""Compare actual game glyphs with browser-native font rendering; check redraw stability."""
from pathlib import Path
import argparse, json, sys
from urllib.parse import urlsplit,urlunsplit
from PIL import Image,ImageChops,ImageDraw,ImageStat
from playwright.sync_api import sync_playwright

parser=argparse.ArgumentParser()
parser.add_argument('--url',default='http://127.0.0.1:8080/')
parser.add_argument('--output',type=Path,default=Path('artifacts/typography'))
args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
url=urlsplit(args.url);url=urlunsplit((url.scheme,url.netloc,url.path,'testbed',''))
results=[]
with sync_playwright() as p:
    browser=p.chromium.launch(headless=True,args=['--enable-webgl','--ignore-gpu-blocklist']+
        ['--disable-software-rasterizer'] + (['--use-angle=d3d11'] if sys.platform=='win32' else []))
    try:
        for dpr in [1,2]:
            page=browser.new_page(viewport={'width':1440,'height':900},device_scale_factor=dpr)
            errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
            page.goto(url,wait_until='domcontentloaded')
            page.wait_for_function("JSON.parse(document.documentElement.getAttribute('data-findcat')||'{}').ready",timeout=180000)
            page.keyboard.press('Enter');page.wait_for_timeout(2500)
            state=page.evaluate("JSON.parse(document.documentElement.getAttribute('data-findcat'))")
            assert state['case']['phase']=={'Story':0}
            targets=[next(t for t in state['texts'] if '“율아, 잡아 봐!”' in t['text']),
                     next(t for t in state['texts'] if '다음 이야기' in t['text'])]
            original=args.output/f'game-dpr{dpr}.png';page.screenshot(path=str(original))
            # Force repeated production UI rebuilds using the toy action.
            for _ in range(8):page.keyboard.press('Space');page.wait_for_timeout(120)
            page.wait_for_timeout(1500)
            repeat=args.output/f'redraw-dpr{dpr}.png';page.screenshot(path=str(repeat))
            for t in targets:
                box=tuple(round(v) for v in t['min']+t['max'])
                a=Image.open(original).crop(box).convert('RGB');b=Image.open(repeat).crop(box).convert('RGB')
                assert sum(ImageStat.Stat(ImageChops.difference(a,b)).sum)==0,'Text changed after repeated UI rebuilds'
            await_fonts="""async () => { for (const [name,file,weight] of [
                ['ProofRegular','Pretendard-Regular.otf','400'],['ProofBold','Pretendard-SemiBold.otf','600']]) {
                const face=new FontFace(name,`url(${new URL('assets/fonts/'+file,document.baseURI)})`,{weight});
                document.fonts.add(await face.load()); }}"""
            page.evaluate(await_fonts)
            for i,t in enumerate(targets):
                page.evaluate("""({t,dpr,i})=>{const el=document.createElement('div');el.id='type-reference';
                    Object.assign(el.style,{position:'fixed',left:t.min[0]/dpr+'px',top:t.min[1]/dpr+'px',
                        width:(t.max[0]-t.min[0])/dpr+'px',height:(t.max[1]-t.min[1])/dpr+'px',
                        fontFamily:i?'ProofBold':'ProofRegular',fontWeight:i?'600':'400',
                        fontSize:t.font_size+'px',lineHeight:'1.4',whiteSpace:'pre-wrap',zIndex:100,
                        color:i?'rgb(252,249,240)':'rgb(23,43,38)',background:i?'rgb(38,77,64)':'rgb(252,249,240)'});
                    el.textContent=t.text;document.body.appendChild(el);} """,{'t':t,'dpr':dpr,'i':i})
                reference=args.output/f'reference-{i}-dpr{dpr}.png';page.screenshot(path=str(reference))
                page.evaluate("document.querySelector('#type-reference').remove()")
                box=tuple(round(v) for v in t['min']+t['max'])
                game=Image.open(original).crop(box).convert('RGB');native=Image.open(reference).crop(box).convert('RGB')
                # Compare integrated stroke coverage, independently of rasterizer hinting/placement.
                def mass(im):
                    channel=im.getchannel('R');bg,fg=(38,252) if i else (252,23)
                    return sum(n*abs(value-bg)/abs(fg-bg) for value,n in enumerate(channel.histogram()))
                ratio=mass(game)/mass(native)
                assert .75<ratio<1.25,{'kind':i,'dpr':dpr,'stroke_coverage_ratio':ratio}
                w=max(game.width,native.width);h=game.height
                sheet=Image.new('RGB',(w,h*2+48),(242,242,238));draw=ImageDraw.Draw(sheet)
                draw.text((5,3),'GAME',(20,30,25));sheet.paste(game,(0,22))
                draw.text((5,h+26),'BROWSER REFERENCE / SAME FONT',(20,30,25));sheet.paste(native,(0,h+48))
                sheet.resize((w*2,(h*2+48)*2),Image.Resampling.NEAREST).save(args.output/f'compare-{i}-dpr{dpr}.png')
                results.append({'dpr':dpr,'kind':'light' if i else 'dark','coverage_ratio':ratio})
            assert not errors,errors
            page.close()
        (args.output/'report.json').write_text(json.dumps({'passed':True,'url':args.url,'redraws':8,'comparisons':results},indent=2),encoding='utf-8')
        print(json.dumps(results),flush=True)
        print('PASS: dark/light glyph coverage matches browser reference; repeated UI redraws are identical.',flush=True)
    finally:browser.close()
