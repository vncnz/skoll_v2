use cairo::Context;

use crate::{config::Config, launcher_data::{AppEntry, get_sample}, pills::PillModuleBase, utils::cr_text_layout};


pub enum LauncherMode {
    Search,
    Calculator
}

pub struct Launcher {
    needs_redraw: bool,
    needs_recalc: bool,
    mode: LauncherMode,
    base: PillModuleBase,
    query: String,
    results: Vec<AppEntry>,
    results_index: usize
}

impl Launcher {
    pub fn new () -> Launcher {
        Launcher {
            needs_redraw: true,
            needs_recalc: true,
            mode: LauncherMode::Search,
            base: PillModuleBase::new(),
            query: "fire".into(),
            results: get_sample(),
            results_index: 0
        }
    }

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64) {

        let sep = 600.0;

        let mode_text = match self.mode {
            LauncherMode::Search => "execute",
            LauncherMode::Calculator => "calculate"
        };

        let (layout, sizes) = cr_text_layout(&cr, &mode_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, mode_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep - 200.0, y, pango::Alignment::Right);






        let result_text: String = if self.results.is_empty() {
            "No results".into()
        } else {
            let entry = &self.results[self.results_index];
            format!("{} - {}", entry.name, entry.comment)
        };

        let (layout, sizes) = cr_text_layout(&cr, &result_text, 18.0, None).unwrap();
        let color = (1.0, 1.0, 1.0, 1.0);
        self.base.set_layout(layout, sizes, result_text.into(), color);

        
        self.base.draw_aligned(cr, 200.0, rect_height, sep + 20.0, y, pango::Alignment::Left);
    }
}

