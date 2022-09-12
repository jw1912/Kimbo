// The enums of tokens are inspired by Rustic
// SOURCE: https://github.com/mvanthoor/rustic/blob/master/src/comm/uci.rs

use super::inputs::uci_to_u16;
use crate::engine::EnginePosition;
use super::errors::UciError;
use crate::io::outputs::{display_board, u16_to_uci};
use crate::search::{Search, Times};
use crate::perft::PerftSearch;
use crate::hash::{perft::PerftTT, search::HashTable};
use std::io;
use std::process;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use super::info::*;

struct State {
    pos: EnginePosition,
    search_handle: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    ttable_size: usize,
    ttable: Arc<HashTable>,
    age: u8,
    move_overhead: u64,
}

impl Default for State {
    fn default() -> Self {
        State {
            pos: EnginePosition::default(),
            search_handle: None,
            stop: Arc::new(AtomicBool::new(false)),
            ttable_size: 1,
            ttable: Arc::new(HashTable::new(1024 * 1024)),
            age: 0,
            move_overhead: 10,
        }
    }
}

/// runs the uci loop
pub fn uci_run() {
    println!("id name Kimbo {}", VERSION);
    println!("id author {}", AUTHOR);
    println!("option name Hash type spin default 32 min 1 max 256");
    println!("option name Clear Hash type button");
    println!("option name Move Overhead type spin default 10 min 0 max 500");
    println!("uciok");
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));

    'uci: loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        let leave = run_commands(state.clone(), commands);
        if leave {
            break 'uci
        }
    }
}

fn run_commands(state: Arc<Mutex<State>>, commands: Vec<&str>) -> bool {
    let result = match commands[0] {
        // standard uci commands
        "go" => go(state, commands),
        "isready" => isready(),
        "position" => position(state, commands),
        "ucinewgame" => ucinewgame(state),
        "setoption" => setoption(state, commands),
        "stop" => stop(state),
        "quit" => process::exit(0),
        // custom commands
        "display" => display(state, commands),
        "break" => return true,
        _ => return false,
    };
    if result.is_err() {
        println!("{}",result.unwrap_err())
    }
    false
}

fn stop(state: Arc<Mutex<State>>) -> Result<(), UciError> {
    state.lock().unwrap().stop.store(true, Ordering::Relaxed);
    Ok(())
}

fn isready() -> Result<(), UciError> {
    println!("readyok");
    Ok(())
}

fn ucinewgame(state: Arc<Mutex<State>>) -> Result<(), UciError> {
    state.lock().unwrap().pos = EnginePosition::default();
    Ok(())
}

fn display(state: Arc<Mutex<State>>, commands: Vec<&str>) -> Result<(), UciError> {
    enum Tokens {
        None,
        Fancy,
        Hash,
    }
    let mut token: Tokens = Tokens::None;
    for command in commands {
        match command {
            "display" => (),
            "fancy" => token = Tokens::Fancy,
            "hash" => token = Tokens::Hash,
            _ => {
                return Err(UciError::Display);
            }
        }
    }
    match token {
        Tokens::None => display_board::<false>(&state.lock().unwrap().pos.board),
        Tokens::Fancy => display_board::<true>(&state.lock().unwrap().pos.board),
        Tokens::Hash => {
            let state_lock = state.lock().unwrap();
            println!("{} / {} entries filled", state_lock.ttable.filled.load(Ordering::Relaxed), state_lock.ttable.num_entries);
            drop(state_lock);
        }
    }
    Ok(())
}

fn position(state: Arc<Mutex<State>>, commands: Vec<&str>) -> Result<(), UciError> {
    let mut state_lock = state.lock().unwrap();
    enum Tokens {
        Nothing,
        Fen,
        Moves,
    }
    let mut fen = String::from("");
    let mut moves: Vec<String> = Vec::new();
    let mut skip_fen = false;
    let mut token = Tokens::Nothing;

    for command in commands {
        match command {
            "position" => (),
            "startpos" => {
                skip_fen = true;
                state_lock.pos = EnginePosition::default();
            }
            "fen" => {
                if !skip_fen {
                    token = Tokens::Fen
                }
            }
            "moves" => token = Tokens::Moves,
            _ => match token {
                Tokens::Nothing => return Err(UciError::Position),
                Tokens::Fen => {
                    fen.push_str(command);
                    fen.push(' ');
                }
                Tokens::Moves => moves.push(command.to_string()),
            },
        }
    }

    if !fen.is_empty() && !skip_fen {
        state_lock.pos = EnginePosition::from_fen(&fen)?;
    }

    for m in moves {
        let mo = uci_to_u16(&state_lock.pos, &m)?;
        state_lock.pos.make_move(mo);
    }
    drop(state_lock);
    Ok(())
}

fn go(state: Arc<Mutex<State>>, commands: Vec<&str>) -> Result<(), UciError> {
    #[derive(PartialEq)]
    enum Tokens {
        Ponder,
        Depth,
        Nodes,
        MoveTime,
        WTime,
        BTime,
        WInc,
        BInc,
        MovesToGo,
        Perft
    }

    let state_lock = state.lock().unwrap();
    state_lock.stop.store(false, Ordering::Relaxed);
    drop(state_lock);

    // fields to be set
    let mut token = Tokens::Ponder;
    let mut max_depth: u8 = u8::MAX;
    let mut max_move_time: u64 = u64::MAX;
    let mut max_nodes: u64 = u64::MAX;
    let mut times: Times = Times::default();
    let mut perft = false;
    let mut perft_depth = 0;

    for command in commands {
        match command {
            "go" => token = Tokens::Ponder,
            "ponder" => token = Tokens::Ponder,
            "depth" => token = Tokens::Depth,
            "nodes" => token = Tokens::Nodes,
            "movetime" => token = Tokens::MoveTime,
            "wtime" => token = Tokens::WTime,
            "btime" => token = Tokens::BTime,
            "winc" => token = Tokens::WInc,
            "binc" => token = Tokens::BInc,
            "movestogo" => token = Tokens::MovesToGo,
            "perft" => token = {
                perft = true;
                Tokens::Perft
            },
            _ => {
                let mut parse: u64 = 0;
                if token != Tokens::Ponder {
                    parse = command.parse::<u64>()?;
                }
                match token {
                    Tokens::Ponder => return Err(UciError::Go),
                    Tokens::Depth => max_depth = parse as u8,
                    Tokens::Nodes => max_nodes = parse,
                    Tokens::MoveTime => max_move_time = parse,
                    Tokens::WTime => times.wtime = parse,
                    Tokens::BTime => times.btime = parse,
                    Tokens::WInc => times.winc = parse,
                    Tokens::BInc => times.binc = parse,
                    Tokens::MovesToGo => times.moves_to_go = Some(parse as u8),
                    Tokens::Perft => perft_depth = parse as u8,
                }
            },
        }
    }

    if !times.is_default() {
        let state_lock = state.lock().unwrap();
        max_move_time = times.to_movetime(state_lock.pos.board.side_to_move);
        drop(state_lock);
    }

    if perft {
        let state_2 = state.clone();
        let search_thread = thread::spawn(move || {
            let state_lock = state_2.lock().unwrap();
            let position = state_lock.pos.clone();
            let tt = Arc::new(PerftTT::new(state_lock.ttable_size * 1024 * 1024));
            drop(state_lock);
            let mut search = PerftSearch::new(
                position,
                tt
            );
            let count = search.go(perft_depth);
            println!("Move count: {count}");
        });
        state.lock().unwrap().search_handle = Some(search_thread);
        return Ok(());
    }

    // SEARCHING ON SECOND THREAD
    let state_2 = state.clone();
    let search_thread = thread::spawn(move || {
        let state_lock = state_2.lock().unwrap();
        let position = state_lock.pos.clone();
        let abort_signal = state_lock.stop.clone();
        let tt = state_lock.ttable.clone();
        let age = state_lock.age;
        let move_overhead = state_lock.move_overhead;
        drop(state_lock);

        let move_time = if max_move_time <= move_overhead {
            move_overhead
        } else {
            max_move_time - move_overhead
        };

        let mut search = Search::new(
            position,
            abort_signal,
            move_time,
            max_depth,
            max_nodes,
            tt,
            age,
        );
        let best_move = search.go::<false>();
        println!("bestmove {}", u16_to_uci(&best_move));
        let mut state_lock = state_2.lock().unwrap();
        state_lock.age += 1;
        drop(state_lock);
    });
    // join handle provided to master thread
    state.lock().unwrap().search_handle = Some(search_thread);
    Ok(())
}

fn setoption(state: Arc<Mutex<State>>, commands: Vec<&str>) -> Result<(), UciError>{
    let mut reading_name = false;
    let mut reading_value = false;
    let mut name_token = Vec::new();
    let mut value_token = Vec::new();

    for parameter in commands {
        match parameter {
            "setoption" => (),
            "name" => {
                reading_name = true;
                reading_value = false;
            }
            "value" => {
                reading_name = false;
                reading_value = true;
            }
            _ => {
                if reading_name {
                    name_token.push(parameter);
                } else if reading_value {
                    value_token.push(parameter);
                }
            }
        }
    }
    match name_token.join(" ").as_str() {
        "Hash" => {
            let size = value_token[0].parse::<usize>()?;
            let mut state_lock = state.lock().unwrap();
            state_lock.ttable_size = size;
            state_lock.ttable = Arc::new(HashTable::new(state_lock.ttable_size * 1024 * 1024));
            state_lock.age = 0;
            drop(state_lock)
        }
        "Clear Hash" => {
            let mut state_lock = state.lock().unwrap();
            state_lock.ttable = Arc::new(HashTable::new(state_lock.ttable_size * 1024 * 1024));
            state_lock.age = 0;
            drop(state_lock)
        }
        "Move Overhead" => {
            let size = value_token[0].parse::<u64>()?;
            let mut state_lock = state.lock().unwrap();
            state_lock.move_overhead = size;
            drop(state_lock)
        }
        _ => return Err(UciError::SetOption),
    }
    Ok(())
}
