use std::sync::{Arc, Mutex};
use rustyline::history::{DefaultHistory, History, SearchDirection};
use rustyline::{
    Changeset, Cmd, Completer, CompletionType, ConditionalEventHandler, Context, DefaultEditor,
    Editor, Event, EventContext, EventHandler, Helper, Highlighter, KeyCode, KeyEvent, Modifiers,
    RepeatCount, Result, Validator, 
};
use std::io::stdout;


#[derive(Clone)]
pub struct HistHelper {
    pub history: Arc<Mutex<DefaultHistory>>,
    pub current_index: Arc<Mutex<usize>>,
}

impl HistHelper {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(DefaultHistory::new())),
            current_index: Arc::new(Mutex::new(0)),
        }
    }
    pub fn add_history(&mut self, line: &str) {
        let mut history = self.history.lock().expect("History mutex poisoned");
        history.add(line);
    }

    pub fn get_len(&self) -> usize {
        let mut history = self.history.lock().expect("History mutex poisoned");
        return history.len();
    }

    pub fn show_all_history(&self) {
        let mut history = self.history.lock().expect("History mutex poisoned");
        for (idx, val) in history.iter().enumerate() {
            println!("{:>4} {}", idx + 1, val);
        }
    }

    pub fn show_n_lines_history(&self, hist_limit: usize) {
        let start = self.get_len() - hist_limit;
        let mut history = self.history.lock().expect("History mutex poisoned");
        for (idx, val) in history.iter().enumerate().skip(start) {
            println!("{:>4} {}", idx + 1, val);
        }
    }

    pub fn show_history_by_arrow(&self, dir: SearchDirection) -> Option<String> {
        let check_empty = {
            if self.get_len() == 0 {
                return None;
            }
        };

        let dir_config = {
            // decrease index for future call
            let len = self.get_len();
            let mut current_index_guard = self
                .current_index
                .lock()
                .expect("error mutex current index");

            // configure dir
            match dir {
                SearchDirection::Forward => {
                    if *current_index_guard < len - 1 {
                        *current_index_guard += 1
                    } else {
                        *current_index_guard = 0
                    }
                }
                SearchDirection::Reverse => {
                    if *current_index_guard > 0 {
                        // keep the first
                        *current_index_guard -= 1
                    } else {
                        *current_index_guard = len - 1
                    }
                }
            }
        };

        let entry = {
            let mut current_index_guard = self
                .current_index
                .lock()
                .expect("error mutex current index");
            let mut history = self.history.lock().expect("History mutex poisoned");

            let res = history
                .get(*current_index_guard, SearchDirection::Forward)
                .unwrap();

            match res {
                Some(hist) => {
                    let entry = hist.entry.as_ref();
                    //  stdout().flush().unwrap();
                    println!("{entry}");
                    // stdout().flush().unwrap();
                    return Some(entry.to_string());
                }
                None => {
                    println!("Error searching previous history");
                    return None;
                }
            }
        };
    }

    pub fn set_index_to_end(&mut self) {
        let length = self.get_len();
        let mut current_index_guard = self
            .current_index
            .lock()
            .expect("error mutex current index");
        *current_index_guard = length;
    }

    pub fn set_index_to_start(&mut self) {
        let mut current_index_guard = self
            .current_index
            .lock()
            .expect("error mutex current index");
        *current_index_guard = 0;
    }
}

impl ConditionalEventHandler for HistHelper {
    fn handle(
        &self,
        evt: &Event,
        n: RepeatCount,
        positive: bool,
        ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // Check for Up/Down keys
        if let Some(k) = evt.get(0) {
            if *k == KeyEvent(KeyCode::Up, Modifiers::NONE) {
                let res = self.show_history_by_arrow(SearchDirection::Reverse);
                let hist =res.as_ref().map(String::as_str).unwrap();
                
               
                return Some(Cmd::Repaint);
            }
            if *k == KeyEvent(KeyCode::Down, Modifiers::NONE) {
        
                let res = self.show_history_by_arrow(SearchDirection::Forward);
                let hist =res.as_ref().map(String::as_str).unwrap();
               
                return Some(Cmd::Repaint);
            }
        }
        None
    }
}


