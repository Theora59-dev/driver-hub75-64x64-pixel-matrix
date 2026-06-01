use esp_hal::gpio::Output;

/// Pilote des 14 signaux HUB75 pour piloter une matrice LED.
///
/// Chaque signal correspond à une broche GPIO configurée en sortie :
/// - **R1, G1, B1** : données RGB de la moitié haute (lignes 0–31)
/// - **R2, G2, B2** : données RGB de la moitié basse (lignes 32–63)
/// - **A, B, C, D, E** : adresse de la paire de lignes (0–31 pour 1/32 scan)
/// - **CLK** (horloge) : À chaque fois qu'on fait un « tic » sur CLK, le
///   panneau décale les données d'un cran vers la droite, comme une file
///   d'attente. C'est comme appuyer sur « Entrée » après chaque pixel pour
///   le faire entrer dans la mémoire du panneau. On attend ~30 cycles CPU
///   entre le front haut et le front bas pour que le registre ait le temps
///   de réagir (fiche technique du SM52606P).
/// - **LAT** (latch) : Une fois qu'on a décalé tous les pixels d'une ligne,
///   on envoie un « petit coup » sur LAT pour tout figer d'un coup.
///   Imagine que tu écris lettre par lettre sur un brouillon (shift), puis
///   que tu fais un copier-coller vers le document final (latch). La pause
///   est plus longue (> 100 ns) car le transfert est plus lent que le
///   simple décalage.
/// - **OE** (output enable) : C'est l'interrupteur général des LED.
///   On laisse OE éteint (= `HIGH`) pendant qu'on remplit la ligne pour
///   éviter que l'image précédente clignote n'importe comment. Ensuite on
///   allume OE (= `LOW`) pour que la ligne s'affiche pendant un petit
///   moment. En jouant sur le temps où OE reste allumé, on fait du
///   « faux » PWM sans avoir besoin de vrais PWM matériels.
pub struct Hub75Pins<'d> {
    r1: Output<'d>,
    g1: Output<'d>,
    b1: Output<'d>,
    r2: Output<'d>,
    g2: Output<'d>,
    b2: Output<'d>,
    a: Output<'d>,
    b: Output<'d>,
    c: Output<'d>,
    d: Output<'d>,
    e: Output<'d>,
    clk: Output<'d>,
    lat: Output<'d>,
    oe: Output<'d>,
}

/// Micro-pause utilisée pour respecter les temps de maintien des signaux
/// CLK et LAT. À 240 MHz, chaque itération dure environ 3–4 cycles.
fn delay_ns(n: u32) {
    for _ in 0..n {
        core::hint::spin_loop();
    }
}

impl<'d> Hub75Pins<'d> {
    /// Crée une nouvelle instance à partir des 14 sorties GPIO.
    ///
    /// Chaque paramètre correspond à un signal HUB75. L'ordre est :
    /// `r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe`.
    pub fn new(
        r1: Output<'d>,
        g1: Output<'d>,
        b1: Output<'d>,
        r2: Output<'d>,
        g2: Output<'d>,
        b2: Output<'d>,
        a: Output<'d>,
        b: Output<'d>,
        c: Output<'d>,
        d: Output<'d>,
        e: Output<'d>,
        clk: Output<'d>,
        lat: Output<'d>,
        oe: Output<'d>,
    ) -> Self {
        Self {
            r1,
            g1,
            b1,
            r2,
            g2,
            b2,
            a,
            b,
            c,
            d,
            e,
            clk,
            lat,
            oe,
        }
    }

    /// Positionne les 5 bits d'adresse (A–E) pour sélectionner la paire de
    /// lignes `row`. Pour un scan 1/32, `row` va de 0 à 31.
    pub fn set_row(&mut self, row: u8) {
        if row & 0x01 != 0 {
            self.a.set_high();
        } else {
            self.a.set_low();
        }
        if row & 0x02 != 0 {
            self.b.set_high();
        } else {
            self.b.set_low();
        }
        if row & 0x04 != 0 {
            self.c.set_high();
        } else {
            self.c.set_low();
        }
        if row & 0x08 != 0 {
            self.d.set_high();
        } else {
            self.d.set_low();
        }
        if row & 0x10 != 0 {
            self.e.set_high();
        } else {
            self.e.set_low();
        }
    }

    /// Fixe l'état des 6 bits de données (R1,G1,B1, R2,G2,B2).
    pub fn set_data(&mut self, r1: bool, g1: bool, b1: bool, r2: bool, g2: bool, b2: bool) {
        if r1 {
            self.r1.set_high();
        } else {
            self.r1.set_low();
        }
        if g1 {
            self.g1.set_high();
        } else {
            self.g1.set_low();
        }
        if b1 {
            self.b1.set_high();
        } else {
            self.b1.set_low();
        }
        if r2 {
            self.r2.set_high();
        } else {
            self.r2.set_low();
        }
        if g2 {
            self.g2.set_high();
        } else {
            self.g2.set_low();
        }
        if b2 {
            self.b2.set_high();
        } else {
            self.b2.set_low();
        }
    }

    /// Génère une impulsion sur CLK (front montant puis front descendant).
    /// La micro-pause assure un temps haut suffisant pour le registre à
    /// décalage du panneau (~120 ns à 240 MHz).
    pub fn pulse_clk(&mut self) {
        self.clk.set_high();
        delay_ns(30);
        self.clk.set_low();
        delay_ns(30);
    }

    /// Génère une impulsion sur LAT pour transférer le contenu des
    /// registres à décalage vers les sorties. La pause est plus longue
    /// que celle de CLK (~400 ns).
    pub fn latch(&mut self) {
        self.lat.set_high();
        delay_ns(100);
        self.lat.set_low();
    }

    /// Active (`true`) ou désactive (`false`) la sortie des LEDs.
    ///
    /// # Rappel
    /// Le signal OE est actif bas : `LOW` = LEDs allumées.
    pub fn set_oe_enabled(&mut self, enabled: bool) {
        if enabled {
            self.oe.set_low();
        } else {
            self.oe.set_high();
        }
    }

    /// Shifte un pixel (6 bits) dans les registres : pose les données
    /// puis envoie une impulsion CLK.
    pub fn shift_pixel(&mut self, r1: bool, g1: bool, b1: bool, r2: bool, g2: bool, b2: bool) {
        self.set_data(r1, g1, b1, r2, g2, b2);
        self.pulse_clk();
    }

    /// Éteint les LEDs (OE = HIGH).
    pub fn oe_off(&mut self) {
        self.oe.set_high();
    }

    /// Allume les LEDs (OE = LOW).
    pub fn oe_on(&mut self) {
        self.oe.set_low();
    }
}
