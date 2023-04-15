use crate::{
    engine::{util::perft, Engine},
    state::{consts::Side, *},
    AUTHOR, NAME, VERSION, tables::HashTable,
};
use std::{
    cmp::max,
    io, process,
    sync::{atomic::{AtomicBool, Ordering},  Arc, Mutex},
    time::Instant,
};

struct UciState {
    position: Position,
    abort_signal: Arc<AtomicBool>,
    hash_table: Arc<HashTable>,
}

pub fn run() {
    // uci preamble
    println!("id name {NAME} {VERSION}");
    println!("id author {AUTHOR}");
    println!("option name Hash type spin default 1 min 1 max 512");
    println!("option name Clear Hash type button");
    println!("option name UCI_Chess960 type check default false");
    println!("uciok");

    // set up state
    let state = Arc::new(Mutex::new(UciState {
        position: Position::default(),
        abort_signal: Arc::new(AtomicBool::new(false)),
        hash_table: Arc::new(HashTable::new(1)),
    }));

    // command loop
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let commands = input.split_whitespace().collect::<Vec<&str>>();
        if let Err(err) = parse_commands(state.clone(), commands) {
            println!("{err}");
        }
    }
}

fn parse_commands(state: Arc<Mutex<UciState>>, commands: Vec<&str>) -> Result<(), String> {
    match *commands.first().unwrap_or(&"oops") {
        // core UCI commands
        "isready" => println!("readyok"),
        "ucinewgame" => ucinewgame(state),
        "position" => return parse_position(state, commands),
        "go" => parse_go(state, commands),
        "setoption" => parse_setoption(state, commands),

        // commands during search
        "stop" => handle_stop(state),
    
        // other commands
        "perft" => parse_perft::<false>(state, &commands),
        "splitperft" => parse_perft::<true>(state, &commands),
        "quit" => process::exit(0),
        _ => {},
    }
    Ok(())
}

fn ucinewgame(state: Arc<Mutex<UciState>>) {
    let mut stateref = state.lock().unwrap();
    stateref.position = Position::default();
    stateref.hash_table.clear();
}

fn handle_stop(state: Arc<Mutex<UciState>>) {
    let stateref = state.lock().unwrap();
    stateref.abort_signal.store(true, Ordering::Relaxed)
}

fn uci_to_u16(pos: &Position, m_str: &str) -> Result<u16, String> {
    // basic move info
    let from = square_str_to_index(&m_str[0..2])?;
    let mut to = square_str_to_index(&m_str[2..4])?;
    let stm = pos.stm();

    // chess960
    if pos.chess960() && pos.side(stm) & (1 << to) > 0 {
        let side = 56 * (from / 56);
        let castle;
        (to, castle) = if to == pos.castling_rooks()[Side::WHITE] as u16 + side {
            (2 + side, MoveFlag::QS_CASTLE)
        } else {
            (6 + side, MoveFlag::KS_CASTLE)
        };

        return Ok(castle | from << 6 | to);
    }

    let r#move = from << 6 | to;

    // promotion?
    let flag = match m_str.chars().nth(4).unwrap_or('f') {
        'n' => MoveFlag::KNIGHT_PROMO,
        'b' => MoveFlag::BISHOP_PROMO,
        'r' => MoveFlag::ROOK_PROMO,
        'q' => MoveFlag::QUEEN_PROMO,
        _ => MoveFlag::QUIET,
    };

    // match to move in move list
    let possible_moves = pos.generate::<{ MoveType::ALL }>();
    for i in 0..possible_moves.len() {
        let m = possible_moves[i].r#move();
        let mflag = m & 0xF000;
        if r#move == m & 0xFFF
            && (m_str.len() < 5 || flag == mflag & 0xB000)
            && (!pos.chess960() || (mflag != MoveFlag::KS_CASTLE && mflag != MoveFlag::QS_CASTLE))
        {
            return Ok(m);
        }
    }

    Err(String::from("error parsing {m_str:?}"))
}

fn parse_position(state: Arc<Mutex<UciState>>, commands: Vec<&str>) -> Result<(), String> {
    let mut stateref = state.lock().unwrap();

    let mut fen = String::new();
    let mut move_list = Vec::new();
    let mut moves = false;

    // process string
    for cmd in commands {
        match cmd {
            "position" | "fen" => {}
            "startpos" => stateref.position = Fens::STARTPOS.parse()?,
            "kiwipete" => stateref.position = Fens::KIWIPETE.parse()?,
            "moves" => moves = true,
            _ => {
                if moves {
                    move_list.push(cmd.to_string())
                } else {
                    fen.push_str(format!("{cmd} ").as_str())
                }
            }
        }
    }

    // set position
    if !fen.is_empty() {
        stateref.position = fen.parse()?;
    }

    for m_str in move_list {
        let m = uci_to_u16(&stateref.position, &m_str)?;
        stateref.position.r#do(m);
    }

    Ok(())
}

fn parse_perft<const SPLIT: bool>(state: Arc<Mutex<UciState>>, commands: &[&str]) {
    let mut stateref = state.lock().unwrap();
    let depth = commands[1].parse().unwrap();
    let now = Instant::now();
    let count = perft::<SPLIT>(&mut stateref.position, depth);
    let time = now.elapsed();
    println!(
        "perft {depth} time {} nodes {count} ({:.2} Mnps)",
        time.as_millis(),
        count as f64 / time.as_micros() as f64
    );
}

fn parse_go(state: Arc<Mutex<UciState>>, commands: Vec<&str>) {
    let stateref = state.lock().unwrap();
    let mut token = 0;
    let mut times = [0, 0];
    let mut mtg = None;
    let mut alloc = 1000;
    let mut incs = [0, 0];
    const COMMANDS: [&str; 7] = [
        "go",
        "movetime",
        "wtime",
        "btime",
        "movestogo",
        "winc",
        "binc",
    ];
    for cmd in commands {
        if let Some(x) = COMMANDS.iter().position(|&y| y == cmd) {
            token = x
        } else {
            match token {
                1 => alloc = cmd.parse::<i64>().unwrap(),
                2 => times[0] = max(cmd.parse::<i64>().unwrap(), 0),
                3 => times[1] = max(cmd.parse::<i64>().unwrap(), 0),
                4 => mtg = Some(cmd.parse::<i64>().unwrap()),
                5 => incs[0] = max(cmd.parse::<i64>().unwrap(), 0),
                6 => incs[1] = max(cmd.parse::<i64>().unwrap(), 0),
                _ => {}
            }
        }
    }

    let mut engine = Engine::new(
        stateref.position.clone(),
        stateref.hash_table.clone(),
        stateref.abort_signal.clone(),
    );

    // timing
    let side = engine.position.stm();
    let mytime = times[side];
    let myinc = incs[side];
    if mytime != 0 {
        alloc = mytime / mtg.unwrap_or(25) + 3 * myinc / 4
    }

    engine.limits.set_time(max(10, alloc - 10) as u128);

    std::thread::spawn(move || {
        engine.go();
    });
}

fn parse_setoption(state: Arc<Mutex<UciState>>, commands: Vec<&str>) {
    let mut stateref = state.lock().unwrap();
    match &commands[1..] {
        ["name", "Hash", "value", x] => {
            stateref.hash_table = Arc::new(HashTable::new(x.parse().unwrap()));
        }
        ["name", "Clear", "Hash"] => {
            stateref.hash_table.clear();
        }
        ["name", "UCI_Chess960", "value", _] => {}
        _ => println!("unrecognised option"),
    }
}
