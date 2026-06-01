# driver-64x64-pixel-matrix

HUB75 RGB LED matrix driver for ESP32. Designed for 64×64 panels in 1/32 scan, but configurable for other sizes via generics const.

## Features

- Bit-bang HUB75 protocol over GPIO (no RMT/I2S required)
- 1-bit-per-color display (8 colors)
- Configurable panel size via const generics (`PixelMap<W, H>`)
- `no_std` compatible
- FPS counter in demo binaries

## Hardware

### Default pinout

| HUB75 | GPIO |
| ----- | ---- |
| R1    | 25   |
| G1    | 26   |
| B1    | 27   |
| R2    | 14   |
| G2    | 12   |
| B2    | 13   |
| A     | 23   |
| B     | 22   |
| C     | 5    |
| D     | 17   |
| E     | 32   |
| CLK   | 16   |
| LAT   | 4    |
| OE    | 15   |

Pins are fully configurable via `Hub75Pins::new()`.

## Usage

```rust
use driver_64x64_pixel_matrix::{display_frame, Hub75Pins, PixelMap, Rgb565};

let mut pins = Hub75Pins::new(/* 14 GPIO outputs */);
let mut fb = PixelMap::<64, 64>::new();

fb.fill(Rgb565::red());

loop {
    display_frame(&mut pins, &fb);
}
```

## Examples

```bash
# Solid colors cycling
cargo run --example solid_colors

# Checkerboard pattern
cargo run --example checkerboard

# Vertical color bars
cargo run --example color_bars
```

## License

Use of this repository is subject to the following license:

- MIT license (LICENSE-MIT or https://opensource.org/licenses/MIT)
