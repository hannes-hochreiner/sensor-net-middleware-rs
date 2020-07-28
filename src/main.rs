extern crate libc;
mod serial_stream;
mod sensor_message;
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

    loop {
        if let Some(res) = stream.next().await {
            info!("received message");
            debug!("res: {}", res);
            match process_message(&res, &key) {
                Ok(()) => info!("message processed successfully"),
                Err(err) => error!("error processing message: {}", err)
            }
        }
    }
}

fn process_message(msg: &String, key: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let msg: Message = serde_json::from_str(msg)?;
    debug!("message: {:?}", msg);
    let enc_data = Vec::from_hex(&msg.data)?;
    let cipher = Aes128Ecb::new_var(&key, Default::default())?;
    let plain_data = cipher.decrypt_vec(&enc_data)?;
    let sens_msg = SensorMessage::parse(&msg.rssi, &plain_data)?;
    debug!("{}", serde_json::to_string(&sens_msg)?);

    Ok(())
}
