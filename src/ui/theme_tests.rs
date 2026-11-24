use super::{Color, DarkPalette, palette};

fn luminance(color: Color) -> f32 {
    // Palette colors are already linear, decoded by rgb().
    color.r * 0.2126 + color.g * 0.7152 + color.b * 0.0722
}

fn contrast(a: Color, b: Color) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn dark_palettes_keep_body_text_and_accents_readable_on_all_surfaces() {
    for theme in DarkPalette::ALL {
        let p = palette(true, theme);
        for background in [p.bg, p.sidebar, p.tint] {
            for foreground in [p.ink, p.muted, p.accent] {
                assert!(
                    contrast(foreground, background) >= 4.5,
                    "{theme:?}: low text contrast"
                );
            }
            assert!(
                contrast(p.faint, background) >= 3.,
                "{theme:?}: low secondary contrast"
            );
        }
    }
}

#[test]
fn vanilla_keeps_text_readable_on_pastel_yellow_surfaces() {
    let p = super::light_palette(super::LightPalette::Vanilla);
    for background in [p.bg, p.sidebar, p.tint] {
        for foreground in [p.ink, p.muted, p.accent] {
            assert!(contrast(foreground, background) >= 4.5);
        }
        assert!(contrast(p.faint, background) >= 3.);
    }
}
