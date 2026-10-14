use std::time::Instant;

use cairo::Context;
use colored::Colorize;

use crate::{config::Config, mode_calculator::Calculator, mode_search::Search};

pub static X_POINT: f64 = 600.0;
pub static Y_POINT: f64 = 200.0;

pub trait LauncherModule {
    // fn recalc (&mut self);
    fn new () -> Self where Self: Sized;
    fn needs_redraw (&self) -> bool;
    fn on_activate (&mut self);
    fn key_tab (&mut self);
    fn key_home (&mut self);
    fn key_down (&mut self);
    fn key_up (&mut self);
    fn key_enter (&mut self);
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

macro_rules! forward_key {
    ($($method:ident($($arg:ident: $ty:ty),*)),* $(,)?) => {
        $(
            pub fn $method(&mut self $(, $arg: $ty)*) {
                match self.mode {
                    LauncherMode::Search => self.search.$method($($arg),*),
                    LauncherMode::Calculator => self.calculator.$method($($arg),*),
                };
            }
        )*
    };
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

    /* To replace the forward_key macro in key events management
    fn active(&mut self) -> &mut dyn LauncherModule {
        match self.mode {
            LauncherMode::Search => &mut self.search,
            LauncherMode::Calculator => &mut self.calculator,
        }
    } */

    forward_key!(
        on_activate(),
        key_tab(),
        key_home(),
        key_down(),
        key_up(),
        key_enter()
        // key_char(c: char), key_arrow(dir: Arrow),
    );

    /* pub fn on_activate (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.on_activate(),
            LauncherMode::Calculator => self.calculator.on_activate()
        };
    } */

    /* pub fn key_tab (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.key_tab(),
            LauncherMode::Calculator => self.calculator.key_tab()
        };
    } */
    /* pub fn key_home (&mut self) {
        match self.mode {
            LauncherMode::Search => self.search.key_home(),
            LauncherMode::Calculator => self.calculator.key_home()
        };
    } */
    /* pub fn key_down (&mut self) {
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
    } */

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64) {
        let start = Instant::now();
        match self.mode {
            LauncherMode::Calculator => self.calculator.draw(config, cr, rect_width, rect_height, x, y),
            LauncherMode::Search => self.search.draw(config, cr, rect_width, rect_height, x, y)
        }
        let elapsed = Instant::now().saturating_duration_since(start);
        eprintln!("{} {}ms", "Layer draw time:".red(), elapsed.as_millis());
    }
}





/*
macro_rules! forward_key {
    ($($method:ident($($arg:ident: $ty:ty),*)),* $(,)?) => {
        $(
            pub fn $method(&mut self $(, $arg: $ty)*) {
                match self.mode {
                    LauncherMode::Search => self.search.$method($($arg),*),
                    LauncherMode::Calculator => self.calculator.$method($($arg),*),
                };
            }
        )*
    };
}

impl Launcher {
    forward_key!(
        key_tab(), key_enter(), key_backspace(),
        key_char(c: char), key_arrow(dir: Arrow),
    );
}
*/