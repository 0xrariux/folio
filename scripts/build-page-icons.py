"""Build original Folio page icons as SVGs and an editable Blender contact sheet.

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


def ellipse(x, y, rx, ry, angle=0):
    radians = math.radians(angle)
    c, s = math.cos(radians), math.sin(radians)
    result = []
    for command in circle(0, 0, 1):
        coordinates = []
        for px, py in zip(command[1::2], command[2::2]):
            coordinates.extend((x + px * rx * c - py * ry * s,
                                y + px * rx * s + py * ry * c))
        result.append((command[0], *coordinates))
    return result


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
    # Finance and business.
    ("coins", "Coins", [ellipse(9, 6, 6, 3),
                         [("M", 3, 6), ("L", 3, 11), ("C", 3, 15, 15, 15, 15, 11), ("L", 15, 6)],
                         [("M", 3, 10), ("C", 3, 14, 15, 14, 15, 10)],
                         [("M", 15, 10), ("C", 23, 9, 23, 16, 15, 16)],
                         [("M", 9, 14), ("L", 9, 18), ("C", 9, 22, 21, 22, 21, 18), ("L", 21, 13)]]),
    ("bank", "Banking", [poly((3, 8), (12, 3), (21, 8), closed=True),
                          *[poly((x, 11), (x, 18)) for x in [6, 12, 18]],
                          poly((4, 21), (20, 21)), poly((3, 18), (21, 18))]),
    ("credit-card", "Payments", [rounded_rect(2.5, 5, 19, 14, 2), poly((2.5, 10), (21.5, 10)),
                                 poly((6, 15), (10, 15))]),
    ("receipt", "Expenses", [poly((5, 3), (19, 3), (19, 21), (15.5, 19), (12, 21), (8.5, 19), (5, 21), closed=True),
                             poly((9, 7), (15, 7)), poly((9, 11), (15, 11)), poly((9, 15), (12, 15))]),
    ("calculator", "Accounting", [rounded_rect(5, 2.5, 14, 19, 2), rounded_rect(8, 6, 8, 4, .5),
                                  *[poly((x, y), (x + 1, y)) for x in [8, 11.5, 15] for y in [14, 18]]]),
    # Health and wellbeing.
    ("medical-kit", "Healthcare", [rounded_rect(3, 7, 18, 14, 2),
                                   poly((8, 7), (8, 3), (16, 3), (16, 7)),
                                   poly((12, 11), (12, 17)), poly((9, 14), (15, 14))]),
    ("stethoscope", "Medicine", [[("M", 4, 3), ("L", 4, 9), ("C", 4, 16, 14, 16, 14, 9), ("L", 14, 3)],
                                  poly((3, 3), (5, 3)), poly((13, 3), (15, 3)),
                                  [("M", 9, 14), ("L", 9, 17), ("C", 9, 23, 20, 23, 20, 17), ("L", 20, 13)],
                                  circle(20, 10, 2.5)]),
    ("pill", "Medication", [[("M", 4, 12), ("L", 12, 4), ("C", 17, -1, 25, 7, 20, 12),
                             ("L", 12, 20), ("C", 7, 25, -1, 17, 4, 12), ("Z",)],
                            poly((8, 8), (16, 16))]),
    ("activity", "Fitness", [poly((2, 12), (6, 12), (9, 4), (14, 20), (17, 12), (22, 12))]),
    ("brain", "Mental wellbeing", [[("M", 12, 5), ("C", 10, 1, 5, 2, 5, 6),
                                    ("C", 0, 7, 2, 12, 4, 13), ("C", 0, 17, 5, 22, 8, 20),
                                    ("C", 10, 23, 12, 20, 12, 18), ("Z",)],
                                   [("M", 12, 5), ("C", 14, 1, 19, 2, 19, 6),
                                    ("C", 24, 7, 22, 12, 20, 13), ("C", 24, 17, 19, 22, 16, 20),
                                    ("C", 14, 23, 12, 20, 12, 18)],
                                   [("M", 5, 6), ("C", 5, 9, 7, 10, 8, 10)],
                                   [("M", 19, 6), ("C", 19, 9, 17, 10, 16, 10)],
                                   poly((7, 15), (9, 17)), poly((17, 15), (15, 17))]),
    # Research and sciences.
    ("microscope", "Research", [poly((9, 3), (13, 5), (8, 15), (4, 13), closed=True),
                                poly((10, 2), (14, 4)), poly((5, 17), (12, 17)),
                                [("M", 13, 9), ("C", 22, 9, 22, 21, 13, 21)], poly((4, 21), (20, 21))]),
    ("atom", "Physics", [ellipse(12, 12, 10, 4), ellipse(12, 12, 10, 4, 60),
                          ellipse(12, 12, 10, 4, 120), circle(12, 12, 1)]),
    ("dna", "Biology", [[("M", 7, 2), ("C", 7, 12, 17, 12, 17, 22)],
                         [("M", 17, 2), ("C", 17, 12, 7, 12, 7, 22)],
                         *[poly((x, y), (24 - x, y)) for x, y in [(7.5, 5), (9, 8), (9, 16), (7.5, 19)]]]),
    ("telescope", "Astronomy", [poly((3, 10), (17, 3), (20, 9), (6, 16), closed=True),
                                poly((17, 3), (18, 2.5)), poly((20, 9), (21, 8.5)),
                                poly((11, 13.5), (11, 17), (7, 22)), poly((11, 17), (16, 22))]),
    ("magnet", "Engineering", [[("M", 4, 4), ("L", 4, 13), ("C", 4, 24, 20, 24, 20, 13),
                                ("L", 20, 4), ("L", 15, 4), ("L", 15, 13),
                                ("C", 15, 17, 9, 17, 9, 13), ("L", 9, 4), ("Z",)],
                               poly((4, 8), (9, 8)), poly((15, 8), (20, 8))]),
    # Arts and visual creation.
    ("theatre", "Performing arts", [[("M", 3, 3), ("C", 8, 5, 12, 5, 17, 3), ("L", 17, 9),
                                     ("C", 17, 18, 3, 18, 3, 9), ("Z",)],
                                    poly((6, 8), (7, 8)), poly((12, 8), (13, 8)),
                                    [("M", 7, 11), ("C", 8, 13, 11, 13, 12, 11)],
                                    [("M", 18, 8), ("L", 21, 7), ("L", 21, 14), ("C", 21, 21, 14, 23, 10, 18)]]),
    ("picture", "Visual arts", [rounded_rect(3, 3, 18, 18, 2), circle(8, 8, 2),
                                poly((3, 17), (9, 12), (13, 16), (17, 12), (21, 16))]),
    ("easel", "Painting", [rounded_rect(4, 5, 16, 12, 1), poly((12, 2), (12, 5)),
                           poly((3, 17), (21, 17)), poly((8, 17), (5, 22)), poly((16, 17), (19, 22)),
                           poly((8, 13), (11, 9), (16, 13))]),
    ("sculpture", "Sculpture", [circle(12, 6, 3.5),
                                [("M", 9, 9), ("L", 9, 12), ("C", 5, 13, 4, 15, 4, 18),
                                 ("L", 20, 18), ("C", 20, 15, 19, 13, 15, 12), ("L", 15, 9)],
                                poly((6, 18), (6, 21), (18, 21), (18, 18))]),
    ("swatches", "Color design", [rounded_rect(3, 3, 6, 18, 1.5), poly((3, 9), (9, 9)), poly((3, 15), (9, 15)),
                                  poly((9, 7), (12, 4), (17, 9), (9, 17)),
                                  poly((17, 11), (21, 11), (21, 21), (6, 21)), poly((15, 15), (15, 21))]),
    # Music and audio.
    ("guitar", "Guitar", [[("M", 10, 9), ("C", 6, 6, 3, 9, 5, 12),
                           ("C", -1, 16, 8, 25, 12, 19), ("C", 15, 21, 18, 18, 15, 14),
                           ("L", 20, 6), ("L", 18, 4), ("Z",)],
                          poly((18, 4), (20, 2), (22, 4), (20, 6)), circle(10, 14, 2), poly((6, 17), (8, 19))]),
    ("piano", "Piano", [rounded_rect(3, 4, 18, 16, 1),
                         *[poly((x, 4), (x, 20)) for x in [9, 15]],
                         *[rounded_rect(x, 4, 2, 9, .4) for x in [8, 14]]]),
    ("drum", "Percussion", [ellipse(12, 10, 9, 3),
                            [("M", 3, 10), ("L", 3, 18), ("C", 3, 22, 21, 22, 21, 18), ("L", 21, 10)],
                            poly((4, 12), (8, 19), (12, 13), (16, 19), (20, 12)),
                            poly((5, 3), (12, 7)), poly((19, 2), (14, 7))]),
    ("speaker", "Audio", [rounded_rect(5, 2.5, 14, 19, 2), circle(12, 14.5, 4), circle(12, 6.5, 1.3)]),
    ("waveform", "Sound production", [*[poly((x, 12 - h), (x, 12 + h))
                                         for x, h in [(3, 2), (7.5, 6), (12, 9), (16.5, 4), (21, 7)]]]),
    # Games and play.
    ("dice", "Board games", [rounded_rect(3, 3, 18, 18, 3),
                             *[circle(x, y, .8) for x, y in [(7, 7), (17, 7), (12, 12), (7, 17), (17, 17)]]]),
    ("chess-knight", "Chess", [poly((6, 18), (6, 15), (11, 10), (7, 12), (3, 9), (8, 4), (8, 2.5),
                                   (13, 4), (17, 7), (19, 18)),
                               poly((5, 18), (20, 18), (20, 21), (5, 21), closed=True), circle(10, 7, .65)]),
    ("cards", "Card games", [rounded_rect(7, 3, 14, 18, 2), poly((7, 4), (3, 5), (3, 19), (7, 19)),
                             poly((14, 8), (17, 12), (14, 16), (11, 12), closed=True)]),
    ("joystick", "Arcade", [poly((3, 15), (6, 12), (9, 12)), poly((15, 12), (18, 12), (21, 15)),
                            rounded_rect(3, 15, 18, 6, 1.5), circle(12, 5.5, 3), poly((12, 8.5), (12, 15)),
                            poly((17, 18), (18, 18))]),
    ("ghost", "Video games", [[("M", 4, 11), ("C", 4, 0, 20, 0, 20, 11),
                               ("L", 20, 21), ("L", 16, 18), ("L", 12, 21), ("L", 8, 18), ("L", 4, 21), ("Z",)],
                              circle(9, 10, 1), circle(15, 10, 1)]),
    # Home and everyday life.
    ("lamp", "Lighting", [poly((8, 3), (16, 3), (20, 13), (4, 13), closed=True),
                           poly((12, 13), (12, 21)), poly((8, 21), (16, 21)), poly((17, 13), (17, 17))]),
    ("sofa", "Living room", [[("M", 5, 11), ("L", 5, 7), ("C", 5, 3, 19, 3, 19, 7), ("L", 19, 11)],
                             [("M", 6, 15), ("L", 6, 12), ("C", 6, 8, 2, 8, 2, 12), ("L", 2, 19),
                              ("L", 22, 19), ("L", 22, 12), ("C", 22, 8, 18, 8, 18, 12), ("L", 18, 15), ("Z",)],
                             poly((5, 19), (5, 21)), poly((19, 19), (19, 21))]),
    ("bed", "Bedroom", [poly((3, 21), (3, 5), (21, 5), (21, 21)), rounded_rect(3, 12, 18, 6, 1),
                         rounded_rect(6, 8, 5, 4, .8), rounded_rect(13, 8, 5, 4, .8)]),
    ("cooking-pot", "Cooking", [[("M", 5, 10), ("L", 5, 17), ("C", 5, 23, 19, 23, 19, 17), ("L", 19, 10), ("Z",)],
                                poly((2, 13), (5, 13)), poly((19, 13), (22, 13)), poly((4, 10), (20, 10)),
                                [("M", 8, 6), ("C", 11, 4, 6, 4, 9, 2)],
                                [("M", 14, 6), ("C", 17, 4, 12, 4, 15, 2)]]),
    ("bath", "Bathroom", [[("M", 3, 12), ("L", 3, 15), ("C", 3, 22, 21, 22, 21, 15), ("L", 21, 12)],
                           poly((2, 12), (22, 12)), poly((6, 19), (5, 22)), poly((18, 19), (19, 22)),
                           [("M", 5, 12), ("L", 5, 5), ("C", 5, 1, 10, 1, 10, 5)], poly((8, 6), (12, 6))]),
    # Industry, services and logistics.
    ("hard-hat", "Construction", [[("M", 4, 16), ("L", 4, 12), ("C", 4, 2, 20, 2, 20, 12), ("L", 20, 16)],
                                   rounded_rect(2, 16, 20, 4, 1), poly((9, 10), (9, 3), (15, 3), (15, 10))]),
    ("wrench", "Maintenance", [[("M", 14, 3), ("C", 20, 0, 24, 7, 20, 11),
                                ("C", 18, 13, 16, 13, 14, 12), ("L", 6, 21),
                                ("C", 3, 24, 0, 20, 3, 18), ("L", 12, 10),
                                ("C", 10, 8, 10, 5, 11, 4), ("L", 14, 7), ("L", 17, 6), ("Z",)]]),
    ("building", "Real estate", [rounded_rect(5, 3, 14, 18, 1), poly((10, 21), (10, 16), (14, 16), (14, 21)),
                                  *[poly((x, y), (x + 1, y)) for x in [9, 14] for y in [7, 11]]]),
    ("scales", "Legal", [poly((12, 3), (12, 21)), poly((7, 21), (17, 21)), poly((4, 6), (20, 6)),
                          poly((5, 6), (2, 14), (8, 14), closed=True),
                          poly((19, 6), (16, 14), (22, 14), closed=True),
                          [("M", 2, 14), ("C", 3, 18, 7, 18, 8, 14)],
                          [("M", 16, 14), ("C", 17, 18, 21, 18, 22, 14)]]),
    ("package", "Logistics", [poly((3, 7), (12, 3), (21, 7), (21, 17), (12, 21), (3, 17), closed=True),
                              poly((3, 7), (12, 11), (21, 7)), poly((12, 11), (12, 21)),
                              poly((7.5, 5), (16.5, 9), (16.5, 13))]),
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

    columns = 8
    rows = math.ceil(len(ICONS) / columns)
    width, height = columns * 44 + 16, rows * 44 + 64
    label('Folio / More ways to make a page yours', columns * 22, 32, 6, ink)
    label(f'{len(ICONS)} original symbols / 24-point grid / rounded strokes', columns * 22, 23, 3.2, muted)
    for index, (key, name, paths) in enumerate(ICONS):
        (DEST / f'{key}.svg').write_text(svg(paths))
        collection = bpy.data.collections.new(f'{index + 36:02} / {name}')
        bpy.context.scene.collection.children.link(collection)
        origin = ((index % columns) * 44 + 10, -(index // columns) * 44)
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
    scene.render.resolution_x = width * 5; scene.render.resolution_y = height * 5
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'; scene.render.image_settings.color_mode = 'RGBA'
    scene.view_settings.view_transform = 'Standard'
    scene.world.use_nodes = True
    scene.world.node_tree.nodes['Background'].inputs['Color'].default_value = tuple(linear(c) for c in (29, 31, 30)) + (1,)
    scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 1
    camera = bpy.data.objects.new('Folio / orthographic', bpy.data.cameras.new('Folio / orthographic'))
    bpy.context.collection.objects.link(camera); camera.location = (columns * 22, -(rows - 1) * 22 - 1, 400)
    camera.data.type = 'ORTHO'; camera.data.sensor_fit = 'HORIZONTAL'; camera.data.ortho_scale = width
    scene.camera = camera
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
    print(f'Generated {len(ICONS)} SVG icons, the Blender master and its preview.')


if __name__ == '__main__':
    build()
