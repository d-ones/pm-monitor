use super::display::DisplayState;
extern crate alloc;
use alloc::boxed::Box;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config as i2cConfig, I2c};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::Blocking;
use mipidsi::interface::SpiInterface;
use mipidsi::options::ColorOrder;
use mipidsi::Display;
use mipidsi::{models::ST7789, Builder};

pub type TftDisplay<'a> = Display<
    SpiInterface<
        'a,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    mipidsi::models::ST7789,
    Output<'static>,
>;

pub struct AppHardware<'a> {
    pub display: TftDisplay<'a>,
    pub i2c: I2c<'a, Blocking>,
    pub delay: Delay,
}

pub fn init_hardware() -> (AppHardware<'static>, DisplayState) {
    // Init essential pieces of controller
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut delay = Delay::new();
    // let mut buffer = [0_u8; 4096];
    // Allocate on the heap and "leak" it so it becomes &'static mut [u8]
    let buffer = Box::leak(Box::new([0u8; 4096]));

    // Can't forget the backlight lol
    let mut display_power = Output::new(peripherals.GPIO7, Level::High, OutputConfig::default());
    display_power.set_high();

    let mut tft_backlight = Output::new(peripherals.GPIO45, Level::High, OutputConfig::default());
    tft_backlight.set_high();

    // Pins for display, direct current and chip select
    let dc = Output::new(peripherals.GPIO40, Level::High, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO42, Level::High, OutputConfig::default());

    // Init SPI for display purposes
    let spi_config = Config::default()
        .with_frequency(Rate::from_mhz(3))
        .with_mode(Mode::_0);

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(peripherals.GPIO36)
        .with_mosi(peripherals.GPIO35)
        .with_miso(peripherals.GPIO37);

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let si = SpiInterface::new(spi_device, dc, buffer);

    let display = Builder::new(ST7789, si)
        .color_order(ColorOrder::Rgb)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(Output::new(
            peripherals.GPIO41,
            Level::High,
            OutputConfig::default(),
        ))
        .orientation(
            mipidsi::options::Orientation::default().rotate(mipidsi::options::Rotation::Deg90),
        )
        .init(&mut delay)
        .unwrap();

    let i2c = I2c::new(
        peripherals.I2C0,
        i2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO3)
    .with_scl(peripherals.GPIO4);

    (
        AppHardware {
            display,
            i2c,
            delay,
        },
        DisplayState::new(),
    )
}

