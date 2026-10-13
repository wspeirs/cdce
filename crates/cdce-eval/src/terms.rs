use gears::games::chess::pieces::PieceType::*;
use gears::games::chess::squares::{ChessboardSize, Square};
use gears::games::chess::Color::White;
use gears::games::chess::{Board, ChessBitboardTrait, Color};
use gears::games::ColorTrait;
use gears::general::squares::RectangularCoordinates;
use gears::general::bitboards::chessboard::{Bitboard, KINGS};
use gears::general::bitboards::{BitboardTrait, KnownSizeBitboard, RawBitboardTrait};
use gears::general::board::{BitboardBoard, BoardTrait};
use crate::attack_maps::AttackMaps;
use crate::values::CdceValues;

/// Material evaluation term for the given side.
pub fn eval_material<V: CdceValues>(pos: &Board, values: &V, side: Color) -> V::Score {
    let mut score = V::Score::default();
    for piece in [Pawn, Knight, Bishop, Rook, Queen] {
        let count = pos.col_piece_bb(side, piece).num_ones();
        if count > 0 {
            score += values.material(piece) * count;
        }
    }
    score
}

/// One-way attacks evaluation term for the given side.
/// For each attacker square `a` of `side` and `t ∈ att[a] & enemy`:
/// if `!att[t].has(a)`, add `OW[type(a)][type(t)]`.
pub fn eval_one_way<V: CdceValues, const DEFENDED_ONLY: bool>(
    pos: &Board,
    maps: &AttackMaps,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let own_bb = pos.player_bb(side);
    let enemy_bb = pos.player_bb(side.other());

    for a_sq in own_bb {
        let piece_a = pos.piece_type_on(a_sq);
        let targets = maps.att[a_sq.bb_idx()] & enemy_bb;
        for t_sq in targets {
            if !maps.att[t_sq.bb_idx()].has(a_sq) {
                if DEFENDED_ONLY && !maps.attacks_by_side[side.other() as usize].has(t_sq) {
                    continue;
                }
                let piece_t = pos.piece_type_on(t_sq);
                score += values.one_way(piece_a, piece_t);
            }
        }
    }
    score
}

/// Knight forks evaluation term for the given side.
/// For each knight: `(att[n] & enemy & FORK_TARGETS).count() >= 2` adds Fork_Bonus (40).
/// FORK_TARGETS = {Knight, Rook, Queen, King}.
pub fn eval_forks<V: CdceValues>(
    pos: &Board,
    maps: &AttackMaps,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let enemy = side.other();
    let enemy_fork_targets = pos.col_piece_bb(enemy, Knight)
        | pos.col_piece_bb(enemy, Rook)
        | pos.col_piece_bb(enemy, Queen)
        | pos.col_piece_bb(enemy, King);

    for n_sq in pos.col_piece_bb(side, Knight) {
        if (maps.att[n_sq.bb_idx()] & enemy_fork_targets).num_ones() >= 2 {
            score += values.knight_fork();
        }
    }
    score
}

/// Rook activity evaluation term for the given side.
pub fn eval_rook_activity<V: CdceValues>(
    pos: &Board,
    maps: &AttackMaps,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let rooks = pos.col_piece_bb(side, Rook);
    let home_rank: u8 = if side == White { 0 } else { 7 };
    let rank3: u8 = if side == White { 2 } else { 5 };
    let rank4: u8 = if side == White { 3 } else { 4 };

    for r_sq in rooks {
        // Off home rank (mg only)
        if r_sq.rank() != home_rank {
            score += values.rook_off_home_rank();
        }
        // On relative rank 3 (mg only)
        if r_sq.rank() == rank3 {
            score += values.rook_relative_rank3();
        }
        // Open and semi-open files
        let file_bb = Bitboard::file(r_sq.file());
        let own_pawns = pos.col_piece_bb(side, Pawn) & file_bb;
        let enemy_pawns = pos.col_piece_bb(side.other(), Pawn) & file_bb;
        if own_pawns.is_zero() {
            if enemy_pawns.is_zero() {
                score += values.rook_file_open();
            } else {
                score += values.rook_file_semi_open();
            }
        }
        // Rooks connected (shared file/rank unobstructed)
        if rooks.num_ones() >= 2 && (maps.att[r_sq.bb_idx()] & rooks).has_any() {
            score += values.rook_connected();
        }
    }

    // A/H pawn on relative rank 3 or 4 while phase > gate
    if pos.phase() > values.rook_pawn_phase_gate() {
        let own_pawns = pos.col_piece_bb(side, Pawn);
        for pawn_sq in own_pawns {
            let f = pawn_sq.file();
            if f == 0 || f == 7 {
                let r = pawn_sq.rank();
                if r == rank3 || r == rank4 {
                    score += values.rook_pawn_gate();
                }
            }
        }
    }

    score
}

/// Mobility evaluation term for the given side.
/// `popcount(att[sq] & !own & !enemy_pawn_attacks)` per piece type.
pub fn eval_mobility<V: CdceValues>(
    pos: &Board,
    maps: &AttackMaps,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let enemy_pawn_attacks = pos.col_piece_bb(side.other(), Pawn).pawn_attacks(side.other());
    let own_bb = pos.player_bb(side);

    for piece in [Knight, Bishop, Rook, Queen] {
        for sq in pos.col_piece_bb(side, piece) {
            let mob_bb = maps.att[sq.bb_idx()] & !own_bb & !enemy_pawn_attacks;
            let count = mob_bb.num_ones();
            if count > 0 {
                score += values.mobility(piece) * count;
            }
        }
    }
    score
}

/// King safety evaluation term for the given side.
pub fn eval_king_safety<V: CdceValues>(
    pos: &Board,
    maps: &AttackMaps,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let king_sq = pos.king_sq(side);
    let king_zone = Bitboard::single_piece_for(king_sq, ChessboardSize::default()) | KINGS[king_sq.bb_idx()];

    // Enemy attacks on the king zone
    for piece in [Knight, Bishop, Rook, Queen] {
        for e_sq in pos.col_piece_bb(side.other(), piece) {
            let hits = (maps.att[e_sq.bb_idx()] & king_zone).num_ones();
            if hits > 0 {
                score -= values.king_attacker(piece) * hits;
            }
        }
    }

    // Pawn shield count on 3 files in front of king (mg only)
    let kf = king_sq.file();
    let f_min = kf.saturating_sub(1);
    let f_max = (kf + 1).min(7);
    let mut shield_bb = Bitboard::default();

    for f in f_min..=f_max {
        if side == White {
            for r in (king_sq.rank() + 1)..=((king_sq.rank() + 2).min(7)) {
                shield_bb |= Bitboard::single_piece_for(Square::from_rank_file(r, f), ChessboardSize::default());
            }
        } else {
            for r in (king_sq.rank().saturating_sub(2))..=(king_sq.rank().saturating_sub(1)) {
                shield_bb |= Bitboard::single_piece_for(Square::from_rank_file(r, f), ChessboardSize::default());
            }
        }
    }

    let shield_pawns = (pos.col_piece_bb(side, Pawn) & shield_bb).num_ones();
    if shield_pawns > 0 {
        score += values.king_shield() * shield_pawns;
    }

    score
}

/// Pawn structure evaluation term for the given side.
pub fn eval_pawn_structure<V: CdceValues>(
    pos: &Board,
    values: &V,
    side: Color,
) -> V::Score {
    let mut score = V::Score::default();
    let pawns = pos.col_piece_bb(side, Pawn);
    let enemy_pawns = pos.col_piece_bb(side.other(), Pawn);

    // Doubled pawns
    let mut doubled = 0;
    for file in 0..8 {
        let count = (pawns & Bitboard::file(file)).num_ones();
        if count > 1 {
            doubled += count - 1;
        }
    }
    if doubled > 0 {
        score -= values.pawn_doubled() * doubled;
    }

    // Isolated pawns
    for p_sq in pawns {
        let f = p_sq.file();
        let mut adj_files = Bitboard::default();
        if f > 0 {
            adj_files |= Bitboard::file(f - 1);
        }
        if f < 7 {
            adj_files |= Bitboard::file(f + 1);
        }
        if (pawns & adj_files).is_zero() {
            score -= values.pawn_isolated();
        }
    }

    // Passed pawns
    for p_sq in pawns {
        let f = p_sq.file();
        let r = p_sq.rank();
        let mut files_span = Bitboard::file(f);
        if f > 0 {
            files_span |= Bitboard::file(f - 1);
        }
        if f < 7 {
            files_span |= Bitboard::file(f + 1);
        }

        let mut front_ranks = Bitboard::default();
        if side == White {
            for rank in (r + 1)..8 {
                front_ranks |= Bitboard::rank(rank);
            }
        } else {
            for rank in 0..r {
                front_ranks |= Bitboard::rank(rank);
            }
        }

        if (enemy_pawns & files_span & front_ranks).is_zero() {
            let rel_rank = if side == White { r as usize } else { 7 - r as usize };
            score += values.pawn_passed(rel_rank);
        }
    }

    score
}
