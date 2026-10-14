use cairo::Context;
use std::collections::BTreeMap;

use crate::config::Config;
use crate::launcher_ui::{LauncherModule, X_POINT, Y_POINT};
use crate::utils::cr_text_aligned;

pub struct Calculator {
    dirty_ui: bool,
    query: String,
    result: String
}

impl LauncherModule for Calculator {
    fn new () -> Calculator {
        Calculator {
            query: String::new(),
            dirty_ui: true,
            result: String::new()
        }
    }
    fn on_activate (&mut self) {}
    fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    fn key_tab (&mut self) {
        // Handle key tab event for calculator
    }

    fn key_home (&mut self) {
        // Handle key home event for calculator
    }

    fn key_down (&mut self) {
        // Handle key down event for calculator
    }

    fn key_up (&mut self) {
        // Handle key up event for calculator
    }

    fn key_enter (&mut self) {
        // Handle key enter event for calculator
    }

    fn update_query (&mut self, query: &str) {
        let mut map: BTreeMap<String, f64> = BTreeMap::new();
        self.result = if let Ok(res) = fasteval::ez_eval(query, &mut map) { format!(" = {res}") } else { " ?".to_string() };
        self.query = query.to_string();
        self.dirty_ui = true;
    }

    fn draw (&mut self, _config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        self.dirty_ui = false;

        cr.set_font_size(18.0);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "calculate".into(), X_POINT, Y_POINT, 1.0, 0.5);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt,_yt) = cr_text_aligned(cr, self.query.clone(), X_POINT + 36.0, Y_POINT, 0.0, 0.5);

        if !self.result.is_empty() {
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            let (_xt,_yt) = cr_text_aligned(cr, self.result.to_string(), X_POINT + 36.0 + xt + 10.0, Y_POINT, 0.0, 0.5);
        }
    }
}