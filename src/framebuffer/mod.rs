mod color;

use crate::hub75::Hub75Pins;
use embedded_hal::digital::OutputPin;
pub use color::Rgb565;

/// Number of busy-wait iterations per row to control LED persistence
/// (and thus brightness). CPU-speed dependent — at 240 MHz this yields
/// roughly the correct on-time for 1/32 scan panels.
const ROW_DISPLAY_CYCLES: u32 = 20_000;

/// Renders one full frame to the HUB75 panel.
///
/// Sequences through all `H/2` scanline pairs, shifting `W` pixels per row pair,
/// latching, and displaying each row for a brief period (busy-wait).
///
/// `W` is the panel width in columns, `H` the panel height in rows.
pub fn display_frame<const W: usize, const H: usize, P: OutputPin>(pins: &mut Hub75Pins<P>, fb: &PixelMap<W, H>) {
    let scan_pairs = H / 2;
    for row in 0..scan_pairs {
        pins.oe_off();
        pins.set_row(row as u8);

        for col in 0..W {
            let (r1, g1, b1) = fb.read(col, row).to_1bit();
            let (r2, g2, b2) = fb.read(col, row + scan_pairs).to_1bit();
            pins.shift_pixel(r1, g1, b1, r2, g2, b2);
        }

        pins.latch();
        pins.oe_on();
        for _ in 0..ROW_DISPLAY_CYCLES {
            core::hint::spin_loop();
        }
    }

    pins.oe_off();
}

/// Framebuffer storing a `W × H` image in RGB565 format.
///
/// Storage is a 2D array indexed as `pixels[y][x]`.
/// Out-of-bounds accesses are silently ignored (clipped).
pub struct PixelMap<const W: usize, const H: usize> {
    pixels: [[Rgb565; W]; H],
}

impl<const W: usize, const H: usize> PixelMap<W, H> {
    /// Creates a new framebuffer initialized to black.
    pub const fn new() -> Self {
        Self {
            pixels: [[Rgb565::black(); W]; H],
        }
    }

    /// Writes a pixel at coordinates `(x, y)`.
    /// Out-of-bounds coordinates are silently ignored.
    pub fn write_color_at(&mut self, x: usize, y: usize, color: Rgb565) {
        if x < W && y < H {
            self.pixels[y][x] = color;
        }
    }

    /// Reads the pixel at coordinates `(x, y)`.
    /// Returns `Rgb565::black()` if out of bounds.
    pub fn read(&self, x: usize, y: usize) -> Rgb565 {
        if x < W && y < H {
            self.pixels[y][x]
        } else {
            Rgb565::black()
        }
    }

    /// Fills the entire framebuffer with a uniform color.
    pub fn fill(&mut self, color: Rgb565) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = color;
            }
        }
    }

    /// Clears the entire framebuffer to black.
    pub fn clear(&mut self) {
        self.fill(Rgb565::black());
    }

    /// Returns a reference to the raw pixel array.
    pub fn pixels(&self) -> &[[Rgb565; W]; H] {
        &self.pixels
    }
}

impl<const W: usize, const H: usize> Default for PixelMap<W, H> {
    fn default() -> Self {
        Self::new()
    }
}
