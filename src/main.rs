extern crate libc;
mod serial_stream;
mod sensor_message;
mod auth_request;
#[macro_use] extern crate log;

use std::error::Error;
use std::result::Result;
use serial_stream::SerialStream;
use futures::stream::StreamExt;
use serde::{Deserialize};
use hex::FromHex;
use aes::Aes128;
use block_modes::{BlockMode, Ecb, block_padding::NoPadding};
use sensor_message::SensorMessage;
use std::env;
use auth_request::{AuthRequestConfig, AuthRequest};

type Aes128Ecb = Ecb<Aes128, NoPadding>;

#[derive(Deserialize, Debug)]
struct Message {
    r#type: String,
    rssi: String,
    data: String,
}

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
    let msg: Message = serde_json::from_str(msg)?;
    debug!("message: {:?}", msg);
    let enc_data = Vec::from_hex(&msg.data)?;
    let cipher = Aes128Ecb::new_var(&key, Default::default())?;
    let plain_data = cipher.decrypt_vec(&enc_data)?;
    let sens_msg = SensorMessage::parse(&msg.rssi, &plain_data)?;
    let string_result = serde_json::to_string(&sens_msg)?;
    debug!("{}", string_result);

    Ok(string_result)
}
