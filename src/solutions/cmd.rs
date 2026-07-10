enum Cmd {
    CmdEcho,
    CmdCat
}

pub struct CmdEcho {
    pub args: String
}

pub struct CmdCat {
    pub cmd: String,
    pub args: Vec<String>
}

impl CmdEcho {
    pub fn new( args: String) -> CmdEcho {
        CmdEcho {
            args: args
        }
    }

    pub fn print_result(&self) {
        
    }
}