#![no_std]
#![no_main]

use driver_64x64_pixel_matrix::framebuffer::{PixelMap, Rgb565, display_frame};
use driver_64x64_pixel_matrix::hub75::Hub75Pins;
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
    let mut color_idx = 0;
    let mut last_change = Instant::now();
    let mut fps_timer = Instant::now();
    let mut frame_count = 0u32;

    loop {
        display_frame(&mut pins, &fb);

        frame_count += 1;
        // Calcul du nombre de trames par seconde toutes les secondes
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {}", frame_count);
            frame_count = 0;
            fps_timer = Instant::now();
        }

        if last_change.elapsed() >= Duration::from_secs(1) {
            color_idx = (color_idx + 1) % colors.len();
            fb.fill(colors[color_idx]);
            last_change = Instant::now();
        }
    }
}
