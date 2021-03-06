use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::convert::TryInto;
use serde::Serialize;
use chrono::{Utc, SecondsFormat};

#[derive(Serialize, Debug, Clone)]
pub struct ParameterValue {
    pub value: f32,
    pub unit: String
}

impl PartialEq for ParameterValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.unit == other.unit
    }
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
        match u16::from_le_bytes(data[0..2].try_into()?) {
            2 => {
                Ok(SensorMessage {
                    r#type: String::from("rfm"),
                    rssi: rssi.clone(),
                    timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    message: Message {
                        mcuId: format!("{:08x}-{:08x}-{:08x}",
                            u32::from_le_bytes(data[4..8].try_into()?),
                            u32::from_le_bytes(data[8..12].try_into()?),
                            u32::from_le_bytes(data[12..16].try_into()?),
                        ),
                        index: u32::from_le_bytes(data[16..20].try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(data[2..4].try_into()?)),
                            parameters: [
                                (String::from("temperature"), ParameterValue {value: f32::from_le_bytes(data[20..24].try_into()?), unit: String::from("°C")}),
                                (String::from("relativeHumidity"), ParameterValue {value: f32::from_le_bytes(data[24..28].try_into()?), unit: String::from("%")}),
                                (String::from("pressure"), ParameterValue {value: f32::from_le_bytes(data[28..32].try_into()?), unit: String::from("mbar")})
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
                            u32::from_le_bytes(data[10..14].try_into()?),
                            u64::from_le_bytes(data[2..10].try_into()?),
                        ),
                        index: u32::from_le_bytes(data[14..18].try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(data[18..20].try_into()?)),
                            parameters: [
                                (String::from("acceleration_x"), ParameterValue {value: i16::from_le_bytes(data[20..22].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("acceleration_y"), ParameterValue {value: i16::from_le_bytes(data[22..24].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("acceleration_z"), ParameterValue {value: i16::from_le_bytes(data[24..26].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_x"), ParameterValue {value: i16::from_le_bytes(data[26..28].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_y"), ParameterValue {value: i16::from_le_bytes(data[28..30].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_z"), ParameterValue {value: i16::from_le_bytes(data[30..32].try_into()?) as f32, unit: String::from("au")}),
                            ].iter().cloned().collect()
                        }]
                    }
                })
            },
            4 => {
                Ok(SensorMessage {
                    r#type: String::from("rfm"),
                    rssi: rssi.clone(),
                    timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    message: Message {
                        mcuId: format!("{:0>8x}-{:0>16x}",
                            u32::from_le_bytes(data[10..14].try_into()?),
                            u64::from_le_bytes(data[2..10].try_into()?),
                        ),
                        index: u32::from_le_bytes(data[14..18].try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(data[18..20].try_into()?)),
                            parameters: [
                                (String::from("temperature"), ParameterValue {value: f32::from_le_bytes(data[20..24].try_into()?), unit: String::from("°C")}),
                                (String::from("relativeHumidity"), ParameterValue {value: f32::from_le_bytes(data[24..28].try_into()?), unit: String::from("%")}),
                            ].iter().cloned().collect()
                        }]
                    }
                })
            },
            5 => {
                Ok(SensorMessage {
                    r#type: String::from("rfm"),
                    rssi: rssi.clone(),
                    timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    message: Message {
                        mcuId: format!("{:0>8x}-{:0>16x}",
                            u32::from_le_bytes(data[10..14].try_into()?),
                            u64::from_le_bytes(data[2..10].try_into()?),
                        ),
                        index: u32::from_le_bytes(data[14..18].try_into()?),
                        measurements: vec![Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(data[18..20].try_into()?)),
                            parameters: [
                                (String::from("acceleration_x"), ParameterValue {value: i16::from_le_bytes(data[20..22].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("acceleration_y"), ParameterValue {value: i16::from_le_bytes(data[22..24].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("acceleration_z"), ParameterValue {value: i16::from_le_bytes(data[24..26].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_x"), ParameterValue {value: i16::from_le_bytes(data[26..28].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_y"), ParameterValue {value: i16::from_le_bytes(data[28..30].try_into()?) as f32, unit: String::from("au")}),
                                (String::from("magneticField_z"), ParameterValue {value: i16::from_le_bytes(data[30..32].try_into()?) as f32, unit: String::from("au")}),
                            ].iter().cloned().collect()
                        }, Measurement {
                            sensorId: format!("{:04x}", u16::from_le_bytes(data[32..34].try_into()?)),
                            parameters: [
                                (String::from("temperature"), ParameterValue {value: f32::from_le_bytes(data[34..38].try_into()?), unit: String::from("°C")}),
                                (String::from("relativeHumidity"), ParameterValue {value: f32::from_le_bytes(data[38..42].try_into()?), unit: String::from("%")}),
                            ].iter().cloned().collect()
                        }, Measurement {
                            sensorId: format!("{:04x}", 0u16),
                            parameters: [
                                (String::from("batteryVoltage"), ParameterValue {value: f32::from_le_bytes(data[42..46].try_into()?), unit: String::from("V")}),
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

#[test]
fn parse_type_3() {
    use hex::FromHex;
    use serde_json::{Value};
    
    let s = "{\"type\": \"gateway-bl651-radio\",\"rssi\": -65,\"data\": \"03008477642fb9e155f6102805002e01000001ab60fd404050fce8ffb9fd3a00\"}";
    let msg: Value = serde_json::from_str(&String::from(s)).unwrap();
    let data = &Vec::from_hex(&msg["data"].as_str().unwrap()).unwrap();
    let rssi = msg["rssi"].as_i64().unwrap();
    let sen_msg = SensorMessage::parse(&format!("{}", rssi), data).unwrap();

    assert_eq!(sen_msg.r#type, "rfm");
    assert_eq!(sen_msg.rssi, "-65");
    assert_eq!(sen_msg.message.mcuId, "00052810-f655e1b92f647784");
    assert_eq!(sen_msg.message.index, 302);
    assert_eq!(sen_msg.message.measurements.len(), 1);
    assert_eq!(sen_msg.message.measurements[0].sensorId, "ab01");
    assert_eq!(sen_msg.message.measurements[0].parameters.len(), 6);
    assert_eq!(sen_msg.message.measurements[0].parameters["acceleration_x"], ParameterValue {value: -672.0, unit: String::from("au")});
    assert_eq!(sen_msg.message.measurements[0].parameters["acceleration_y"], ParameterValue {value: 16448.0, unit: String::from("au")});
    assert_eq!(sen_msg.message.measurements[0].parameters["acceleration_z"], ParameterValue {value: -944.0, unit: String::from("au")});
    assert_eq!(sen_msg.message.measurements[0].parameters["magneticField_x"], ParameterValue {value: -24.0, unit: String::from("au")});
    assert_eq!(sen_msg.message.measurements[0].parameters["magneticField_y"], ParameterValue {value: -583.0, unit: String::from("au")});
    assert_eq!(sen_msg.message.measurements[0].parameters["magneticField_z"], ParameterValue {value: 58.0, unit: String::from("au")});
}
