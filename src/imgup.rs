use anyhow::bail;
use anyhow::Result;
use core::str;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use serde::Deserialize;
use serde_json;
use std::io::Write;

#[derive(Deserialize)]
struct Response {
    file: heapless::String<40>,
}

pub fn send(apikey: &str, data: &[u8]) -> Result<heapless::String<64>> {
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        raw_request_body: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);

    // TODO: make it more random
    let boundary: &'static str = "----zzzz";

    // Having fun with minimizing heap allocations
    let mut data_head = Vec::with_capacity(512);

    write!(
        &mut data_head,
        "--{}\r\n{}\r\n\r\n{}\r\n--{}\r\n{}\r\n\r\n{}\r\n",
        boundary,
        "Content-Disposition: form-data; name=\"UPLOADCARE_PUB_KEY\"",
        apikey,
        boundary,
        "Content-Disposition: form-data; name=\"UPLOADCARE_STORE\"",
        "auto"
    )?;

    write!(
        &mut data_head,
        "--{}\r\n{}\r\n{}\r\n\r\n",
        boundary,
        "Content-Disposition: form-data; name=\"file\"; filename=\"file.jpg\"",
        "Content-Type: application/octet-stream",
    )?;

    // Having fun with minimizing even more heap allocations
    let mut tail = [0u8; 32];
    write!(tail.as_mut_slice(), "\r\n--{}--\r\n", boundary)?;
    let tail = str::from_utf8(&tail)?.trim_matches(char::from(0));

    let mut conttype = [0u8; 64];
    write!(
        conttype.as_mut_slice(),
        "multipart/form-data; boundary={}",
        boundary
    )?;

    let mut contlen = [0u8; 8];
    write!(
        contlen.as_mut_slice(),
        "{}",
        data_head.len() + data.len() + tail.len()
    )?;

    // conttype and contlen buffers are plagued by trailing zeros,
    // since we allocated them as fixed arrays in the first place

    let contlen = str::from_utf8(&contlen)?.trim_matches(char::from(0));
    let conttype = str::from_utf8(&conttype)?.trim_matches(char::from(0));

    let headers = [
        ("Accept", "*/*"),
        ("Content-Length", contlen),
        ("Content-Type", conttype),
    ];

    // TODO: make debug mode configurable via the config file
    // let mut request = client.post("http://192.168.10.100:8080", &headers)?;
    let mut request = client.post("https://upload.uploadcare.com/base/", &headers)?;
    request.write(data_head.as_slice())?;
    request.write(data)?;
    request.write(&tail.as_bytes())?;

    let mut response = request.submit()?;

    println!("Response status: {}", response.status());

    if response.status() == 200 {
        let mut resp_body = [0u8; 64];
        response.read(&mut resp_body)?;
        let resp_str = str::from_utf8(&resp_body)?
            .trim()
            .trim_matches(char::from(0));
        let r: Response = serde_json::from_str(&resp_str)?;

        let mut rs = heapless::String::new();
        // TODO: map error properly from rs.push_str()
        let _ = rs.push_str("https://ucarecdn.com/");
        let _ = rs.push_str(&r.file);
        Ok(rs)
    } else {
        bail!("response received: {}", response.status());
    }
}
