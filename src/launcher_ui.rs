use cairo::Context;
// use colored::Colorize;

use crate::{config::Config, mode_calculator::Calculator, mode_search::Search};

pub static X_POINT: f64 = 600.0;
pub static Y_POINT: f64 = 200.0;

pub trait LauncherModule {
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

