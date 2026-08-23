use rustyline::history::{DefaultHistory, History};
use std::fmt::write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
        let mut idx = 0;
        for val in self.history.iter() {
            if val == "#V2" {
                continue;
            }
            println!("{:>4} {}", idx + 1, val);
            idx += 1;
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

    pub fn write_history_to_file(&mut self, path: String) -> std::io::Result<()> {
        // remove first line
        //let path_string = path.to_string_lossy().into_owned();
        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
        for val in self.history.iter() {
            writeln!(file, "{}", val)?;
        }

        file.sync_all();

        return Ok(());
    }

    pub fn append_to_history(&mut self, path: String) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = BufReader::new(file);
        for line in buffer.lines() {
            let line = line.unwrap();
            if line == "#V2" {
                continue;
            }
            self.add_history(&line);
        }
       
        return Ok(());

    }
}
