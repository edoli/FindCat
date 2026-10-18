"""Run in Blender's bundled Python. All geometry is original and deterministic.

blender --background --python python/generate_assets.py
"""
from pathlib import Path
import math
import random
import sys
import json
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "models"
OUT.mkdir(parents=True, exist_ok=True)
random.seed(41)

def material(name, color, metallic=0):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*color, 1)
    m.use_nodes = True
    p = m.node_tree.nodes.get('Principled BSDF')
    p.inputs['Base Color'].default_value = (*color, 1)
    p.inputs['Roughness'].default_value = .78
    p.inputs['Metallic'].default_value = metallic
    return m

M = {n: material(n, c) for n,c in {
    'cream':(.96,.88,.72), 'paper':(1,.96,.86), 'white':(.99,.97,.9),
    'ginger':(.64,.38,.20), 'lightginger':(.77,.52,.30), 'stripe':(.47,.28,.16),
    'pink':(.84,.46,.44), 'nose':(.57,.29,.29), 'ink':(.025,.030,.025),
    'gold':(.96,.65,.24), 'mint':(.43,.66,.55), 'leaf':(.25,.49,.36),
    'sage':(.52,.66,.38), 'grass':(.50,.63,.40), 'ground':(.80,.77,.59),
    'path':(.94,.85,.67), 'stone':(.69,.74,.66), 'wood':(.39,.26,.19),
    'pale_wood':(.65,.47,.31), 'terracotta':(.76,.36,.25), 'rose':(.87,.57,.49),
    'roof':(.39,.52,.49), 'glass':(.43,.67,.65), 'water':(.34,.62,.66),
    'navy':(.21,.33,.39), 'lavender':(.66,.59,.74), 'yellow':(.98,.77,.29)
}.items()}

def p(v): return Vector((v[0], -v[2], v[1]))
def finish(o,name,mat):
    o.name=name
    o.data.materials.append(M[mat])
    return o
def cube(name, pos, scale, mat, bevel=.05):
    bpy.ops.mesh.primitive_cube_add(size=1, location=p(pos))
    o=bpy.context.object; o.scale=(scale[0],scale[2],scale[1])
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    if bevel:
        b=o.modifiers.new('Soft handmade edges','BEVEL'); b.width=bevel; b.segments=2
        bpy.ops.object.modifier_apply(modifier=b.name)
        o.modifiers.new('Weighted normals','WEIGHTED_NORMAL')
    return finish(o,name,mat)
def ball(name,pos,scale,mat,segments=20):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments,ring_count=12, radius=1,location=p(pos))
    o=bpy.context.object; o.scale=(scale[0],scale[2],scale[1])
    for f in o.data.polygons:f.use_smooth=True
    return finish(o,name,mat)
def cyl(name,pos,r,depth,mat,vertices=24):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices,radius=r,depth=depth,location=p(pos))
    return finish(bpy.context.object,name,mat)
def beam(name,a,b,r,mat):
    d=p(b)-p(a)
    o=cyl(name,tuple((a[i]+b[i])/2 for i in range(3)),r,d.length,mat,12)
    o.rotation_euler=d.to_track_quat('Z','Y').to_euler();return o
def curve(name,pts,r,mat):
    c=bpy.data.curves.new(name,'CURVE');c.dimensions='3D';c.bevel_depth=r;c.bevel_resolution=3
    s=c.splines.new('BEZIER');s.bezier_points.add(len(pts)-1)
    for bp,pt in zip(s.bezier_points,pts):bp.co=p(pt);bp.handle_left_type='AUTO';bp.handle_right_type='AUTO'
    o=bpy.data.objects.new(name,c);bpy.context.collection.objects.link(o);o.data.materials.append(M[mat]);return o
def tri(name,verts,mat):
    me=bpy.data.meshes.new(name);me.from_pydata([p(v) for v in verts],[],[(0,1,2)]);me.update()
    o=bpy.data.objects.new(name,me);bpy.context.collection.objects.link(o);o.data.materials.append(M[mat]);return o
def clear():
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
def group(name,position,items):
    root=bpy.data.objects.new(name,None);bpy.context.collection.objects.link(root);root.location=p(position)
    bpy.context.view_layer.update()
    for obj in items:
        if obj.parent is not None:continue
        world_matrix=obj.matrix_world.copy();obj.parent=root;obj.matrix_world=world_matrix
    return root
def export(name):
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.export_scene.gltf(filepath=str(OUT/f'{name}.glb'),export_format='GLB',export_apply=True,export_yup=True)
    print('EXPORTED',name)

def cat():
    clear()
    body=ball('Round cream ginger body',(0,.43,-.08),(.34,.38,.56),'lightginger')
    body.data.materials.append(M['white'])
    for face in body.data.polygons:
        if .12<face.center.y<.42 and face.center.z>0:face.material_index=1
    ball('White chest',(0,.45,.40),(.28,.31,.17),'white')
    for x in [-.23,.23]:
        ball('White little front socks',(x,.14,.32),(.12,.15,.18),'white')
        ball('Soft back haunch',(x,.28,-.37),(.19,.26,.25),'lightginger')
        ball('White back socks',(x,.12,-.31),(.14,.12,.19),'white')
    head_before=set(bpy.context.scene.objects)
    ball('Large round ginger head',(0,.86,.40),(.39,.35,.30),'lightginger')
    ball('White face blaze',(0,.81,.647),(.10,.255,.062),'white')
    for x in [-.26,.26]:
        verts=[(x-.13,1.05,.47),(x+.12,1.05,.47),(x*1.24,1.34,.36),(x,1.04,.23)]
        me=bpy.data.meshes.new('Soft pointed ear');me.from_pydata([p(v) for v in verts],[],[(0,1,2),(0,2,3),(1,3,2),(0,3,1)]);me.update()
        ear=bpy.data.objects.new('Soft pointed ear',me);bpy.context.collection.objects.link(ear);ear.data.materials.append(M['lightginger'])
        tri('Pink inner ear',[(x-.085,1.09,.478),(x+.075,1.09,.478),(x*1.20,1.285,.39)],'pink')
    for x in [-.185,.185]:
        eyes_before=set(bpy.context.scene.objects)
        ball('Eyeliner',(x,.89,.650),(.144,.156,.054),'stripe')
        ball('Amber gold eyes',(x,.89,.679),(.124,.135,.032),'gold')
        ball('Round pupils',(x,.895,.705),(.085,.106,.022),'ink')
        ball('Big eye sparkle',(x-.025,.945,.727),(.029,.031,.009),'white')
        ball('Tiny eye sparkle',(x+.031,.861,.727),(.012,.014,.007),'white')
        group('CatEyeRigL' if x<0 else 'CatEyeRigR',(x,.89,.68),set(bpy.context.scene.objects)-eyes_before)
    for x in [-.092,.092]:ball('White plush muzzle',(x,.702,.679),(.115,.087,.077),'white')
    tri('Pink heart nose',[(-.065,.768,.759),(.065,.768,.759),(0,.708,.782)],'nose')
    curve('Little smile',[(-.055,.672,.739),(0,.684,.758),(.055,.672,.739)],.009,'stripe')
    beam('Nose line',(0,.712,.765),(0,.683,.761),.007,'stripe')
    for sign in [-1,1]:
        for j in range(3):
            curve('Fine white whiskers',[(sign*.12,.72-j*.025,.741),(sign*.30,.73-j*.035,.75),(sign*.51,.77-j*.055,.70)],.004,'white')
    for x in [-.12,.12]:
        curve('Forehead tabby detail',[(x,1.15,.45),(x*.8,1.12,.56),(x*.7,1.065,.62)],.022,'ginger')
    group('CatHeadRig',(0,.86,.40),set(bpy.context.scene.objects)-head_before)
    tail_before=set(bpy.context.scene.objects)
    curve('Curled cream tail',[(0,.37,-.48),(.35,.25,-.68),(.55,.35,-.5),(.56,.68,-.40),(.46,.80,-.41)],.105,'lightginger')
    ball('Rounded tail tip',(.46,.80,-.41),(.105,.105,.105),'lightginger')
    group('CatTailRig',(0,.37,-.48),set(bpy.context.scene.objects)-tail_before)
    for z in [-.23,-.03]:curve('Subtle back markings',[(-.21,.70,z),(0,.805,z),(.21,.70,z)],.018,'ginger')
    export('cat')

def detective():
    clear()
    for x in [-.13,.13]:
        before=set(bpy.context.scene.objects)
        cube('Boot',(x,.10,.06),(.18,.19,.29),'wood')
        cube('Trouser',(x,.32,0),(.15,.34,.15),'navy')
        group('LegRigL' if x<0 else 'LegRigR',(x,.47,0),set(bpy.context.scene.objects)-before)
    ball('Coat',(0,.65,0),(.29,.34,.19),'cream')
    cube('Coat hem',(0,.43,0),(.47,.19,.31),'cream',.07)
    for x in [-.31,.31]:
        before=set(bpy.context.scene.objects)
        beam('Sleeve',(x*.82,.81,0),(x,.48,.04),.095,'cream')
        ball('Hand',(x,.45,.06),(.075,.085,.075),'rose')
        group('ArmRigL' if x<0 else 'ArmRigR',(x*.82,.81,0),set(bpy.context.scene.objects)-before)
    before=set(bpy.context.scene.objects)
    ball('Head',(0,1.08,.02),(.22,.23,.21),'rose')
    ball('Hair',(0,1.19,-.04),(.23,.17,.22),'wood')
    for x in [-.082,.082]:ball('Eyes',(x,1.09,.219),(.022,.029,.012),'ink')
    ball('Nose',(0,1.04,.23),(.025,.025,.025),'pink')
    cyl('Detective hat brim',(0,1.27,0),.34,.065,'pale_wood')
    cyl('Detective hat',(0,1.36,0),.225,.18,'cream')
    cyl('Hat band',(0,1.31,0),.23,.055,'wood')
    group('DetectiveHeadRig',(0,1.08,.02),set(bpy.context.scene.objects)-before)
    cube('Mint scarf',(0,.895,.14),(.36,.105,.16),'mint')
    cube('Scarf end',(.13,.76,.19),(.1,.25,.07),'mint')
    cube('Detective satchel',(0,.65,-.24),(.34,.35,.15),'terracotta')
    for y in [.52,.66,.80]:ball('Golden coat buttons',(0,y,.194),(.025,.025,.016),'gold')
    export('detective')

def fairy():
    clear()
    ball('Mori light body',(0,.0,0),(.17,.22,.14),'paper')
    ball('Mori head',(0,.22,.025),(.22,.18,.16),'paper')
    for x in [-.13,.13]:
        tri('Mori ear',[(x-.08,.30,.07),(x+.08,.30,.07),(x*1.3,.48,.025)],'paper')
        ball('Mori eye',(x*.60,.24,.17),(.022,.03,.009),'ink')
        ball('Mori blush',(x,.18,.15),(.035,.019,.009),'pink')
        ball('Mori wing',(x*2,.04,-.02),(.17,.09,.045),'gold')
    curve('Mori tail',[(0,-.1,-.07),(.2,-.13,-.16),(.24,-.02,-.17)],.035,'paper')
    ball('Mori star',(0,-.06,.16),(.07,.07,.035),'gold')
    export('fairy')

def pot(x,z,s=1,flower=False):
    cyl('Terracotta flowerpot',(x,.18*s,z),.21*s,.34*s,'terracotta',12)
    cyl('Pot rim',(x,.34*s,z),.24*s,.075*s,'rose',12)
    cyl('Pot soil',(x,.38*s,z),.19*s,.025,'wood',12)
    for j in range(3):
        a=j*2.1;dx=math.cos(a)*.14*s;dz=math.sin(a)*.14*s
        beam('Plant stem',(x,.36*s,z),(x+dx,.68*s,z+dz),.015*s,'leaf')
        ball('Succulent leaves',(x+dx,.59*s,z+dz),(.14*s,.06*s,.09*s),'sage')
        if flower:
            for k in range(5):
                a=k*math.tau/5
                ball('Flower petals',(x+dx+math.cos(a)*.065*s,.73*s,z+dz+math.sin(a)*.065*s),(.055*s,.032*s,.055*s),'rose' if j%2 else 'yellow',12)
            ball('Flower heart',(x+dx,.75*s,z+dz),(.04*s,.025*s,.04*s),'gold',12)

def tree(x,z,s=1):
    cyl('Tree trunk',(x,.85*s,z),.13*s,1.7*s,'pale_wood',12)
    for dx,dz,y,r in [(-.35,0,1.8,.7),(.35,.12,2,.72),(0,-.25,2.4,.8),(0,.4,1.9,.65)]:
        ball('Round sage canopy',(x+dx*s,y*s,z+dz*s),(r*s,r*.9*s,r*s),'sage' if dx<=0 else 'leaf',16)

def window(x,y,z,w=.8):
    cube('Window frame',(x,y,z),(w,.85,.09),'cream')
    cube('Teal glass',(x,y,z+.052),(w-.12,.7,.04),'glass')
    cube('Window mullion',(x,y,z+.08),(.045,.74,.025),'paper')
    cube('Window bar',(x,y,z+.08),(w-.08,.045,.025),'paper')
    cube('Window ledge',(x,y-.47,z+.08),(w+.15,.1,.22),'paper')

def world():
    clear()
    cube('Floating neighborhood foundation',(0,-.43,.2),(15.5,.8,12.6),'pale_wood',.38)
    cube('Cream foundation stripe',(0,-.15,.2),(15.65,.17,12.75),'cream',.26)
    cube('Soft moss island',(0,-.035,.2),(15.6,.17,12.7),'grass',.25)
    cube('Main courtyard path',(-.4,.06,.4),(11.2,.08,2.8),'path',.20)
    cube('Cross path',(-2,.065,-.3),(2.5,.085,9.6),'path',.15)
    cube('Greenhouse path',(3.3,.067,2.6),(4.1,.09,1.5),'path',.12)
    # Handmade pavers with slight offsets.
    for i in range(12):
        for j in range(2):
            x=-5.4+i*.87;z=-.22+j*1.1
            cube('Limestone courtyard paver',(x,.12,z),(.78,.055,.96),'cream' if (i+j)%4 else 'ground',.07)
    for i in range(8):cube('Little stepping stone',(-2.05,.12,-4+i*1.13),(1.5,.06,.75),'cream',.12)
    for i in range(4):cube('Greenhouse stepping stone',(2.2+i*.78,.12,2.65),(.6,.06,.78),'cream',.10)
    # House with soft roof tiles, shutters and welcoming open door.
    cube('Our honey house',(-5,1.35,-4),(3.5,2.7,3),'cream',.10)
    cube('Home rose skirting',(-5,.25,-2.45),(3.6,.4,.17),'rose')
    for x in [-6.1,-3.9]:window(x,1.65,-2.46,.75)
    cube('Dark open doorway',(-5,.91,-2.45),(.8,1.8,.12),'wood')
    cube('Mint door ajar',(-4.55,.93,-2.22),(.1,1.83,.8),'mint')
    ball('Door knob',(-4.47,.91,-1.99),(.04,.04,.04),'gold')
    cube('Welcome step',(-5,.12,-1.93),(1.1,.18,.7),'paper')
    for k in range(2):
        o=cube('Sloping sage roof',(-5,2.98,-4+(-.82 if k==0 else .82)),(3.95,.18,1.96),'roof',.04)
        o.rotation_euler[0]=(-1 if k==0 else 1)*math.radians(25)
    beam('Roof ridge',(-7.02,3.4,-4),(-2.98,3.4,-4),.10,'cream')
    for i in range(12):
        x=-6.83+i*.335
        beam('Roof tile seam',(x,3.4,-4),(x,2.65,-2.27),.018,'mint')
    cube('Brick chimney',(-5.9,3.3,-4.5),(.5,1.15,.55),'rose')
    cube('Chimney cap',(-5.9,3.9,-4.5),(.67,.13,.7),'cream')
    pot(-6.55,-1.65,1.2,True);pot(-3.5,-1.8,.9,True)
    # Cafe.
    cube('Moon bakery', (3.6,1.36,-4.25),(4.4,2.72,3.3),'rose',.12)
    cube('Bakery flat roof',(3.6,2.83,-4.25),(4.7,.23,3.6),'paper',.11)
    cube('Bakery roof garden',(3.6,2.99,-4.3),(4.15,.14,2.9),'roof',.07)
    for x in [2.1,4.0]:window(x,1.36,-2.56,1.25)
    cube('Bakery mint door',(5.12,1.04,-2.56),(.78,2,.10),'roof')
    window(5.12,1.36,-2.48,.55)
    for i in range(11):
        o=cube('Striped bakery awning',(1.62+i*.37,2.22,-1.99),(.37,.09,1.25),'cream' if i%2 else 'terracotta',.015)
        o.rotation_euler[0]=-.17
        cube('Scalloped awning edge',(1.62+i*.37,2.05,-1.37),(.37,.23,.10),'cream' if i%2 else 'terracotta',.06)
    cube('Bakery sign',(3.4,2.58,-2.52),(2.45,.36,.10),'wood')
    # Abstract golden bread emblem, no font dependencies in Blender.
    for dx in [-.23,0,.23]:ball('Bread loaf emblem',(3.4+dx,2.6,-2.43),(.22,.105,.05),'gold')
    for x in [1.8,5.6]:pot(x,-1.1,.85,True)
    cyl('Cafe table',(5.8,.75,.4),.49,.10,'pale_wood')
    cyl('Cafe table leg',(5.8,.39,.4),.07,.7,'wood')
    for z in [-.3,1.1]:
        cyl('Round cafe stool',(5.8,.42,z),.23,.11,'terracotta')
        for dx in [-.12,.12]:beam('Stool legs',(5.8+dx,.04,z),(5.8+dx,.39,z),.035,'wood')
    cyl('Tea cup',(5.8,.87,.4),.09,.16,'paper')
    # Greenhouse: open front, reassuringly visible interior.
    cube('Greenhouse foundation',(5, .12,4),(3.6,.2,3),'stone')
    for x in [3.2,6.8]:
        for z in [2.5,5.5]:cube('Greenhouse corner',(x,1.12,z),(.12,2.05,.12),'cream')
    for x in [3.2,6.8]:
        cube('Greenhouse low wall',(x,.46,4),(.12,.72,3),'mint')
        for z in [3.2,4.5]:cube('Greenhouse side pane',(x,1.43,z),(.035,1.05,1.1),'glass')
    cube('Greenhouse back wall',(5,.24,5.5),(3.6,.32,.13),'mint')
    for x in [3.2,6.8]:
        beam('Roof rafters',(x,2.14,2.5),(x,2.9,4),.055,'cream')
        beam('Roof rafters',(x,2.9,4),(x,2.14,5.5),.055,'cream')
    beam('Greenhouse ridge',(3.1,2.9,4),(6.9,2.9,4),.065,'cream')
    beam('Greenhouse front eave',(3.1,2.14,2.5),(6.9,2.14,2.5),.06,'cream')
    cube('Potting bench',(5.3,.75,4.7),(2.4,.12,.65),'pale_wood')
    for x in [4.4,6.2]:
        for z in [4.5,4.95]:cube('Potting bench leg',(x,.39,z),(.08,.7,.08),'wood')
    for x,z,s in [(3.6,3.0,1.5),(6.4,3.0,1.5),(3.7,4.8,1.3),(5,4.2,.9),(6.3,5,.8)]:pot(x,z,s,True)
    cube('Dark safe hiding nook',(4.7,.63,2.92),(1.3,1.12,.10),'wood')
    cube('Shelter roof',(4.7,1.26,3.3),(1.5,.10,1.05),'pale_wood')
    for x in [4.02,5.38]:beam('Shelter legs',(x,.1,3.3),(x,1.25,3.3),.045,'pale_wood')
    # Wind chime outside, reachable from front.
    beam('Chime post',(2.1,.1,4),(2.1,2.2,4),.055,'wood')
    beam('Chime arm',(2.1,2.17,4),(2.65,2.17,4),.04,'wood')
    for i in range(3):
        x=2.4+i*.11;beam('Chime string',(x,2.13,4),(x,1.9-i*.12,4),.009,'cream')
        cyl('Golden wind chime',(x,1.7-i*.12,4),.035,.37,'gold',12)
    # Bench and box, little laundry.
    for j in range(4):cube('Bench seat slats',(-4.5,.58,2.65+j*.13),(2.4,.10,.10),'pale_wood')
    for y in [.85,1.1]:cube('Bench backrest',(-4.5,y,2.55),(2.4,.16,.12),'pale_wood')
    for x in [-5.4,-3.6]:
        for z in [2.7,3.05]:cube('Bench legs',(x,.30,z),(.10,.6,.10),'wood')
    cube('Cat cardboard box',(-4.2,.24,2.55),(.65,.45,.55),'ground')
    cube('Dark box opening',(-4.2,.24,2.26),(.43,.29,.012),'wood',.04)
    for x in [-6.8,-4.8]:beam('Clothesline pole',(x,.05,.0),(x,1.85,0),.055,'pale_wood')
    curve('Clothesline',[(-6.8,1.8,0),(-5.8,1.65,0),(-4.8,1.8,0)],.012,'cream')
    cube('Familiar striped blanket',(-5.6,1.11,.0),(.85,1.10,.035),'lavender',.02)
    for y in [.72,.9,1.08,1.26,1.44]:cube('Blanket soft stripe',(-5.6,y,.023),(.85,.045,.015),'paper',.006)
    # Fountain concentric sculpted bowls.
    cyl('Fountain base',(-.2,.18,1),.95,.28,'stone',48)
    cyl('Fountain cream rim',(-.2,.34,1),.88,.18,'cream',48)
    cyl('Still blue water',(-.2,.44,1),.73,.035,'water',48)
    cyl('Fountain stem',(-.2,.72,1),.14,.65,'stone')
    cyl('Fountain upper bowl',(-.2,1.02,1),.45,.15,'cream',32)
    ball('Fountain finial',(-.2,1.2,1),(.13,.18,.13),'gold')
    # Streetlamps.
    for x,z in [(-6.9,2),(1.0,-1.2),(1.2,4.9)]:
        cyl('Lamp foot',(x,.15,z),.19,.25,'roof')
        cyl('Lamp post',(x,1.2,z),.045,2.2,'roof')
        cube('Warm lantern',(x,2.4,z),(.31,.42,.31),'yellow')
        cube('Lantern top',(x,2.65,z),(.42,.10,.42),'roof')
        for dx in [-.15,.15]:
            for dz in [-.15,.15]:beam('Lantern frame',(x+dx,2.16,z+dz),(x+dx,2.6,z+dz),.017,'roof')
    # Boundary picket fence and shrubs.
    for x in [-7+i*.43 for i in range(34)]:
        cube('Back fence picket',(x,.49,-5.85),(.16,.92,.09),'cream')
    for y in [.25,.65]:cube('Long fence rail',(0,y,-5.91),(14.5,.09,.08),'pale_wood')
    for x,z,s in [(-7,4.8,.9),(-6.8,-.7,.8),(-.4,-4.2,1.0),(7,-.4,.8),(-2.3,4.7,.7),(0,5.1,.75)]:tree(x,z,s)
    for x,z in [(-6.7,4.8),(-5.9,5.2),(-3.4,5.3),(-1.3,-4.7),(.3,-5.0),(6.7,-5.1),(7,1.7),(1.4,5.6)]:
        for j in range(3):ball('Rounded garden shrub',(x+j*.28,.38,z),(.42,.42,.4),'sage' if j%2 else 'leaf',12)
    for x,z,s in [(-2.3,-3.4,.85),(-2.9,-2.6,.7),(1.5,3,.75),(.9,3.6,.65),(-3.1,1.9,.65)]:pot(x,z,s,True)
    for i in range(58):
        x=random.uniform(-7.1,7.1);z=random.choice([random.uniform(4.8,5.7),random.uniform(-5.5,-4.9)])
        if x>2.5 and z>0:continue
        ball('Meadow blossom',(x,.18,z),(.07,.085,.07),random.choice(['paper','yellow','rose']),10)
    # Paw-print trail, subtle enough to discover.
    for x,z in [(-4.2,-1.75),(-3.6,-2),(-2.9,-2.3),(1.2,1.2),(2.1,1.6),(3,1.9)]:
        ball('Paw pad in earth',(x,.16,z),(.065,.008,.06),'pale_wood',12)
        for dx,dz in [(-.06,-.065),(0,-.085),(.06,-.065)]:ball('Paw toe',(x+dx,.16,z+dz),(.026,.008,.028),'pale_wood',10)
    enrich_neighborhood()
    # Bake static scenery by material: hundreds of hand-built props, a few draw calls.
    original=list(bpy.context.scene.objects)
    (OUT/'neighborhood-manifest.json').write_text(json.dumps({'objects_before_batching':len(original),'named_props':sorted(o.name for o in original)},indent=2))
    bpy.ops.object.select_all(action='SELECT');bpy.context.view_layer.objects.active=original[0]
    bpy.ops.object.convert(target='MESH')
    batches={}
    for obj in list(bpy.context.scene.objects):
        if obj.type=='MESH':batches.setdefault(obj.data.materials[0].name if obj.data.materials else 'none',[]).append(obj)
    for mat,objects in batches.items():
        bpy.ops.object.select_all(action='DESELECT')
        for obj in objects:obj.select_set(True)
        bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.join();objects[0].name=f'Neighborhood_{mat}'
    export('neighborhood')

def basket(x,y,z):
    cube('Woven bread basket',(x,y,z),(.65,.23,.42),'pale_wood',.05)
    for j in range(3):
        ball('Fresh baguette',(x-.2+j*.20,y+.17,z),(.083,.073,.22),'gold')
        for k in [-.09,0,.09]:cube('Bread scoring',(x-.2+j*.20,y+.225,z+k),(.075,.01,.014),'cream',.002)

def tulip(x,z,color='rose',height=.50):
    beam('Tulip stem',(x,.10,z),(x,height,z),.013,'leaf')
    leaf=ball('Long tulip leaf',(x+.06,height*.5,z),(.025,.14,.045),'leaf',12);leaf.rotation_euler[1]=-.4
    for i in range(3):
        a=i*math.tau/3
        ball('Tulip flower',(x+math.cos(a)*.035,height,z+math.sin(a)*.035),(.063,.10,.065),color,12)

def enrich_neighborhood():
    # Porch details, familiar objects and a small life lived here.
    cube('Woven doorstep mat',(-5,.17,-1.48),(1.05,.045,.51),'terracotta',.08)
    for j in range(7):cube('Doormat weave',(-5.43+j*.14,.199,-1.48),(.025,.007,.43),'cream',.003)
    for x in [-5.13,-4.88]:ball('House slippers',(x,.22,-1.18),(.09,.07,.16),'navy')
    beam('Mailbox post',(-6.7,.1,-2.),(-6.7,1.1,-2.),.06,'pale_wood')
    cube('Rose letter box',(-6.7,1.12,-2.),(.45,.36,.27),'terracotta',.1)
    cube('Mail slot',(-6.7,1.13,-1.857),(.24,.025,.015),'wood',.005)
    cube('White envelope',(-6.69,1.08,-1.83),(.23,.13,.025),'paper',.008)
    beam('Broom handle',(-3.35,.12,-2.33),(-3.6,1.60,-2.38),.026,'pale_wood')
    cube('Broom bristles',(-3.35,.23,-2.33),(.23,.36,.10),'ground',.04)
    for j in range(5):
        z=-4.5+j*.34
        curve('Ivy climbing the house',[(-3.23,.5+j*.25,z),(-3.18,.8+j*.25,z+.12),(-3.22,1.1+j*.25,z+.03)],.014,'leaf')
        for k in range(3):ball('Heart shaped ivy leaves',(-3.16,.65+j*.25+k*.18,z+(.08 if k%2 else -.03)),(.08,.055,.06),'leaf',12)
    # Laundry, folded linens, and the bench's little reading corner.
    cube('Folded mint cloth',(-6.3,.25,.65),(.5,.24,.36),'mint',.08)
    cube('Wicker laundry basket',(-6.3,.17,.65),(.61,.24,.47),'pale_wood',.05)
    cube('Bench cushion',(-5.15,.73,2.85),(.5,.20,.5),'terracotta',.10)
    cube('Bench folded blanket',(-4.60,.69,2.88),(.55,.12,.40),'lavender')
    for i,mat in enumerate(['navy','rose','cream']):cube('Book stack',(-3.65,.69+i*.065,2.88),(.35,.06,.28),mat,.01)
    # A loafing grey neighbor: a visual reason the box clue is a false lead.
    ball('Neighbour cat loaf',(-4.8,.90,2.9),(.22,.19,.31),'stone')
    ball('Neighbour cat head',(-4.8,1.06,3.12),(.20,.19,.17),'stone')
    for dx in [-.12,.12]:
        tri('Neighbour cat ear',[(-4.8+dx-.08,1.17,3.14),(-4.8+dx+.08,1.17,3.14),(-4.8+dx,1.34,3.11)],'stone')
        curve('Neighbour sleepy eyes',[(-4.8+dx*.65-.035,1.07,3.279),(-4.8+dx*.65,1.055,3.29),(-4.8+dx*.65+.035,1.07,3.279)],.009,'ink')
    # Bakery goods, fruit crates, a signboard and a cafe guest.
    cube('Bakery outdoor display',(3.3,.62,-1.20),(1.5,.12,.54),'pale_wood')
    for x in [2.68,3.92]:beam('Display table leg',(x,.1,-1.2),(x,.6,-1.2),.04,'wood')
    basket(2.94,.77,-1.2);basket(3.66,.77,-1.2)
    for x,z in [(6.5,-2.1),(6.5,-1.42)]:
        cube('Slatted fruit crate',(x,.31,z),(.64,.48,.54),'pale_wood')
        for y in [.18,.34,.48]:cube('Crate horizontal gap',(x,y,z+.28),(.6,.035,.008),'wood',.001)
        for j in range(5):ball('Fresh market apples',(x+math.cos(j*2.4)*.18,.58,z+math.sin(j*2.4)*.15),(.095,.095,.095),'terracotta' if j%2 else 'sage',12)
    cube('Bakery sandwich board',(4.52,.56,-.65),(.53,.92,.10),'pale_wood')
    cube('Chalkboard face',(4.52,.59,-.585),(.42,.73,.015),'roof',.01)
    for y,length in [(.78,.25),(.64,.28),(.49,.19),(.36,.24)]:cube('Chalk menu strokes',(4.52,y,-.571),(length,.018,.009),'paper',.003)
    for x,z in [(5.55,.4),(6.1,.55)]:
        cyl('Saucer',(x,.82,z),.12,.025,'cream');cyl('Small coffee',(x,.88,z),.07,.12,'paper')
    x,z=5.20,-1.15
    for dx in [-.10,.10]:
        cube('Baker shoes',(x+dx,.16,z+.025),(.15,.15,.22),'wood')
        cube('Baker trousers',(x+dx,.34,z),(.13,.27,.13),'navy')
        beam('Baker sleeves',(x+dx*2.1,.86,z),(x+dx*2.4,.55,z+.07),.072,'terracotta')
        ball('Baker hands',(x+dx*2.4,.52,z+.07),(.06,.07,.06),'rose')
    ball('Baker shirt',(x,.72,z),(.23,.30,.17),'terracotta')
    cube('Baker apron',(x,.66,z+.16),(.30,.38,.04),'cream',.04)
    ball('Friendly baker face',(x,1.11,z+.02),(.18,.21,.17),'rose')
    cyl('Baker cap',(x,1.30,z),.20,.13,'paper')
    for dx in [-.065,.065]:ball('Baker eyes',(x+dx,1.13,z+.18),(.018,.022,.009),'ink',12)
    # A hand-built bicycle with wheels facing the courtyard.
    for x in [6.32,7.22]:
        bpy.ops.mesh.primitive_torus_add(major_radius=.29,minor_radius=.028,major_segments=28,minor_segments=8,location=p((x,.4,.5)),rotation=(math.pi/2,0,0))
        finish(bpy.context.object,'Bicycle tire','ink')
        for a in range(0,360,45):beam('Bicycle spokes',(x,.4,.5),(x+math.cos(math.radians(a))*.27,.4+math.sin(math.radians(a))*.27,.5),.008,'cream')
    for a,b in [((6.32,.4,.5),(6.6,.9,.5)),((6.6,.9,.5),(6.78,.4,.5)),((6.32,.4,.5),(6.78,.4,.5)),((6.6,.9,.5),(7.02,.9,.5)),((7.02,.9,.5),(6.78,.4,.5)),((7.02,.9,.5),(7.22,.4,.5))]:beam('Mint bicycle frame',a,b,.023,'mint')
    beam('Bicycle handle stem',(7.05,.9,.5),(7.08,1.16,.5),.023,'cream')
    beam('Bicycle handle',(7.08,1.16,.34),(7.08,1.16,.65),.023,'wood')
    cube('Bicycle saddle',(6.55,1.03,.5),(.26,.06,.15),'wood')
    # Greenhouse craft, shelves, hanging plants and watering can.
    for x in [3.65,6.25]:
        for y in [1.30,1.85]:
            cube('Greenhouse shelf',(x,y,4.95),(.7,.09,.38),'pale_wood')
            for dx in [-.20,.20]:
                cyl('Shelf seedling pot',(x+dx,y+.17,4.95),.10,.25,'terracotta',12)
                ball('Shelf seedling',(x+dx,y+.37,4.95),(.13,.16,.12),'sage',12)
    for x in [3.8,6.15]:
        for dx in [-.12,.12]:beam('Hanging basket cord',(x,2.6,3.9),(x+dx,1.9,3.9),.01,'cream')
        cyl('Hanging flower basket',(x,1.86,3.9),.22,.18,'terracotta',16)
        for j in range(5):
            a=j*math.tau/5
            curve('Trailing basket vine',[(x,1.9,3.9),(x+math.cos(a)*.2,1.7,3.9+math.sin(a)*.2),(x+math.cos(a)*.25,1.48,3.9+math.sin(a)*.25)],.022,'leaf')
    cyl('Garden watering can',(6.32,.34,5.),.23,.43,'mint',20)
    beam('Watering can spout',(6.45,.43,5.),(6.92,.71,5.),.05,'mint')
    curve('Watering can handle',[(6.15,.48,5.),(5.98,.75,5.),(6.33,.78,5.),(6.45,.49,5.)],.028,'cream')
    cube('Seed packets',(5.8,.88,4.7),(.25,.23,.06),'paper',.01)
    ball('Seed packet flower',(5.8,.90,4.66),(.065,.065,.012),'rose',12)
    # Flower beds have a deliberate color rhythm and stay outside the walking route.
    for x,z,length in [(-5.0,5.35,2.1),(-1.0,-5.05,1.6),(0.3,4.9,1.4),(6.9,2.4,.8)]:
        cube('Raised flower bed',(x,.16,z),(length,.22,.58),'pale_wood',.05)
        cube('Dark flowerbed soil',(x,.29,z),(length-.1,.03,.48),'wood',.02)
        for j in range(int(length/.2)):
            tulip(x-length*.43+j*.2,z+(.10 if j%2 else -.1),['rose','yellow','lavender'][j%3],.57+(j%3)*.08)
    # Additional soft grasses and stone edging create detail at ground level.
    for i in range(24):
        x=-7.1+(i%12)*1.15;z=5.75 if i<12 else -5.65
        for k in range(3):
            leaf=ball('Meadow grass blade',(x+k*.06,.19,z),(.025,.15,.034),'leaf',10);leaf.rotation_euler[1]=(k-1)*.25
    for x,z in [(-6.9,3.9),(-5.8,4.2),(1.1,4.2),(6.9,-3.),(.7,-3.6)]:
        ball('Rounded garden stone',(x,.18,z),(.19,.12,.15),'stone',12)
    # Garland with individually colored flags.
    curve('Courtyard celebration garland',[(-3.0,2.65,-1.9),(.4,2.27,-1.9),(5.9,2.6,-1.9)],.014,'cream')
    for i in range(16):
        x=-2.8+i*.56;y=2.28+.32*((x-.4)/5.0)**2
        tri('Pastel bunting',[(x-.12,y,-1.91),(x+.12,y,-1.91),(x,y-.28,-1.91)],['rose','cream','mint','gold'][i%4])

generators={'cat':cat,'detective':detective,'fairy':fairy,'neighborhood':world}
requested=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else list(generators)
for name in requested:generators[name]()
