use cairo::Context;

use crate::{config::Config, launcher_data::{AppEntry, get_apps, get_sample, load_app_entries}, pills::PillModuleBase, utils::{cr_draw_image, cr_text_aligned, cr_text_layout}};


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
    results_index: usize
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
            results_index: 0
        }
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

    pub fn handle_key_num(&mut self, c: char) {
        match self.mode {
            LauncherMode::Search => {
                self.mode = LauncherMode::Calculator;
                self.query = String::new();
            },
            _ => {}
        };
        self.query.push(c);
        self.needs_recalc = true;
        self.needs_redraw = true;
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
        cr_text_aligned(cr, "execute".into(), sep, y + 12.0, 1.0, 0.5);

        if self.results.len() > 0 {
            let stepy = 24.0;

            let (skip, take, mut yline, alphas) = if self.results_index > 0 {
                (self.results_index - 1, 4, y - stepy, vec![0.7, 1.0, 0.7, 0.4])
            } else {
                (0, 3, y, vec![1.0, 0.7, 0.4])
            };
            let end = (skip+take).min(self.results.len());

            eprintln!("({}) {skip}..{end}   results:{}", self.results_index, self.results.len());

            cr.set_font_size(18.0);



            /*for idx in skip..end {
                // eprintln!("{idx} in {skip}..{end}");
                let entry = &self.results[idx];
                let result_text = format!("{} - {}", entry.name, entry.comment);
                let alpha = alphas[idx - skip];
                cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
                cr_text_aligned(cr, result_text.clone(), sep + 36.0, yline + 12.0, 0.0, 0.5);
                if let Some(icon_path) = entry.icon_path.as_ref() {
                    cr_draw_image(&cr, icon_path.clone(), sep + 10.0, yline + 2.0, 20.0, 20.0);
                }
                yline += stepy;
            }*/
            let entry: &AppEntry = &self.results[self.results_index];
            let (s1, s2) = if entry.name.to_lowercase().starts_with(&self.query.to_lowercase()) {
                (self.query.clone(), entry.name[self.query.len()..entry.name.len()].to_string())
            } else {
                (self.query.clone(), entry.name.to_string())
            };
            // let result_text = format!("{} - {}", entry.name, entry.comment);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let (xt, _yt) = cr_text_aligned(cr, s1, sep + 36.0, yline + 12.0, 0.0, 0.5);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.6);
            cr_text_aligned(cr, s2, sep + 36.0 + xt + 2.0, yline + 12.0, 0.0, 0.5);

            if let Some(icon_path) = entry.icon_path.as_ref() {
                cr_draw_image(&cr, icon_path.clone(), sep + 10.0, yline + 2.0, 20.0, 20.0);
            }
            yline += stepy;

            /* let entry = &self.results[self.results_index];
            if let Some(icon_path) = entry.icon_path.as_ref() {
                cr_draw_image(&cr, icon_path.clone(), 10.0, 10.0, 40.0, 40.0);
            } else {
                eprintln!("----- No image ----")
            } */
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
        cr_text_aligned(cr, "calculate".into(), sep, y + 12.0, 1.0, 0.5);

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        let (xt,yt) = cr_text_aligned(cr, self.query.clone(), sep + 36.0, y + 12.0, 0.0, 0.5);

        cr.set_source_rgba(0.7, 0.7, 0.7, 1.0);
        let (xt,yt) = cr_text_aligned(cr, "= 123".into(), sep + 36.0 + xt + 10.0, y + 12.0, 0.0, 0.5);

    }
}

