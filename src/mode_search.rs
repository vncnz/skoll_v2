use cairo::Context;
use colored::Colorize;
use std::collections::HashMap;

use crate::config::Config;
use crate::launcher_data::{AppEntry, launch_detached, update_launch_history};
use crate::launcher_ui::{LauncherModule, X_POINT, Y_POINT};
use crate::niri::{NiriWindow, focus_niri_window, get_niri_windows};
use crate::time_of_scope::TimeOfScope;
use crate::utils::{cr_draw_image, cr_text_aligned, cr_color};
use crate::launcher_data::{load_app_entries, get_apps};

pub struct Search {
    dirty_ui: bool,
    query: String,
    apps: Vec<AppEntry>,
    results: Vec<AppEntry>,
    results_index: usize,
    option_index: usize,
    options: Vec<String>,
    niri_windows: HashMap<String,Vec<NiriWindow>>
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
            results_index: 0,
            option_index: 0,
            options: vec![],
            niri_windows: HashMap::new()
        }
    }
    fn needs_redraw (&self) -> bool {
        self.dirty_ui
    }

    fn on_activate (&mut self) {
        let windows = get_niri_windows();
        // eprintln!("{windows:?}");
        let mut windows_map: HashMap<String,Vec<NiriWindow>> = HashMap::new();
        for w in windows {
            if !windows_map.contains_key(&w.app_id) {
                windows_map.insert(w.app_id.to_owned(), vec![w.clone()]);
            } else if let Some(arr) = windows_map.get_mut(&w.app_id) {
                arr.push(w);
            }
        }
        // eprintln!("{windows_map:?}");
        self.niri_windows = windows_map;
        // self.results_index = 0;
        self.update_query("");
        self.reset_options();
    }

    fn key_tab (&mut self) {
        if self.options.len() > 0 {
            self.option_index = (self.option_index + 1) % self.options.len();
            self.dirty_ui = true;
        }
    }

    fn key_home (&mut self) {
        self.results_index = 0;
        self.dirty_ui = true;
    }

    fn key_down (&mut self) {
        if self.results_index < self.results.len() - 1 {
            self.results_index += 1;
            self.reset_options();
            self.dirty_ui = true;
        }
    }

    fn key_up (&mut self) {
        if self.results_index > 0 {
            self.results_index -= 1;
            self.reset_options();
            self.dirty_ui = true;
        }
    }

    fn key_enter (&mut self) {
        let entry: &AppEntry = &self.results[self.results_index];
        eprintln!("{} {:?}", "ENTER PRESSED".yellow(), entry);
        eprintln!("self.option_index: {}", self.option_index);
        eprintln!("{:?}", self.niri_windows.get(&entry.appid));

        if self.option_index > 0 {
            if let Some(windows) = self.niri_windows.get(&entry.appid) {
                // eprintln!("{windows:?}");
                if self.option_index - 1 < windows.len() {
                    let res = focus_niri_window(windows[self.option_index - 1].id);
                    eprintln!("Result {res:?}");
                }
            }
        } else {
            update_launch_history(&entry);
            let _ = launch_detached(&entry.exec);
            eprintln!("launch_detached ran!");
        }
    }

    /// Update the query, reset candidate options, requires UI redraw
    fn update_query (&mut self, query: &str) {
        self.query = query.to_string();
        self.results = get_apps(&self.apps, &self.query);
        self.results_index = 0;
        self.reset_options();
        self.dirty_ui = true;
    }

    fn draw (&mut self, config: &Config, cr: &Context, _rect_width: f64, _rect_height: f64, _x: f64, _y: f64) {

        eprintln!("{} {:?}", "DRAWING SEARCH".green(), self.query);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, "execute".into(), X_POINT, Y_POINT, 1.0, 0.0);

        // let accent = (1.0, 0.2, 0.5, 1.0);
        // let white = (1.0, 1.0, 1.0, 1.0);
        let gray0 = (0.7, 0.7, 0.7, 1.0);
        let gray1 = (0.5, 0.5, 0.5, 1.0);
        let gray2 = (0.35, 0.35, 0.35, 1.0);
        let gray3 = (0.2, 0.2, 0.2, 1.0);

        if self.results.len() > 0 {

            cr.set_font_size(18.0);

            // Previous option (if any)
            if self.results_index > 0 {
                let (_xt, _yt) = self.draw_app_result(&cr, self.results_index - 1, true, X_POINT, Y_POINT - 24.0, gray1);
            }

            // Selected option
            let (xt, _yt) = self.draw_app_result(&cr, self.results_index, false, X_POINT, Y_POINT, gray0);

            // Draw options for the selected line
            let mut xx = X_POINT + xt + 20.0;
            let opts = self.options.len();
            for opt in 0..opts {
                cr_color(&cr, if self.option_index == opt { config.accent_color } else { gray1 });
                let (lx, _) = cr_text_aligned(cr, self.options[opt].clone().into(), xx, Y_POINT, 0.0, 0.0);
                xx += lx + 8.0;
            }

            // Next options (if any)
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

        self.dirty_ui = false;
    }
}

impl Search {

    /// Draw the single query result on the cairo::Context
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
        let (xt1, yt1) = cr_text_aligned(cr, s2, x + 36.0 + xt + 2.0, y, 0.0, 0.0);

        if let Some(icon_path) = entry.icon_path.as_ref() {
            cr_draw_image(&cr, &icon_path, x + 10.0, y - 16.0, 20.0, 20.0);
            // eprintln!("{icon_path:?}");
        }
        (36.0 + xt + 2.0 + xt1, yt.max(yt1))
    }

    /// Recalc options for candidate-for-launch application
    fn reset_options (&mut self) {
        self.option_index = 0;
        self.options = vec![];
        if self.results_index < self.results.len() {
            let key = &self.results[self.results_index].appid;
            if self.niri_windows.contains_key(key) {
                self.options.push("[new]".to_string());
                for w in self.niri_windows.get(key).expect("niri_windows missing key") {
                    self.options.push(format!("[ws {}]", w.workspace_id));
                }
            }
        }
    }
}