use crate::{
    state::{consts::Side, *},
    engine::{util::perft, Engine},
    AUTHOR, NAME, VERSION,
};
use std::{cmp::max, io, process, sync::{Arc, atomic::AtomicBool}, time::Instant};

pub fn run() {
    // uci preamble
    println!("id name {NAME} {VERSION}");
    println!("id author {AUTHOR}");
    println!("option name Hash type spin default 128 min 1 max 512");
    println!("uciok");

    // set up engine
    let abort_signal = Arc::new(AtomicBool::new(false));
    let mut engine = Engine::new(abort_signal);
    engine.hash_table.resize(1);

    // command loop
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands = input.split_whitespace().collect::<Vec<&str>>();
        if let Err(err) = match *commands.first().unwrap_or(&"oops") {
            // core UCI commands
            "isready" => Ok(println!("readyok")),
            "ucinewgame" => Ok(ucinewgame(&mut engine)),
            "position" => parse_position(&mut engine.position, commands),
            "go" => Ok(parse_go(&mut engine, commands)),

            // other commands
            "perft" => Ok(parse_perft::<false>(&mut engine.position, &commands)),
            "splitperft" => Ok(parse_perft::<true>(&mut engine.position, &commands)),
            "quit" => process::exit(0),
            _ => Ok(()),
        } {
            println!("{err}");
        }
    }
}

fn ucinewgame(engine: &mut Engine) {
    engine.position = Position::default();
    engine.hash_table.clear();
}

fn uci_to_u16(pos: &Position, m_str: &str) -> Result<u16, String> {
    // basic move info
    let from = square_str_to_index(&m_str[0..2])?;
    let mut to = square_str_to_index(&m_str[2..4])?;
    let stm = usize::from(pos.stm());

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

fn parse_position(pos: &mut Position, commands: Vec<&str>) -> Result<(), String> {
    let mut fen = String::new();
    let mut move_list = Vec::new();
    let mut moves = false;

    // process string
    for cmd in commands {
        match cmd {
            "position" | "fen" => {}
            "startpos" => *pos = Fens::STARTPOS.parse()?,
            "kiwipete" => *pos = Fens::KIWIPETE.parse()?,
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
        *pos = fen.parse()?;
    }

    for m in move_list {
        pos.r#do(uci_to_u16(pos, &m)?);
    }

    Ok(())
}

fn parse_perft<const SPLIT: bool>(pos: &mut Position, commands: &[&str]) {
    let depth = commands[1].parse().unwrap();
    let now = Instant::now();
    let count = perft::<SPLIT>(pos, depth);
    let time = now.elapsed();
    println!(
        "perft {depth} time {} nodes {count} ({:.2} Mnps)",
        time.as_millis(),
        count as f64 / time.as_micros() as f64
    );
}

fn parse_go(engine: &mut Engine, commands: Vec<&str>) {
    let mut token = 0;
    let mut times = [0, 0];
    let mut mtg = None;
    let mut alloc = 1000;
    let mut incs = [0, 0];
    const COMMANDS: [&str; 7] = ["go", "movetime", "wtime", "btime", "movestogo", "winc", "binc"];
    for cmd in commands {
        if let Some(x) = COMMANDS.iter().position(|&y| y == cmd) { token = x }
        else {
            match token {
                1 => alloc = cmd.parse::<i64>().unwrap(),
                2 => times[0] = max(cmd.parse::<i64>().unwrap(), 0),
                3 => times[1] = max(cmd.parse::<i64>().unwrap(), 0),
                4 => mtg = Some(cmd.parse::<i64>().unwrap()),
                5 => incs[0] = max(cmd.parse::<i64>().unwrap(), 0),
                6 => incs[1] = max(cmd.parse::<i64>().unwrap(), 0),
                _ => {},
            }
        }
    }

    // timing
    let side = engine.position.stm();
    let mytime = times[side];
    let myinc = incs[side];
    if mytime != 0 {
        alloc = mytime / mtg.unwrap_or(25) + 3 * myinc / 4
    }

    engine.limits.set_time(max(10, alloc - 10) as u128);
    engine.go();
}
