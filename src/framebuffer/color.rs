/// Couleur 16 bits au format RGB565 (5 bits rouge, 6 bits vert, 5 bits bleu).
///
/// Le stockage interne est un `u16` organisé ainsi, du MSB au LSB :
/// ```text
/// RRRR RGGG GGGB BBBB
/// ```
/// - bits 15–11 : rouge (5 bits, 32 niveaux)
/// - bits 10–5  : vert (6 bits, 64 niveaux)
/// - bits 4–0   : bleu (5 bits, 32 niveaux)
#[derive(Clone, Copy)]
pub struct Rgb565(u16);

impl Rgb565 {
    /// Crée une couleur RGB565 à partir de composantes 8 bits (0–255).
    ///
    /// Chaque composante est réduite au nombre de bits correspondant par
    /// masquage des bits de poids faible.
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb565 {
        let r = (r as u16 >> 3) & 0x1F;
        let g = (g as u16 >> 2) & 0x3F;
        let b = (b as u16 >> 3) & 0x1F;
        Rgb565((r << 11) | (g << 5) | b)
    }

    /// Retourne la valeur brute `u16` du pixel.
    pub const fn get_raw_color(&self) -> u16 {
        self.0
    }

    /// Noir (toutes les LEDs éteintes).
    pub const fn black() -> Self {
        Self(0)
    }

    /// Rouge pur.
    pub const fn red() -> Self {
        Self(0xF800)
    }

    /// Vert pur.
    pub const fn green() -> Self {
        Self(0x07E0)
    }

    /// Bleu pur.
    pub const fn blue() -> Self {
        Self(0x001F)
    }

    /// Blanc (toutes les LEDs allumées).
    pub const fn white() -> Self {
        Self(0xFFFF)
    }

    /// Jaune (rouge + vert).
    pub const fn yellow() -> Self {
        Self(0xFFE0)
    }

    /// Cyan (vert + bleu).
    pub const fn cyan() -> Self {
        Self(0x07FF)
    }

    /// Magenta (rouge + bleu).
    pub const fn magenta() -> Self {
        Self(0xF81F)
    }

    /// Retourne vrai si au moins une composante de couleur est non nulle.
    pub const fn non_zero(&self) -> bool {
        self.0 != 0
    }

    /// Convertit la couleur en 3 booléens (rouge, vert, bleu) pour un
    /// affichage 1‑bit. Chaque booléen indique si la composante correspondante
    /// est active (vraie) ou éteinte (fausse). (Utile pour le débuggage)
    pub const fn to_1bit(&self) -> (bool, bool, bool) {
        let r = (self.0 >> 11) & 0x1F != 0;
        let g = (self.0 >> 5) & 0x3F != 0;
        let b = self.0 & 0x1F != 0;
        (r, g, b)
    }
}
