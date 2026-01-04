use super::display::DisplayState;
extern crate alloc;
use alloc::boxed::Box;
use embassy_time::Delay;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::gpio::Pin;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config as i2cConfig, I2c};
use esp_hal::peripherals::{
    GPIO3, GPIO35, GPIO36, GPIO37, GPIO4, GPIO40, GPIO41, GPIO42, GPIO45, GPIO7, I2C0, SPI2,
};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::Blocking;
use mipidsi::interface::SpiInterface;
use mipidsi::options::ColorOrder;
use mipidsi::Display;
use mipidsi::{models::ST7789, Builder};

pub struct PreflightHardware<'a> {
    pub i2c0: I2C0<'a>,
    pub spi2: SPI2<'a>,
    pub pin_display_pw: GPIO7<'a>,
    pub pin_backlight: GPIO45<'a>,
    pub pin_dc: GPIO40<'a>,
    pub pin_cs: GPIO42<'a>,
    pub pin_reset: GPIO41<'a>,
    pub pin_sck: GPIO36<'a>,
    pub pin_mosi: GPIO35<'a>,
    pub pin_miso: GPIO37<'a>,
    pub pin_sda: GPIO3<'a>,
    pub pin_scl: GPIO4<'a>,
}

pub type TftDisplay<'a> = Display<
    SpiInterface<
        'a,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    mipidsi::models::ST7789,
    Output<'static>,
>;

pub struct AppHardware {
    pub display: TftDisplay<'static>,
    pub i2c: I2c<'static, Blocking>,
    pub delay: Delay,
}

pub fn init_hardware<'a>(p: PreflightHardware<'a>) -> (AppHardware, DisplayState) {
    // Init essential pieces of controller
    let mut delay = Delay;
    // let mut buffer = [0_u8; 4096];
    // Allocate on the heap and "leak" it so it becomes &'static mut [u8]
    let buffer = Box::leak(Box::new([0u8; 4096]));

    // Can't forget the backlight lol
    let mut display_power = Output::new(
        p.pin_display_pw.degrade(),
        Level::High,
        OutputConfig::default(),
    );
    display_power.set_high();

    let mut tft_backlight = Output::new(
        p.pin_backlight.degrade(),
        Level::High,
        OutputConfig::default(),
    );
    tft_backlight.set_high();

    // Pins for display, direct current and chip select
    let dc = Output::new(p.pin_dc.degrade(), Level::High, OutputConfig::default());
    let cs = Output::new(p.pin_cs.degrade(), Level::High, OutputConfig::default());

    // Init SPI for display purposes
    let spi_config = Config::default()
        .with_frequency(Rate::from_mhz(3))
        .with_mode(Mode::_0);

    let spi = Spi::new(p.spi2, spi_config)
        .unwrap()
        .with_sck(p.pin_sck.degrade())
        .with_mosi(p.pin_mosi.degrade())
        .with_miso(p.pin_miso.degrade());

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let si = SpiInterface::new(spi_device, dc, buffer);

    let display = Builder::new(ST7789, si)
        .color_order(ColorOrder::Rgb)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(Output::new(
            p.pin_reset.degrade(),
            Level::High,
            OutputConfig::default(),
        ))
        .orientation(
            mipidsi::options::Orientation::default().rotate(mipidsi::options::Rotation::Deg90),
        )
        .init(&mut delay)
        .unwrap();

    let i2c = I2c::new(
        p.i2c0,
        i2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(p.pin_sda.degrade())
    .with_scl(p.pin_scl.degrade());

    (
        AppHardware {
            // Hitting the special "I think my hardware won't get deallocated" button
            display: unsafe { core::mem::transmute(display) },
            i2c: unsafe { core::mem::transmute(i2c) },
            delay: embassy_time::Delay,
        },
        DisplayState::new(),
    )
}
