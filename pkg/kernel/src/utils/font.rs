use embedded_graphics::{
    geometry::Size,
    image::ImageRaw,
    mono_font::{mapping::ASCII, DecorationDimensions, MonoFont}
};

pub const JBMONO: MonoFont = MonoFont {
    image: ImageRaw::new_binary(
        include_bytes!("../assets/font.raw"),
        15 * 16,
    ),
    glyph_mapping: &ASCII,
    character_size: Size::new(15, 30),
    character_spacing: 0,
    baseline: 30,
    underline: DecorationDimensions::new(30, 2),
    strikethrough: DecorationDimensions::new(16, 2),
};
