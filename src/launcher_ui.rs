use cairo::Context;
use std::collections::BTreeMap;

use crate::{config::Config, launcher_data::{AppEntry, get_apps, load_app_entries}, utils::{cr_draw_image, cr_text_aligned}};


pub enum LauncherMode {
    Search,
    Calculator
}

pub struct Launcher {
    pub needs_redraw: bool,
    pub needs_recalc: bool,
    mode: LauncherMode,
    // base: PillModuleBase,
    query: String,
    apps: Vec<AppEntry>,
    results: Vec<AppEntry>,
    results_index: usize,
    result_calc: String
}

impl Launcher {
    pub fn new () -> Launcher {
        Launcher {
            needs_redraw: true,
            needs_recalc: false,
            mode: LauncherMode::Search,
            // base: PillModuleBase::new(),
            query: "".into(),
            apps: load_app_entries().unwrap_or_default(), // get_sample(),
            results: Vec::new(),
            results_index: 0,
            result_calc: String::new()
        }
    }

    pub fn refresh_calc (&mut self) {
        self.needs_redraw = true;
    }

    pub fn refresh_results (&mut self) {
        self.results = get_apps(&self.apps, &self.query);
        self.results_index = 0;
        self.needs_redraw = true;
    }

    pub fn handle_key_char(&mut self, c: char) {
        match self.mode {
            LauncherMode::Calculator => {
                self.mode = LauncherMode::Search;
                self.query = String::new();
            },
            _ => {}
        };
        self.query.push(c);
        self.refresh_results();
        self.needs_recalc = true;
        self.needs_redraw = true;
    }

    pub fn handle_key_space (&mut self) {
        self.query.push(' ');
    }

    pub fn handle_key_num(&mut self, c: char) {
        match self.mode {
            LauncherMode::Search => {
                self.mode = LauncherMode::Calculator;
                self.query = String::new();
            },
            _ => {}
        };
        self.query.push(c);
        self.refresh_calc();
        self.needs_recalc = true;
        self.needs_redraw = true;
    }

    pub fn key_reset_query (&mut self) {
        self.query = String::new();
        self.refresh_results();
        self.needs_recalc = true;
        self.needs_redraw = true;
    }

    pub fn key_backspace (&mut self) {
        if !self.query.is_empty() {
            self.query.pop();
            self.refresh_results();
            self.needs_recalc = true;
            self.needs_redraw = true;
        }
    }

    pub fn key_down (&mut self) {
        // eprintln!("{} {}", self.results_index, self.results.len());
        if self.results_index < self.results.len() - 1 {
            self.results_index += 1;
            self.needs_recalc = true;
            self.needs_redraw = true;
        }
    }
    pub fn key_up (&mut self) {
        if self.results_index > 0 {
            self.results_index -= 1;
            self.needs_recalc = true;
            self.needs_redraw = true;
        }
    }

    pub fn key_enter (&mut self) {
        // TODO
    }

    pub fn recalc (&mut self) {
        match self.mode {
            LauncherMode::Calculator => {},
            LauncherMode::Search => {}
        };
        self.needs_recalc = false;
    }

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64) {
        match self.mode {
            LauncherMode::Calculator => self.draw_calc(config, cr, rect_width, rect_height, x, y),
            LauncherMode::Search => self.draw_search(config, cr, rect_width, rect_height, x, y)
        }
        
    }

    fn draw_app_result (&mut self, cr: &Context, idx: usize, plain: bool, x: f64, y: f64, color: (f64, f64, f64, f64)) -> (f64, f64) {
        let entry: &AppEntry = &self.results[idx];
        let (s1, s2) = if plain {
            ("".to_string(), entry.name.clone())
        } else if entry.name.to_lowercase().starts_with(&self.query.to_lowercase()) {
            (self.query.clone(), entry.name[self.query.len()..entry.name.len()].to_string())
        } else {
            (self.query.clone(), format!(" ({})", entry.name.to_string()))
        };
        // let result_text = format!("{} - {}", entry.name, entry.comment);
        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt, yt) = cr_text_aligned(cr, s1, x + 36.0, y, 0.0, 0.0);
        cr.set_source_rgba(color.0, color.1, color.2, color.3);
        cr_text_aligned(cr, s2, x + 36.0 + xt + 2.0, y, 0.0, 0.0);

        if let Some(icon_path) = entry.icon_path.as_ref() {
            cr_draw_image(&cr, icon_path.clone(), x + 10.0, y - 16.0, 20.0, 20.0);
        }
        (xt, yt)
    }

    pub fn draw_search (&mut self, _config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        self.needs_redraw = false;

        let sep = 600.0;
        let y = 200.0;

        /* let (layout, sizes) = cr_text_layout(&cr, &mode_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, mode_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep - 200.0, y, pango::Alignment::Right); */

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "execute".into(), sep, y, 1.0, 0.0);

        // let white = (1.0, 1.0, 1.0, 1.0);
        let gray0 = (0.7, 0.7, 0.7, 1.0);
        let gray1 = (0.5, 0.5, 0.5, 1.0);
        let gray2 = (0.35, 0.35, 0.35, 1.0);
        let gray3 = (0.2, 0.2, 0.2, 1.0);

        if self.results.len() > 0 {

            cr.set_font_size(18.0);
            if self.results_index > 0 {
                let (_xt, _yt) = self.draw_app_result(&cr, self.results_index - 1, true, sep, y - 24.0, gray1);
            }

            let (_xt, _yt) = self.draw_app_result(&cr, self.results_index, false, sep, y, gray0);

            for (idx, color) in vec![
                (1, gray1),
                (2, gray2),
                (3, gray3)
            ] {
                if self.results_index + idx < self.results.len() {
                    let (_xt, _yt) = self.draw_app_result(&cr, self.results_index + idx, true, sep, y + (24.0 * idx as f64), color);
                }
            }

            // rounded_rect_gradient(cr, sep, 0.0, 10.0, 30.0, 0.0, vec![(0.0, (1.0, 0.0, 0.0, 0.4))], crate::utils::GradientDirection::Horizontal, false, None);

        } else {
            cr_text_aligned(cr, "No results".into(), sep + 80.0, y, 0.0, 0.0);
        }

        // cr_text_aligned(cr, result_text.clone(), sep + 20.0, y, 0.0, 0.0);

        /* let (layout, sizes) = cr_text_layout(&cr, &result_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, result_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep + 20.0, y, pango::Alignment::Left); */
    }

    pub fn draw_calc (&mut self, _config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        self.needs_redraw = false;

        cr.set_font_size(18.0);

        let sep = 600.0;
        let y = 200.0;

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "calculate".into(), sep, y, 1.0, 0.5);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt,_yt) = cr_text_aligned(cr, self.query.clone(), sep + 36.0, y, 0.0, 0.5);


        // TODO: move it in recalc method!
        let mut map: BTreeMap<String, f64> = BTreeMap::new();
        self.result_calc = if let Ok(res) = fasteval::ez_eval(&self.query, &mut map) { format!(" = {res}") } else { " ?".to_string() };

        if !self.result_calc.is_empty() {
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            let (_xt,_yt) = cr_text_aligned(cr, self.result_calc.to_string(), sep + 36.0 + xt + 10.0, y, 0.0, 0.5);
        }

    }
}

