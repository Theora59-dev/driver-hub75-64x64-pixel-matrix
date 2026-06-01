#![no_std]
#![no_main]
pub mod maths;
use core::cmp::max;
use core::cmp::min;
use maths::*;

use driver_64x64_pixel_matrix::{Hub75Pins, PixelMap, Rgb565, display_frame};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    const WIN_HEIGHT: usize = 64;
    const WIN_WIDTH: usize = 64;

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

    fn put_triangle(canvas: &mut PixelMap<WIN_HEIGHT, WIN_WIDTH>, tri: &Triangle) {
        fn eq(p: &Vec2, a: &Vec2, b: &Vec2) -> i32 {
            (a.x - p.x) * (b.y - p.y) - (a.y - p.y) * (b.x - p.x)
        }

        let xmin = min(min(tri.v1.x, tri.v2.x), tri.v3.x);
        let xmax = max(max(tri.v1.x, tri.v2.x), tri.v3.x);
        let ymin = min(min(tri.v1.y, tri.v2.y), tri.v3.y);
        let ymax = max(max(tri.v1.y, tri.v2.y), tri.v3.y);

        for yc in ymin..ymax {
            if 0 <= yc && yc < WIN_HEIGHT as i32 {
                for xc in xmin..xmax {
                    if 0 <= xc && xc < WIN_WIDTH as i32 {
                        let pos = Vec2 { x: xc, y: yc };
                        let w1 = eq(&pos, &tri.v3, &tri.v1);
                        let w2 = eq(&pos, &tri.v1, &tri.v2);
                        let w3 = eq(&pos, &tri.v2, &tri.v3);

                        if (w1 > 0 && w2 > 0 && w3 > 0) || (w1 < 0 && w2 < 0 && w3 < 0) {
                            canvas.write_color_at(
                                pos.x as usize,
                                pos.y as usize,
                                Rgb565::magenta(),
                            );
                        }
                    }
                }
            }
        }
    }

    let mut t: f64 = 0.0;
    let tri = Triangle3D {
        v1: Vec3f {
            x: -0.5,
            y: -0.5,
            z: 0.0,
        },
        v2: Vec3f {
            x: 0.0,
            y: 0.5,
            z: 0.0,
        },
        v3: Vec3f {
            x: 0.5,
            y: -0.5,
            z: 0.0,
        },
    };

    loop {
        fb.clear();

        t += 0.01;
        put_triangle(
            &mut fb,
            &tri.rotation_y(t)
                .translate(Vec3f {
                    x: 0.0,
                    y: 0.1,
                    z: 1.0,
                })
                .projection()
                .to_screen(),
        );
        display_frame(&mut pins, &fb);
    }
}
