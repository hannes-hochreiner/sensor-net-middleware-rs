extern crate libc;
mod serial_stream;
mod sensor_message;
mod auth_request;
#[macro_use] extern crate log;

use std::error::Error;
use std::result::Result;
use std::fmt;
use serial_stream::SerialStream;
use futures::stream::StreamExt;
use serde_json::{Value};
use hex::FromHex;
use aes::Aes128;
use block_modes::{BlockMode, Ecb, block_padding::NoPadding};
use sensor_message::{SensorMessage, Message, Measurement, ParameterValue};
use std::env;
use auth_request::{AuthRequestConfig, AuthRequest};
use chrono::{Utc, SecondsFormat};

type Aes128Ecb = Ecb<Aes128, NoPadding>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("starting...");

    let device = env::var("SENSOR_NET_DEVICE")?;
    let key = Vec::from_hex(env::var("SENSOR_NET_KEY")?)?;

    let mut stream = SerialStream::new(&device)?;

    info!("initialized stream...");

    let auth_config = AuthRequestConfig {
        client_id: env::var("AUTH0_CLIENT_ID")?,
        client_secret: env::var("AUTH0_CLIENT_SECRET")?,
        audience: env::var("AUTH0_CLIENT_AUDIENCE")?,
        tenant: env::var("AUTH0_TENANT")?,
        region: env::var("AUTH0_REGION")?,
        endpoint: env::var("SENSOR_NET_ENDPOINT")?
    };
    let mut auth = AuthRequest::new(&auth_config);
    let mut buffer = String::new();

    loop {
        if let Some(res) = stream.next().await {
            info!("received message");
            debug!("buffer: {}", buffer);
            debug!("received message: {}", res);

            buffer.push_str(&res);

            debug!("buffer + message: {}", buffer);

            let mut parts: Vec<String> = buffer.split('\n').map(|elem| String::from(elem)).collect();

            debug!("parts: {:?}", parts);

            buffer = match parts.pop() {
                Some(elem) => elem,
                None => String::new()
            };

            for message in parts {
                let msg = match process_message(&message, &key) {
                    Ok(message) => {
                        info!("message processed successfully");
                        message
                    },
                    Err(err) => {
                        error!("error processing message: {}", err);
                        continue
                    }
                };
    
                match auth.send_message(msg).await {
                    Ok(()) => info!("message sent successfully"),
                    Err(err) => {
                        error!("error sending message: {}", err);
                        continue
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
            let cipher = Aes128Ecb::new_var(&key, Default::default())?;
            sens_msg = SensorMessage::parse(&String::from(msg["rssi"].as_str().unwrap()), &cipher.decrypt_vec(&enc_data)?)?;
        },
        Some("gateway-bl651-radio") => {
            sens_msg = SensorMessage::parse(&format!("{}", msg["rssi"].as_i64().unwrap()), &Vec::from_hex(&msg["data"].as_str().unwrap())?)?;
        },
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
                            (String::from("temperature"), ParameterValue {value: msg["message"]["temperature"].as_f64().unwrap() as f32, unit: String::from("°C")}),
                            (String::from("relativeHumidity"), ParameterValue {value: msg["message"]["humidity"].as_f64().unwrap() as f32, unit: String::from("%")}),
                        ].iter().cloned().collect()
                    }]
                }
            };
        },
        _ => {
            return Err(Box::new(MiddlewareError{description: String::from("unknown message type")}))
        }
    }

    let string_result = serde_json::to_string(&sens_msg)?;
    debug!("{}", string_result);

    Ok(string_result)
}

#[derive(Debug)]
struct MiddlewareError {
    description: String
}

impl Error for MiddlewareError {}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MiddlewareError: {}", self.description)
    }
}
