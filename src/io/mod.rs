/// error handling
pub mod errors;
pub mod fen;
mod info;
pub mod inputs;
/// internal representations to readable outputs
pub mod outputs;
/// uci interface
pub mod uci;

use info::*;
use outputs::u16_to_uci;
use std::io;
use std::process;
use uci::uci_run;

use crate::eval::tuner::optimise;
use crate::eval::tuner_eval::ParamContainer;

// used in inputs/outputs
const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

/// description of version
fn description() {
    println!("{}", DESCRIPTION);
}
/// output listed features
fn features() {
    println!("{}", FEATURES);
}

/// run Texel tuner
fn run_tuner(commands: Vec<&str>) {
    if !(2..=3).contains(&commands.len()) {
        println!("invalid command");
        return;
    }
    let initial_params = ParamContainer::default();
    let best = optimise::<true>(commands[1], initial_params);
    println!("Best parameters:");
    println!("{:#?}", best);
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
            "tune" => run_tuner(commands),
            _ => println!("Unknown command!"),
        }
    }
}

// Stats for a full iterative deepening search
pub struct SearchStats {
    depth_reached: i8,
    nodes_to_depth: u64,
    time_to_depth: u64,
    pv: Vec<u16>,
}

impl SearchStats {
    pub fn new(depth_reached: i8, time_to_depth: u64, nodes_to_depth: u64, pv: Vec<u16>) -> Self {
        Self {
            depth_reached,
            nodes_to_depth,
            time_to_depth,
            pv,
        }
    }

    pub fn report(&self) {
        println!(
            "depth reached {} nodes {} time {}",
            self.depth_reached, self.nodes_to_depth, self.time_to_depth
        );
        println!("pv {}", self.pv.iter().map(u16_to_uci).collect::<String>());
    }
}
