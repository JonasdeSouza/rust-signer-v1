use std::fmt::Pointer;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Error, Result};

use bitcoin::secp256k1::SecretKey;
use esp_idf_svc::hal::peripheral::{Peripheral, PeripheralRef};
use esp_idf_svc::hal::{
    delay::Ets, gpio::*, peripheral, peripherals::Peripherals, prelude::*, spi::config::*, spi::*,
    units::FromValueType,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::*};
use mipidsi::interface::{self, SpiInterface};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::*,
    prelude::*,
    text::*,
};
use mipidsi::{models::ST7789, options::*, Builder};

use bitcoin::consensus::{deserialize, encode, serialize};
use bitcoin::psbt::{self, serialize};
use bitcoin::{Psbt, Transaction};
use nvs::memory::nvs_example;
use std::io::{self, Write};
use ui::display::{self, example_display};

extern crate bitcoin;

use hex;

//mod comm;
mod bitcoin_mod;
mod nvs;
mod ui;

//use comm::wifi::config_and_connect_wifi;
use bitcoin_mod::signature::sig_example;

#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn initialize_runtime() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
}

fn main() {
    initialize_runtime();
    example_display();
     nvs_example();
    //config_and_connect_wifi();
    sig_example();
}
