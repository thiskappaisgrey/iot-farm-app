use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi as SvcWifi};
use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use embedded_svc::{
    http::{client::Client as HttpClient, Method, Status},
    io::Write,
    utils::io,
};
use crate::peripherals::WifiPeripherals;
use log::*;
pub struct Network {
    // client:  HttpClient<EspHttpConnection>,
    _wifi: EspWifi<'static>
}
use anyhow::anyhow;


/* Configure all of these environment variables in the
<project-root>/.cargo/config.toml
Example:
[env]
WIFI_SSID="123"
WIFI_PASS="pass"
SERVER_ADDR="http://<ip>"
 */

const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");
const SERVER_ADDR: &str = env!("SERVER_ADDR");

impl Network {
    // Use the builder pattern to build a "Network" object. Tbh, dont' really need this.
    pub fn init(peripherals: WifiPeripherals) -> anyhow::Result<Self> {
	let _wifi = Self::connect_wifi(peripherals)?;
	Ok(Network {
	    _wifi
	})
    }
    
    // Connect to the wifi and return the reference
    pub fn connect_wifi(peripherals: WifiPeripherals) -> anyhow::Result<EspWifi<'static>> {
        // Wifi stuff
        // monitor: https://stackoverflow.com/questions/75540291/esp32-wifi-wpa2-enterprise-on-rust
        // low level bindings in esp-idf-sys might help..
	let eventloop = EspSystemEventLoop::take()?;
	let event_partition = EspDefaultNvsPartition::take()?;
	let mut wifi_driver = EspWifi::new(
	    peripherals.modem,
	    eventloop,
	    Some(event_partition),
	)?;
	    
        wifi_driver
            .set_configuration(&Configuration::Client(ClientConfiguration {
                // See .cargo/config.toml to set WIFI_SSID and WIFI_PWD env variables
                ssid: SSID.into(),
                password: PASS.into(),
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            }))?;
            
        // TODO before starting, I need to hack the WPA enterprise identity for eduroam
        // TODO https://github.com/espressif/esp-idf/blob/afbdb0f3ef195ab51690a64e22bfb8a5cd487914/examples/wifi/wifi_enterprise/main/wifi_enterprise_main.c
	wifi_driver.start()?;
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
	let mut tries = 0;
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
	    if tries > 10 {
		return Err(anyhow!("Tries exceeded 10. Stopping connection to wifi."));
	    }
	    tries +=  1;
	    FreeRtos::delay_ms(1000);
	    
	}


	// wifi_driver.connect()?;
	Ok(wifi_driver)

    }
    // Make a get request to the sensors server
    pub fn get_request(&mut self) -> anyhow::Result<()> {
	let mut client = HttpClient::wrap(EspHttpConnection::new(&HttpConfiguration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // Needed for HTTPS support
            ..Default::default()
	})?);

	// Prepare headers and URL
	let headers = [("accept", "text/plain"), ("connection", "close")];
	// get the data I guess
	let url = format!("{SERVER_ADDR}/data");
	// let url = "http://ifconfig.net/";

	// Send request
	//
	// Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
	let request = client.request(Method::Get, &url, &headers)?;
	info!("-> GET {url}");
	
	// there's an error here
	let mut response = request.submit()?;

	// Process response
	let status = response.status();
	info!("<- {status}");
	let (_headers, mut body) = response.split();
	let mut buf = [0u8; 1024];
	let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
	info!("Read {bytes_read} bytes");
	match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => info!(
		"Response body (truncated to {} bytes): {:?}",
		buf.len(),
		body_string
            ),
            Err(e) => error!("Error decoding response body: {e}"),
	};

	// Drain the remaining response bytes
	while body.read(&mut buf)? > 0 {}

	Ok(())
    }
    // make a put request to my flask server and send logging data there
    pub fn put_request(&mut self, temp: u16, capacitance: u16) -> anyhow::Result<()> {
	let mut client = HttpClient::wrap(EspHttpConnection::new(&HttpConfiguration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach), // Needed for HTTPS support
            ..Default::default()
	})?);
	
	// Prepare headers and URL
	// let content_length_header = format!("{}", payload.len());
	let headers = [
            ("accept", "text/plain"),
            ("content-type", "text/plain"),
            ("connection", "close"),
	];
	// TODO 
	let url = format!("{SERVER_ADDR}/api/addsensorlog/?sensor_id=1&temperature={temp}&capacitance={capacitance}&api_key=w23cs190bherbfarmproject");

	// Send request
	let mut request = client.put(&url, &headers)?;
	// request.write_all(payload)?;
	request.flush()?;
	info!("-> POST {url}");
	let mut response = request.submit()?;

	// Process response
	let status = response.status();
	info!("<- {status}");
	let (_headers, mut body) = response.split();
	let mut buf = [0u8; 1024];
	let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
	info!("Read {bytes_read} bytes");
	match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => info!(
		"Response body (truncated to {} bytes): {:?}",
		buf.len(),
		body_string
            ),
            Err(e) => error!("Error decoding response body: {e}"),
	};

	// Drain the remaining response bytes
	while body.read(&mut buf)? > 0 {}

	Ok(())
    }

}

