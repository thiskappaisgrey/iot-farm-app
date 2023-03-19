use core::mem;
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
use esp_idf_hal::prelude::*;
// wifi stuff
use esp_idf_hal::i2c::*;
use esp_idf_hal::delay::FreeRtos;
mod display;
mod peripherals;
use display::*;
use gfx_xtra::draw_target::Flushable;
use log::*;
use peripherals::*;
mod network;
use soil::SoilSensor;
use network::Network;
mod soil;

// The  "main loop" after getting all of the peripherals
fn main_loop(mut network:  Network, mut soil_sensor: SoilSensor, mut led: PinDriver<AnyOutputPin, Output>, display: &mut impl Flushable <Color = Rgb565, Error = impl core::fmt::Debug + 'static>) -> anyhow::Result<()> {
    loop {
        // maybe upwraps here is not a good idea..
	
        let temp: u16 = unsafe { soil_sensor.get_temp().unwrap().to_int_unchecked::<u16>() };
        let cap = soil_sensor.get_capacitance().unwrap();
        info!("Temperature is: {}C", temp);
        info!("Capacitance is: {}", cap);
	display::write_text_center( display, format!("Temp: {temp}C and Cap: {cap}").as_str()).unwrap();
        led.toggle()?;
	// network requests have some panics..
	if let Err(e) = network.put_request(temp, cap) {
	    error!("error with request {e}");
	}
        // delay for 10 seconds to not spam the server
        FreeRtos::delay_ms(5000);
    }

}



fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // TODO clean this code later, since I'd prob just want to copy anemometer's stuff
    // let peripherals = Peripherals::take().unwrap();
    let peripherals = FeatherPeripherals::take();
    let led = PinDriver::output(peripherals.led)?;
    debug!("Powering on the vdd");
    let mut vdd = PinDriver::output(peripherals.power)?;
    vdd.set_high()?;
    mem::forget(vdd);

    info!("initializing display");
    // I can use monadic code "and_then" but that's annoying to write
    // if display creation fails, I want to print an error
    
    // TODO abstract the display in a builder pattern
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
    let text = "Hello! Initializing wifi!";
    // TODO abstract into display module
    display::write_text_center(&mut display, text).unwrap();
    
    info!("Starting I2C");
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(
        peripherals.i2c_peripherals.i2c,
        peripherals.i2c_peripherals.sda,
        peripherals.i2c_peripherals.scl,
        &config,
    )?;

    info!("Starting network and soil sensor");
    
    // TODO stop the program when network fails - I can use map_or_else or something else to handle errors
    // TODO for now, I'll just write monadic code
    let val: anyhow::Result<()> = network::Network::init(peripherals.wifi_peripherals).and_then(|network| {
	soil::SoilSensor::init(i2c).and_then(|soil_sensor| {
	    main_loop(network, soil_sensor,  led, &mut display)
	})
    });
    // // TODO unwrap the error - since main_loop loops, the code will (hopefully) never get here unless there is an error
    match val {
	Ok(()) => {
	    info!("unexpected");
	    loop {}
	}
	Err(err) => {
	    // TODO probably want some sort of text wrapping.. errors dont' look good on screen
	    let txt: String = err.to_string();
	    error!("Error in initializing the main loop: {}", txt);
	    display::write_text_center(&mut display, format!("Error in initializing main loop: {txt}").as_str()).unwrap();
	}
   } 
    loop {}

}

