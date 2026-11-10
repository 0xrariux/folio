"""Rebuild standalone SVG sheets from the shipped icons and Rust catalogs."""
from html import escape
from math import ceil
from pathlib import Path
import re
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]


def catalog(source, entry):
    return re.findall(
        rf'{entry}\s*\{{\s*key:\s*"([^"]+)",\s*name:\s*"([^"]+)"',
        (root / source).read_text(),
    )


def sheet(choices, title, destination):
    height = max(760, 170 + ceil(len(choices) / 4) * 144)
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1040 {height}">',
        f'<title>Folio — {escape(title)}</title>',
        f'<rect width="1040" height="{height}" fill="#fdfcf9"/>',
        f'<rect x="520" width="520" height="{height}" fill="#1d1f1e"/>',
    ]
    for panel, (ink, muted, line, accent) in enumerate([
        ("#2b312a", "#697265", "#e2e5db", "#41653e"),
        ("#e8eae4", "#a3aaa0", "#353a35", "#99bd92"),
    ]):
        left = 32 + panel * 520
        parts += [
            f'<g font-family="Inter, system-ui, sans-serif" fill="{ink}">',
            f'<text x="{left}" y="54" font-size="26" font-weight="500">Folio / {escape(title)}</text>',
            f'<text x="{left}" y="84" font-size="12" fill="{muted}">{len(choices)} formes · une même écriture</text>',
            f'<path d="M{left} 108h456" stroke="{line}"/>',
        ]
        for index, (key, name) in enumerate(choices):
            path = root / f"assets/icons/{key}.svg"
            ET.parse(path)
            svg = path.read_text().split(">", 1)[1].rsplit("</svg>", 1)[0]
            x, y = left + (index % 4) * 116, 138 + (index // 4) * 144
            parts += [
                f'<rect x="{x}" y="{y}" width="108" height="128" rx="8" fill="none" stroke="{line}"/>',
                f'<g color="{accent}" transform="translate({x + 34} {y + 22}) scale(1.666667)">{svg}</g>',
                f'<g color="{muted}" transform="translate({x + 46} {y + 74}) scale(.666667)">{svg}</g>',
                f'<text x="{x + 54}" y="{y + 112}" text-anchor="middle" font-size="10.5">{escape(name)}</text>',
            ]
        parts.append('</g>')
    parts.append('</svg>')
    (root / f"assets/icons/{destination}.svg").write_text("\n".join(parts) + "\n")


pages = [("folio", "Folio")] + catalog("src/icons.rs", "Choice")
controls = catalog("src/icons/controls.rs", "Symbol")
assert len(pages) == 16 and len(controls) == 43, "Unexpected catalog size"
assert len({key for key, _ in pages + controls}) == 59, "Duplicate pictogram"
sheet(pages, "Signes", "catalog")
sheet(controls, "Commandes", "controls")
