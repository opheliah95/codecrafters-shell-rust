use rustyline::history::{DefaultHistory, History};
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

    pub fn write_history_to_file(&mut self, path: &Path) -> std::io::Result<()> {
        // remove first line
        let path_string = path.to_string_lossy().into_owned();
        let mut file =OpenOptions::new().read(true).write(true).open(path_string)?;
        let mut reader = BufReader::new(&file);

        let mut first_line = String::new();

        if reader.read_line(&mut first_line)? > 0 {
            if ["#V2", "", "\n"].contains(&first_line.trim()) {
                println!("first line is {}", first_line);
                first_line.pop();
            }

            let mut rest = String::new();
            reader.read_to_string(&mut rest)?;

            // set buffer
            file.set_len(0)?;
            file.write_all(rest.as_bytes());

        }

        self.history.save(path);

        return Ok(());
    }
}
