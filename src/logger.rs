use colored::*;

pub struct Logger {
    debug_main: bool,
    #[allow(dead_code)]
    debug_frida: bool,
}

impl Logger {
    pub fn new(debug_main: bool, debug_frida: bool) -> Self {
        Self {
            debug_main,
            debug_frida,
        }
    }

    pub fn info(&self, msg: &str) {
        println!("{}", msg);
    }

    pub fn error(&self, msg: &str) {
        eprintln!("{}", msg.red());
    }

    pub fn main_debug(&self, msg: &str) {
        if self.debug_main {
            println!("{}", msg.dimmed());
        }
    }

    #[allow(dead_code)]
    pub fn frida_debug(&self, msg: &str) {
        if self.debug_frida {
            println!("{}", msg.cyan());
        }
    }

    #[allow(dead_code)]
    pub fn main_debug_raw(&self, msg: &str) {
        if self.debug_main {
            println!("{}", msg);
        }
    }
}
