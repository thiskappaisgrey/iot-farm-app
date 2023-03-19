use std::rc::Rc;
use esp_idf_hal::{gpio::*, spi::*, prelude::*, i2c::{I2c, I2C0}, modem::Modem};

// code adapted / copied from https://github.com/taunusflieger/anemometer

pub struct DisplayPeripherals {
    pub dc: AnyOutputPin,
    pub rst: AnyOutputPin,
    pub cs: AnyOutputPin,
    // pub power: Gpio21,
}

pub struct I2cPeripherals {
    pub    scl: AnyIOPin,
    pub sda: AnyIOPin,
    pub i2c: I2C0
}

pub struct WifiPeripherals {
    pub modem: Modem
}
pub struct FeatherPeripherals {
    pub display_peripherals: DisplayPeripherals,
    pub spi_driver: Rc<SpiDriver<'static>>,
    pub led: AnyOutputPin,
    pub i2c_peripherals: I2cPeripherals,
    pub power: Gpio21, // power pin or vdd
    pub backlight: AnyOutputPin,
    pub wifi_peripherals: WifiPeripherals
}

impl FeatherPeripherals {
    // Get all of the peripherals of the feather using the builder pattern
    pub fn take() -> Self {
	let peripherals = Peripherals::take().unwrap();
	let spi_driver = std::rc::Rc::new(
            SpiDriver::new(
		peripherals.spi2,
		peripherals.pins.gpio36,
		peripherals.pins.gpio35,
		Some(peripherals.pins.gpio37),
		Dma::Disabled,
            )
		.unwrap(),
	);
	let display_peripherals = 
            DisplayPeripherals {
		dc: peripherals.pins.gpio39.into(),
		rst: peripherals.pins.gpio40.into(),
		cs: peripherals.pins.gpio7.into(),
		// Both are power pins I guess
		// power: peripherals.pins.gpio21,
            };
	let i2c_peripherals = I2cPeripherals {
	    scl: peripherals.pins.gpio41.into(),
	    sda: peripherals.pins.gpio42.into(),
	    i2c: peripherals.i2c0
	    // power: peripherals.pins.gpio21.into(),

	};
	let modem = peripherals.modem;
	let wifi_peripherals = WifiPeripherals {
	    modem
	};
	FeatherPeripherals {
	    spi_driver,
	    display_peripherals,
	    i2c_peripherals,
	    led: peripherals.pins.gpio13.into(),
	    power: peripherals.pins.gpio21, // power is gpio 21
	    backlight: peripherals.pins.gpio45.into(),
	    wifi_peripherals
	}
    }
}
