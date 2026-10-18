"""Copy the locked dependency notices into a built static game (standard library only)."""
from pathlib import Path
import json
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
dest = root / 'dist/web/licenses'
dest.mkdir(parents=True, exist_ok=True)
metadata = json.loads(subprocess.check_output([
    'cargo','metadata','--locked','--format-version','1',
    '--filter-platform','wasm32-unknown-unknown','--features','web',
], cwd=root))
records = []
for package in metadata['packages']:
    if package['name'] == 'findcat':
        continue
    texts = []
    for file in Path(package['manifest_path']).parent.iterdir():
        if file.is_file() and file.name.upper().startswith(('LICENSE','COPYING','NOTICE')):
            name = f"{package['name']}-{package['version']}-{file.name}"
            shutil.copy2(file, dest / name)
            texts.append(name)
    records.append({k:package.get(k) for k in ['name','version','license','repository']} | {'texts':texts})
(dest / 'dependencies.json').write_text(json.dumps(records, indent=2), encoding='utf-8')
print(f'Included {len(records)} dependency license records in {dest}')
