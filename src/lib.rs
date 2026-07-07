pub mod solutions {
    pub fn type_impl(input: &str) {
        let args = input.split(" ").collect::<Vec<&str>>();
         match args.as_slice() {
            [] => (),
            ["echo"] => println!(),
            ["echo", ..] => println!("{}", &input[5..]),
            ["type", args @ ("echo" | "exit" | "type")] => println!("{args} is a shell builtin"),
            ["type", args @ ..] => println!("{}: not found", args[0]),
            _ => println!("{}: command not found", input),
        }
    }
}
