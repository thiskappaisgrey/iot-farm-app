// TODO

fn wifi() {
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
    
}
