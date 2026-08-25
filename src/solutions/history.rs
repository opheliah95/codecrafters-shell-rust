use rustyline::history::{DefaultHistory, History};
use std::fmt::write;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tokio::fs::metadata;

pub struct HistHelper {
    pub history: DefaultHistory,
    pub current_index: usize,
    pub append_index: usize,
}

impl HistHelper {
    pub fn new() -> Self {
        Self {
            history: DefaultHistory::new(),
            current_index: 0,
            append_index: 0,
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

    pub fn clear_and_reload_hist_from_path(&mut self, path: &Path) -> std::io::Result<()>{
        // check if file is empty
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    return Ok(());
                }
            }
            Err(e) => {
                println!("Error loading metadata");
                return Err(e);
            }
        }

        // init setup
        let prev_hist: Vec<String> = self.history.iter().map (|v| v.to_owned()).collect();
        self.history.clear();
        self.load_history(path);
        // add existing history
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        for val in prev_hist.iter() {
            writeln!(file, "{}", val)?;
        }
        self.history.add("history");

        return Ok(());
    }

    pub fn write_history_to_file(&mut self, path: &Path) -> std::io::Result<()> {
        // remove first line
        //let path_string = path.to_string_lossy().into_owned();
        let mut file = OpenOptions::new().append(true).open(path)?;
        for val in self.history.iter() {
            //println!("write to file {}", val);
            writeln!(file, "{}", val)?;
        }

        return Ok(());
    }

    pub fn append_to_history(&mut self, path: &String) -> std::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        let content: Vec<String> = self
            .history
            .iter()
            .skip(self.append_index)
            .map(|s| s.to_string())
            .collect();
        //println!("content: {:?}", content);
        for (idx, cmd) in content.iter().enumerate() {
            let path_str = path.as_str();
            let existing = fs::read_to_string(path_str)?;
            if existing.contains(cmd) && !cmd.contains("history -a") {
                //println!("{} exists", cmd);
                continue;
            }
            writeln!(file, "{}", cmd)?;
        }

        self.append_index = self.get_len();
        return Ok(());
    }
}
