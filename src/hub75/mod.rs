use esp_hal::gpio::Output;

/// Driver for the 14 HUB75 signals used to control an RGB LED matrix.
///
/// Each signal corresponds to a GPIO output pin:
/// - **R1, G1, B1**: RGB data for the top half (rows 0–31)
/// - **R2, G2, B2**: RGB data for the bottom half (rows 32–63)
/// - **A, B, C, D, E**: row address lines (0–31 for 1/32 scan)
/// - **CLK** (clock): each rising edge shifts data one column forward
///   into the panel's shift registers. A ~30-cycle delay between
///   edges ensures sufficient hold time per the SM52606P datasheet.
/// - **LAT** (latch): transfers shift-register contents to the output
///   latches. Requires a longer pulse (>100 ns) than CLK.
/// - **OE** (output enable): master LED switch. Active low —
///   `LOW` turns LEDs on, `HIGH` turns them off.
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

/// Micro-delay used to meet CLK and LAT signal hold times.
/// At 240 MHz each iteration takes approximately 3–4 cycles.
fn delay_ns(n: u32) {
    for _ in 0..n {
        core::hint::spin_loop();
    }
}

impl<'d> Hub75Pins<'d> {
    /// Creates a new instance from 14 GPIO outputs.
    ///
    /// Parameter order: `r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe`.
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

    /// Sets the 5 address lines (A–E) to select the row pair `row`.
    /// For a 1/32 scan panel `row` ranges from 0 to 31.
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

    /// Sets the 6 data bits (R1, G1, B1, R2, G2, B2).
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

    /// Generates a CLK pulse (rising then falling edge).
    /// The micro-delay ensures sufficient high time for the panel's
    /// shift register (~120 ns at 240 MHz).
    pub fn pulse_clk(&mut self) {
        self.clk.set_high();
        delay_ns(30);
        self.clk.set_low();
        delay_ns(30);
    }

    /// Generates a LAT pulse to transfer shift-register contents
    /// to the output latches. The delay is longer than for CLK (~400 ns).
    pub fn latch(&mut self) {
        self.lat.set_high();
        delay_ns(100);
        self.lat.set_low();
    }

    /// Enables (`true`) or disables (`false`) the LED output.
    ///
    /// Note: OE is active low — `LOW` turns LEDs on.
    pub fn set_oe_enabled(&mut self, enabled: bool) {
        if enabled {
            self.oe.set_low();
        } else {
            self.oe.set_high();
        }
    }

    /// Shifts one pixel (6 bits) into the shift registers: sets the data
    /// lines then pulses CLK.
    pub fn shift_pixel(&mut self, r1: bool, g1: bool, b1: bool, r2: bool, g2: bool, b2: bool) {
        self.set_data(r1, g1, b1, r2, g2, b2);
        self.pulse_clk();
    }

    /// Turns LEDs off (OE = HIGH).
    pub fn oe_off(&mut self) {
        self.oe.set_high();
    }

    /// Turns LEDs on (OE = LOW).
    pub fn oe_on(&mut self) {
        self.oe.set_low();
    }
}
