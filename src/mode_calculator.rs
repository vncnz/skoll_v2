use cairo::Context;
use colored::Colorize;
use fasteval::Error;
use std::collections::BTreeMap;
use std::eprint;

use crate::config::Config;
use crate::launcher_ui::{LauncherModule, X_POINT, Y_POINT};
use crate::utils::{cr_color, cr_text_aligned};

pub struct Calculator {
    dirty_ui: bool,
    query: String,
    result: Result<f64, Error>,
    history: Vec<(String, f64)>
}

impl LauncherModule for Calculator {
    fn new () -> Calculator {
        Calculator {
            query: String::new(),
            dirty_ui: true,
            result: Err(Error::EOF),
            history: vec![]
        }
    }
    fn on_activate (&mut self, query: &str) {
        self.history.clear();
        self.update_query(query);
    }
    fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    fn key_tab (&mut self) {
        // TODO: Handle key tab event for calculator
    }

    fn key_home (&mut self) {
        // TODO: Handle key home event for calculator
    }

    fn key_down (&mut self) {
        // TODO: Handle key down event for calculator
    }

    fn key_up (&mut self) {
        // TODO: Handle key up event for calculator
    }

    fn key_enter (&mut self) -> bool {
        if let Ok(r) = self.result {
            self.history.insert(0, (self.query.clone(), r));
            self.query = String::new();
            self.result = Err(Error::EOF);
            self.dirty_ui = true;
        }
        false
    }

    fn update_query (&mut self, query: &str) {
        eprint!("{} {}", "Query:".yellow(), query);
        let mut map: BTreeMap<String, f64> = BTreeMap::new();

        let len = self.history.len();
        for i in 0..len {
            let idx = len - i - 1;
            if let Ok(c) = std::str::from_utf8(&[(idx as u8 + 97)]) {
                eprint!("{} {c} {}", "Looking for char:".yellow(), query.contains(c));
                if query.contains(c) && let Some((_q, v)) = self.history.get(i) {
                    eprint!("{} {c} = {v}", "Inserting char:".yellow());
                    map.insert(c.into(), *v);
                }
            }
        }
        self.result = fasteval::ez_eval(query, &mut map);
        self.query = query.to_string();
        self.dirty_ui = true;
    }

    fn draw (&mut self, config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        self.dirty_ui = false;

        cr.set_font_size(18.0);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "calculate".into(), X_POINT, Y_POINT, 1.0, 0.5);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt,_yt) = cr_text_aligned(cr, self.query.clone(), X_POINT + 36.0, Y_POINT, 0.0, 0.5);

        if !self.query.is_empty() {
            let text: String = if let Ok(res) = self.result { format!(" = {res}").to_string() } else { " ?".to_string() };
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            let (_xt,_yt) = cr_text_aligned(cr, text, X_POINT + 36.0 + xt + 10.0, Y_POINT, 0.0, 0.5);
        }

        let len = self.history.len();
        for i in 0..len {

            let (query, result) = self.history.get(i).unwrap();
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let (xt,_yt) = cr_text_aligned(cr, query.clone(), X_POINT + 36.0, Y_POINT + ((i+1) as f64)*24.0, 0.0, 0.5);

            let text: String = format!(" = {result}").to_string();
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            let (_xt,_yt) = cr_text_aligned(cr, text, X_POINT + 36.0 + xt + 10.0, Y_POINT + ((i+1) as f64)*24.0, 0.0, 0.5);

            let idx = len - i - 1;
            if let Ok(c) = std::str::from_utf8(&[(idx as u8 + 97)]) {
                let text: String = format!("{c}").to_string();
                cr_color(&cr, config.accent_color);
                let (_xt,_yt) = cr_text_aligned(cr, text, X_POINT + 30.0, Y_POINT + ((i+1) as f64)*24.0, 1.0, 0.5);
            }
        }

    }
}