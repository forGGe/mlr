use anyhow::{anyhow, Result};
use core::str;
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttEvent, MqttClientConfiguration, QoS};
use std::io::Write;

pub fn send(tbkey: &str, imgurl: &str) -> Result<()> {
    let conf = MqttClientConfiguration {
        client_id: Some("mrl_rust_example_id"),
        username: Some(tbkey),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    let (tx, rx) = std::sync::mpsc::channel();

    let cb = move |e: EspMqttEvent| {
        let msg = match e.payload() {
            esp_idf_svc::mqtt::client::EventPayload::Connected(_) => Ok(()),
            esp_idf_svc::mqtt::client::EventPayload::Disconnected => Err(anyhow!("disconnected")),
            esp_idf_svc::mqtt::client::EventPayload::Published(_) => Ok(()),
            esp_idf_svc::mqtt::client::EventPayload::Error(e) => Err(anyhow!("failed")),
            _ => return,
        };

        tx.send(msg).unwrap();
    };

    let mut cli = EspMqttClient::new_cb("mqtt://demo.thingsboard.io:1883", &conf, cb)?;

    let mut payload = [' ' as u8; 128];
    write!(payload.as_mut_slice(), r#"{{ "url": "{}" }}"#, imgurl)?;
    let payload = str::from_utf8(&payload)?.trim().as_bytes();

    // Waiting for "Connected"
    rx.recv().unwrap()?;

    cli.publish("v1/devices/me/telemetry", QoS::AtLeastOnce, false, payload)?;

    // Waiting for "Published"
    rx.recv().unwrap()?;

    Ok(())
}
