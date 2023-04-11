//! Kimbo, a chess engine writte in Rust.

mod engine;
mod macros;
mod state;
mod uci;

#[cfg(test)]
mod test;

use std::{io::stdin, process};

const NAME: &str = env!("CARGO_PKG_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("{NAME} {VERSION}, created by {AUTHOR}");
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let commands = input.split_whitespace().collect::<Vec<&str>>();
        match *commands.first().unwrap_or(&"oops") {
            "uci" => uci::run(),
            "quit" => process::exit(0),
            _ => {}
        }
    }
}
