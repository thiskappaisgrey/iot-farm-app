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
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi as SvcWifi};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use esp_idf_hal::i2c::*;
use esp_idf_hal::prelude::*;
mod display;
mod peripherals;
use display::*;
use log::*;
use peripherals::*;

// wifi ssid and password
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

mod soil;
// TODO create the display
const SOIL_ADDRESS: u8 = 0x36;

// TODO then, figure out how to use the i2c soil sensor
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // TODO clean this code later, since I'd prob just want to copy anemometer's stuff
    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio13)?;

    // info!("initializing display");
    // // the spi bus driver - I honestly don't know why we need this but we do..
    // let driver = std::rc::Rc::new(
    //     SpiDriver::new(
    //         peripherals.spi2,
    //         peripherals.pins.gpio36,
    //         peripherals.pins.gpio35,
    //         Some(peripherals.pins.gpio37),
    //         Dma::Disabled,
    //     )
    //     .unwrap(),
    // );
    // let mut display = create_display(
    //     DisplayPeripherals {
    //         dc: peripherals.pins.gpio39.into(),
    //         rst: peripherals.pins.gpio40.into(),
    //         cs: peripherals.pins.gpio7.into(),
    //         power: peripherals.pins.gpio21,
    //     },
    //     driver,
    // )
    // .unwrap();

    // display.clear(Rgb565::BLACK).unwrap();

    // // Enable the blacklight
    // let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    // let backlight: AnyOutputPin = peripherals.pins.gpio45.into();
    // let mut bl = PinDriver::output(backlight).unwrap();
    // bl.set_drive_strength(DriveStrength::I40mA).unwrap();
    // bl.set_high().unwrap();
    // mem::forget(bl); // TODO: For now
    
    // TODO.. i2c - abstract this
    // I swapped the pins lmao
    let scl = peripherals.pins.gpio41;
    let sda = peripherals.pins.gpio42;
    // TODO power on the i2c
    let i2c_power = peripherals.pins.gpio21;
    info!("Toggling the i2c power pin");
    let mut i2c_power_status = PinDriver::output(i2c_power).expect("Failed to get the  i2c power pin");
    // read the i2c_power pin & toggle it
    let power = i2c_power_status.is_set_high();
    info!("i2c power is: {}", power);
    // write t
    // toggle the pin
    i2c_power_status.toggle().expect("Failed to toggle the i2c power pin");
    
    info!("Starting I2C");
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;
    let soil_sensor = soil::SoilSensor::init(i2c);
    match soil_sensor {
	Ok(mut sensor) => {
	    info!("Found soil sensor: {:#}", sensor.addr);
	    loop  {
		// maybe upwraps here is not a good idea..
		let temp = sensor.get_temp().unwrap();
		let cap = sensor.get_capacitance().unwrap();
		info!("Temperature is: {}C", temp);
		info!("Capacitance is: {}", cap);
		// delay
		FreeRtos::delay_ms(250);
	    }
	}
	Err(err) => {
	    warn!("couldn't find soil sensor: {:#}", err);
	}
    }
    
    // let text = "Hello World";
    // some text stuff - I'll just print out errors onto the screen lol
    // Text::with_alignment(
    //     text,
    //     display.bounding_box().center() + Point::new(0, 15),
    //     character_style,
    //     Alignment::Center,
    // )
    // .draw(&mut display)
    // .unwrap();

    loop {
	// info!("toggling led");
        // led.set_high()?;
        // // we are sleeping here to make sure the watchdog isn't triggered
        // FreeRtos::delay_ms(500);
        // led.set_low()?;
        // FreeRtos::delay_ms(500);
    }
}
