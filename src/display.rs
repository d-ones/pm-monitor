use super::hardware_init::DisplaySystem;
use super::sensor_reading::DATA_BUS;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_graphics::{pixelcolor::Rgb565, pixelcolor::RgbColor, prelude::*};
use esp_hal::gpio::Input;
use u8g2_fonts::{fonts, FontRenderer};

// NOTE -- this section is a work in progress.
// Button doesn't work exactly as expected but is mainly for debugging
// Press it twice at the start to turn the screen off (inits on high)

static SCREEN_ON: AtomicBool = AtomicBool::new(false);
static SCREEN_TOGGLE_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();
// Check if we already registered the toggle command
static IS_POWERED_ON: AtomicBool = AtomicBool::new(false);

pub const DEFAULT_FONT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvB24_tr>();

#[embassy_executor::task]
pub async fn button_task(button: Input<'static>) {
    loop {
        Timer::after_millis(50).await;
        if button.is_high() {
            let new_state = !SCREEN_ON.load(Ordering::Relaxed);
            SCREEN_ON.store(new_state, Ordering::Relaxed);
            SCREEN_TOGGLE_SIGNAL.signal(new_state);
            // Ideally you can hold it until it goes off
            Timer::after_millis(3000).await;
        }
    }
}

#[embassy_executor::task]
pub async fn render(display_sys: &'static mut DisplaySystem) {
    let mut sub = DATA_BUS.subscriber().unwrap();
    loop {
        match select(sub.next_message_pure(), SCREEN_TOGGLE_SIGNAL.wait()).await {
            Either::Second(_) => {
                if SCREEN_ON.load(Ordering::Relaxed) {
                    if !IS_POWERED_ON.load(Ordering::Relaxed) {
                        display_sys.backlight.set_high();
                        IS_POWERED_ON.store(true, Ordering::Relaxed);
                    }
                } else {
                    if IS_POWERED_ON.load(Ordering::Relaxed) {
                        display_sys.backlight.set_low();
                        IS_POWERED_ON.store(false, Ordering::Relaxed);
                    }
                }
            }
            Either::First(frame) => {
                if SCREEN_ON.load(Ordering::Relaxed) {
                    let mut buf: heapless::String<32> = heapless::String::new();
                    let _ = write!(
                        buf,
                        "{} - {} - {}",
                        frame.pm2_5_atm.get(),
                        frame.pm10_atm.get(),
                        frame.counts_0_3.get()
                    );
                    display_sys.tft.clear(Rgb565::BLACK).unwrap();
                    // eat the complexity of the alignment and colors here
                    DEFAULT_FONT
                        .render_aligned(
                            buf.as_str(),
                            Point::new(
                                display_sys.tft.bounding_box().center().x,
                                display_sys.tft.bounding_box().center().y,
                            ),
                            u8g2_fonts::types::VerticalPosition::Baseline,
                            u8g2_fonts::types::HorizontalAlignment::Center,
                            u8g2_fonts::types::FontColor::Transparent(Rgb565::CYAN),
                            &mut display_sys.tft,
                        )
                        .unwrap();
                }
            }
        }
        // debounce
        Timer::after_millis(50).await;
    }
}
