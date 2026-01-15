This program runs on the ESP32-S2 version of the [TFT Reverse Feather](https://learn.adafruit.com/esp32-s3-reverse-tft-feather/overview) and the [Adafruit PMSA003I Air Quality Breakout](https://learn.adafruit.com/pmsa003i).

While the primary output of this sensor as configured are PM2.5, PM10, and 0.3um readings, more readings can be produced from the sensor struct.

This program uses async Rust in order to post readings to UDP (currently) and optionally the included display without significant overhead. The output string can be configured easily for parsing to a dashboard.

<img width="960" height="1280" alt="image" src="https://github.com/user-attachments/assets/4180a16d-e832-45e3-81f2-6f8a1276a69a" />
