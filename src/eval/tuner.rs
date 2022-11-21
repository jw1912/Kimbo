use crate::position::{Position, zobrist::ZobristVals};
use super::{*, tuner_eval::NUM_PARAMS};
use std::io::BufRead;
use std::time::Instant;
use std::{sync::Arc, io::BufReader};
use std::fs::File;

use super::tuner_eval::{ParamContainer, tuner_eval, tuner_pawn_score, tuner_mobility_score};

#[derive(Debug)]
pub struct TunerPosition {
    pub pst: i16,
    pub pawns: [i16; 5],
    pub phase: i16,
    pub mob: [i16; 12],
    pub result: f64,
}

fn parse_epd(s: &str, zvals: Arc<ZobristVals>) -> TunerPosition {
    let commands: Vec<&str> = s.split("c9").map(|v| v.trim()).collect();
    let pos = Position::from_fen(commands[0], zvals).unwrap();
    let r = match commands[1] {
        "\"1-0\";" => 1.0,
        "\"0-1\";" => 0.0,
        "\"1/2-1/2\";" => 0.5,
        _ => panic!("invalid results")
    };
    let mut phase = pos.phase as i32;
    if phase > TOTALPHASE {
        phase = TOTALPHASE
    };
    let pst = eval_factor(phase, pos.pst_mg, pos.pst_eg) + eval_factor(phase, pos.mat_mg, pos.mat_eg);
    let mut pawns = tuner_pawn_score(&pos, 0);
    let bp = tuner_pawn_score(&pos, 1);
    for i in 0..5 {
        pawns[i] -= bp[i]
    }
    let mut mob = tuner_mobility_score(&pos, 0);
    let bm = tuner_mobility_score(&pos, 1);
    for i in 0..12 {
        mob[i] -= bm[i];
    }
    TunerPosition { pst, pawns, phase: phase as i16, result: r, mob }
}

fn get_positions(filename: &str) -> Vec<TunerPosition> {
    let mut positions: Vec<TunerPosition> = Vec::new();
    let file = match File::open(filename) {
        Ok(f) => f,
        _ => {
            println!("Couldn't load file!");
            return positions;
        }
    };
    let zvals = Arc::new(ZobristVals::default());
    let mut count = 0;
    let now = Instant::now();
    for line in BufReader::new(file).lines() {
        positions.push(parse_epd(&line.unwrap(), zvals.clone()));
        count += 1;
        if count & 65535 == 0 {println!("Loaded {count} positions, {} per sec. {:?}", count * 1000 / now.elapsed().as_millis(), positions.last().unwrap().mob)}
    }
    println!("Completed: Loaded {count} positions.");
    positions
}

fn sigmoid(k: f64, x: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(- k * x))
}

fn calculate_error(positions: &Vec<TunerPosition>, params: &[i16; NUM_PARAMS], num_positions: f64, k: f64) -> f64 {
    let mut error = 0.0;
    for pos in positions {
        error += (pos.result - sigmoid(k, tuner_eval(pos, params) as f64 / 100.0)).powi(2);
    }
    error / num_positions
}

fn optimise_k(positions: &Vec<TunerPosition>, params: &[i16; NUM_PARAMS], num_positions: f64, mut initial_guess: f64, step_size: f64) -> f64 {
    let mut best_error = calculate_error(positions, params, num_positions, initial_guess);
    let step = if calculate_error(positions, params, num_positions, initial_guess - step_size) < calculate_error(positions, params, num_positions, initial_guess + step_size) {
        -step_size
    } else {
        step_size
    };
    loop {
        initial_guess += step;
        let new_error = calculate_error(positions, params, num_positions, initial_guess);
        if new_error < best_error {
            best_error = new_error;
        } else {
            break;
        }
    }
    initial_guess - step
}

// source: https://www.chessprogramming.org/Texel%27s_Tuning_Method
pub fn optimise<const PRINT_PARAMS: bool>(filename: &str, mut best_params: ParamContainer) -> ParamContainer {
    let start = Instant::now();
    let mut params: [i16; NUM_PARAMS] = best_params.into();
    let positions = get_positions(filename);
    if positions.is_empty() {
        return best_params
    }
    let num_positions = positions.len() as f64;
    println!("{}ms to load positions", start.elapsed().as_millis());

    // optimising K value
    let k = optimise_k(&positions, &params, num_positions, 1.0, 0.01);
    let mut best_error = calculate_error(&positions, &params, num_positions, k);
    println!("Initial error: {}, with optimal K: {}", best_error, k);
    let mut dir = [1; NUM_PARAMS];

    let mut improved = true;
    let mut count = 1;
    while improved {
        let runtime = Instant::now();
        improved = false;
        for i in 0..NUM_PARAMS {
            let mut new_params = params;
            new_params[i] += dir[i];
            let new_error = calculate_error(&positions, &new_params, num_positions, k);
            if new_error < best_error {
                best_error = new_error;
                params = new_params;
                improved = true;
            } else {
                new_params[i] -= 2 * dir[i];
                let new_error2 = calculate_error(&positions, &new_params, num_positions, k);
                if new_error2 < best_error {
                    best_error = new_error2;
                    params = new_params;
                    improved = true;
                    dir[i] = -dir[i];
                }
            }
        }
        println!("Run {} in {}ms, error: {}", count, runtime.elapsed().as_millis(), best_error);
        best_params = params.into();
        if PRINT_PARAMS { println!("{:#?}", best_params) }
        count += 1;
    }
    println!("Finished optimisation.");
    best_params
}