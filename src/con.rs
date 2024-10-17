use std::str::FromStr;

use anyhow::Result;
use esp_idf_hal::modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};
use heapless;

pub fn associate(modem: modem::Modem, ssid: &str, pass: &str) -> Result<EspWifi<'static>> {
    log::info!("connecting to {} using passkey: {}", ssid, pass);

    let sl = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi = EspWifi::new(modem, sl, Some(nvs))?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::from_str(ssid).unwrap(),
        password: heapless::String::from_str(pass).unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;

    Ok(wifi)
}
