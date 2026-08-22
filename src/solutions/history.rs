use rustyline::history::{DefaultHistory, History};
use std::io::stdout;
use std::path::Path;
use std::sync::Arc;

pub struct HistHelper {
    pub history: DefaultHistory,
    pub current_index: usize,
}

impl HistHelper {
    pub fn new() -> Self {
        Self {
            history: DefaultHistory::new(),
            current_index: 0,
        }
    }
    pub fn add_history(&mut self, line: &str) {
        self.history.add(line);
    }

    pub fn get_len(&self) -> usize {
        return self.history.len();
    }

    pub fn show_all_history(&self) {
        for (idx, val) in self.history.iter().enumerate() {
            println!("{:>4} {}", idx + 1, val);
        }
    }

    pub fn show_n_lines_history(&self, hist_limit: usize) {
        let start = self.get_len() - hist_limit;
        for (idx, val) in self.history.iter().enumerate().skip(start) {
            println!("{:>4} {}", idx + 1, val);
        }
    }

    pub fn set_index_to_end(&mut self) {
        self.current_index = 0
    }

    pub fn set_index_to_start(&mut self) {
        self.current_index = 0;
    }

    pub fn load_history(&mut self, path: &Path) {
        let res = self.history.load(path);
        match (res) {
            Ok(v) => {
                //add to history
                return;
            }
            Err(e) => println!("error history: {e:?}"),
        }
    }

    pub fn write_history_to_file(&mut self, path: &Path) {
        self.history.save(path);
    }
}
