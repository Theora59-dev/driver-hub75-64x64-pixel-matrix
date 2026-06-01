#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

use driver_64x64_pixel_matrix::framebuffer::{PixelMap, Rgb565};
use driver_64x64_pixel_matrix::hub75::Hub75Pins;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn delay(duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {}
}

esp_bootloader_esp_idf::esp_app_desc!();

fn display_frame(pins: &mut Hub75Pins, fb: &PixelMap) {
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
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut pins = Hub75Pins::new(
        Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO12, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO22, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO17, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO15, Level::High, OutputConfig::default()),
    );

    let mut fb = PixelMap::new();
    let colors = [
        Rgb565::white(),
        Rgb565::blue(),
        Rgb565::red(),
        Rgb565::green(),
        Rgb565::black(),
        Rgb565::yellow(),
        Rgb565::cyan(),
        Rgb565::magenta(),
    ];

    for color in colors {
        for y in 0..64 {
            for x in 0..64 {
                fb.write_color_at(x, y, color);
            }
        }
        display_frame(&mut pins, &fb);
        delay(Duration::from_secs(1));
    }

    println!("Affichage !");

    loop {}
}
