# Folio identity — folded leaves

A solid F with two diagonal ends suggesting folded leaves. Rounded left and
bottom edges and wide openings keep the mark readable in the sidebar. Sage
and graphite follow the Folio palette.

- [`folio-logo.blend`](folio-logo.blend): editable Blender scene with separate
  badge and mark, materials, orthographic camera and lighting; created in Blender 5.2.3.
- [`folio-logo.svg`](folio-logo.svg): complete sage/graphite vector badge.
- [`folio-logo.png`](folio-logo.png): front Blender render, 1024 px, transparent outside.
- [`folio-logo-studio.png`](folio-logo-studio.png): shallow relief on a graphite background.
- [`../icons/folio.svg`](../icons/folio.svg): monochrome mark embedded by Rune.

Blender and SVG outlines share the definition in
[`scripts/build-brand.py`](../../scripts/build-brand.py). No external model,
texture or font is required. The model has a slight thickness and bevels;
the interface uses the vector silhouette with the theme color for readability.

The native badge is 32 px, with a 9 px radius and a 28 px mark. The SVG badge
and Blender scene use the same proportions.

Regenerate from the project root:

```sh
/Applications/Blender.app/Contents/MacOS/Blender \
  --background --factory-startup --threads 4 --python scripts/build-brand.py
python3 scripts/build-icon-sheet.py
```

Use your Blender executable location on other systems. Regeneration replaces
exports and the scene; preserve any manual scene edits before running it.
Files use the project's MIT license.
