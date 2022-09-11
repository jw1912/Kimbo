pub mod inputs;
/// internal representations to readable outputs
pub mod outputs;
/// uci interface
pub mod uci;
/// error handling
pub mod errors;

use uci::uci_run;
use std::io;
use std::process;

const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const FEATURES: [&str; 7] = [
    "fully-legal move generation",
    "alpha-beta search",
    "quiescence search",
    "mvv-lva move ordering",
    "transposition table",
    "iterative deepening",
    "check extensions",
];

/// description of of version
pub fn description() {
    println!("{}", DESCRIPTION);
}
/// output listed features
pub fn features() {
    for feature in FEATURES {
        println!("{}", feature);
    }
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
            _ => println!("Unknown command!"),
        }
    }
}