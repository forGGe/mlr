use anyhow::Result;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

pub fn send(apikey: &str, data: &[u8]) -> Result<()> {
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        raw_request_body: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);

    // TODO: make it more random
    let boundary = "----zzzz";

    let head = format!(
        "--{}\r\n{}\r\n\r\n{}\r\n--{}\r\n{}\r\n\r\n{}\r\n",
        boundary,
        "Content-Disposition: form-data; name=\"UPLOADCARE_PUB_KEY\"",
        apikey,
        boundary,
        "Content-Disposition: form-data; name=\"UPLOADCARE_STORE\"",
        "auto"
    );

    let data_head = format!(
        "--{}\r\n{}\r\n{}\r\n\r\n",
        boundary,
        "Content-Disposition: form-data; name=\"file\"; filename=\"file.jpg\"",
        "Content-Type: application/octet-stream",
    );

    let tail = format!("\r\n--{}--\r\n", boundary);

    let content_type = &format!("multipart/form-data; boundary={}", boundary);
    let content_len = &format!("{}", head.len() + data_head.len() + data.len() + tail.len());
    let headers = [
        ("Accept", "*/*"),
        ("Content-Length", content_len.as_str()),
        ("Content-Type", content_type.as_str()),
    ];
    // let mut request = client.post("http://192.168.10.100:8080", &headers)?;
    let mut request = client.post("https://upload.uploadcare.com/base/", &headers)?;
    request.write(head.as_bytes())?;
    request.write(data_head.as_bytes())?;
    request.write(data)?;
    request.write(tail.as_bytes())?;

    let response = request.submit()?;

    println!("Response: {}", response.status());

    Ok(())
}
