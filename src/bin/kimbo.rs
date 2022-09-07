use std::io;
use std::process;
use kimbo::io::uci::uci_run;

fn main() {
    println!("Kimbo, created by Jamie Whiting");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "uci" => uci_run(),
            "quit" => process::exit(0),
            _ => println!("Unknown command!"),
        }
    }
}