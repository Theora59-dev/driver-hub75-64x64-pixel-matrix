mod color;

use crate::hub75::Hub75Pins;
pub use color::Rgb565;

pub fn display_frame(pins: &mut Hub75Pins, fb: &PixelMap) {
    for row in 0..32u8 {
        pins.oe_off();
        pins.set_row(row);

        for col in 0..64u8 {
            let (r1, g1, b1) = fb.read(col as usize, row as usize).to_1bit();
            let (r2, g2, b2) = fb.read(col as usize, (row as usize) + 32).to_1bit();
            pins.shift_pixel(r1, g1, b1, r2, g2, b2);
        }

        pins.latch();
        pins.oe_on();
        for _ in 0..20000 {
            core::hint::spin_loop();
        }
    }

    pins.oe_off();
}

/// Dimensions du panneau : 64×64 pixels.
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 64;

/// Framebuffer stockant une image de `WIDTH × HEIGHT` pixels en RGB565.
/// Le stockage est un tableau plat indexé par `y * WIDTH + x`.
/// Les accès hors limites sont silencieusement ignorés (clipping).
pub struct PixelMap {
    pixels: [Rgb565; WIDTH * HEIGHT],
}

impl PixelMap {
    /// Crée un nouveau framebuffer initialisé en noir.
    pub const fn new() -> Self {
        Self {
            pixels: [Rgb565::black(); WIDTH * HEIGHT],
        }
    }

    /// Écrit un pixel aux coordonnées `(x, y)`.
    /// Les coordonnées hors limites sont ignorées.
    pub fn write_color_at(&mut self, x: usize, y: usize, color: Rgb565) {
        if x < WIDTH && y < HEIGHT {
            self.pixels[y * WIDTH + x] = color;
        }
    }

    /// Lit le pixel aux coordonnées `(x, y)`.
    /// Retourne `Rgb565::black()` si les coordonnées sont hors limites.
    pub fn read(&self, x: usize, y: usize) -> Rgb565 {
        if x < WIDTH && y < HEIGHT {
            self.pixels[y * WIDTH + x]
        } else {
            Rgb565::black()
        }
    }

    /// Remplit tout le framebuffer avec une couleur uniforme.
    pub fn fill(&mut self, color: Rgb565) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
    }

    /// Efface tout le framebuffer (remet à noir).
    pub fn clear(&mut self) {
        self.fill(Rgb565::black());
    }

    /// Retourne une référence sur le tableau brut de pixels.
    pub fn pixels(&self) -> &[Rgb565; WIDTH * HEIGHT] {
        &self.pixels
    }
}

impl Default for PixelMap {
    fn default() -> Self {
        Self::new()
    }
}
