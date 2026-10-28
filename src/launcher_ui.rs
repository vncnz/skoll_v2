use cairo::Context;

use crate::{config::Config, mode_calculator::Calculator, mode_search::Search, time_of_scope::TimeOfScope};

pub trait LauncherModule {
    // fn recalc (&mut self);
    fn new () -> Self where Self: Sized;
    fn needs_redraw (&self) -> bool;
    fn on_activate (&mut self, query: &str);
    fn key_tab (&mut self);
    fn key_home (&mut self);
    fn key_down (&mut self);
    fn key_up (&mut self);
    fn key_enter (&mut self) -> bool;
    // fn needs_recalc (&self) -> bool;
    fn update_query (&mut self, query: &str);
    fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64, x: f64, y: f64);
}


























#[derive(PartialEq)]
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
        if self.query.len() == 0 && self.mode != LauncherMode::Search { self.mode = LauncherMode::Search; }
        self.query.push(c);
        // self.search.update_query(self.query.clone().as_str());
        match self.mode {
            LauncherMode::Search => self.search.update_query(self.query.clone().as_str()),
            LauncherMode::Calculator => self.calculator.update_query(self.query.clone().as_str())
        }
    }

    pub fn handle_key_space (&mut self) {
        self.query.push(' ');
        self.search.update_query(self.query.clone().as_str());
        self.calculator.update_query(self.query.clone().as_str());
    }

    pub fn handle_key_num(&mut self, c: char) {
        if self.query.len() == 0 && self.mode != LauncherMode::Calculator { self.mode = LauncherMode::Calculator; }
        self.query.push(c);
        // self.calculator.update_query(self.query.clone().as_str());
        match self.mode {
            LauncherMode::Search => self.search.update_query(self.query.clone().as_str()),
            LauncherMode::Calculator => self.calculator.update_query(self.query.clone().as_str())
        }
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
        key_tab(),
        key_home(),
        key_down(),
        key_up()
        // key_char(c: char), key_arrow(dir: Arrow),
    );

    pub fn on_activate (&mut self) {
        self.query = String::new();
        match self.mode {
            LauncherMode::Search => self.search.on_activate(&self.query),
            LauncherMode::Calculator => self.calculator.on_activate(&self.query)
        };
    }

    pub fn key_enter (&mut self) -> bool {
        self.query = String::new();
        match self.mode {
            LauncherMode::Search => self.search.key_enter(),
            LauncherMode::Calculator => self.calculator.key_enter()
        }
    }

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

    pub fn draw (&mut self, config: &Config, cr: &Context, rect_width: f64, rect_height: f64) {
        let _time = TimeOfScope::new("launcher draw");
        let x = rect_width / 3.0;
        let y = rect_height / 4.0;
        match self.mode {
            LauncherMode::Calculator => self.calculator.draw(config, cr, rect_width, rect_height, x, y),
            LauncherMode::Search => self.search.draw(config, cr, rect_width, rect_height, x, y)
        };
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