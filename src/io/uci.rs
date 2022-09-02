use std::io;
use std::process;
use crate::engine::EnginePosition;

/// runs the uci loop
pub fn run() {
    println!("id name Kimbo");
    println!("id auther Jamie Whiting");
    let mut position = EnginePosition::default();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            "go" => println!("handling go"),
            "isready" => println!("handling isready"),
            "position" => println!("handling position"),
            "ucinewgame" => ucinewgame(&mut position),
            "setoption" => println!("do i need this?"),
            "stop" => println!("STOPPING"),
            "quit" => process::exit(0),
            _ => println!("Unknown command!"),
        }
    }
}

fn ucinewgame(position: &mut EnginePosition) {
    *position = EnginePosition::default();
}