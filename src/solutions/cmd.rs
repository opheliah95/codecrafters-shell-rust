use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CompleteCmd {
    pub path: String,
    pub cmd: String,
    pub flag: String,
}

pub struct RegisteredCompleter {
    pub name: String,
}

impl CompleteCmd {
    pub fn new(path: String, cmd: String, flag: String) -> CompleteCmd {
        CompleteCmd {
            path: path,
            cmd: cmd,
            flag: flag,
        }
    }
    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn set_cmd(&mut self, cmd: String) {
        self.cmd = cmd;
    }
    pub fn run_complete(&self) {
        //println!("the path is {}", self.path);
        if self.path.len() > 0 {
            println!("complete -C '{}' {}", self.path, self.cmd)
        } else {
            println!("complete: {}: no completion specification", self.cmd)
        }
    }

    pub fn read_stdout(&self) -> String {
        let path = &self.path;
        let mut result = Command::new(path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Completer Path Not Found");
        let mut stdout = result.stdout.take().expect(("Error no stdout found"));
        let mut output = String::new();
        stdout
            .read_to_string(&mut output)
            .expect("Failed to read stdout");

        return output;
    }
}

impl RegisteredCompleter {
    pub fn run_cmd(self, completer: CompleteCmd) {
        let path = completer.path;
        let mut result = Command::new(path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Completer Path Not Found");
    }
}

#[derive(Clone)]
pub struct Job {
    pub num: i32,
    pub pid: u32,
    pub status: Arc<Mutex<String>>,
    pub cmd: String,
    pub marker: String,
    pub finished: Arc<Mutex<bool>>,
    pub delete_counter: i32,
}

impl Job {
    pub fn show_job(&mut self) {
        self.update_delete_counter();
        let current_status = self.status.lock().unwrap();
        let new_cmd = &self.cmd[0..self.cmd.len() - 2].trim();
        let output = format!(
            "[{}]{}  {}                 {}",
            &self.num, &self.marker, *current_status, new_cmd
        );
        println!("{output}");
    }

    pub fn update_marker(&mut self, marker: String) {
        self.marker = marker;
    }

    pub fn update_delete_counter(&mut self) {
        let finished = self.finished.lock().unwrap();
        if *finished {
            self.delete_counter += 1;
            return;
        }
    }
}

pub struct JobList {
    pub jobs: Vec<Job>,
}

impl JobList {
    pub fn list_jobs(&mut self) {
        for job in &mut self.jobs {
            job.show_job();
        }
    }
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn remove_job(&mut self) {
        self.jobs.retain(|job| {
            // Lock the mutex to check the condition
            let finished = job.finished.lock().unwrap();
            let is_finished = *finished;
            // Drop the lock
            drop(finished);

            !(is_finished && job.delete_counter >= 1)
        });
    }

    pub fn sequence(&mut self) {
        //println!("total jobs {}", self.jobs.len());
        let mut last = self.jobs.clone().pop().unwrap();
        last.update_marker("+".to_string());

        if self.jobs.len() == 2 {
            self.jobs[0].update_marker("-".to_string());
            return;
        }

        if self.jobs.len() >= 3 {
            let last_len = self.jobs.len() - 2;
            let last_sec = &mut self.jobs[last_len];
            last_sec.update_marker("-".to_string());

            for i in 0..last_len {
                self.jobs[i].update_marker(" ".to_string());
            }
            return;
        }
    }
}
