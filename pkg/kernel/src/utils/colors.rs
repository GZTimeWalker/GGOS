use embedded_graphics::pixelcolor::Rgb888;

/// default background color
///
/// #202020
pub const BACKGROUND: Rgb888 = Rgb888::new(0x20, 0x20, 0x20);

/// default frontground color
///
/// #efefef
pub const FRONTGROUND: Rgb888 = Rgb888::new(0xef, 0xef, 0xef);

/// default warning color
///
/// #e53e30
pub const RED: Rgb888 = Rgb888::new(0xe5, 0x5e, 0x30);

/// default hint color
///
/// #328e2e
pub const GREEN: Rgb888 = Rgb888::new(0x32, 0x8e, 0x2e);
