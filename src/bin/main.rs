#![no_std]
#![no_main]
//Eating this for the wifi and some hardware items
extern crate alloc;
use esp_alloc::heap_allocator;

use airqual::hardware_init::init_hardware;
use airqual::sensor_reading::PlantowerFrame;
use embedded_graphics::{pixelcolor::Rgb565, pixelcolor::RgbColor, prelude::*};
use esp_backtrace as _;

use esp_hal::main;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    heap_allocator!(size: 72 * 1024);

    let (mut hw, mut state) = init_hardware();

    loop {
        let mut i2c_buffer = [0u8; 32];

        let frame = hw
            .i2c
            .read(0x12, &mut i2c_buffer)
            .ok()
            .and_then(|_| PlantowerFrame::parse(&i2c_buffer));

        state.tick();
        state.update(frame);

        hw.display.clear(Rgb565::BLACK).unwrap();
        state.render(&mut hw.display).unwrap();

        hw.delay.delay_millis(1000);
    }
}
