/// Color roles for the native workbench. Avoid per-screen hard-coded colors.
/// Quiet surfaces and a restrained clay accent keep controls legible in both modes.
#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: u32, pub surface: u32, pub sidebar: u32, pub text: u32,
    pub muted: u32, pub subtle: u32, pub border: u32, pub hover: u32,
    pub selected: u32, pub logo: u32, pub logo_text: u32, pub avatar: u32,
    pub status_bg: u32, pub status_text: u32,
    pub accent_soft: u32, pub accent_ink: u32,
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                canvas: 0x171b19, surface: 0x202521, sidebar: 0x1b201d,
                text: 0xf2f1ec, muted: 0xb8bdb4, subtle: 0x929b91,
                border: 0x394039, hover: 0x2b342e, selected: 0x323b34,
                logo: 0xe7d5bd, logo_text: 0x242b26, avatar: 0x435448,
                status_bg: 0x2a3d32, status_text: 0xa0cfac,
                accent_soft: 0x45362c, accent_ink: 0xf0b992,
            }
        } else {
            Self {
                canvas: 0xf6f7f3, surface: 0xfefefc, sidebar: 0xeff0eb,
                text: 0x242b25, muted: 0x57615a, subtle: 0x616b62,
                border: 0xdde1d9, hover: 0xe6e9e2, selected: 0xdfe5db,
                logo: 0x303d35, logo_text: 0xffffff, avatar: 0xd7dfd5,
                status_bg: 0xe7efe9, status_text: 0x376547,
                accent_soft: 0xf6e9df, accent_ink: 0x885035,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Palette;

    fn luminance(hex: u32) -> f64 {
        let channel = |shift| {
            let value = ((hex >> shift) & 0xffu32) as f64 / 255.0;
            if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
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
}
