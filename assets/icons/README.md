# Folio icons

35 page icons share a 24 × 24 grid, 1.5 px strokes, rounded caps, and rounded
joins. They are original SVGs supplied under the project's MIT license.
The Folio monogram and command icons use the same visual language.

[Page icons](catalog.svg) · [Color variations](colors.svg) · [Controls](controls.svg)

## Page customization

Click a page's icon to choose a symbol and color. The scrollable picker contains
all 35 symbols. Colors update the preview immediately; choosing a symbol closes
the picker. The reset swatch returns to the theme's default color.

Eight colors are available: Sage, Teal, Blue, Violet, Rose, Coral, Amber, and Gray.
Each has a darker variant for the light theme and a softer variant for the dark
theme. Every variant has at least 3:1 contrast against its page background.
The chosen color follows the page in the sidebar, page links, and record cards.

Symbols and colors are stored separately. Existing emoji identifiers are still
recognized, and documents without a color keep their previous theme color.
The optional `icon_color` field is retained in database saves, history, backups,
and protected notes. Existing notes are not recolored automatically.

## Sources

- [Page catalog](../../src/icons.rs) maps stored identifiers to SVGs.
- [Palette](../../src/icons/colors.rs) defines the shared light/dark color pairs.
- [Controls](../../src/icons/controls.rs) maps commands and block types.
- [UI](../../src/ui/icons.rs) applies page colors wherever the icons are rendered.

All icons use `currentColor`, with no font, texture, or external asset dependencies.
The [Folio logo and Blender source](../brand/README.md) remain available separately.

Regenerate the three SVG sheets after editing the catalog or palette:

```sh
python3 scripts/build-icon-sheet.py
```

Preview the native application using fictional demo content:

```sh
FOLIO_DEMO=1 cargo run --locked -- --scene icon-catalog
FOLIO_DEMO=1 cargo run --locked -- --script scripts/icon-colors.json
```
