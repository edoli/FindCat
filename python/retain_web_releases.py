"""Keep bootstraps referenced by the last two published Pages documents.

Pages caches HTML for 600 seconds. Removing old hashed JS/WASM on deployment
breaks browsers with that HTML. Only each archive's own references are copied,
so files retained by older releases do not accumulate.
"""
from pathlib import Path, PurePosixPath
import argparse
import re
import shutil
import tarfile

BOOTSTRAP = re.compile(r'(?<![\w-])(?:findcat-[0-9a-f]+(?:_bg)?\.(?:wasm|js)|[0-9a-f]+-initializer\.mjs)\b')


def retain(archive: Path, destination: Path) -> list[str]:
    with tarfile.open(archive) as bundle:
        members = {str(PurePosixPath(m.name)): m for m in bundle.getmembers() if m.isfile()}
        index = members.get('index.html')
        if index is None:
            raise ValueError(f'{archive}: missing index.html')
        with bundle.extractfile(index) as stream:
            referenced = sorted(set(BOOTSTRAP.findall(stream.read().decode('utf-8'))))
        if not referenced:
            raise ValueError(f'{archive}: no bootstrap references')
        for name in referenced:
            if name not in members:
                raise ValueError(f'{archive}: missing {name}')
            target = destination / name
            # Never extract paths, links, assets, or an older index over the new build.
            if not target.exists():
                with bundle.extractfile(members[name]) as source, target.open('wb') as output:
                    shutil.copyfileobj(source, output)
        # The retained binaries also retain their own dependency notices. Ignore
        # nested historical notices so each deploy contains only two prior releases.
        notices = destination / 'licenses' / 'previous' / archive.parent.name
        for name, member in members.items():
            parts = PurePosixPath(name).parts
            if name in ('LICENSE', 'THIRD_PARTY.md') or (
                len(parts) == 2 and parts[0] == 'licenses' and parts[1] != '..'
            ):
                notices.mkdir(parents=True, exist_ok=True)
                with bundle.extractfile(member) as source, (notices / parts[-1]).open('wb') as output:
                    shutil.copyfileobj(source, output)
        return referenced


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archives', type=Path)
    parser.add_argument('--dist', type=Path, default=Path('dist/web'))
    args = parser.parse_args()
    for archive in sorted(args.archives.glob('*/artifact.tar')):
        print(f'{archive.parent.name}: retained {", ".join(retain(archive, args.dist))}')
