use esp_idf_hal::{gpio::Pin, prelude::Peripherals};

mod espcam;

mod con {
    use std::str::FromStr;

    use anyhow::Result;
    use esp_idf_hal::{modem, prelude::Peripherals};
    use esp_idf_svc::{
        espnow::PeerInfo,
        eventloop::EspSystemEventLoop,
        nvs::EspDefaultNvsPartition,
        wifi::{ClientConfiguration, Configuration, EspWifi},
    };
    use heapless;

    pub fn associate(modem: modem::Modem) -> Result<EspWifi<'static>> {
        let sl = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();
        let mut wifi = EspWifi::new(modem, sl, Some(nvs))?;
        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: heapless::String::from_str("test_network").unwrap(),
            password: heapless::String::from_str("1234567890").unwrap(),
            ..Default::default()
        }))?;
        wifi.start()?;
        wifi.connect()?;

        Ok(wifi)
    }
}

mod tbup {}
mod imgup {
    use anyhow::Result;

    pub fn send(data: &[u8]) -> Result<()> {
        Ok(())
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let pfs = Peripherals::take().unwrap();
    let cpf = espcam::CameraPeriphs {
        i2c: pfs.i2c0,
        sda: pfs.pins.gpio13,
        sdl: pfs.pins.gpio3,
        pin_pwdn: pfs.pins.gpio9.pin(),
        pin_reset: pfs.pins.gpio11.pin(),
        pin_xclk: pfs.pins.gpio8.pin(),
        pin_d7: pfs.pins.gpio12.pin(),
        pin_d6: pfs.pins.gpio18.pin(),
        pin_d5: pfs.pins.gpio17.pin(),
        pin_d4: pfs.pins.gpio15.pin(),
        pin_d3: pfs.pins.gpio6.pin(),
        pin_d2: pfs.pins.gpio4.pin(),
        pin_d1: pfs.pins.gpio5.pin(),
        pin_d0: pfs.pins.gpio7.pin(),
        pin_vsync: pfs.pins.gpio10.pin(),
        pin_href: pfs.pins.gpio40.pin(),
        pin_pclk: pfs.pins.gpio16.pin(),
    };

    log::info!("Intercepting power...");
    log::info!("Initializing and connecting to WiFi...");
    let wifi = con::associate(pfs.modem).unwrap();
    log::info!("Configuring I2C...");
    log::info!("Configuring camera...");
    let camera = espcam::Camera::configure(cpf).unwrap();
    log::info!("Capturing image...");
    let fb = camera.get_data().unwrap();
    log::info!("Captured data: {} bytes", fb.slice().len());
    log::info!("Uploading image to HTTP endpoint...");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        log::info!("tick...");
    }
}
