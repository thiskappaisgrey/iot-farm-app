use esp_idf_hal::{spi::*, gpio::*, prelude::*, delay};
use crate::peripherals::*;
use display_interface_spi::SPIInterfaceNoCS;
use gfx_xtra::draw_target::{Flushable, OwnedDrawTargetExt};
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
use core::fmt::Debug;
use std::mem;
use mipidsi::{Builder, Orientation};

// create the display object
pub fn create_display(
    display_peripherals: DisplayPeripherals,
    driver: std::rc::Rc<SpiDriver<'static>>,
) -> anyhow::Result<impl Flushable<Color = Rgb565, Error = impl Debug + 'static> + 'static> {
    let spi_display = SpiDeviceDriver::new(
        driver,
        Some(display_peripherals.cs),
        &SpiConfig::new().baudrate(10.MHz().into()),
    )
    .unwrap();

    let dc = PinDriver::output(display_peripherals.dc).unwrap();
    let di = SPIInterfaceNoCS::new(spi_display, dc);
    let rst = PinDriver::output(display_peripherals.rst).unwrap();
    let display = Builder::st7789_pico1(di)
        .with_display_size(135, 240)
        // set default orientation
        .with_orientation(Orientation::Landscape(true))
        // initialize
        .init(&mut delay::Ets, Some(rst))
        .unwrap();
    let display = display.owned_noop_flushing();

    Ok(display)
}

// Write the text to the center of the screen - clearing the screen first
pub fn write_text_center<D>(display: &mut D, text: &str) -> Result<(), D::Error>
where
    D: Flushable<Color = Rgb565>,
    D::Error: Debug,
{
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    // clear the display first
    display.clear(Rgb565::BLACK).unwrap();
    Text::with_alignment(
	text,
	display.bounding_box().center() + Point::new(0, 15),
	character_style,
	Alignment::Center,
    )
	.draw(display)?;
    Ok(())
}
