#![no_std]
#![no_main]
//Eating this for the wifi and some hardware items
extern crate alloc;
use esp_alloc::heap_allocator;

use airqual::hardware_init::init_hardware;
use airqual::hardware_init::AppHardware;
use airqual::hardware_init::PreflightHardware;
use airqual::sensor_reading::PlantowerFrame;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, pixelcolor::RgbColor, prelude::*};
use esp_backtrace as _;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

// Managing peripherals across embassy timer loop + preflight init
static HW: StaticCell<AppHardware> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    heap_allocator!(size: 72 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let p = PreflightHardware {
        i2c0: peripherals.I2C0,
        spi2: peripherals.SPI2,
        pin_display_pw: peripherals.GPIO7,
        pin_backlight: peripherals.GPIO45,
        pin_dc: peripherals.GPIO40,
        pin_cs: peripherals.GPIO42,
        pin_reset: peripherals.GPIO41,
        pin_sck: peripherals.GPIO36,
        pin_mosi: peripherals.GPIO35,
        pin_miso: peripherals.GPIO37,
        pin_sda: peripherals.GPIO3,
        pin_scl: peripherals.GPIO4,
    };
    // Not wrestling with lifetime specifiers in preflight struct
    let (hw_instance, mut state) = init_hardware(p);

    let hw = HW.init(hw_instance);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

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

        Timer::after(Duration::from_millis(1000)).await;
    }
}
