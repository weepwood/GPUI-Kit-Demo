/// Color roles for the native workbench. Avoid per-screen hard-coded colors.
/// Quiet surfaces and a restrained clay accent keep controls legible in both modes.
#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: u32,
    pub surface: u32,
    pub sidebar: u32,
    pub text: u32,
    pub muted: u32,
    pub subtle: u32,
    pub border: u32,
    pub hover: u32,
    pub selected: u32,
    pub logo: u32,
    pub logo_text: u32,
    pub avatar: u32,
    pub status_bg: u32,
    pub status_text: u32,
    pub accent_soft: u32,
    pub accent_ink: u32,
    /// Opacity of the roles when the frosted window background is enabled.
    /// Content layers stack over the canvas, so text areas stay nearly opaque.
    pub alpha_canvas: f32,
    pub alpha_sidebar: f32,
    pub alpha_surface: f32,
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                canvas: 0x171b19,
                surface: 0x202521,
                sidebar: 0x1b201d,
                text: 0xf2f1ec,
                muted: 0xb8bdb4,
                subtle: 0x929b91,
                border: 0x394039,
                hover: 0x2b342e,
                selected: 0x323b34,
                logo: 0xe7d5bd,
                logo_text: 0x242b26,
                avatar: 0x435448,
                status_bg: 0x2a3d32,
                status_text: 0xa0cfac,
                accent_soft: 0x45362c,
                accent_ink: 0xf0b992,
                alpha_canvas: 0.94,
                alpha_sidebar: 0.80,
                alpha_surface: 0.86,
            }
        } else {
            Self {
                canvas: 0xf6f7f3,
                surface: 0xfefefc,
                sidebar: 0xeff0eb,
                text: 0x242b25,
                muted: 0x57615a,
                subtle: 0x616b62,
                border: 0xdde1d9,
                hover: 0xe6e9e2,
                selected: 0xdfe5db,
                logo: 0x303d35,
                logo_text: 0xfffcf8,
                avatar: 0xd7dfd5,
                status_bg: 0xe7efe9,
                status_text: 0x376547,
                accent_soft: 0xf6e9df,
                accent_ink: 0x885035,
                alpha_canvas: 0.96,
                alpha_sidebar: 0.84,
                alpha_surface: 0.90,
            }
        }
    }
}

/// Clay accent applied to the GPUI Kit theme so native controls (primary
/// buttons, switches, sliders, focus rings) stay in the workbench color system.
pub const PRIMARY_LIGHT: u32 = 0xa85432;
pub const PRIMARY_LIGHT_HOVER: u32 = 0x9a4a2b;
pub const PRIMARY_LIGHT_ACTIVE: u32 = 0x8a4226;
pub const PRIMARY_DARK: u32 = 0xe2a97f;
pub const PRIMARY_DARK_HOVER: u32 = 0xecbb98;
pub const PRIMARY_DARK_ACTIVE: u32 = 0xcd9066;
/// Foregrounds drawn on top of the clay fills above.
pub const PRIMARY_LIGHT_FOREGROUND: u32 = 0xfffcf8;
pub const PRIMARY_DARK_FOREGROUND: u32 = 0x2a170d;

/// Mirror the clay accent onto the active GPUI Kit theme. `Theme::change`
/// restores the stock palette for a mode, so call this once at startup and
/// again after every appearance switch.
pub fn apply_theme_overrides(cx: &mut gpui_kit::App) {
    use gpui_kit::component::Theme;

    let to_hsla = |hex: u32| -> gpui_kit::Hsla { gpui_kit::rgb(hex).into() };

    Theme::update(cx, |theme| {
        let dark = theme.mode.is_dark();
        let (primary, hover, active, foreground) = if dark {
            (
                to_hsla(PRIMARY_DARK),
                to_hsla(PRIMARY_DARK_HOVER),
                to_hsla(PRIMARY_DARK_ACTIVE),
                to_hsla(PRIMARY_DARK_FOREGROUND),
            )
        } else {
            (
                to_hsla(PRIMARY_LIGHT),
                to_hsla(PRIMARY_LIGHT_HOVER),
                to_hsla(PRIMARY_LIGHT_ACTIVE),
                to_hsla(PRIMARY_LIGHT_FOREGROUND),
            )
        };
        theme.colors.primary = primary;
        theme.colors.primary_hover = hover;
        theme.colors.primary_active = active;
        theme.colors.primary_foreground = foreground;
        theme.colors.accent = primary;
        theme.colors.accent_foreground = foreground;
    });
}

#[cfg(test)]
mod tests {
    use super::Palette;

    fn luminance(hex: u32) -> f64 {
        let channel = |shift| {
            let value = ((hex >> shift) & 0xffu32) as f64 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }

    fn contrast(a: u32, b: u32) -> f64 {
        let first = luminance(a);
        let second = luminance(b);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    #[test]
    fn foreground_contrast_on_main_surfaces() {
        for dark in [false, true] {
            let p = Palette::new(dark);
            assert!(contrast(p.text, p.canvas) >= 7.0);
            assert!(contrast(p.text, p.surface) >= 7.0);
            assert!(contrast(p.muted, p.surface) >= 4.5);
            assert!(contrast(p.subtle, p.canvas) >= 4.5);
            assert!(contrast(p.subtle, p.sidebar) >= 4.5);
            assert!(contrast(p.accent_ink, p.accent_soft) >= 4.5);
        }
    }

    #[test]
    fn theme_primary_contrast_with_its_foreground() {
        use super::{
            PRIMARY_DARK, PRIMARY_DARK_FOREGROUND, PRIMARY_LIGHT, PRIMARY_LIGHT_FOREGROUND,
        };
        assert!(contrast(PRIMARY_LIGHT, PRIMARY_LIGHT_FOREGROUND) >= 4.5);
        assert!(contrast(PRIMARY_DARK, PRIMARY_DARK_FOREGROUND) >= 4.5);
    }
}
