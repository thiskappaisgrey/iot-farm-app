// TODO figure out how logging works.. https://esp-rs.github.io/esp-idf-svc/esp_idf_svc/log/struct.EspLogger.html
// TODO figure out how to connect to wifi
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

// TODO the debug port doesn't get created..
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio13).unwrap();
    // println!("hello");
    info!("ESP32-Logging");
    loop {
	// esp_println::println!("hello");
	info!("starting to toggle led");
	// TODO not sure how to change the logging of "expect"
	led.toggle().expect("Failed to toggle LED");
	FreeRtos::delay_ms(500);
	info!("toggling led");
    }
}
