use std::process::{Command, Stdio};
use std::io::{Read, BufReader};

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
