use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;
use gfx_xtra::draw_target::{Flushable, OwnedDrawTargetExt};
use mipidsi::{Builder, Orientation};
use core::mem;
use display_interface_spi::SPIInterfaceNoCS;
use esp_idf_hal::delay;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
    // mock_display::MockDisplay,
};

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    // the led is pin13 - prob should make my own peripherals module
    // (like aneometer.. prob gonna just copy his code) for the feather
    
    let mut led = PinDriver::output(peripherals.pins.gpio13)?;

    let display_dc: AnyOutputPin = peripherals.pins.gpio39.into();
    let display_rst: AnyOutputPin = peripherals.pins.gpio40.into();
    let display_cs: AnyOutputPin =  peripherals.pins.gpio7.into();
    // i2c power or "vdd" - for the display
    let display_power: Gpio21 = peripherals.pins.gpio21;

    // power on the display
    let mut vdd = PinDriver::output(display_power)?;
    vdd.set_high()?;
    mem::forget(vdd);
    // the spi bus driver - I honestly don't know why we need this but we do..
    let driver = std::rc::Rc::new(
                    SpiDriver::new(
                        peripherals.spi2,
                        peripherals.pins.gpio36,
                        peripherals.pins.gpio35,
                        Some(peripherals.pins.gpio37),
                        Dma::Disabled,
                    )
                    .unwrap());

    let spi_display = SpiDeviceDriver::new(
	driver,
	Some(display_cs),
	&SpiConfig::new().baudrate(10.MHz().into()),
    ).unwrap();
    let dc = PinDriver::output(display_dc).unwrap();
    let di = SPIInterfaceNoCS::new(spi_display, dc);
    let rst = PinDriver::output(display_rst).unwrap();
    let display = Builder::st7789_pico1(di)
        .with_display_size(135, 240)
        // set default orientation
        .with_orientation(Orientation::Landscape(true))
        // initialize
        .init(&mut delay::Ets, Some(rst))
        .unwrap();

    let mut display = display.owned_noop_flushing();
    display.clear(Rgb565::BLACK).unwrap();


    

    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let backlight : AnyOutputPin = peripherals.pins.gpio45.into();
    let mut bl = PinDriver::output(backlight).unwrap();
    bl.set_drive_strength(DriveStrength::I40mA).unwrap();
    bl.set_high().unwrap();
    mem::forget(bl); // TODO: For now
    // some text stuff - I'll just print out errors onto the screen lol
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    ).draw(&mut display).unwrap();
    
    loop {
        led.set_high()?;
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);

        led.set_low()?;
	let text = "Hello World";


        FreeRtos::delay_ms(500);
    }
}
