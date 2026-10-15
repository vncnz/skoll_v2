use cairo::Context;
use colored::Colorize;
use std::collections::HashMap;

use crate::config::Config;
use crate::utils::{cr_text_aligned, cr_color};

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
        let info0 = InfoUnit {
            text: "test".into(),
            color: (1.0, 0.4, 0.2, 1.0)
        };
        let info1 = InfoUnit {
            text: "test1".into(),
            color: (0.7, 0.4, 1.0, 1.0)
        };
        let mut info_units = HashMap::new();
        info_units.insert("test0".into(), info0);
        info_units.insert("test1".into(), info1);
        let sorted_keys = vec!["test0".to_string(), "test1".to_string()];
        RatatoskrModule {
            info_units,
            sorted_keys,
            dirty_ui: true
        }
    }

    pub fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    pub fn update (&mut self, id: String, unit: InfoUnit) {
        if !self.sorted_keys.contains(&id) {
            self.sorted_keys.push(id.clone());
        }
        self.info_units.insert(id, unit);
        self.dirty_ui = true;
    }

    pub fn draw (&mut self, _config: &Config, cr: &Context, _rect_width: f64, rect_height: f64) {

        // let gray1 = (0.5, 0.5, 0.5, 1.0);
        // let red = (1.0, 0.4, 0.1, 1.0);

        let mut xx = 8.0;
        for key in &self.sorted_keys {
            if let Some(unit) = self.info_units.get(key) {
                cr_color(&cr, unit.color);
                let (lx, _) = cr_text_aligned(cr, (&unit.text).into(), xx, rect_height - 10.0, 0.0, 1.0);
                xx += lx + 8.0;
            }
        }
        self.dirty_ui = false;
    }
}