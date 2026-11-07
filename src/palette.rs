/// Application design tokens. Keep shell styling independent from business views.
#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas:u32, pub surface:u32, pub sidebar:u32, pub text:u32,
    pub muted:u32, pub subtle:u32, pub border:u32, pub hover:u32,
    pub selected:u32, pub logo:u32, pub logo_text:u32, pub avatar:u32,
    pub status_bg:u32, pub status_text:u32,
}
impl Palette {
    pub fn new(dark:bool)->Self {
        if dark {
            Self {canvas:0x151617,surface:0x1c1d1e,sidebar:0x202122,
                text:0xf0f0ef,muted:0xa5a5a3,subtle:0x858583,
                border:0x343638,hover:0x303234,selected:0x36383a,
                logo:0xf1f1ef,logo_text:0x1b1c1e,avatar:0x686d77,
                status_bg:0x263d32,status_text:0x81d6a6}
        } else {
            Self {canvas:0xfafaf9,surface:0xffffff,sidebar:0xf1f2f0,
                text:0x222322,muted:0x646864,subtle:0x939693,
                border:0xe5e7e4,hover:0xe9ebe8,selected:0xe1e4e0,
                logo:0x232626,logo_text:0xffffff,avatar:0xd8ded6,
                status_bg:0xe9f5eb,status_text:0x2d8150}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Palette;
    #[test]
    fn theme_palette_has_contrast() {
        for dark in [false,true] {
            let p=Palette::new(dark);
            assert_ne!(p.canvas,p.text);
            assert_ne!(p.surface,p.border);
            assert_ne!(p.status_bg,p.status_text);
        }
    }
}
