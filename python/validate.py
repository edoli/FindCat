"""Validate the real game in an isolated testbed without controlling the desktop.

uv run --project python python python/validate.py [--release] [--exe PATH]
"""
from pathlib import Path
import argparse
import json
import os
import subprocess
import sys
from PIL import Image, ImageStat

ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--release',action='store_true')
parser.add_argument('--exe',type=Path)
args=parser.parse_args()
os.chdir(ROOT)
art=ROOT/'artifacts'
art.mkdir(exist_ok=True)
if not args.exe:
    subprocess.run(['cargo','test','--locked'],check=True)
    command=['cargo','build','--locked']+(['--release'] if args.release else [])
    subprocess.run(command,check=True)
exe=args.exe.resolve() if args.exe else ROOT/'target'/('release' if args.release else 'debug')/'findcat.exe'
report=art/'testbed-report.json'
if report.exists():report.unlink()
with (art/'testbed.log').open('w',encoding='utf-8') as log:
    process=subprocess.run([str(exe),'--testbed'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=300)
if process.returncode:
    print((art/'testbed.log').read_text(encoding='utf-8')[-8000:]);sys.exit(process.returncode)
result=json.loads(report.read_text())
assert result['passed'] and result['all_clues']==8 and result['trust']==3
for path in result['screenshots']:
    im=Image.open(ROOT/path).convert('RGB')
    assert im.width>=1100 and im.height>=760
    assert max(ImageStat.Stat(im).stddev)>15, f'Blank render: {path}'
    assert len(im.resize((180,100)).getcolors(20000) or [])>300, f'Missing scene: {path}'
if 'camera_views' in result:
    heights=[v['height'] for v in result['camera_views']]
    assert max(heights)-min(heights)>8, 'Cinematic camera did not change framing'
    assert result['ui_bounds_checks']>40
print(json.dumps({k:v for k,v in result.items() if k!='camera_views'},ensure_ascii=False,indent=2))
print('PASS: real GPU rendering, navigation, all clues, deductions, rescue, save reload, screenshots.')
