#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    //TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if &command.trim().to_lowercase() == "exit" {
            break;
        }
        match command.trim() {
            i if i.len() >=4 && &i[0..4].to_lowercase()== "echo" => println!("{}", &i[5..]),
            i if i.len() >=4 && &i[0..4].to_lowercase()=="type" => {
                let target= &i[5..9];
                let arr = vec! [String::from("echo"),
                                String::from("exit"),
                                String::from("type")];
               let found = arr.iter().any(|result| result == target);
               if found {
                    println!("{} is a shell builtin", target);
               } else {
                    println!("{}: command not found", target);
               }
            }
            _=> println!("{}: command not found", command.trim())
        }
    }
}
// match input.trim() {
//     "xyz" =>println!("xyz: command not found"),
//     _ => ()
// }
