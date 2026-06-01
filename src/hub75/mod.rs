use embedded_hal::digital::OutputPin;

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
pub struct Hub75Pins<P> {
    r1: P,
    g1: P,
    b1: P,
    r2: P,
    g2: P,
    b2: P,
    a: P,
    b: P,
    c: P,
    d: P,
    e: P,
    clk: P,
    lat: P,
    oe: P,
}

/// Busy-wait spin loop used to meet CLK and LAT signal hold times.
/// The actual delay depends on the CPU clock speed. At 240 MHz each
/// iteration takes approximately 3–4 cycles.
fn delay_spin(n: u32) {
    for _ in 0..n {
        core::hint::spin_loop();
    }
}

impl<P: OutputPin> Hub75Pins<P> {
    /// Creates a new instance from 14 GPIO outputs.
    ///
    /// Parameter order: `r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe`.
    pub fn new(
        r1: P,
        g1: P,
        b1: P,
        r2: P,
        g2: P,
        b2: P,
        a: P,
        b: P,
        c: P,
        d: P,
        e: P,
        clk: P,
        lat: P,
        oe: P,
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
            self.a.set_high().ok();
        } else {
            self.a.set_low().ok();
        }
        if row & 0x02 != 0 {
            self.b.set_high().ok();
        } else {
            self.b.set_low().ok();
        }
        if row & 0x04 != 0 {
            self.c.set_high().ok();
        } else {
            self.c.set_low().ok();
        }
        if row & 0x08 != 0 {
            self.d.set_high().ok();
        } else {
            self.d.set_low().ok();
        }
        if row & 0x10 != 0 {
            self.e.set_high().ok();
        } else {
            self.e.set_low().ok();
        }
    }

    /// Sets the 6 data bits (R1, G1, B1, R2, G2, B2).
    pub fn set_data(&mut self, r1: bool, g1: bool, b1: bool, r2: bool, g2: bool, b2: bool) {
        if r1 {
            self.r1.set_high().ok();
        } else {
            self.r1.set_low().ok();
        }
        if g1 {
            self.g1.set_high().ok();
        } else {
            self.g1.set_low().ok();
        }
        if b1 {
            self.b1.set_high().ok();
        } else {
            self.b1.set_low().ok();
        }
        if r2 {
            self.r2.set_high().ok();
        } else {
            self.r2.set_low().ok();
        }
        if g2 {
            self.g2.set_high().ok();
        } else {
            self.g2.set_low().ok();
        }
        if b2 {
            self.b2.set_high().ok();
        } else {
            self.b2.set_low().ok();
        }
    }

    /// Generates a CLK pulse (rising then falling edge).
    /// The spin-loop ensures sufficient high time for the panel's
    /// shift register (~120 ns at 240 MHz).
    pub fn pulse_clk(&mut self) {
        self.clk.set_high().ok();
        delay_spin(30);
        self.clk.set_low().ok();
        delay_spin(30);
    }

    /// Generates a LAT pulse to transfer shift-register contents
    /// to the output latches. The spin-loop is longer than for CLK (~400 ns).
    pub fn latch(&mut self) {
        self.lat.set_high().ok();
        delay_spin(100);
        self.lat.set_low().ok();
    }

    /// Shifts one pixel (6 bits) into the shift registers: sets the data
    /// lines then pulses CLK.
    pub fn shift_pixel(&mut self, r1: bool, g1: bool, b1: bool, r2: bool, g2: bool, b2: bool) {
        self.set_data(r1, g1, b1, r2, g2, b2);
        self.pulse_clk();
    }

    /// Turns LEDs off (OE = HIGH).
    pub fn oe_off(&mut self) {
        self.oe.set_high().ok();
    }

    /// Turns LEDs on (OE = LOW).
    pub fn oe_on(&mut self) {
        self.oe.set_low().ok();
    }
}
