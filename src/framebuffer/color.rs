/// 16-bit RGB565 color (5 bits red, 6 bits green, 5 bits blue).
///
/// Internal `u16` storage layout, MSB to LSB:
/// ```text
/// RRRR RGGG GGGB BBBB
/// ```
/// - bits 15–11: red (5 bits, 32 levels)
/// - bits 10–5:  green (6 bits, 64 levels)
/// - bits 4–0:   blue (5 bits, 32 levels)
#[derive(Clone, Copy)]
pub struct Rgb565(u16);

impl Rgb565 {
    /// Creates an RGB565 color from 8-bit components (0–255).
    ///
    /// Each component is reduced to its corresponding bit depth by
    /// discarding the least significant bits.
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb565 {
        let r = (r as u16 >> 3) & 0x1F;
        let g = (g as u16 >> 2) & 0x3F;
        let b = (b as u16 >> 3) & 0x1F;
        Rgb565((r << 11) | (g << 5) | b)
    }

    /// Returns the raw `u16` pixel value.
    pub const fn get_raw_color(&self) -> u16 {
        self.0
    }

    /// Black (all LEDs off).
    pub const fn black() -> Self {
        Self(0)
    }

    /// Pure red.
    pub const fn red() -> Self {
        Self(0xF800)
    }

    /// Pure green.
    pub const fn green() -> Self {
        Self(0x07E0)
    }

    /// Blue.
    pub const fn blue() -> Self {
        Self(0x001F)
    }

    /// White (all LEDs on).
    pub const fn white() -> Self {
        Self(0xFFFF)
    }

    /// Yellow (red + green).
    pub const fn yellow() -> Self {
        Self(0xFFE0)
    }

    /// Cyan (green + blue).
    pub const fn cyan() -> Self {
        Self(0x07FF)
    }

    /// Magenta (red + blue).
    pub const fn magenta() -> Self {
        Self(0xF81F)
    }

    /// Returns `true` if at least one color component is non-zero.
    pub const fn non_zero(&self) -> bool {
        self.0 != 0
    }

    /// Converts the color to 3 booleans (red, green, blue) for 1-bit
    /// display. Each boolean indicates whether the corresponding component
    /// is active (true) or off (false).
    pub const fn to_1bit(&self) -> (bool, bool, bool) {
        let r = (self.0 >> 11) & 0x1F != 0;
        let g = (self.0 >> 5) & 0x3F != 0;
        let b = self.0 & 0x1F != 0;
        (r, g, b)
    }
}
