use std::fs;

use rand::Rng;
use serde::{Deserialize, Deserializer};

fn default_show_devices_battery_max_level() -> f64 {
    80.0
}

fn default_background_color() -> (f64, f64, f64, f64) {
    (0.0, 0.0, 0.0, 1.0)
}

#[derive(Debug, Clone)]
pub enum FrameColor {
    None,
    Rgba(f64, f64, f64, f64),
}

impl Default for FrameColor {
    fn default() -> Self {
        Self::None
    }
}

impl<'de> Deserialize<'de> for FrameColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        match value {
            serde_json::Value::Null => Ok(FrameColor::None),
            serde_json::Value::String(s) if s == "random" => {
                let mut rng = rand::rng();
                Ok(FrameColor::Rgba(rng.random(), rng.random(), rng.random(), 1.0))
            }
            serde_json::Value::String(s) => {
                eprintln!("Unrecognized color value: {:?}; using None", s);
                Ok(FrameColor::None)
            }
            serde_json::Value::Array(arr) if arr.len() == 4 => {
                let mut iter = arr.into_iter();
                let mut rgba = Vec::with_capacity(4);

                for value in &mut iter {
                    let component = value.as_f64().ok_or_else(|| {
                        serde::de::Error::custom("color entries must be numbers")
                    })?;
                    rgba.push(component);
                }

                if rgba.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
                    Ok(FrameColor::Rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
                } else {
                    eprintln!("Invalid frame_color array; using None");
                    Ok(FrameColor::None)
                }
            }
            _ => {
                eprintln!("Invalid color in JSON configuration: {:?}; using None", value);
                Ok(FrameColor::None)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub frame_color: FrameColor,
    pub show_devices_battery_max_level: f64,
    pub hide_missing_ratatoskr: bool,
    pub show_watts: bool,
    #[serde(deserialize_with = "deserialize_background_color")]
    pub background_color: (f64, f64, f64, f64),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frame_color: FrameColor::None,
            show_devices_battery_max_level: default_show_devices_battery_max_level(),
            hide_missing_ratatoskr: false,
            show_watts: false,
            background_color: default_background_color(),
        }
    }
}

fn deserialize_background_color<'de, D>(
    deserializer: D,
) -> Result<(f64, f64, f64, f64), D::Error>
where
    D: Deserializer<'de>,
{
    let rgba: [f64; 4] = Deserialize::deserialize(deserializer)?;

    if rgba.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
        Ok((rgba[0], rgba[1], rgba[2], rgba[3]))
    } else {
        Err(serde::de::Error::custom(
            "background_color must be a 4-tuple with each value between 0.0 and 1.0",
        ))
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Self {
        let expanded_path = shellexpand::tilde(path);
        let data = fs::read_to_string(expanded_path.as_ref()).unwrap_or_else(|err| {
            eprintln!("Cannot read configuration file '{}': {}", path, err);
            String::new()
        });

        match serde_json::from_str::<Config>(&data) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Config JSON is invalid, using default values: {}", err);
                Config::default()
            }
        }
    }
}

