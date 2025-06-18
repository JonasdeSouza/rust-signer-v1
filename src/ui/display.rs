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


pub struct LcdController {
    tx: mpsc::Sender<String>,
    running: Arc<AtomicBool>,
}

impl LcdController {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        
        // Spawn the LCD thread
        let lcd_running = Arc::clone(&running);
        screen_thread(rx, lcd_running);
        
        // Give the thread time to initialize
        thread::sleep(Duration::from_millis(100));
        
        Self { tx, running }
    }
    
    /// Write a message to the LCD screen
    pub fn write_message(&self, message: &str) -> Result<(), mpsc::SendError<String>> {
        self.tx.send(format!("Message: {}", message))
    }
    
    /// Clear the LCD screen
    pub fn clear(&self) -> Result<(), mpsc::SendError<String>> {
        self.tx.send("Action: clear".to_string())
    }
    
    /// Turn off the LCD backlight
    pub fn turn_off_backlight(&self) -> Result<(), mpsc::SendError<String>> {
        self.tx.send("Action: backlight_off".to_string())
    }
    
    /// Turn on the LCD backlight
    pub fn turn_on_backlight(&self) -> Result<(), mpsc::SendError<String>> {
        self.tx.send("Action: backlight_on".to_string())
    }
    
    /// Display a multi-line message
    pub fn write_lines(&self, lines: &[&str]) -> Result<(), mpsc::SendError<String>> {
        let message = lines.join("\n");
        self.write_message(&message)
    }
    
    /// Shutdown the LCD thread and turn off the display
    pub fn shutdown(self) {
        // Turn off backlight before shutting down
        let _ = self.turn_off_backlight();
        thread::sleep(Duration::from_millis(50));
        
        // Signal thread to stop
        self.running.store(false, Ordering::SeqCst);
        
        // Drop the sender to close the channel
        drop(self.tx);
    }
}

// Update your ParsedInput enum to handle more actions
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

// Update your screen_thread to handle the new actions
fn screen_thread(rx: mpsc::Receiver<String>, running: Arc<AtomicBool>) {
    let builder = thread::Builder::new().stack_size(8192);
    builder
        .spawn(move || {
            let peripherals = Peripherals::take().expect("Failed to take peripherals");
            let spi = peripherals.spi2;

            let mut delay = Ets;

            // Initialize display components
            let rst = PinDriver::input_output_od(peripherals.pins.gpio23)
                .expect("Failed to initialize rst pin");
            let dc = PinDriver::input_output_od(peripherals.pins.gpio16)
                .expect("Failed to initialize dc pin");
            let mut backlight = PinDriver::output(peripherals.pins.gpio4)
                .expect("Failed to initialize backlight pin");
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

            let mut spi_buffer = [0u8; 256];
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
            
            while running.load(Ordering::SeqCst) {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(message) => {
                        match parse_input(&message) {
                            Some(ParsedInput::Message(value)) => {
                                display
                                    .clear(Rgb565::BLACK)
                                    .expect("Failed to clear display");
                                
                                // Handle multi-line text
                                let lines: Vec<&str> = value.split('\n').collect();
                                let character_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
                                
                                // Calculate starting Y position for centered text
                                let line_height = 22; // Adjust based on your font
                                let total_height = lines.len() as i32 * line_height;
                                let start_y = (display.bounding_box().size.height as i32 - total_height) / 2;
                                
                                for (i, line) in lines.iter().enumerate() {
                                    let y_pos = start_y + (i as i32 * line_height) + line_height/2;
                                    let position = Point::new(
                                        display.bounding_box().center().x,
                                        y_pos
                                    );
                                    
                                    Text::with_alignment(
                                        line,
                                        position,
                                        character_style,
                                        Alignment::Center,
                                    )
                                    .draw(&mut display)
                                    .expect("Failed to draw text");
                                }
                            }
                            Some(ParsedInput::Action(value)) => {
                                match value {
                                    "clear" => {
                                        display
                                            .clear(Rgb565::BLACK)
                                            .expect("Failed to clear display");
                                    }
                                    "backlight_off" => {
                                        backlight.set_low().expect("Failed to turn off backlight");
                                    }
                                    "backlight_on" => {
                                        backlight.set_high().expect("Failed to turn on backlight");
                                    }
                                    _ => println!("Unknown action: {}", value),
                                }
                            }
                            None => println!("Invalid input: {}", message),
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Continue checking if thread should stop
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        println!("Channel disconnected, stopping LCD thread");
                        break;
                    }
                }
            }
            
            // Clean up before exiting
            display.clear(Rgb565::BLACK).ok();
            backlight.set_low().ok();
            println!("LCD thread stopped");
        })
        .expect("Thread spawn failed");
}

// Simple helper functions if you prefer not to use the struct
pub fn write_lcd(controller: &LcdController, message: &str) {
    controller.write_message(message)
        .expect("Failed to write to LCD");
}

pub fn turn_off_lcd(controller: LcdController) {
    controller.shutdown();
}

// Example usage
pub fn example_display() -> Result<()> {
    // Create the LCD controller
    let lcd = LcdController::new();
    
    // Simple message
    lcd.write_message("Hello ESP32!")?;
    thread::sleep(Duration::from_secs(2));
    
    // Multi-line message
    lcd.write_lines(&[
        "Bitcoin Signer",
        "Ready to sign",
        "transactions"
    ])?;
    thread::sleep(Duration::from_secs(2));
    
    // Clear screen
    lcd.clear()?;
    thread::sleep(Duration::from_secs(1));
    
    // Another message
    lcd.write_message("Signing...")?;
    thread::sleep(Duration::from_secs(2));
    
    // Turn off backlight
    lcd.turn_off_backlight()?;
    thread::sleep(Duration::from_secs(1));
    
    // Turn on backlight
    lcd.turn_on_backlight()?;
    lcd.write_message("Done!")?;
    thread::sleep(Duration::from_secs(2));
    
    // Shutdown the LCD
    lcd.shutdown();
    
    println!("Main finished.");
    Ok(())
}

// For your Bitcoin transaction signing workflow, you could use it like:
pub fn display_transaction_info(lcd: &LcdController, tx_id: &str, amount: &str) {
    lcd.write_lines(&[
        "Transaction:",
        &format!("ID: {}...", &tx_id[..8]),
        &format!("Amount: {}", amount),
        "Press OK to sign"
    ]).expect("Failed to display transaction info");
}