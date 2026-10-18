"""Produce an asset-complete Windows portable ZIP and third-party license inventory."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
DIST=ROOT/'dist'
PACKAGE=DIST/'FindCat-1.0.0-windows-x64'
PACKAGE.mkdir(parents=True,exist_ok=True)
exe=ROOT/'target/release/findcat.exe'
if not exe.exists():raise SystemExit('Run cargo build --release --locked first.')
shutil.copy2(exe,PACKAGE/'FindCat.exe')
shutil.copytree(ROOT/'assets',PACKAGE/'assets',dirs_exist_ok=True)
for name in ['README.md','LICENSE','THIRD_PARTY.md']:shutil.copy2(ROOT/name,PACKAGE/name)
licenses=PACKAGE/'licenses'
licenses.mkdir(exist_ok=True)
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1'],cwd=ROOT))
index=[]
for item in metadata['packages']:
    if item['name']=='findcat':continue
    folder=Path(item['manifest_path']).parent
    copied=[]
    for f in folder.iterdir():
        if f.is_file() and f.name.upper().startswith(('LICENSE','COPYING','NOTICE')):
            dest=f"{item['name']}-{item['version']}-{f.name}"
            shutil.copy2(f,licenses/dest);copied.append(dest)
    index.append({'name':item['name'],'version':item['version'],'license':item.get('license'),'repository':item.get('repository'),'texts':copied})
(licenses/'dependencies.json').write_text(json.dumps(index,indent=2),encoding='utf-8')
manifest={str(f.relative_to(PACKAGE)).replace('\\','/'):hashlib.sha256(f.read_bytes()).hexdigest() for f in PACKAGE.rglob('*') if f.is_file() and f!=PACKAGE/'SHA256SUMS.json'}
(PACKAGE/'SHA256SUMS.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
archive=DIST/f'{PACKAGE.name}.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=7,strict_timestamps=False) as z:
    for f in PACKAGE.rglob('*'):
        if f.is_file():z.write(f,f.relative_to(DIST))
print(f'{archive}\n{archive.stat().st_size/1024/1024:.1f} MiB\n{len(index)} dependency license records')
