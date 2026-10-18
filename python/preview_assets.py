"""Optional Blender render for geometry review; does not interact with the desktop."""
from pathlib import Path
import bpy
from mathutils import Vector

root=Path(__file__).resolve().parents[1]
out=root/'artifacts'
out.mkdir(exist_ok=True)
for name,position,target,scale in [('cat',(1.7,-3.5,1.8),(0,0,.65),2.2),('neighborhood',(12,-23,17.5),(0,0,.3),19)]:
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(root/'assets/models'/f'{name}.glb'))
    bpy.ops.object.camera_add(location=position)
    cam=bpy.context.object;cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler()
    cam.data.type='ORTHO';cam.data.ortho_scale=scale
    scene=bpy.context.scene;scene.camera=cam
    bpy.ops.object.light_add(type='AREA',location=(-3,-4,8));bpy.context.object.data.energy=750;bpy.context.object.data.shape='DISK';bpy.context.object.data.size=6
    scene.world.color=(.45,.48,.4)
    scene.render.engine='CYCLES';scene.cycles.samples=16
    scene.render.resolution_x=1000;scene.render.resolution_y=850;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG';scene.render.filepath=str(out/f'asset-{name}.png')
    bpy.ops.render.render(write_still=True)
