use esp_idf_hal::{gpio::Pin, prelude::Peripherals};
use esp_idf_sys::ESP_LWIP_ARP;

mod con;
mod espcam;
mod imgup;

mod tbup {}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    pass: &'static str,
    #[default("")]
    uri: &'static str,
    #[default("")]
    apikey: &'static str,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let pfs = Peripherals::take().unwrap();

    log::info!("Intercepting power...");
    log::info!("Initializing and connecting to WiFi...");
    let wifi = con::associate(pfs.modem, CONFIG.ssid, CONFIG.pass).unwrap();
    log::info!("Configuring I2C...");
    log::info!("Configuring camera...");
    let cpf = espcam::CameraPeriphs {
        i2c: pfs.i2c0,
        sda: pfs.pins.gpio13.into(),
        sdl: pfs.pins.gpio3.into(),
        pin_pwdn: pfs.pins.gpio9.into(),
        pin_reset: pfs.pins.gpio11.into(),
        pin_xclk: pfs.pins.gpio8.into(),
        pin_d7: pfs.pins.gpio12.into(),
        pin_d6: pfs.pins.gpio18.into(),
        pin_d5: pfs.pins.gpio17.into(),
        pin_d4: pfs.pins.gpio15.into(),
        pin_d3: pfs.pins.gpio6.into(),
        pin_d2: pfs.pins.gpio4.into(),
        pin_d1: pfs.pins.gpio5.into(),
        pin_d0: pfs.pins.gpio7.into(),
        pin_vsync: pfs.pins.gpio10.into(),
        pin_href: pfs.pins.gpio40.into(),
        pin_pclk: pfs.pins.gpio16.into(),
    };

    let camera = espcam::Camera::configure(cpf).unwrap();

    log::info!("Capturing image...");
    let fb = camera.get_data().unwrap();
    log::info!("Captured data: {} bytes", fb.slice().len());

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        log::info!("Uploading image to HTTP endpoint...");
        imgup::send(CONFIG.apikey, fb.slice()).unwrap();
        log::info!("tick...");
    }
}
