use cairo::Context;
use std::collections::HashMap;

use crate::config::Config;
use crate::utils::{cr_color, cr_text_aligned_pango};

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