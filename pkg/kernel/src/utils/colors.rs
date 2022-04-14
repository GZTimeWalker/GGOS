use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;

/// default background color
///
/// #202020
pub const BACKGROUND: Rgb888 = Rgb888::new(0x20, 0x20, 0x20);

/// default frontground color
///
/// #efefef
pub const FRONTGROUND: Rgb888 = Rgb888::new(0xef, 0xef, 0xef);

/// default red color
///
/// #e53e30
pub const RED: Rgb888 = Rgb888::new(0xe5, 0x5e, 0x30);

/// default green color
///
/// #328e2e
pub const GREEN: Rgb888 = Rgb888::new(0x32, 0x8e, 0x2e);

/// default blue color
///
/// #3887fe
pub const BLUE: Rgb888 = Rgb888::new(0x38, 0x87, 0xfe);

/// default grey color
///
/// #555555
pub const GREY: Rgb888 = Rgb888::new(0x55, 0x55, 0x55);

pub const WHITE: Rgb888 = Rgb888::WHITE;
pub const BLACK: Rgb888 = Rgb888::BLACK;
