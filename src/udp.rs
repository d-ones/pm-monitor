use super::secrets::{WIFI_PASS, WIFI_SSID};
use super::sensor_reading::DATA_BUS;
use core::fmt::Write;
use embassy_net::Stack;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_radio::wifi::{
    AuthMethod, ClientConfig, ModeConfig, WifiController, WifiDevice, WifiStaState,
};

#[embassy_executor::task]
pub async fn connection_task(mut controller: WifiController<'static>) {
    // wait for the S2 power rails to settle
    Timer::after_millis(500).await;

    loop {
        if esp_radio::wifi::sta_state() != WifiStaState::Connected {
            esp_println::println!("WiFi: Connecting to Goose House...");

            let config = ModeConfig::Client(
                ClientConfig::default()
                    .with_auth_method(AuthMethod::Wpa2Personal)
                    .with_ssid(WIFI_SSID.try_into().unwrap())
                    .with_password(WIFI_PASS.try_into().unwrap()),
            );
            if controller.set_config(&config).is_ok() {
                if !controller.is_started().unwrap_or(false) {
                    controller.start_async().await.ok();
                };
                match controller.connect_async().await {
                    Ok(_) => esp_println::println!("WiFi: Connected!"),
                    Err(e) => esp_println::println!("WiFi: Connect failed {:?}", e),
                }
            }
        }

        Timer::after_secs(10).await;
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn send_data(stack: &'static Stack<'static>, remote_ip: [u8; 4], port: u16) {
    let mut rx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
    let mut rx_payload = [0u8; 512];
    let mut tx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
    let mut tx_payload = [0u8; 1024];

    let mut socket = embassy_net::udp::UdpSocket::new(
        *stack,
        &mut rx_meta,
        &mut rx_payload,
        &mut tx_meta,
        &mut tx_payload,
    );

    // Bind to a local port
    socket.bind(12345).ok();
    let remote_endpoint = (embassy_net::Ipv4Address::from(remote_ip), port);

    let mut sub = DATA_BUS.subscriber().unwrap();

    loop {
        let frame = sub.next_message_pure().await;

        if stack.is_config_up() {
            let mut s = heapless::String::<128>::new();
            // Can be JSON'd or whatever here
            let _ = write!(
                s,
                "PM2.5: {} | PM10: {} | Counts_0.3: {}\n",
                frame.pm2_5_atm.get(),
                frame.pm10_atm.get(),
                frame.counts_0_3.get()
            );

            let _ = socket.send_to(s.as_bytes(), remote_endpoint).await;
            esp_println::println!("Sent measurement to Manjaro!");
        }

        // debounce
        Timer::after_millis(50).await;
    }
}
