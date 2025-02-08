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

enum ParsedInput<'a> {
    Message(&'a str),
    Action(&'a str),
}

fn parse_input(input: &str) -> Option<ParsedInput> {
    if let Some((key, value)) = input.split_once(": ") {
        match key {
            "Message" => Some(ParsedInput::Message(value)),
            "Action" => Some(ParsedInput::Action(value)),
            _ => None, // Unrecognized key
        }
    } else {
        None // Input does not match the expected format
    }
}
//rx: mpsc::Receiver<String>,
fn screen_thread(rx: mpsc::Receiver<String>, running: Arc<AtomicBool>) {
    let peripherals = Peripherals::take().expect("Failed to take peripherals");
    let spi = peripherals.spi2;

    let mut delay = Ets;

    // Initialize display components
    let rst =
        PinDriver::input_output_od(peripherals.pins.gpio23).expect("Failed to initialize rst pin");
    let dc =
        PinDriver::input_output_od(peripherals.pins.gpio16).expect("Failed to initialize dc pin");
    let mut backlight =
        PinDriver::output(peripherals.pins.gpio4).expect("Failed to initialize backlight pin");
    let sclk = peripherals.pins.gpio18;
    let sdo = peripherals.pins.gpio19;
    let cs = peripherals.pins.gpio5;

    let spi_driver = SpiDriver::new(
        spi,
        sclk,
        sdo,
        None::<AnyIOPin>,
        &DriverConfig::new().dma(Dma::Channel1(240 * 135)),
    )
    .expect("Failed to initialize SPI driver");

    let spi_device_driver = SpiDeviceDriver::new(
        spi_driver,
        Some(cs),
        &esp_idf_svc::hal::spi::config::Config::new()
            .baudrate(26.MHz().into())
            .data_mode(MODE_3),
    )
    .expect("Failed to initialize SPI device driver");

    let mut spi_buffer = [0u8; 256]; // Buffer of 256 bytes
    let spi_interface = SpiInterface::new(spi_device_driver, dc, &mut spi_buffer);
    let mut display = Builder::new(ST7789, spi_interface)
        .display_size(135, 240)
        .display_offset(52, 40)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut delay)
        .expect("Failed to initialize display");

    backlight.set_high().expect("Failed to set backlight high");

    println!("Display initialized.");
    thread::spawn(move || {
        println!("LCD thread started");
        // Keep the thread running as long as `running` is true
        while running.load(Ordering::SeqCst) {
            match rx.recv() {
                Ok(message) => {
                    println!("LCD received: {}", message);

                    //             match parse_input(&message) {
                    //                 Some(ParsedInput::Message(value)) => {
                    //                     println!("Message received: {}", value);
                    // display
                    //     .clear(Rgb565::BLACK)
                    //     .expect("Failed to clear display");
                    //                     let character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
                    //                     match Text::with_alignment(
                    //                         value,
                    //                         display.bounding_box().center(),
                    //                         character_style,
                    //                         Alignment::Center,
                    //                     )
                    //                     .draw(&mut display)
                    //                     {
                    //                         Ok(_) => println!("Text drawn successfully."),
                    //                         Err(e) => println!("Error drawing text: {:?}", e),
                    //                     }
                    //                 }
                    //                 Some(ParsedInput::Action(value)) => {
                    //                     println!("Action received: {}", value);
                    //                     // Handle action here
                    //                 }
                    //                 None => println!("Invalid input: {}", message),
                    //             //}
                }
                Err(_) => {
                    println!("Error receiving message.");
                    break; // Exit if receiving fails (channel disconnected)
                }
            }
        }
    });
}

pub fn example_display() -> Result<()> {

    let (tx_lcd, rx_lcd) = mpsc::channel();
    let running = Arc::new(AtomicBool::new(true));

    // Spawn the LCD thread and pass the `running` flag to keep it active
    let lcd_running = Arc::clone(&running);
    screen_thread(rx_lcd, lcd_running);

    // Send messages to the LCD thread
    tx_lcd
        .send(format!("Message: Hello"))
        .expect("Failed to send message: Hello");
    thread::sleep(Duration::from_secs(2));
    tx_lcd
        .send(format!("Message: Goodbye"))
        .expect("Failed to send message: Goodbye");

    // Stop the thread after some time by setting `running` to false
    thread::sleep(Duration::from_secs(2));
    running.store(false, Ordering::SeqCst); // Signal to stop the thread
    drop(tx_lcd); // Close the channel

    // Wait for the LCD thread to finish before exiting main
    thread::sleep(Duration::from_secs(1));

    println!("Main finished.");

    Ok(())
}
