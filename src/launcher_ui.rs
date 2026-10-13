use cairo::Context;
use colored::Colorize;
use std::{collections::BTreeMap, eprintln};

use crate::{config::Config, launcher_data::{AppEntry, get_apps, load_app_entries}, utils::{cr_draw_image, cr_text_aligned}};

static X_POINT: f64 = 600.0;
static Y_POINT: f64 = 200.0;

trait LauncherModule {
    // fn recalc (&mut self);
    fn new () -> Self;
    fn needs_redraw (&self) -> bool;
    fn key_home (&mut self);
    fn key_down (&mut self);
    fn key_up (&mut self);
    // fn needs_recalc (&self) -> bool;
    fn update_query (&mut self, query: &str);
    fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64);
}

struct Calculator {
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
    fn needs_redraw (&self) -> bool {
        self.dirty_ui
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

    fn update_query (&mut self, query: &str) {
        let mut map: BTreeMap<String, f64> = BTreeMap::new();
        self.result = if let Ok(res) = fasteval::ez_eval(query, &mut map) { format!(" = {res}") } else { " ?".to_string() };
        self.query = query.to_string();
        self.dirty_ui = true;
    }

    fn draw (&mut self, _config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        self.dirty_ui = false;

        cr.set_font_size(18.0);

        let sep = 600.0;
        let y = 200.0;

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "calculate".into(), sep, y, 1.0, 0.5);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt,_yt) = cr_text_aligned(cr, self.query.clone(), sep + 36.0, y, 0.0, 0.5);

        if !self.result.is_empty() {
            cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
            let (_xt,_yt) = cr_text_aligned(cr, self.result.to_string(), sep + 36.0 + xt + 10.0, y, 0.0, 0.5);
        }
    }
}

struct Search {
    dirty_ui: bool,
    query: String,
    apps: Vec<AppEntry>,
    results: Vec<AppEntry>,
    results_index: usize
}

impl LauncherModule for Search {
    fn new () -> Search {
        let apps = load_app_entries().unwrap_or_default();
        let results = get_apps(&apps, &"".to_string());
        Search {
            query: String::new(),
            dirty_ui: true,
            apps,
            results,
            results_index: 0
        }
    }
    fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    fn key_home (&mut self) {
        self.results_index = 0;
        self.dirty_ui = true;
    }

    fn key_down (&mut self) {
        if self.results_index < self.results.len() - 1 {
            self.results_index += 1;
            self.dirty_ui = true;
        }
    }

    fn key_up (&mut self) {
        if self.results_index > 0 {
            self.results_index -= 1;
            self.dirty_ui = true;
        }
    }

    fn update_query (&mut self, query: &str) {
        self.query = query.to_string();
        self.results = get_apps(&self.apps, &self.query);
        self.results_index = 0;
        self.dirty_ui = true;
    }

    fn draw (&mut self, _config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        eprintln!("{} {:?}", "DRAWING SEARCH".green(), self.query);

        self.dirty_ui = false;

        /* let (layout, sizes) = cr_text_layout(&cr, &mode_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, mode_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep - 200.0, y, pango::Alignment::Right); */

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "execute".into(), X_POINT, Y_POINT, 1.0, 0.0);

        // let white = (1.0, 1.0, 1.0, 1.0);
        let gray0 = (0.7, 0.7, 0.7, 1.0);
        let gray1 = (0.5, 0.5, 0.5, 1.0);
        let gray2 = (0.35, 0.35, 0.35, 1.0);
        let gray3 = (0.2, 0.2, 0.2, 1.0);

        if self.results.len() > 0 {

            cr.set_font_size(18.0);
            if self.results_index > 0 {
                let (_xt, _yt) = self.draw_app_result(&cr, self.results_index - 1, true, X_POINT, Y_POINT - 24.0, gray1);
            }

            let (_xt, _yt) = self.draw_app_result(&cr, self.results_index, false, X_POINT, Y_POINT, gray0);

            for (idx, color) in vec![
                (1, gray1),
                (2, gray2),
                (3, gray3)
            ] {
                if self.results_index + idx < self.results.len() {
                    let (_xt, _yt) = self.draw_app_result(&cr, self.results_index + idx, true, X_POINT, Y_POINT + (24.0 * idx as f64), color);
                }
            }

            // rounded_rect_gradient(cr, sep, 0.0, 10.0, 30.0, 0.0, vec![(0.0, (1.0, 0.0, 0.0, 0.4))], crate::utils::GradientDirection::Horizontal, false, None);

        } else {
            cr_text_aligned(cr, "No results".into(), X_POINT + 80.0, Y_POINT, 0.0, 0.0);
        }

        // cr_text_aligned(cr, result_text.clone(), sep + 20.0, y, 0.0, 0.0);

        /* let (layout, sizes) = cr_text_layout(&cr, &result_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, result_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep + 20.0, y, pango::Alignment::Left); */
    }
}

impl Search {
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
}

























pub enum LauncherMode {
    Search,
    Calculator
}

pub struct Launcher {
    mode: LauncherMode,
    query: String,
    // base: PillModuleBase,
    search: Search,
    calculator: Calculator
}

impl Launcher {
    pub fn new () -> Launcher {
        Launcher {
            query: String::new(),
            mode: LauncherMode::Search,
            search: Search::new(),
            calculator: Calculator::new()
        }
    }

    pub fn needs_redraw (&self) -> bool {
        match self.mode {
            LauncherMode::Search => self.search.needs_redraw(),
            LauncherMode::Calculator => self.calculator.needs_redraw()
        }
    }

    pub fn handle_key_char(&mut self, c: char) {
        self.query.push(c);
        match self.mode {
            LauncherMode::Calculator => {
                self.mode = LauncherMode::Search;
            },
            _ => {}
        };
        self.search.update_query(self.query.clone().as_str());
    }

    pub fn handle_key_space (&mut self) {
        self.query.push(' ');
        self.search.update_query(self.query.clone().as_str());
        self.calculator.update_query(self.query.clone().as_str());
    }

    pub fn handle_key_num(&mut self, c: char) {
        self.query.push(c);
        match self.mode {
            LauncherMode::Search => {
                self.mode = LauncherMode::Calculator;
            },
            _ => {}
        };
        self.calculator.update_query(self.query.clone().as_str());
    }

    pub fn key_reset_query (&mut self) {
        self.query = String::new();
        self.search.update_query(self.query.clone().as_str());
        self.calculator.update_query(self.query.clone().as_str());
    }

    pub fn key_backspace (&mut self) {
        if !self.query.is_empty() {
            self.query.pop();
            self.search.update_query(self.query.clone().as_str());
            self.calculator.update_query(self.query.clone().as_str());
        }
    }

    pub fn key_home (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.key_home(),
            LauncherMode::Calculator => self.calculator.key_home()
        };
    }
    pub fn key_down (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.key_down(),
            LauncherMode::Calculator => self.calculator.key_down()
        };
    }
    pub fn key_up (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.key_up(),
            LauncherMode::Calculator => self.calculator.key_up()
        };
    }

    pub fn key_enter (&mut self) {
        // TODO
    }

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64) {
        match self.mode {
            LauncherMode::Calculator => self.calculator.draw(config, cr, rect_width, rect_height, x, y),
            LauncherMode::Search => self.search.draw(config, cr, rect_width, rect_height, x, y)
        }
        
    }
}

