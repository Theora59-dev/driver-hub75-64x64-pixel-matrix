#![no_std]
#![no_main]

use driver_64x64_pixel_matrix::{Hub75Pins, PixelMap, Rgb565, display_frame};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

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

    let mut fb = PixelMap::<64, 64>::new();
    let mut fps_timer = Instant::now();
    let mut frame_count = 0u32;

    for y in 0..64 {
        for x in 0..64 {
            let is_white = (x / 8 + y / 8) % 2 == 0;
            fb.write_color_at(
                x,
                y,
                if is_white {
                    Rgb565::white()
                } else {
                    Rgb565::black()
                },
            );
        }
    }

    loop {
        display_frame(&mut pins, &fb);

        frame_count += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {}", frame_count);
            frame_count = 0;
            fps_timer = Instant::now();
        }
    }
}
