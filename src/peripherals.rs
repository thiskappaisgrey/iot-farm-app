use esp_idf_hal::gpio::*;

pub struct DisplayPeripherals {
    pub dc: AnyOutputPin,
    pub rst: AnyOutputPin,
    pub cs: AnyOutputPin,
    pub power: Gpio21,
}
