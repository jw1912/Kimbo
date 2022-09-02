use std::io;
use std::process;

fn main() {
    println!("Kimbo, created by Jamie Whiting");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "uci" => println!("Trying uci"),
            "quit" => process::exit(0),
            _ => println!("Unknown command!"),
        }
    }
}