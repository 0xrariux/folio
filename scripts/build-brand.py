"""Build Folio's folded-paper monogram, its Blender master and shared SVGs.

Run with Blender, from any directory:
  blender --background --factory-startup --threads 4 --python scripts/build-brand.py
No downloaded models, fonts or textures are required.
"""
import math
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "assets/brand"
DEST.mkdir(parents=True, exist_ok=True)

# SVG coordinates are the shared master: two diagonal paper cuts, a softened
# spine and a generous open counter. Blender samples these very same curves.
OUTLINE = [
    ("M", 6, 20),
    ("C", 5.448, 20, 5, 19.552, 5, 19),
    ("L", 5, 6.4),
    ("C", 5, 5.075, 6.075, 4, 7.4, 4),
    ("L", 19, 4),
    ("L", 15.6, 7.4),
    ("L", 9, 7.4),
    ("L", 9, 10.6),
    ("L", 16.8, 10.6),
    ("L", 13.4, 14),
    ("L", 9, 14),
    ("L", 9, 18),
    ("C", 9, 19.105, 8.105, 20, 7, 20),
    ("Z",),
]


def path_data():
    return " ".join(command[0] + " ".join(f"{n:g}" for n in command[1:]) for command in OUTLINE)


def sample_outline():
    points = []
    for command in OUTLINE:
        kind, *v = command
        if kind in ("M", "L"):
            points.append(tuple(v))
        elif kind == "C":
            a = Vector(points[-1])
            b, c, d = Vector(v[:2]), Vector(v[2:4]), Vector(v[4:])
            for step in range(1, 25):
                t = step / 24
                p = (1 - t) ** 3 * a + 3 * (1 - t) ** 2 * t * b + 3 * (1 - t) * t ** 2 * c + t ** 3 * d
                points.append(tuple(p))
    return points


def rounded_square(half, radius):
    points = []
    for cx, cy, start in [
        (half - radius, half - radius, 0),
        (-half + radius, half - radius, 90),
        (-half + radius, -half + radius, 180),
        (half - radius, -half + radius, 270),
    ]:
        for step in range(25):
            angle = math.radians(start + step * 90 / 24)
            points.append((cx + radius * math.cos(angle), cy + radius * math.sin(angle)))
    return points


def linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def material(name, rgb, roughness):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = tuple(linear(c / 255) for c in rgb) + (1,)
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = mat.diffuse_color
    shader.inputs["Roughness"].default_value = roughness
    return mat


def solid(name, points, bottom, top, mat, bevel):
    # Use a filled curve for robust triangulation of the concave F outline.
    curve = bpy.data.curves.new(name, "CURVE")
    curve.dimensions = "2D"
    curve.fill_mode = "BOTH"
    curve.resolution_u = 1
    curve.extrude = (top - bottom) / 2
    spline = curve.splines.new("POLY")
    spline.points.add(len(points) - 1)
    for point, (x, y) in zip(spline.points, points):
        point.co = (x, y, 0, 1)
    spline.use_cyclic_u = True
    obj = bpy.data.objects.new(name, curve)
    bpy.context.collection.objects.link(obj)
    obj.location.z = (top + bottom) / 2
    obj.data.materials.append(mat)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.convert(target="MESH")
    obj = bpy.context.object
    edge = obj.modifiers.new("Soft manufactured edges", "BEVEL")
    edge.width = bevel
    edge.segments = 4
    edge.limit_method = "ANGLE"
    obj.modifiers.new("Weighted face normals", "WEIGHTED_NORMAL")
    obj.select_set(False)
    return obj


def area(name, position, power, size):
    light = bpy.data.lights.new(name, "AREA")
    light.energy = power
    light.shape = "DISK"
    light.size = size
    obj = bpy.data.objects.new(name, light)
    bpy.context.collection.objects.link(obj)
    obj.location = position
    obj.rotation_euler = (-obj.location).to_track_quat("-Z", "Y").to_euler()


def camera_at(position, scale):
    camera.location = position
    camera.rotation_euler = (Vector((0, 0, 0.1)) - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.ortho_scale = scale


def render(filename, position, scale, transparent):
    camera_at(position, scale)
    scene.render.film_transparent = transparent
    scene.render.filepath = str(DEST / filename)
    bpy.ops.render.render(write_still=True)


# The native monogram uses only a solid currentColor path, so it stays crisp
# and inherits the theme in Rune. Full-colour exports are available separately.
path = path_data()
(ROOT / "assets/icons/folio.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">\n'
    f'  <path fill="currentColor" d="{path}"/>\n</svg>\n'
)
(DEST / "folio-logo.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">\n'
    '  <title>Folio — folded paper</title>\n'
    '  <rect width="64" height="64" rx="18" fill="#99bd92"/>\n'
    f'  <path fill="#1d1f1e" transform="translate(4 4) scale(2.333333)" d="{path}"/>\n'
    '</svg>\n'
)

bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.context.preferences.filepaths.save_version = 0
sage = material("Folio · sage", (153, 189, 146), 0.44)
ink = material("Folio · graphite", (29, 31, 30), 0.5)
tile = solid("Folio / soft square", rounded_square(2.4, 1.35), -0.12, 0.12, sage, 0.055)
glyph = solid("Folio / folded F", [((x - 12) * 0.175, (12 - y) * 0.175) for x, y in sample_outline()], 0.122, 0.18, ink, 0.012)
glyph["design"] = "Two diagonal paper cuts; no font or external texture."

scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 48
scene.cycles.use_denoising = True
scene.render.resolution_x = 1024
scene.render.resolution_y = 1024
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = "PNG"
scene.render.image_settings.color_mode = "RGBA"
scene.view_settings.view_transform = "Standard"
scene.world.use_nodes = True
scene.world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.08, 0.09, 0.08, 1)
scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.45
area("Key / large softbox", (-3, 4, 8), 650, 6)
area("Fill / softbox", (4, -1, 6), 240, 5)

cam_data = bpy.data.cameras.new("Folio / orthographic")
camera = bpy.data.objects.new("Folio / orthographic", cam_data)
bpy.context.collection.objects.link(camera)
camera.data.type = "ORTHO"
scene.camera = camera
render("folio-logo.png", (0, 0, 12), 5.8, True)

floor_mat = material("Studio / graphite", (24, 26, 25), 0.85)
bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -0.19))
bpy.context.object.name = "Studio / ground"
bpy.context.object.data.materials.append(floor_mat)
render("folio-logo-studio.png", (0, -4, 20), 6.2, False)
bpy.context.view_layer.objects.active = glyph
bpy.ops.object.select_all(action="DESELECT")
glyph.select_set(True)
# Blender saves fixed-size path buffers, including bytes beyond the terminating
# NUL. Overwrite their old contents before setting portable, relative paths.
for screen in bpy.data.screens:
    for screen_area in screen.areas:
        for space in screen_area.spaces:
            if space.type == "FILE_BROWSER" and space.params:
                space.params.directory = b"_" * 1023
                space.params.directory = b"//"
scene.render.filepath = "_" * 1023
scene.render.filepath = "//folio-logo-studio.png"
bpy.ops.wm.save_as_mainfile(filepath=str(DEST / "folio-logo.blend"))
print("Folio logo: Blender master, SVGs and two PNG renders exported.")
