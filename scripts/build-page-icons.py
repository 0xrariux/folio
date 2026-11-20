"""Build 30 original Folio page icons as SVGs and an editable Blender contact sheet.

Run with Blender: blender --background --factory-startup --threads 4
    --python scripts/build-page-icons.py
The same 24-point stroke paths generate the native SVGs and Blender curves.
No external fonts, textures or models are required.
"""
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "assets/icons"


def poly(*points, closed=False):
    commands = [("M", *points[0]), *[("L", *point) for point in points[1:]]]
    return commands + ([("Z",)] if closed else [])


def circle(x, y, radius):
    k = radius * 0.5522847498
    return [("M", x + radius, y),
            ("C", x + radius, y + k, x + k, y + radius, x, y + radius),
            ("C", x - k, y + radius, x - radius, y + k, x - radius, y),
            ("C", x - radius, y - k, x - k, y - radius, x, y - radius),
            ("C", x + k, y - radius, x + radius, y - k, x + radius, y), ("Z",)]


def rounded_rect(x, y, width, height, radius=1.5):
    k = radius * 0.5522847498
    right, bottom = x + width, y + height
    return [("M", x + radius, y), ("L", right - radius, y),
            ("C", right - radius + k, y, right, y + radius - k, right, y + radius),
            ("L", right, bottom - radius),
            ("C", right, bottom - radius + k, right - radius + k, bottom, right - radius, bottom),
            ("L", x + radius, bottom),
            ("C", x + radius - k, bottom, x, bottom - radius + k, x, bottom - radius),
            ("L", x, y + radius),
            ("C", x, y + radius - k, x + radius - k, y, x + radius, y), ("Z",)]


ICONS = [
    ("bolt", "Lightning", [poly((13.5, 2.5), (5, 13), (11, 13), (10.5, 21.5), (19, 11), (13, 11), closed=True)]),
    ("cloud", "Cloud", [[("M", 6.5, 18), ("C", 1, 18, 1, 10, 6.4, 9.5),
                          ("C", 7, 3, 16.4, 3, 17.5, 10), ("C", 23.5, 9.5, 23, 18, 18, 18), ("Z",)]]),
    ("umbrella", "Umbrella", [[("M", 3, 11), ("C", 4, 1, 20, 1, 21, 11), ("Z",)],
                              [("M", 12, 11), ("L", 12, 18), ("C", 12, 22, 17, 22, 17, 18)], poly((12, 2), (12, 3))]),
    ("flame", "Flame", [[("M", 12, 2.5), ("C", 14, 7, 11, 9, 15, 11),
                          ("C", 17, 10, 18, 8, 18, 8), ("C", 24, 18, 16, 23, 10, 21),
                          ("C", 3, 20, 2, 13, 7, 8), ("C", 7, 12, 9, 12, 9, 10),
                          ("C", 9, 7, 12, 6, 12, 2.5), ("Z",)],
                         [("M", 12, 14), ("C", 8, 18, 11, 21, 13, 20), ("C", 16, 19, 14, 16, 12, 14)]]),
    ("droplet", "Water", [[("M", 12, 2.5), ("C", 9, 7, 5, 10, 5, 14),
                            ("C", 5, 23, 19, 23, 19, 14), ("C", 19, 10, 15, 7, 12, 2.5), ("Z",)],
                           [("M", 8, 15), ("C", 8, 17, 9, 18, 11, 18)]]),
    ("moon-stars", "Moon and stars", [[("M", 14, 3), ("C", 4, 0, 0, 13, 7, 19),
                                        ("C", 12, 23, 19, 20, 20, 15), ("C", 10, 18, 6, 9, 14, 3), ("Z",)],
                                       poly((19, 2.5), (19, 7.5)), poly((16.5, 5), (21.5, 5)),
                                       poly((21, 10), (21, 12)), poly((20, 11), (22, 11))]),
    ("headphones", "Headphones", [[("M", 4, 14), ("L", 4, 10), ("C", 4, 0, 20, 0, 20, 10), ("L", 20, 14)],
                                   rounded_rect(3, 12, 4, 8), rounded_rect(17, 12, 4, 8)]),
    ("microphone", "Microphone", [rounded_rect(9, 2.5, 6, 12, 3),
                                  [("M", 5.5, 11), ("L", 5.5, 12), ("C", 5.5, 21, 18.5, 21, 18.5, 12), ("L", 18.5, 11)],
                                  poly((12, 18.5), (12, 22)), poly((8.5, 22), (15.5, 22))]),
    ("film", "Film", [rounded_rect(3, 3, 18, 18), poly((7, 3), (7, 21)), poly((17, 3), (17, 21)),
                       *[poly((x, y), (x + 4, y)) for x in [3, 17] for y in [7.5, 12, 16.5]]]),
    ("gamepad", "Game controller", [[("M", 7, 7), ("L", 17, 7), ("C", 21, 7, 23, 19, 19, 20),
                                     ("C", 17, 20, 16, 16, 14, 16), ("L", 10, 16),
                                     ("C", 8, 16, 7, 20, 5, 20), ("C", 1, 19, 3, 7, 7, 7), ("Z",)],
                                    poly((8, 10), (8, 14)), poly((6, 12), (10, 12)),
                                    circle(16, 11, .65), circle(18, 13, .65)]),
    ("brush", "Paintbrush", [[("M", 10, 13), ("L", 18, 3), ("C", 20, 1, 23, 4, 21, 6),
                               ("L", 13, 16), ("Z",)],
                              [("M", 12, 15), ("C", 4, 11, 7, 20, 3, 21),
                               ("C", 10, 23, 16, 21, 12, 15), ("Z",)]]),
    ("pen-tool", "Design pen", [poly((8, 21), (3, 10), (12, 3), (21, 10), (16, 21)),
                                 poly((12, 3), (12, 11)), circle(12, 13, 2), poly((7, 21), (17, 21))]),
    ("scissors", "Scissors", [circle(6, 6, 3), circle(6, 18, 3), poly((8.2, 8.2), (20, 20)),
                               poly((8.2, 15.8), (20, 4))]),
    ("ruler", "Ruler", [poly((3, 16), (16, 3), (21, 8), (8, 21), closed=True),
                         poly((7, 12), (9, 14)), poly((10, 9), (12, 11)), poly((13, 6), (15, 8))]),
    ("pencil", "Pencil", [poly((4, 15), (15, 4), (20, 9), (9, 20), (3, 21), closed=True),
                           poly((13, 6), (18, 11)), poly((4, 15), (9, 20)),
                           [("M", 15, 4), ("L", 16.5, 2.5), ("C", 18, 1, 23, 6, 21.5, 7.5), ("L", 20, 9)]]),
    ("map", "Map", [poly((3, 5), (9, 3), (15, 5), (21, 3), (21, 19), (15, 21), (9, 19), (3, 21), closed=True),
                     poly((9, 3), (9, 19)), poly((15, 5), (15, 21))]),
    ("plane", "Travel", [poly((21, 3), (14, 21), (10.5, 13.5), (3, 10), closed=True), poly((10.5, 13.5), (21, 3))]),
    ("car", "Car", [rounded_rect(3, 10, 18, 9, 2), poly((5, 10), (7, 4), (17, 4), (19, 10)),
                     poly((5, 19), (5, 21)), poly((19, 19), (19, 21)), poly((6, 14), (8, 14)), poly((16, 14), (18, 14))]),
    ("bicycle", "Cycling", [circle(5.5, 16.5, 4), circle(18.5, 16.5, 4),
                             poly((5.5, 16.5), (9, 9), (15, 9), (11, 16.5), (5.5, 16.5)),
                             poly((11, 16.5), (8, 6)), poly((6, 6), (10, 6)), poly((18.5, 16.5), (14, 4), (18, 4))]),
    ("ship", "Sailing", [poly((12, 2.5), (12, 16), (3, 16), closed=True),
                          poly((15, 6), (21, 14), (15, 14), closed=True),
                          [("M", 2.5, 18), ("C", 4, 22, 20, 22, 21.5, 18), ("Z",)]]),
    ("tree", "Tree", [poly((12, 2.5), (6, 9), (9, 9), (4, 15), (7, 15), (3, 19), (21, 19),
                           (17, 15), (20, 15), (15, 9), (18, 9), closed=True), poly((12, 19), (12, 22))]),
    ("flower", "Flower", [circle(12, 11, 2),
                           [("M", 12, 7), ("C", 6, 0, 3, 9, 8, 10), ("C", 0, 13, 7, 20, 10, 15),
                            ("C", 12, 23, 20, 18, 16, 14), ("C", 24, 13, 21, 5, 16, 8),
                            ("C", 18, 1, 10, 0, 12, 7), ("Z",)]]),
    ("paw", "Pets", [[("M", 12, 11), ("C", 8, 11, 8, 14, 5, 16), ("C", 2, 20, 7, 22, 12, 19),
                       ("C", 17, 22, 22, 20, 19, 16), ("C", 16, 14, 16, 11, 12, 11), ("Z",)],
                      circle(4.5, 9, 2), circle(9, 5, 2), circle(15, 5, 2), circle(19.5, 9, 2)]),
    ("fish", "Fish", [[("M", 17, 9), ("C", 11, 2, 5, 7, 2, 12), ("C", 5, 17, 11, 22, 17, 15),
                        ("L", 22, 19), ("L", 22, 5), ("Z",)], circle(7, 11, .65),
                       [("M", 12, 7), ("C", 14, 10, 14, 14, 12, 17)]]),
    ("fruit", "Food", [[("M", 12, 8), ("C", 3, 3, 1, 11, 5, 18), ("C", 8, 24, 10, 20, 12, 20),
                         ("C", 14, 20, 16, 24, 19, 18), ("C", 23, 11, 21, 3, 12, 8), ("Z",)],
                        [("M", 12, 8), ("C", 11, 4, 15, 2, 18, 3), ("C", 17, 6, 14, 7, 12, 6)]]),
    ("cake", "Celebration", [rounded_rect(3, 11, 18, 10), poly((7, 11), (7, 7)), poly((12, 11), (12, 7)),
                              poly((17, 11), (17, 7)),
                              [("M", 3, 15), ("C", 6, 12, 8, 18, 11, 15), ("C", 14, 12, 17, 18, 21, 15)],
                              *[poly((x, 4.5), (x, 3)) for x in [7, 12, 17]]]),
    ("gift", "Gift", [rounded_rect(3, 8, 18, 4, 1), poly((5, 12), (5, 21), (19, 21), (19, 12)),
                       poly((12, 8), (12, 21)),
                       [("M", 12, 8), ("C", 2, 8, 5, 0, 9, 3), ("C", 11, 4, 12, 8, 12, 8)],
                       [("M", 12, 8), ("C", 22, 8, 19, 0, 15, 3), ("C", 13, 4, 12, 8, 12, 8)]]),
    ("trophy", "Achievement", [[("M", 7, 3), ("L", 17, 3), ("L", 17, 10), ("C", 17, 18, 7, 18, 7, 10), ("Z",)],
                                [("M", 7, 5), ("L", 3, 5), ("L", 3, 8), ("C", 3, 12, 7, 12, 7, 12)],
                                [("M", 17, 5), ("L", 21, 5), ("L", 21, 8), ("C", 21, 12, 17, 12, 17, 12)],
                                poly((12, 16), (12, 21)), poly((8, 21), (16, 21))]),
    ("shield", "Protection", [[("M", 12, 2.5), ("L", 20, 6), ("L", 20, 12),
                               ("C", 20, 17, 15, 20, 12, 22), ("C", 9, 20, 4, 17, 4, 12),
                               ("L", 4, 6), ("Z",)], poly((8, 12), (11, 15), (16, 9))]),
    ("key", "Key", [circle(8, 8, 5), poly((11.5, 11.5), (21, 21)),
                     poly((17, 17), (20, 14)), poly((20, 20), (22, 18))]),
]


def svg(paths):
    data = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">',
            '  <g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">']
    for commands in paths:
        d = ' '.join(command[0] + ' '.join(f'{n:g}' for n in command[1:]) for command in commands)
        data.append(f'    <path d="{d}"/>')
    return '\n'.join(data + ['  </g>', '</svg>', ''])


def samples(commands):
    points = []
    for command in commands:
        kind, *values = command
        if kind == 'M':
            points.append(tuple(values))
        elif kind in ['L', 'Z']:
            a = points[-1]
            b = tuple(values) if kind == 'L' else points[0]
            steps = max(1, math.ceil(math.dist(a, b) * 2))
            points.extend(tuple(a[i] + (b[i] - a[i]) * step / steps for i in range(2))
                          for step in range(1, steps + 1))
        elif kind == 'C':
            a = points[-1]
            b, c, d = values[:2], values[2:4], values[4:]
            for step in range(1, 25):
                t = step / 24
                points.append(tuple((1-t)**3*a[i] + 3*(1-t)**2*t*b[i] + 3*(1-t)*t*t*c[i] + t**3*d[i] for i in range(2)))
    while len(points) > 1 and points[0] == points[-1]:
        points.pop()
    return points, commands[-1][0] == 'Z'


def build():
    import bpy
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    bpy.context.preferences.filepaths.save_version = 0
    DEST.mkdir(exist_ok=True)

    def linear(c):
        c /= 255
        return c / 12.92 if c <= .04045 else ((c + .055) / 1.055) ** 2.4

    def material(name, rgb):
        m = bpy.data.materials.new(name); m.use_nodes = True
        m.diffuse_color = tuple(linear(c) for c in rgb) + (1,)
        m.node_tree.nodes.clear()
        emission = m.node_tree.nodes.new('ShaderNodeEmission')
        emission.inputs['Color'].default_value = m.diffuse_color
        out = m.node_tree.nodes.new('ShaderNodeOutputMaterial')
        m.node_tree.links.new(emission.outputs[0], out.inputs['Surface'])
        return m

    sage = material('Folio / sage', (153, 189, 146))
    muted = material('Folio / muted', (163, 170, 160))
    ink = material('Folio / heading', (232, 234, 228))

    def label(text, x, y, size, mat):
        data = bpy.data.curves.new(text, 'FONT'); data.body = text; data.size = size; data.align_x = 'CENTER'
        obj = bpy.data.objects.new(text, data); bpy.context.collection.objects.link(obj)
        obj.location = (x, y, 1); data.materials.append(mat)

    label('Folio / More ways to make a page yours', 132, 32, 6, ink)
    label('30 new symbols / 24-point grid / rounded strokes', 132, 23, 3.2, muted)
    for index, (key, name, paths) in enumerate(ICONS):
        (DEST / f'{key}.svg').write_text(svg(paths))
        collection = bpy.data.collections.new(f'{index + 36:02} / {name}')
        bpy.context.scene.collection.children.link(collection)
        origin = ((index % 6) * 44 + 10, -(index // 6) * 44)
        for n, path in enumerate(paths):
            points, closed = samples(path)
            data = bpy.data.curves.new(f'{key} / stroke {n + 1}', 'CURVE')
            data.dimensions = '3D'; data.resolution_u = 1; data.bevel_depth = .75
            data.twist_mode = 'Z_UP'
            data.bevel_resolution = 4; data.use_fill_caps = True
            spline = data.splines.new('POLY'); spline.points.add(len(points) - 1)
            for point, (x, y) in zip(spline.points, points):
                point.co = (origin[0] + x, origin[1] - y, 0, 1)
            spline.use_cyclic_u = closed
            obj = bpy.data.objects.new(f'{key} / stroke {n + 1}', data); collection.objects.link(obj)
            data.materials.append(sage)
            if not closed:
                for x, y in [points[0], points[-1]]:
                    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=.75,
                                                       location=(origin[0] + x, origin[1] - y, 0))
                    cap = bpy.context.object; cap.name = f'{key} / round cap'; cap.data.materials.append(sage)
                    for owner in list(cap.users_collection): owner.objects.unlink(cap)
                    collection.objects.link(cap)
        label(name, origin[0] + 12, origin[1] - 33, 3.2, muted)

    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'; scene.cycles.device = 'CPU'; scene.cycles.samples = 32
    scene.cycles.use_denoising = True
    scene.render.resolution_x = 1584; scene.render.resolution_y = 1500; scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'; scene.render.image_settings.color_mode = 'RGBA'
    scene.view_settings.view_transform = 'Standard'
    scene.world.use_nodes = True
    scene.world.node_tree.nodes['Background'].inputs['Color'].default_value = tuple(linear(c) for c in (29, 31, 30)) + (1,)
    scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 1
    camera = bpy.data.objects.new('Folio / orthographic', bpy.data.cameras.new('Folio / orthographic'))
    bpy.context.collection.objects.link(camera); camera.location = (132, -89, 400)
    camera.data.type = 'ORTHO'; camera.data.ortho_scale = 276; scene.camera = camera
    scene.render.filepath = str(DEST / 'page-icons-blender.png')
    bpy.ops.render.render(write_still=True)
    # Clear full fixed-size buffers, including bytes after the NUL, before saving.
    for screen in bpy.data.screens:
        for area in screen.areas:
            for space in area.spaces:
                if space.type == 'FILE_BROWSER' and space.params:
                    space.params.directory = b'_' * 1023; space.params.directory = b'//'
    scene.render.filepath = '_' * 1023; scene.render.filepath = '//page-icons-blender.png'
    bpy.ops.wm.save_as_mainfile(filepath=str(DEST / 'page-icons.blend'))
    print('Generated 30 SVG icons, the Blender master and its preview.')


if __name__ == '__main__':
    build()
