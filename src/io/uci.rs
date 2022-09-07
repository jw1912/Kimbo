use std::io;
use std::process;
use std::thread;
use std::thread::JoinHandle;
use crate::engine::EnginePosition;
use crate::io::outputs::display_board;
use crate::io::outputs::u16_to_uci;
use crate::search::Times;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::{Arc, Mutex};
use super::inputs::uci_to_u16;
use crate::search::Search;

struct State {
    pos: EnginePosition,
    search_handle: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl Default for State {
    fn default() -> Self {
        State { 
            pos: EnginePosition::default(), 
            search_handle: None, 
            stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// runs the uci loop
pub fn uci_run() {
    println!("id name Kimbo");
    println!("id author Jamie Whiting");
    println!("uciok");
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));
    
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands: Vec<&str> = input.split(' ').map(|v| v.trim()).collect();
        match commands[0] {
            // standard uci commands
            "go" => go(state.clone(), commands),
            "isready" => isready(),
            "position" => position(state.clone(), commands),
            "ucinewgame" => ucinewgame(state.clone()),
            "setoption" => println!("do i need this?"),
            "stop" => stop(state.clone()),
            "quit" => quit(),
            // custom commands
            "display" => display(state.clone(), commands),
            _ => println!("unknown command!"),
        }
    }
}

fn quit() {
    process::exit(0)
}

fn stop(state: Arc<Mutex<State>>) {
    state.lock().unwrap().stop.store(true, Ordering::Relaxed)
}

fn isready() {
    println!("readyok")
}

fn ucinewgame(state: Arc<Mutex<State>>) {
    state.lock().unwrap().pos = EnginePosition::default();
}

fn display(state: Arc<Mutex<State>>, commands: Vec<&str>) {
    for command in commands {
        match command {
            "display" => (),
            "fancy" => {
                display_board::<true>(&state.lock().unwrap().pos.board);
                return;
            },
            _ => {
                println!("unknown command!");
                return;
            }
        }
    }
    display_board::<false>(&state.lock().unwrap().pos.board);
}

fn position(state: Arc<Mutex<State>>, commands: Vec<&str>) {
    let mut state_lock = state.lock().unwrap();

    // SOURCE: https://github.com/mvanthoor/rustic/blob/master/src/comm/uci.rs
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
            },
            "fen" => {
                if !skip_fen { 
                    token = Tokens::Fen
                }
            },
            "moves" => token = Tokens::Moves,
            _ => match token {
                Tokens::Nothing => (),
                Tokens::Fen => {
                    fen.push_str(command);
                    fen.push(' ');
                },
                Tokens::Moves => {
                    moves.push(command.to_string())
                }
            }
        }
    }

    if !fen.is_empty() && !skip_fen {
        state_lock.pos = EnginePosition::from_fen(&fen);
    }

    for m in moves {
        let mo = uci_to_u16(&state_lock.pos, &m);
        state_lock.pos.make_move(mo);
    }
    drop(state_lock);
}

fn go(state: Arc<Mutex<State>>, commands: Vec<&str>) {
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
            _ => match token {
                Tokens::Ponder => (),
                Tokens::Depth => max_depth = command.parse::<u8>().unwrap_or(u8::MAX),
                Tokens::Nodes => max_nodes = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::MoveTime => max_move_time = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::WTime => times.wtime = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::BTime => times.btime = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::WInc => times.winc = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::BInc => times.binc = command.parse::<u64>().unwrap_or(u64::MAX),
                Tokens::MovesToGo => times.moves_to_go = Some(command.parse::<u8>().unwrap_or(u8::MAX)),
            }
        }
    }

    if !times.is_default() {
        let state_lock = state.lock().unwrap();
        max_move_time = times.to_movetime(state_lock.pos.board.side_to_move);
        drop(state_lock);
    }

    // SEARCHING ON SECOND THREAD
    let state_2 = state.clone();
    let search_thread = thread::spawn(move || {
        let state_lock = state_2.lock().unwrap();
        let position = state_lock.pos.clone();
        let abort_signal = state_lock.stop.clone();
        drop(state_lock);
        let mut search = Search::new(position, abort_signal, max_move_time, max_depth, max_nodes);
        let best_move = search.go();
        println!("bestmove {}", u16_to_uci(&best_move));
    });
    // join handle provided to master thread
    state.lock().unwrap().search_handle = Some(search_thread);
}
