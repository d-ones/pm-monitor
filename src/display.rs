use super::sensor_reading::PlantowerFrame;

use core::fmt::Write;
use embedded_graphics::{pixelcolor::Rgb565, pixelcolor::RgbColor, prelude::*};
use u8g2_fonts::{fonts, FontRenderer};

const DEFAULT_FONT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_helvB24_tr>();

pub enum DeviceMode {
    WarmingUp(u8), // Carries the remaining seconds
    Collecting,    // Active sensor mode
}

pub struct DisplayState {
    pub mode: DeviceMode,
    pub pm2_5: Option<u16>,
    pub pm10: Option<u16>,
    pub status_ok: bool,
}

impl DisplayState {
    pub fn new() -> Self {
        Self {
            mode: DeviceMode::WarmingUp(30),
            pm2_5: None,
            pm10: None,
            status_ok: false,
        }
    }

    pub fn update(&mut self, frame: Option<PlantowerFrame>) {
        // We only care about sensor data if we are in Collecting mode
        if matches!(self.mode, DeviceMode::Collecting) {
            if let Some(f) = frame {
                self.pm2_5 = Some(f.pm2_5_atm.get());
                self.pm10 = Some(f.pm10_atm.get());
                self.status_ok = true;
            } else {
                self.status_ok = false;
            }
        }
    }
    // Used to countdown for sensor initiation
    // 30 seconds is the industry standard for Plantower sensors
    // to reach a stable laminar airflow.
    pub fn tick(&mut self) {
        if let DeviceMode::WarmingUp(secs) = self.mode {
            if secs > 0 {
                self.mode = DeviceMode::WarmingUp(secs - 1);
            } else {
                self.mode = DeviceMode::Collecting;
            }
        }
    }
    pub fn render<D>(
        &self,
        display: &mut D,
    ) -> Result<(), u8g2_fonts::Error<<D as DrawTarget>::Error>>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let mut buf: heapless::String<32> = heapless::String::new();

        match self.pm2_5 {
            Some(val) => core::write!(buf, "PM2.5: \n {} ug/m3", val).ok(),
            None => buf.push_str("Initializing...").ok(),
        };

        // eat the complexity of the alignment and colors here
        DEFAULT_FONT.render_aligned(
            buf.as_str(),
            Point::new(
                display.bounding_box().center().x,
                display.bounding_box().center().y,
            ),
            u8g2_fonts::types::VerticalPosition::Baseline,
            u8g2_fonts::types::HorizontalAlignment::Center,
            u8g2_fonts::types::FontColor::Transparent(if self.status_ok {
                Rgb565::CYAN
            } else {
                Rgb565::RED // Stale data
            }),
            display,
        )?;

        Ok(())
    }
}
