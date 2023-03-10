use core::mem;
use display_interface_spi::SPIInterfaceNoCS;
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
use esp_idf_hal::delay;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::*;
use gfx_xtra::draw_target::{Flushable, OwnedDrawTargetExt};
use mipidsi::{Builder, Orientation};
// wifi stuff
use anyhow::Context;
use core::fmt::Debug;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi as SvcWifi};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

// wifi ssid and password
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

struct DisplayPeripherals {
    dc: AnyOutputPin,
    rst: AnyOutputPin,
    cs: AnyOutputPin,
    power: Gpio21,
}
// TODO create the display
fn create_display(
    display_peripherals: DisplayPeripherals,
    driver: std::rc::Rc<SpiDriver<'static>>,
) -> anyhow::Result<impl Flushable<Color = Rgb565, Error = impl Debug + 'static> + 'static> {
    // let display_power: Gpio21 = peripherals.pins.gpio21;

    // power on the display
    let mut vdd = PinDriver::output(display_peripherals.power)?;
    vdd.set_high()?;
    mem::forget(vdd);

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

    // TODO change this
    Ok(display)
}

// TODO then, figure out how to use the i2c soil sensor
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    // TODO clean this code later, since I'd prob just want to copy anemometer's stuff
    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio13)?;

    // the spi bus driver - I honestly don't know why we need this but we do..
    let driver = std::rc::Rc::new(
        SpiDriver::new(
            peripherals.spi2,
            peripherals.pins.gpio36,
            peripherals.pins.gpio35,
            Some(peripherals.pins.gpio37),
            Dma::Disabled,
        )
        .unwrap(),
    );

    // let display_dc: AnyOutputPin = peripherals.pins.gpio39.into();
    // let display_rst: AnyOutputPin = peripherals.pins.gpio40.into();
    // let display_cs: AnyOutputPin =  peripherals.pins.gpio7.into();
    // i2c power or "vdd" - for the display
    // let display_power: Gpio21 = peripherals.pins.gpio21;

    let mut display = create_display(
        DisplayPeripherals {
            dc: peripherals.pins.gpio39.into(),
            rst: peripherals.pins.gpio40.into(),
            cs: peripherals.pins.gpio7.into(),
            power: peripherals.pins.gpio21,
        },
        driver,
    )
    .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    // Enable the blacklight
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let backlight: AnyOutputPin = peripherals.pins.gpio45.into();
    let mut bl = PinDriver::output(backlight).unwrap();
    bl.set_drive_strength(DriveStrength::I40mA).unwrap();
    bl.set_high().unwrap();
    mem::forget(bl); // TODO: For now

    // TODO use the log crate to prop error to display
    // Wifi stuff
    // TODO this doesn't actually work on eduroam
    // monitor: https://stackoverflow.com/questions/75540291/esp32-wifi-wpa2-enterprise-on-rust
    // low level bindings in esp-idf-sys might help..
    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        EspSystemEventLoop::take().expect("Failed to take system event loop"),
        Some(EspDefaultNvsPartition::take().expect("Failed to take default nvs partition")),
    )
    .expect("Failed to create esp wifi device");
    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            // See .cargo/config.toml to set WIFI_SSID and WIFI_PWD env variables
            ssid: SSID.into(),
            password: PASS.into(),
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }))
        .expect("Failed to set wifi driver configuration");
    // TODO before starting, I need to hack the WPA enterprise identity for eduroam
    // TODO https://github.com/espressif/esp-idf/blob/afbdb0f3ef195ab51690a64e22bfb8a5cd487914/examples/wifi/wifi_enterprise/main/wifi_enterprise_main.c

    // TODO figure out how to do enterprise, eduroam wifi
    
    wifi_driver.start().expect("Failed to start wifi driver");
    loop {
        match wifi_driver.is_started() {
            Ok(true) => {
                #[cfg(debug_assertions)]
                println!("Wifi driver started");
                break;
            }
            Ok(false) => {
                #[cfg(debug_assertions)]
                println!("Waiting for wifi driver to start")
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                println!("Error while starting wifi driver: {_e:?}")
            }
        }
    }

    loop {
        match wifi_driver.is_connected() {
            Ok(true) => {
                #[cfg(debug_assertions)]
                println!("Wifi is connected");
                break;
            }
            Ok(false) => {
                #[cfg(debug_assertions)]
                println!("Waiting for Wifi connection")
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                println!("Failed to connect wifi driver: {_e:?}")
            }
        }

        if let Err(_e) = wifi_driver.connect() {
            #[cfg(debug_assertions)]
            println!("Error while connecting wifi driver: {_e:?}")
        }

        FreeRtos::delay_ms(1000);
    }

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

    loop {
        led.set_high()?;
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);
        led.set_low()?;
        FreeRtos::delay_ms(500);
    }
}
