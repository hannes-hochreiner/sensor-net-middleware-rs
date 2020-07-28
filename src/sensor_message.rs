use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::convert::TryInto;
use std::ops::Range;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
struct ParameterValue {
    value: f32,
    unit: String
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
struct Measurement {
    sensorId: String,
    parameters: HashMap<String, ParameterValue>
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct SensorMessage {
    r#type: String,
    rssi: String,
    mcuId: String,
    index: u32,
    measurements: Vec<Measurement>
}

impl SensorMessage {
    pub fn parse(rssi: &String, data: &Vec<u8>) -> Result<SensorMessage, Box<dyn Error>> {
        if u16::from_le_bytes(get_data_slice(data, 0..2)?.try_into()?) != 2 {
            return Err(Box::new(SensorMessageError { description: String::from("unsupported message type")}));
        }

        Ok(SensorMessage {
            r#type: String::from("rfm"),
            rssi: rssi.clone(),
            mcuId: format!("{:08x}-{:08x}-{:08x}",
                u32::from_le_bytes(get_data_slice(data, 12..16)?.try_into()?),
                u32::from_le_bytes(get_data_slice(data, 8..12)?.try_into()?),
                u32::from_le_bytes(get_data_slice(data, 4..8)?.try_into()?),
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
        })
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
