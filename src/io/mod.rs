pub mod inputs;
/// internal representations to readable outputs
pub mod outputs;
/// uci interface
pub mod uci;
/// error handling
pub mod errors;
pub mod fen;
mod info;

use uci::uci_run;
use info::*;
use std::io;
use std::process;

// used in inputs/outputs
const FILES: [char; 8] = ['a','b','c','d','e','f','g','h'];

/// description of of version
fn description() {
    println!("{}", DESCRIPTION);
}
/// output listed features
fn features() {
    println!("{}", FEATURES);
}

pub fn main_loop() {
    println!("Kimbo, created by Jamie Whiting");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "uci" => uci_run(),
            "quit" => process::exit(0),
            "description" => description(),
            "features" => features(),
            _ => println!("Unknown command!"),
        }
    }
}