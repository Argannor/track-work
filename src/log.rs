use std::sync::Mutex;

use once_cell::sync::Lazy;

pub struct Log<'a> {
    entries: Vec<&'a str>
}

impl<'a> Log<'a> {
    fn new() -> Log<'a> {
        Log {
            entries: vec![]
        }
    }

    pub fn log(&mut self, str: String) {
        let msg = Box::leak(Box::new(str));
        self.entries.append(&mut vec![msg]);
    }

    pub fn last_n(&self, n: usize) -> Vec<&'a str> {
        let skip = if self.entries.len() <= n {
            0
        } else {
            self.entries.len() - n
        };
        self.entries.iter().skip(skip).copied().collect()
    }
}

pub static LOG: Lazy<Mutex<Log>> = Lazy::new(|| Mutex::new(Log::new()));

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg: String = format!($($arg)*);
        $crate::log::LOG.lock().unwrap().log(msg);
    }}
}

pub(crate) use log;