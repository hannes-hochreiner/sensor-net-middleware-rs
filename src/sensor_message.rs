use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::convert::TryInto;
use std::ops::Range;
use serde::Serialize;
use chrono::{Utc, SecondsFormat};

#[derive(Serialize, Debug, Clone)]
pub struct ParameterValue {
    pub value: f32,
    pub unit: String
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct Measurement {
    pub sensorId: String,
    pub parameters: HashMap<String, ParameterValue>
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct Message {
    pub mcuId: String,
    pub index: u32,
    pub measurements: Vec<Measurement>
}

#[derive(Serialize, Debug)]
pub struct SensorMessage {
    pub r#type: String,
    pub rssi: String,
    pub timestamp: String,
    pub message: Message
}

impl SensorMessage {
    pub fn parse(rssi: &String, data: &Vec<u8>) -> Result<SensorMessage, Box<dyn Error>> {
        match u16::from_le_bytes(get_data_slice(data, 0..2)?.try_into()?) {
            2 => {
                Ok(SensorMessage {
                    r#type: String::from("rfm"),
                    rssi: rssi.clone(),
                    timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    message: Message {
                        mcuId: format!("{:08x}-{:08x}-{:08x}",
                            u32::from_le_bytes(get_data_slice(data, 4..8)?.try_into()?),
                            u32::from_le_bytes(get_data_slice(data, 8..12)?.try_into()?),
                            u32::from_le_bytes(get_data_slice(data, 12..16)?.try_into()?),
                        ),
                        index: u32::from_le_bytes(get_data_slice(data, 16..20)?.try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(get_data_slice(data, 2..4)?.try_into()?)),
                            parameters: [
                                (String::from("temperature"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 20..24)?.try_into()?), unit: String::from("°C")}),
                                (String::from("relativeHumidity"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 24..28)?.try_into()?), unit: String::from("%")}),
                                (String::from("pressure"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 28..32)?.try_into()?), unit: String::from("mbar")})
                            ].iter().cloned().collect()
                        }]
                    }
                })
            },
            3 => {
                Ok(SensorMessage {
                    r#type: String::from("rfm"),
                    rssi: rssi.clone(),
                    timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    message: Message {
                        mcuId: format!("{:0>8x}-{:0>16x}",
                            u32::from_le_bytes(get_data_slice(data, 10..14)?.try_into()?),
                            u64::from_le_bytes(get_data_slice(data, 2..10)?.try_into()?),
                        ),
                        index: u32::from_le_bytes(get_data_slice(data, 14..18)?.try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(get_data_slice(data, 18..20)?.try_into()?)),
                            parameters: [
                                (String::from("acceleration_x"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 20..22)?.try_into()?), unit: String::from("au")}),
                                (String::from("acceleration_y"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 22..24)?.try_into()?), unit: String::from("au")}),
                                (String::from("acceleration_z"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 24..28)?.try_into()?), unit: String::from("au")}),
                                (String::from("magneticField_x"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 28..30)?.try_into()?), unit: String::from("au")}),
                                (String::from("magneticField_y"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 30..32)?.try_into()?), unit: String::from("au")}),
                                (String::from("magneticField_z"), ParameterValue {value: f32::from_le_bytes(get_data_slice(data, 32..34)?.try_into()?), unit: String::from("au")}),
                            ].iter().cloned().collect()
                        }]
                    }
                })
            },
            _ => {
                Err(Box::new(SensorMessageError { description: String::from("unsupported message type")}))
            }
        }
    }
}

fn get_data_slice(data: &Vec<u8>, index: Range<usize>) -> Result<&[u8], SensorMessageError> {
    match data.get(index) {
        Some(d) => Ok(d),
        None => Err(SensorMessageError { description: String::from("could not get data")})
    }
}

#[derive(Debug)]
struct SensorMessageError {
    description: String
}

impl Error for SensorMessageError {}

impl fmt::Display for SensorMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageError: {}", self.description)
    }
}
