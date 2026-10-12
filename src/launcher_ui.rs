use cairo::Context;

use crate::{config::Config, launcher_data::{AppEntry, get_apps, get_sample}, pills::PillModuleBase, utils::{cr_text_aligned, cr_text_layout}};


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
            results: get_apps("".into()), // get_sample(),
            results_index: 0
        }
    }

    pub fn refresh_results (&mut self) {
        self.results = get_apps(self.query.clone());
        self.results_index = 0;
    }

    pub fn handle_key_char(&mut self, c: char) {
        self.query.push(c);
        self.refresh_results();
        self.needs_recalc = true;
        self.needs_redraw = true;
    }

    pub fn recalc (&mut self) {
        self.needs_recalc = false;
    }

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64) {

        self.needs_redraw = false;

        let sep = 600.0;
        let y = 200.0;

        let mode_text = match self.mode {
            LauncherMode::Search => "execute",
            LauncherMode::Calculator => "calculate"
        };

        /* let (layout, sizes) = cr_text_layout(&cr, &mode_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, mode_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep - 200.0, y, pango::Alignment::Right); */

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.set_font_size(18.0);
        cr_text_aligned(cr, mode_text.into(), sep, y, 1.0, 0.0);


        let results_to_show = self.results.iter().take(4); // .map(|entry| format!("{} - {}", entry.name, entry.comment)).collect::<Vec<_>>();

        /* let result_text: String = if self.results.is_empty() {
            "No results".into()
        } else {
            let entry = &self.results[self.results_index];
            format!("{} - {}", entry.name, entry.comment)
        }; */

        cr.set_font_size(18.0);
        let mut y = y;
        let mut alpha = 1.0;

        for entry in results_to_show {
            let result_text = format!("{} - {}", entry.name, entry.comment);
            cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
            cr_text_aligned(cr, result_text.clone(), sep + 20.0, y, 0.0, 0.0);
            y += 24.0;
            alpha *= 0.7;
        }

        // cr_text_aligned(cr, result_text.clone(), sep + 20.0, y, 0.0, 0.0);

        /* let (layout, sizes) = cr_text_layout(&cr, &result_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, result_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep + 20.0, y, pango::Alignment::Left); */
    }
}

