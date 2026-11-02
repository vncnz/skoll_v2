use cairo::Context;
use std::collections::HashMap;
use serde::Deserialize;

use crate::config::Config;
use crate::utils::{cr_color, cr_text_aligned_pango, get_color_gradient};
use crate::data::{BluetoothStats, PartialMsg, UPowerDeviceKind};
use crate::jstr;

pub struct InfoUnit {
    text: String,
    color: (f64, f64, f64, f64)
}

impl InfoUnit {
    pub fn new (text: String, color: (f64, f64, f64, f64)) -> InfoUnit {
        return InfoUnit { text, color }
    }
}

pub struct RatatoskrModule {
    info_units: HashMap<String, InfoUnit>,
    sorted_keys: Vec<String>,
    dirty_ui: bool
}

impl RatatoskrModule {
    pub fn new () -> RatatoskrModule {
        let info_units = HashMap::new();
        let sorted_keys = vec!["loadavg", "ram", "disk", "temperature", "volume", "weather", "display"].iter().map(|s| s.to_string()).collect();
        RatatoskrModule {
            info_units,
            sorted_keys,
            dirty_ui: true
        }
    }

    pub fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    pub fn reset (&mut self) {
        self.info_units.clear();
    }

    pub fn process_new_message(&mut self, data: PartialMsg, hide_missing_ratatoskr: bool) {
        let res = data.resource.as_str();

        let mut toadd: Option<(String, InfoUnit)> = None;
        let mut reset = false;
        let mut color = get_color_gradient(data.warning);
        let div = 2.0 - data.warning;
        color = (color.0 / div, color.1 / div, color.2 / div, 1.0);
        match res {
            "loadavg" => {
                if let Some(info) = &data.data {
                    toadd = Some((res.to_string(), InfoUnit::new(format!("[AVG {} {} {}]", info["m1"], info["m5"], info["m15"]), color)));
                }
            },
            "ram" => {
                if let Some(info) = &data.data {
                    toadd = Some((res.to_string(), InfoUnit::new(format!("[MEM {}% / SWP {}%]", info["mem_percent"], info["swap_percent"]), color)));
                }
            },
            "disk" => {
                if let Some(info) = &data.data {
                    toadd = Some((res.to_string(), InfoUnit::new(format!("[DSK {}%]", info["used_percent"]), color)));
                }
            },
            "network" => {
                if let Some(info) = &data.data {
                    if info["conn_type"] == "ethernet" {
                        toadd = Some((res.to_string(), InfoUnit::new("[ETH]".to_string().into(), color)));
                    } else {
                        toadd = Some((res.to_string(), InfoUnit::new(format!("[WLAN {}%] [IP {}] [NET {}]", info["signal"], info["ip"].as_str().unwrap(), info["ssid"].as_str().unwrap()), color)));
                    }
                }
            },
            "temperature" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_f64().unwrap_or(0.0);
                    if v > 0.0 {
                        toadd = Some((res.to_string(), InfoUnit::new(format!("[TEMP {:.0}°C]", v), color)));
                    } else {
                        toadd = Some((res.to_string(), InfoUnit::new("[NO TEMP]".to_owned(), color)));
                    }
                }
            },
            "volume" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_f64().unwrap_or(0.0);
                    if v > 0.0 {
                        toadd = Some((res.to_string(), InfoUnit::new(format!("[VOL {:.0}%]", v), color)));
                    } else {
                        toadd = Some((res.to_string(), InfoUnit::new("[MUTED]".to_owned(),  (0.4, 0.4, 0.4, 1.0))));
                    }
                }
            },
            "battery" => {
                if let Some(info) = &data.data {
                    let (bat_symb, mut color) = match info["state"].as_str() {
                        Some("Charging") => { ("󱐋", get_color_gradient(data.warning)) },
                        Some("Discharging") => { ("󰯆", get_color_gradient(data.warning.max(0.5))) },
                        _ => { ("󱞜", get_color_gradient(data.warning)) }
                    };
                    let eta = info["eta"].as_f64().unwrap_or_default().round() as i32;
                    let h = eta / 60;
                    let m = eta % 60;
                    let div = 2.0 - data.warning;
                    color = (color.0 / div, color.1 / div, color.2 / div, 1.0);
                    let perc = info["percentage"].as_f64().unwrap_or(0.0);
                    let text: String = if eta > 0 { format!("[BAT {:.0}%] [{} {}h{}m]", perc, bat_symb, h, m).to_string() } else { format!("[BAT {perc:.0}%]").to_string() };
                    toadd = Some((res.to_string(), InfoUnit::new(text, color)));
                }
            },
            "ratatoskr" => {
                if data.warning == 1.0 && !hide_missing_ratatoskr {
                    toadd = Some((res.to_string(), InfoUnit::new(format!("[RATATOSKR DISCONNECTED]"), color)));
                }
                reset = true;
            },
            "display" => {},
            "weather" => {
                // {"icon": "", "text": "Fog", "temp": 8, "temp_real": 9, "temp_unit": "°C", "day": "0", "icon_name": "fog.svg", "sunrise": "07:15", "sunset": "16:48", "sunrise_mins": 435, "sunset_mins": 1008, "daylight": 34385.75, "locality": "Desenzano Del Garda", "humidity": 99}
                if let Some(info) = &data.data {
                    toadd = Some((res.to_string(), InfoUnit::new(format!("[{} {} {:.0}{}]", jstr!(info, "icon", ""), jstr!(info, "text", ""), info["temp"], jstr!(info, "temp_unit", "")), color)));
                }
            },
            "bt-batteries" => {
                if let Some(blue) = &data.data {
                    if let Ok(b) = BluetoothStats::deserialize(blue.clone()) {
                        let mut sp = "".to_string();
                        // log_to_file(format!("{:?}", b));
                        for dev in b.devices.clone().iter() { // .filter(|dv| dv.is_bluetooth) {
                            let icon = match (&dev.kind, dev.is_bluetooth) {
                                (UPowerDeviceKind::Mouse, true) => "󰦋",
                                (UPowerDeviceKind::Mouse, false) => "󰍽",
                                (UPowerDeviceKind::Phone, true) => "󰏳",
                                (UPowerDeviceKind::Phone, false) => "󰏲",
                                (UPowerDeviceKind::Tablet, _) => "",
                                (UPowerDeviceKind::RemoteControl, _) => "󰻅",
                                (UPowerDeviceKind::Speakers, _) => "󰦢",
                                (UPowerDeviceKind::Headphones, true) => "󰥰",
                                (UPowerDeviceKind::Headset, true) => "󰥰",
                                (UPowerDeviceKind::GamingInput, _) => "󱤙",
                                (UPowerDeviceKind::Keyboard, _) => "󰌌",
                                (_, true) => "󰂱",
                                (_, false) => "󰾰"
                            };
                            let prev = format!("{}", sp);
                            sp = format!("{} [{} {:.0}%]", prev, icon, dev.percentage);
                            // log_to_file(format!("{}", sp));
                        }
                        toadd = Some((res.to_string(), InfoUnit::new(sp, color)));

                    }
                }
            },
            _ => {
                toadd = Some((res.to_string(), InfoUnit::new(format!("[{}]", data.resource), color)));
            }
        }

        if reset {
            self.reset();
        }
        if let Some((key, unit)) = toadd {
            self.update(key, unit);
        }
    }

    pub fn update (&mut self, id: String, unit: InfoUnit) {
        // eprintln!("Updating RatatoskrModule with id: {}, text: {}, color: {:?}", id, unit.text, unit.color);
        /* if id == "ratatoskr" {
            self.info_units.clear();
        } */
        if !self.sorted_keys.contains(&id) {
            self.sorted_keys.push(id.clone());
        }
        self.info_units.insert(id, unit);
        // self.dirty_ui = true;
    }

    pub fn draw (&mut self, _config: &Config, cr: &Context, _rect_width: f64, rect_height: f64) {
        // let gray1 = (0.5, 0.5, 0.5, 1.0);
        // let red = (1.0, 0.4, 0.1, 1.0);

        cr.set_font_size(12.0);
        let mut xx = 8.0;
        for key in &self.sorted_keys {
            if let Some(unit) = self.info_units.get(key) {
                cr_color(&cr, unit.color);
                let (lx, _) = cr_text_aligned_pango(cr, (&unit.text).into(), 14.0, xx, rect_height - 10.0, 0.0, 1.0).unwrap();
                // let (lx, _) = cr_text_aligned(cr, (&unit.text).into(), xx, rect_height - 10.0, 0.0, 1.0);
                xx += lx + 8.0;
            }
        }
        self.dirty_ui = false;
    }
}