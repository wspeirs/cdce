use cdce_eval::{CdceCore, CdceFull, CdceNoForks, CdceNoMobility, CdceNoOneWay, CdceNoRook};
use gears::games::chess::Board;
use gears::general::board::{BoardHelpers, BoardTrait};
use gears::general::board::Strictness::Relaxed;

/// Flips a FEN string vertically and inverts colors.
fn flip_fen(fen: &str) -> String {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    // 1. Piece placement: reverse ranks, swap piece cases
    let ranks: Vec<&str> = parts[0].split('/').collect();
    let flipped_ranks: Vec<String> = ranks
        .into_iter()
        .rev()
        .map(|rank| {
            rank.chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        c.to_ascii_lowercase()
                    } else if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }
                })
                .collect()
        })
        .collect();
    let flipped_placement = flipped_ranks.join("/");

    // 2. Active color
    let flipped_active = if parts.len() > 1 {
        if parts[1] == "w" { "b" } else { "w" }
    } else {
        "b"
    };

    // 3. Castling rights
    let flipped_castling = if parts.len() > 2 {
        if parts[2] == "-" {
            "-".to_string()
        } else {
            let mut s = String::new();
            for c in parts[2].chars() {
                if c.is_ascii_uppercase() {
                    s.push(c.to_ascii_lowercase());
                } else if c.is_ascii_lowercase() {
                    s.push(c.to_ascii_uppercase());
                } else {
                    s.push(c);
                }
            }
            s
        }
    } else {
        "-".to_string()
    };

    // 4. En passant square
    let flipped_ep = if parts.len() > 3 {
        let ep = parts[3];
        if ep == "-" || ep.len() < 2 {
            "-".to_string()
        } else {
            let file = ep.chars().next().unwrap();
            let rank_char = ep.chars().nth(1).unwrap();
            let rank: u32 = rank_char.to_digit(10).unwrap_or(3);
            let flipped_rank = 9 - rank;
            format!("{file}{flipped_rank}")
        }
    } else {
        "-".to_string()
    };

    let halfmove = parts.get(4).copied().unwrap_or("0");
    let fullmove = parts.get(5).copied().unwrap_or("1");

    format!("{flipped_placement} {flipped_active} {flipped_castling} {flipped_ep} {halfmove} {fullmove}")
}

pub fn flip_board(pos: &Board) -> Board {
    let fen = pos.to_string();
    let flipped_fen = flip_fen(&fen);
    Board::from_fen(&flipped_fen, Relaxed).expect("Failed to parse flipped FEN")
}

#[test]
fn test_color_flip_symmetry_startpos() {
    let pos = Board::startpos();
    let flipped = flip_board(&pos);

    let full = CdceFull::default();
    let core = CdceCore::default();

    assert_eq!(full.do_eval(&pos), full.do_eval(&flipped));
    assert_eq!(core.do_eval(&pos), core.do_eval(&flipped));
}

#[test]
fn test_color_flip_symmetry_bench_positions() {
    let full = CdceFull::default();
    let core = CdceCore::default();
    let no_ow = CdceNoOneWay::default();
    let no_forks = CdceNoForks::default();
    let no_rook = CdceNoRook::default();
    let no_mob = CdceNoMobility::default();

    for pos in Board::bench_positions() {
        let flipped = flip_board(&pos);

        assert_eq!(full.do_eval(&pos), full.do_eval(&flipped), "Failed on pos: {pos}");
        assert_eq!(core.do_eval(&pos), core.do_eval(&flipped), "Failed on pos: {pos}");
        assert_eq!(no_ow.do_eval(&pos), no_ow.do_eval(&flipped), "Failed on pos: {pos}");
        assert_eq!(no_forks.do_eval(&pos), no_forks.do_eval(&flipped), "Failed on pos: {pos}");
        assert_eq!(no_rook.do_eval(&pos), no_rook.do_eval(&flipped), "Failed on pos: {pos}");
        assert_eq!(no_mob.do_eval(&pos), no_mob.do_eval(&flipped), "Failed on pos: {pos}");
    }
}
