extern crate libc;
mod sensor_message;
mod serial_stream;

use chrono::{SecondsFormat, Utc};
use futures::stream::StreamExt;
use hex::FromHex;
use hyper::client::HttpConnector;
use hyper::{Body, Method, Request};
use hyper_tls::native_tls::Identity;
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use openssl::symm as openssl;
use sensor_message::{Measurement, Message, ParameterValue, SensorMessage};
use serde_json::Value;
use serial_stream::SerialStream;
use std::env;
use std::error::Error;
use std::fmt;
use std::result::Result;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_native_tls::native_tls::TlsConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("starting...");

    let device = env::var("SENSOR_NET_DEVICE")?;
    let endpoint = env::var("SENSOR_NET_ENDPOINT")?;
    let key = Vec::from_hex(env::var("SENSOR_NET_KEY")?)?;
    let cert_filename = env::var("CERTIFICATE_FILENAME")?;
    let cert_password = env::var("CERTIFICATE_PASSWORD")?;
    let mut stream = SerialStream::new(&device)?;

    info!("initialized stream...");

    let cert = {
        let mut cert_file = File::open(cert_filename).await?;
        let mut cert_raw = Vec::new();
        cert_file.read_to_end(&mut cert_raw).await?;
        cert_raw
    };

    let tls_connector: tokio_native_tls::TlsConnector = TlsConnector::builder()
        .identity(Identity::from_pkcs12(&cert, &cert_password)?)
        .build()?
        .into();
    let mut http_connector = HttpConnector::new();

    http_connector.enforce_http(false);
    http_connector.set_connect_timeout(Some(Duration::from_secs(3)));

    let mut ssl = HttpsConnector::<HttpConnector>::from((http_connector, tls_connector));

    ssl.https_only(true);

    let client = hyper::Client::builder().build::<_, hyper::Body>(ssl);

    info!("initialized client...");

    let mut buffer = String::new();

    loop {
        if let Some(res) = stream.next().await {
            info!("received message");
            debug!("buffer: {}", buffer);
            debug!("received message: {}", res);

            buffer.push_str(&res);

            debug!("buffer + message: {}", buffer);

            let mut parts: Vec<String> =
                buffer.split('\n').map(|elem| String::from(elem)).collect();

            debug!("parts: {:?}", parts);

            buffer = match parts.pop() {
                Some(elem) => elem,
                None => String::new(),
            };

            for message in parts {
                let msg = match process_message(&message, &key) {
                    Ok(message) => {
                        info!("message processed successfully");
                        message
                    }
                    Err(err) => {
                        error!("error processing message: {}", err);
                        continue;
                    }
                };

                let req = Request::builder()
                    .method(Method::PUT)
                    .uri(endpoint.clone())
                    .header("Content-Type", "application/json")
                    .body(Body::from(msg))?;
                match client.request(req).await {
                    Ok(_) => info!("message sent successfully"),
                    Err(error) => {
                        error!("error sending message: {}", error);
                        continue;
                    }
                };
            }
        }
    }
}

fn process_message(msg: &String, key: &Vec<u8>) -> Result<String, Box<dyn Error>> {
    let msg: Value = serde_json::from_str(msg)?;
    debug!("message: {:?}", msg);

    let sens_msg;

    match msg["type"].as_str() {
        Some("rfm") => {
            let enc_data = Vec::from_hex(&msg["data"].as_str().unwrap())?;
            sens_msg = SensorMessage::parse(
                &String::from(msg["rssi"].as_str().unwrap()),
                &openssl::decrypt(openssl::Cipher::aes_128_ecb(), &key, None, &enc_data)?,
            )?;
        }
        Some("gateway-bl651-radio") => {
            sens_msg = SensorMessage::parse(
                &format!("{}", msg["rssi"].as_i64().unwrap()),
                &Vec::from_hex(&msg["data"].as_str().unwrap())?,
            )?;
        }
        Some("gateway-bl651-sensor") => {
            sens_msg = SensorMessage {
                r#type: String::from("rfm"),
                rssi: String::from("0"),
                timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                message: Message {
                    mcuId: String::from(msg["message"]["mcuId"].as_str().unwrap()),
                    index: msg["message"]["index"].as_u64().unwrap() as u32,
                    measurements: vec![Measurement {
                        sensorId: String::from(msg["message"]["sensorId"].as_str().unwrap()),
                        parameters: [
                            (
                                String::from("temperature"),
                                ParameterValue {
                                    value: msg["message"]["temperature"].as_f64().unwrap() as f32,
                                    unit: String::from("°C"),
                                },
                            ),
                            (
                                String::from("relativeHumidity"),
                                ParameterValue {
                                    value: msg["message"]["humidity"].as_f64().unwrap() as f32,
                                    unit: String::from("%"),
                                },
                            ),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                    }],
                },
            };
        }
        _ => {
            return Err(Box::new(MiddlewareError {
                description: String::from("unknown message type"),
            }))
        }
    }

    let string_result = serde_json::to_string(&sens_msg)?;
    debug!("{}", string_result);

    Ok(string_result)
}

#[derive(Debug)]
struct MiddlewareError {
    description: String,
}

impl Error for MiddlewareError {}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MiddlewareError: {}", self.description)
    }
}
