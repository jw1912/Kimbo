use crate::{state::*, AUTHOR, NAME, VERSION};
use std::{io, process, time::Instant};

pub fn run() {
    // uci preamble
    println!("id name {NAME} {VERSION}");
    println!("id author {AUTHOR}");
    println!("option name Hash type spin default 128 min 1 max 512");
    println!("uciok");

    // set up engine
    let mut pos = Position::default();

    // command loop
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let commands = input.split_whitespace().collect::<Vec<&str>>();
        if let Err(err) = match *commands.first().unwrap_or(&"oops") {
            "isready" => Ok(println!("readyok")),
            "position" => parse_position(&mut pos, commands),
            "perft" => Ok(parse_perft::<false>(&mut pos, &commands)),
            "splitperft" => Ok(parse_perft::<true>(&mut pos, &commands)),
            "quit" => process::exit(0),
            _ => Ok(()),
        } {
            println!("{err}");
        }
    }
}

fn uci_to_u16(pos: &Position, m_str: &str) -> Result<u16, String> {
    // basic move info
    let from = square_str_to_index(&m_str[0..2])?;
    let to = square_str_to_index(&m_str[2..4])?;
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
        if r#move == m & 0xFFF && (m_str.len() < 5 || flag == m & 0xB000) {
            return Ok(m);
        }
    }

    Err(String::from("error parsing {m_str:?}"))
}

fn u16_to_uci(m: u16) -> String {
    let index_to_square = |i| format!("{}{}", ((i & 7) as u8 + b'a') as char, (i / 8) + 1);

    // extract move info
    let from = index_to_square((m >> 6) & 63);
    let to = index_to_square(m & 63);
    let flag = (m & MoveFlag::ALL) >> 12;
    let promo = if flag >= 8 {
        ["n", "b", "r", "q"][usize::from(flag & 0b11)]
    } else {
        ""
    };

    format!("{from}{to}{promo} ")
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

fn perft<const SPLIT: bool>(pos: &mut Position, depth: u8) -> u64 {
    let moves = pos.generate::<{ MoveType::ALL }>();
    let mut positions = 0;
    for i in 0..moves.len() {
        let m = moves[i].r#move();
        if pos.r#do(m) {
            continue;
        }

        let count = if depth > 1 {
            perft::<false>(pos, depth - 1)
        } else {
            1
        };

        pos.undo();

        positions += count;
        if SPLIT {
            println!("{}: {count}", u16_to_uci(m));
        }
    }
    positions
}
