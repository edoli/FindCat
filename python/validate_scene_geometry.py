"""Check the authored scenery budget, world bounds and preservation of all props.

Optional --reference-dir points at the GLB and manifest from a previous release.
No Blender process or game window is needed for these structural checks.
"""
import argparse
import json
from pathlib import Path
import struct

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--reference-dir', type=Path)
args = parser.parse_args()


def describe(directory):
    blob = (directory / 'neighborhood.glb').read_bytes()
    assert blob[:4] == b'glTF' and struct.unpack_from('<I', blob, 4)[0] == 2
    model = json.loads(blob[20:20 + struct.unpack_from('<I', blob, 12)[0]])
    binary_offset = 28 + struct.unpack_from('<I', blob, 12)[0]
    manifest = json.loads((directory / 'neighborhood-manifest.json').read_text())
    geometry_bounds = []
    triangles = 0

    def visit(index, parent):
        nonlocal triangles
        node = model['nodes'][index]
        if 'matrix' in node:
            local = np.array(node['matrix']).reshape(4, 4).T
        else:
            x, y, z, w = node.get('rotation', [0, 0, 0, 1])
            local = np.array([
                [1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w), 0],
                [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w), 0],
                [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y), 0],
                [0, 0, 0, 1],
            ], dtype=float)
            local[:3, :3] *= node.get('scale', [1, 1, 1])
            local[:3, 3] = node.get('translation', [0, 0, 0])
        world = parent @ local
        if 'mesh' in node:
            for primitive in model['meshes'][node['mesh']]['primitives']:
                assert primitive.get('mode', 4) == 4
                triangles += model['accessors'][primitive['indices']]['count'] // 3
                position = model['accessors'][primitive['attributes']['POSITION']]
                assert position['componentType'] == 5126 and position['type'] == 'VEC3'
                view = model['bufferViews'][position['bufferView']]
                offset = binary_offset + view.get('byteOffset', 0) + position.get('byteOffset', 0)
                points = np.ndarray((position['count'], 3), dtype='<f4', buffer=blob,
                    offset=offset, strides=(view.get('byteStride', 12), 4))
                # Transform actual vertices, not the corners of a rotated AABB:
                # the latter overestimates the old material-wide batch bounds.
                vertices = points @ world[:3, :3].T + world[:3, 3]
                geometry_bounds.append([vertices.min(axis=0), vertices.max(axis=0)])
        for child in node.get('children', []):
            visit(child, world)

    for root in model['scenes'][model.get('scene', 0)]['nodes']:
        visit(root, np.eye(4))
    geometry = np.array(geometry_bounds)
    assert np.isfinite(geometry).all() and (geometry[:, 1] >= geometry[:, 0]).all()
    return {'triangles': triangles, 'primitives': len(geometry), 'bytes': len(blob),
        'bounds': [geometry[:, 0].min(axis=0).tolist(), geometry[:, 1].max(axis=0).tolist()],
        'materials': sorted((m['name'], m.get('pbrMetallicRoughness')) for m in model['materials']),
        'manifest': manifest}


current = describe(ROOT / 'assets/models')
assert current['triangles'] < 130000, current['triangles']
assert current['primitives'] == 21, 'Keep material batching without increasing draw submissions'
assert current['manifest']['triangles'] == current['triangles']
assert current['manifest']['batches'] == current['primitives']
assert current['manifest']['objects_before_batching'] == len(current['manifest']['named_props']) == 1184
report = {k: v for k, v in current.items() if k not in ('materials', 'manifest')}
if args.reference_dir:
    reference = describe(args.reference_dir)
    assert current['manifest']['named_props'] == reference['manifest']['named_props'], 'Props removed or renamed'
    assert current['materials'] == reference['materials'], 'Material appearance changed'
    assert np.allclose(current['bounds'], reference['bounds'], atol=.025), (current['bounds'], reference['bounds'])
    assert current['triangles'] < reference['triangles'] * .65
    report['reference_triangles'] = reference['triangles']
    report['triangle_reduction_percent'] = 100 * (1 - current['triangles'] / reference['triangles'])
print(json.dumps(report, indent=2))
print('PASS: scenery budget, material batches, bounds, materials and all 1184 authored props')
