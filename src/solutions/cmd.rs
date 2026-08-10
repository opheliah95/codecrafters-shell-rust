use std::collections::HashSet;
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
    pub num: u32,
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
        let mut new_cmd = self.cmd.trim().to_owned();

        //no & for finished command
        let finished = self.finished.lock().unwrap();
        if *finished {
            let mut new_cmd_clone = new_cmd.clone();
            let amp_pos = new_cmd_clone.rfind("&").unwrap();
            new_cmd = new_cmd_clone[0..amp_pos].to_string();
            // let last_2_length = new_cmd_clone.chars().rev().take(2).count();
            // new_cmd_clone
            //     .to_string()
            //     .truncate(new_cmd_clone.len() - last_2_length);
            // new_cmd = new_cmd_clone.trim().to_owned();
        }

        let output = format!(
            "[{}]{}  {}                 {}",
            &self.num,
            &self.marker,
            *current_status,
            new_cmd.trim()
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
    pub last_finished_job: Arc<Mutex<String>>,
    pub last_job_pid: Arc<Mutex<u32>>,
    //unused job id
    pub discarded_pid: HashSet<u32>,
}

impl JobList {
    // assign unused job id -> u32
    pub fn assign_unused_id(&mut self, id: u32) {
        if !self.discarded_pid.contains(&id) {
            self.discarded_pid.insert(id);
        }
    }
    // add unused job id -> u32
    pub fn remove_unused_id(&mut self, id: u32) {
        self.discarded_pid.retain(|old_id| *old_id != id)
    }

    // return smallest id
    pub fn return_smallest_id(&mut self) -> u32 {
        if self.discarded_pid.len() == 0 {
            return 0;
        }

        let mut discarded_id_vec: Vec<u32> = self.discarded_pid.iter().cloned().collect();
        //println!("discarded id {:?}", self.discarded_pid);
        discarded_id_vec.sort();
        let initial = discarded_id_vec[0];
        self.remove_unused_id(initial);
        return initial;
    }

    pub fn display_last_job(&mut self) {
        let mut finished_job = self.last_finished_job.lock().unwrap();
        let has_finished = self.jobs.iter().any(|job| {
            let finished = job.finished.lock().unwrap();
            *finished
        });
        if finished_job.len() > 0 && has_finished {
            println!("{finished_job}");
        }
        *finished_job = "".to_string();
        drop(finished_job);
        self.remove_job_with_id();
    }
    pub fn list_jobs(&mut self) {
        for job in &mut self.jobs {
            job.show_job();
        }
    }
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    // remove finished job
    pub fn remove_job_with_id(&mut self) {
        if self.jobs.len() == 0 {
            self.assign_unused_id(1);
            return;
        }
        let stored_job_id = self.last_job_pid.lock().unwrap();
        //remove id
        let job_to_remove = self.jobs.iter().find(|job| {
            let finished = job.finished.lock().unwrap();
            (job.pid == *stored_job_id && *finished)
        });

        match job_to_remove {
            Some(result) => {
                let removed_id = result.num;
                self.jobs.retain(|job| job.pid != *stored_job_id);

                drop(stored_job_id);
                //println!("id assigned: {}", removed_id);
                self.assign_unused_id(removed_id);
            }
            None => return,
        }
    }

    pub fn remove_job(&mut self) {
        if self.jobs.len() == 0 {
            self.assign_unused_id(1);
            return;
        }

        let to_remove: Vec<u32> = self
            .jobs
            .iter()
            .filter(|job| {
                let finished = job.finished.lock().unwrap();
                *finished
            })
            .map(|job| job.num)
            .collect();

        //println!("to remove list is {:?}", to_remove);
        for id in to_remove.into_iter() {
            self.assign_unused_id(id);
        }

        self.jobs.retain(|job| {
            // Lock the mutex to check the condition
            let finished = job.finished.lock().unwrap();
            let is_finished = *finished;
            // Drop the lock
            drop(finished);
            !(is_finished && job.delete_counter >= 1)
        });

        self.sequence(); //re-order the jobs
    }

    pub fn sequence(&mut self) {
        // return if 0 value
        if self.jobs.len() == 0 {
            return;
        }

        //println!("total jobs {}", self.jobs.len());
        let mut last = self.jobs.clone().pop().unwrap();
        last.update_marker("+".to_string());

        // when there is only one job
        if self.jobs.len() == 1 {
            self.jobs[0].update_marker("+".to_string());
            return;
        }

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
