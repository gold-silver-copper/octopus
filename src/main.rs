use std::{
    env,
    fs::File,
    io::{self, Write},
};

fn main() {
    // Get the input file path from the first command-line argument
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).unwrap();
    println!("{}", input_path);

    // Write header to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
}
