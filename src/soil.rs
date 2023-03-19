// code copied / modified from:  https://github.com/cfsamson/embedded-stemma-soil-sensor
use esp_idf_hal::i2c::*;
use esp_idf_hal::delay::{self, FreeRtos, BLOCK, NON_BLOCK};
use log::*;
use thiserror::Error;

// Registers
const SENSOR_START_ADDR: u8 = 0x35; // Battery on this is 0xB.. So go past that to scan lmao
const SENSOR_END_ADDR: u8 = 0x49; // scan to pretty high
// TODO not sure if this matters too much, but it doesn't match for some reason
pub const SEESAW_HW_ID_CODE: u8 = 0x55;
pub const SEESAW_STATUS_BASE: u8 = 0x00;
pub const SEESAW_TOUCH_BASE: u8 = 0x0F;
pub const SEESAW_STATUS_HW_ID: u8 = 0x01;
pub const SEESAW_STATUS_TEMP: u8 = 0x04;
pub const SEESAW_TOUCH_CHANNEL_OFFSET: u8 = 0x10;
const STD_PROCESSING_DELAY_MICROS: u32 = 250; // shorter delay than 200ms, so use Ets



pub struct SoilSensor {
    pub addr: u8,
    i2c: I2cDriver<'static>
}
#[derive(Debug, Error)]
pub enum SoilSensErr {
    #[error("Couldn't get a valid reading from the moisture sensor.")]
    MoistureReadErr,
    #[error("Couldn't connect to the sensor.")]
    HwNotFound,
    #[error("Invalid Hardware ID. Expected {0}, got {1}")]
    HardwareMismatch(u8, u8),
    #[error("invalid slave address: {0:#X}")]
    InvalidSlaveAddress(u16),
    // #[error("I2C connection error. {source}")]
    // I2C {
    //     #[from]
    //     source: i2c::Error,
    // },
}

// https://github.com/cfsamson/embedded-stemma-soil-sensor/blob/master/src/lib.rs
// https://github.com/esp-rs/esp-idf-hal/blob/master/examples/i2c_ssd1306.rs
impl SoilSensor {
    // Create a soil sensor using the builder pattern
    pub fn init(mut i2c: I2cDriver<'static>) -> anyhow::Result<Self> {
	info!("initializing soil sensor");
	// Scan for the soil sensor
	let mut hw_found = false;
	let mut addr = 0;
	for adr in SENSOR_START_ADDR..=SENSOR_END_ADDR {
	    info!("looking for sensor at addr: {}", adr);
	    let res = try_read_hw(&mut i2c, adr);
	    match res {
		Ok(hw_code) => {
		    hw_found = true;
		    addr = adr;
		    // check that the hardware code is correct - you can also check the product code as well but yeah
		    if SEESAW_HW_ID_CODE == hw_code {
			info!("Hardware code matches");
		    } else {
			return Err(SoilSensErr::HardwareMismatch(SEESAW_HW_ID_CODE, hw_code).into());
		    }
		    
		    break;
		}
		Err(err) => {
		    warn!("Error reading hardware code: {:#}", err);
		    continue;
		}
	    }

	}
	if !hw_found {
	    return Err(SoilSensErr::HwNotFound.into());
	}
	
	Ok(SoilSensor {
	    i2c,
	    addr
	})


    }
    // Get the temperature reading from the soil sensor
    pub fn get_temp(&mut self) -> anyhow::Result<f32> {
        let l_reg = SEESAW_STATUS_BASE;
        let h_reg = SEESAW_STATUS_TEMP;
        let delay = STD_PROCESSING_DELAY_MICROS;

        let mut buffer = [0u8; 4];
        self.read(l_reg, h_reg, &mut buffer[..], delay)?;
        let tmp_val = i32::from_be_bytes(buffer) as f32;

        // See: https://github.com/adafruit/Adafruit_Seesaw/blob/8728936a5d1a0a7bf2887a82adb0828b70556a45/Adafruit_seesaw.cpp#L664
        let temp_celsius = (1.0 / (1u32 << 16) as f32) * tmp_val;
        Ok(temp_celsius)
    }
    // get the capacitance reading from the soil sensor
    pub fn get_capacitance(&mut self) -> anyhow::Result<u16> {
        let l_reg: u8 = SEESAW_TOUCH_BASE;
        let h_reg: u8 = SEESAW_TOUCH_CHANNEL_OFFSET;
        let mut buff = [0u8; 2];
        let mut retry_counter = 0;

        while retry_counter < 3 {
	    FreeRtos::delay_us(1000);
            // NB! Setting this to 1000 (like in the C library) errors.
            if let Err(e) = self.read(l_reg, h_reg, &mut buff, 5000) {
                debug!("Error reading capacitance: {}. Retry: {}", e, retry_counter + 1);
                retry_counter += 1;
                continue;
            }

            // A read before the chip is ready will be 0xFFFF
            let cap = u16::from_be_bytes(buff);
            if cap < u16::max_value() {
                return Ok(cap);
            }
        }

        Err(SoilSensErr::MoistureReadErr.into())
    }
    // read from an i2c register after connecting to the device
    pub fn read(&mut self, reg_low: u8, reg_high: u8, buff: &mut [u8], delay_us: u32) -> anyhow::Result<()> {
	self.i2c.write(self.addr, &[reg_low, reg_high], BLOCK)?;
	esp_idf_hal::delay::FreeRtos::delay_us(delay_us);
	self.i2c.read(self.addr, buff, BLOCK)?;
	Ok(())
    }
}

// try to find hardware on i2c address
fn try_read_hw(i2c: &mut I2cDriver, addr: u8) -> anyhow::Result<u8>{
    let reg_low = SEESAW_STATUS_BASE;
    let reg_high = SEESAW_STATUS_HW_ID;
    i2c.write(addr, &[reg_low, reg_high], BLOCK)?;
    let mut buffer = [0];
    esp_idf_hal::delay::FreeRtos::delay_us(STD_PROCESSING_DELAY_MICROS);
    i2c.read(addr, &mut buffer, BLOCK)?;
    // chan.read(&mut buffer)?;
    Ok(buffer[0])
}

