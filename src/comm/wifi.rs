use std::thread;
use std::time::Duration;
use std::str::FromStr;

use anyhow::{bail, Error, Result};
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::*,
    peripherals::Peripherals,
    peripheral,
    spi::*,
    spi::config::*,
    units::FromValueType,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::*,
};
use mipidsi::interface::SpiInterface;

use mipidsi::{
    Builder,
    models::ST7789,
    options::*,
};
use embedded_graphics::{
    prelude::*,
    pixelcolor::*,
    text::*,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
};

/// This configuration is picked up at compile time by `build.rs` from the
/// file `cfg.toml`.
#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}


pub fn config_and_connect_wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        log::info!("Wifi Ssid is empty");
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        log::info!("Wifi password is empty");
    }
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    log::info!("Starting wifi...");

    wifi.start()?;

    log::info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        log::info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        log::info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_str(ssid).unwrap(),
        password: heapless::String::<64>::from_str(pass).unwrap(),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    log::info!("Connecting wifi...");

    wifi.connect()?;

    log::info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    log::info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}

pub fn wifi_example(){
    let app_config = CONFIG;
    log::info!("Loaded config file: {CONFIG:?}");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();

    let _wifi = match config_and_connect_wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop
    ) {
        Ok(inner) => inner,
        Err(err) => {
            println!("Could not connect to Wi-Fi network due to error: {:?}", err);
        }
    };
}