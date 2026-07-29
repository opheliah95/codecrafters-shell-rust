enum Cmd {
    CmdEcho,
    CmdCat,
}

pub struct CmdEcho {
    pub args: String,
}

pub struct CmdCat {
    pub cmd: String,
    pub args: Vec<String>,
}

impl CmdEcho {
    pub fn new(args: String) -> CmdEcho {
        CmdEcho { args: args }
    }

    pub fn print_result(&self) {}
}

pub struct CompleteCmd {
    pub path: String,
    pub cmd: String,
    pub flag: String,
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
}
