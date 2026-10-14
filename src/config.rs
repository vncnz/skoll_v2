use std::fs;
use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum FrameColor {
    None,
    Rgba(f64, f64, f64, f64),
    // Random,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub frame_color: FrameColor,
    pub show_devices_battery_max_level: f64,
    pub hide_missing_ratatoskr: bool,
    pub show_watts: bool
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    frame_color: Option<serde_json::Value>,
    show_devices_battery_max_level: Option<f64>,
    hide_missing_ratatoskr: Option<bool>,
    show_watts: Option<bool>,
}

impl FrameColor {
    fn from_json(value: Option<serde_json::Value>) -> Self {
        match value {
            Some(serde_json::Value::Null) => FrameColor::None,
            Option::None => FrameColor::None,

            Some(serde_json::Value::String(s)) => match s.as_str() {
                "random" => {
                    let mut rng = rand::rng();
                    FrameColor::Rgba(
                        rng.random(),
                        rng.random(),
                        rng.random(),
                        1.0,
                    )
                },
                _ => {
                    eprintln!("Unrecognized value in framecolor config: {:?}", s);
                    FrameColor::None
                }
            },

            Some(serde_json::Value::Array(arr)) if arr.len() == 4 => {
                let vals: Vec<f64> = arr
                    .iter()
                    .filter_map(|v| v.as_f64().map(|x| x as f64))
                    .collect();
                if vals.len() == 4 {
                    FrameColor::Rgba(vals[0], vals[1], vals[2], vals[3])
                } else {
                    eprintln!("Invalid color array, using None");
                    FrameColor::None
                }
            }

            _ => {
                eprintln!("Invalid color in JSON configuration {:?}", value);
                FrameColor::None
            }
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let expanded_path = shellexpand::tilde(path);
        let data = fs::read_to_string(expanded_path.as_ref())
            .unwrap_or_else(|_| {
                eprintln!("Cannot read configuration file: {}", path);
                "{}".to_string()
            });

        let raw: RawConfig = serde_json::from_str(&data).unwrap_or_else(|_| {
            eprintln!("Config JSON non valido, uso valori di default");
            RawConfig {
                frame_color: None,
                show_devices_battery_max_level: Some(80.0),
                hide_missing_ratatoskr: None,
                show_watts: None
            }
        });

        Config {
            frame_color: FrameColor::from_json(raw.frame_color),
            show_devices_battery_max_level: raw.show_devices_battery_max_level.unwrap_or(80.0),
            hide_missing_ratatoskr: raw.hide_missing_ratatoskr.unwrap_or(false),
            show_watts: raw.show_watts.unwrap_or(false)
        }
    }
}
