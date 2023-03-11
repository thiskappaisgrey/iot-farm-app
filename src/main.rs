use core::mem;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    // primitives::{
    //     Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    // },
    text::{Alignment, Text},
    // mock_display::MockDisplay,
};
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;
// wifi stuff
use anyhow::Context;
use esp_idf_hal::i2c::*;
use esp_idf_hal::prelude::*;
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
mod display;
mod peripherals;
use display::*;
use log::*;
use peripherals::*;
mod wifi;


use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi as SvcWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use embedded_svc::{
    http::{client::Client as HttpClient, Method, Status},
    io::Write,
    utils::io,
};
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

// wifi ssid and password

mod soil;
// TODO create the display
const SOIL_ADDRESS: u8 = 0x36;

// TODO then, figure out how to use the i2c soil sensor
fn soil_main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // TODO clean this code later, since I'd prob just want to copy anemometer's stuff
    // let peripherals = Peripherals::take().unwrap();
    let peripherals = FeatherPeripherals::take();
    let mut led = PinDriver::output(peripherals.led)?;
    debug!("Powering on the vdd");
    let mut vdd = PinDriver::output(peripherals.power)?;
    vdd.set_high()?;
    mem::forget(vdd);

    info!("initializing display");
    // I can use monadic code "and_then" but that's annoying to write
    // if display creation fails, I want to print an error
    let mut display =
        create_display(peripherals.display_peripherals, peripherals.spi_driver).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // // Enable the blacklight
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let mut bl = PinDriver::output(peripherals.backlight).unwrap();
    bl.set_drive_strength(DriveStrength::I40mA).unwrap();
    bl.set_high().unwrap();
    mem::forget(bl); // TODO: For now

    // TODO.. i2c - abstract this
    // I swapped the pins lmao
    // TODO power on the i2c
    let text = "Hello World";
    // some text stuff - I'll just print out errors onto the screen lol
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    info!("Starting I2C");
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(
        peripherals.i2c_peripherals.i2c,
        peripherals.i2c_peripherals.sda,
        peripherals.i2c_peripherals.scl,
        &config,
    )?;

    let soil_sensor = soil::SoilSensor::init(i2c);
    match soil_sensor {
        Ok(mut sensor) => {
            info!("Found soil sensor: {:#}", sensor.addr);
            loop {
                // maybe upwraps here is not a good idea..
                let temp = sensor.get_temp().unwrap();
                let cap = sensor.get_capacitance().unwrap();
                info!("Temperature is: {}C", temp);
                info!("Capacitance is: {}", cap);
                led.toggle()?;
                // delay
                FreeRtos::delay_ms(250);
            }
        }
        Err(err) => {
            warn!("couldn't find soil sensor: {:#}", err)
        }
    }

    Ok(())
}
// TODO  not sure why this doesnt'  work???
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // // wifi::Network::connect_wifi(peripherals.wifi_peripherals).unwrap();
    let peripherals = peripherals::FeatherPeripherals::take();

    let network = wifi::Network::init(peripherals.wifi_peripherals);
    match network {
	// TODO wifi is dropped here
	Ok(mut network) => {
	    FreeRtos::delay_ms(5000);
	    network.get_request().unwrap();
	    // // // make the requests
	    network.post_request().unwrap();
	}
	Err(e) => {
	    warn!("can't connet to network: {e}")
	}
    }
    
    let mut led = PinDriver::output(peripherals.led)?;
    
    loop {
	led.toggle()?;
        // delay
        FreeRtos::delay_ms(250);

    }
    Ok(())
}
