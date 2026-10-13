use cdce_eval::attack_maps::AttackMaps;
use cdce_eval::params::CdceParams;
use cdce_eval::terms::*;
use cdce_eval::CdceFull;
use gears::games::chess::pieces::PieceType::Rook;
use gears::games::chess::Color::{Black, White};
use gears::games::chess::Board;
use gears::general::board::BoardHelpers;
use gears::general::board::Strictness::Relaxed;
use gears::score::{p, Score};

#[test]
fn test_one_way_knight_on_rook() {
    let fen = "7k/8/6r1/4N3/8/8/8/4K3 w - - 0 1";
    let pos = Board::from_fen(fen, Relaxed).unwrap();
    let maps = AttackMaps::compute(&pos);
    let params = CdceParams::default();

    // White knight e5 attacks black rook g6 (one-way): OW[N][R] = 12 * 500 / 100 = 60
    let ow_white = eval_one_way::<_, false>(&pos, &maps, &params, White);
    assert_eq!(ow_white, p(60, 60));

    let ow_black = eval_one_way::<_, false>(&pos, &maps, &params, Black);
    assert_eq!(ow_black, p(0, 0));

    // Knight attacks only one piece, so no fork
    let fork_white = eval_forks(&pos, &maps, &params, White);
    assert_eq!(fork_white, p(0, 0));
}

#[test]
fn test_mutual_rook_on_rook() {
    let fen = "k2r4/8/8/8/8/8/8/K2R4 w - - 0 1";
    let pos = Board::from_fen(fen, Relaxed).unwrap();
    let maps = AttackMaps::compute(&pos);
    let params = CdceParams::default();

    // Mutual rook on rook: mutual attacks contribute 0 one-way
    let ow_white = eval_one_way::<_, false>(&pos, &maps, &params, White);
    assert_eq!(ow_white, p(0, 0));

    let ow_black = eval_one_way::<_, false>(&pos, &maps, &params, Black);
    assert_eq!(ow_black, p(0, 0));

    // Both rooks on open d-file
    let rook_white = eval_rook_activity(&pos, &maps, &params, White);
    assert_eq!(rook_white, p(25, 25));

    let rook_black = eval_rook_activity(&pos, &maps, &params, Black);
    assert_eq!(rook_black, p(25, 25));
}

#[test]
fn test_pawn_attacking_rook() {
    let fen = "k7/8/8/3r4/2P5/8/8/K7 w - - 0 1";
    let pos = Board::from_fen(fen, Relaxed).unwrap();
    let maps = AttackMaps::compute(&pos);
    let params = CdceParams::default();

    // White pawn c4 attacks black rook d5: OW[P][R] = 4 * 500 / 100 = 20
    let ow_white = eval_one_way::<_, false>(&pos, &maps, &params, White);
    assert_eq!(ow_white, p(20, 20));

    let ow_black = eval_one_way::<_, false>(&pos, &maps, &params, Black);
    assert_eq!(ow_black, p(0, 0));
}

#[test]
fn test_royal_fork() {
    // Note: White king on h1 instead of a1 because a1 is on the g7-a1 diagonal,
    // which put White in check when it is Black's turn to move.
    let fen = "3k4/6q1/4N3/8/8/8/8/7K b - - 0 1";
    let pos = Board::from_fen(fen, Relaxed).unwrap();
    let maps = AttackMaps::compute(&pos);
    let params = CdceParams::default();

    // Knight e6 forks Kd8 and Qg7 (targets in {N, R, Q, K})
    let forks_white = eval_forks(&pos, &maps, &params, White);
    assert_eq!(forks_white, p(40, 40));

    // One-way: OW[N][Q] = 12 * 900 / 100 = 108; OW[N][K] = 0
    let ow_white = eval_one_way::<_, false>(&pos, &maps, &params, White);
    assert_eq!(ow_white, p(108, 108));
}

#[test]
fn test_fork_of_two_bishops() {
    let fen = "k7/2b1b3/8/3N4/8/8/8/K7 w - - 0 1";
    let pos = Board::from_fen(fen, Relaxed).unwrap();
    let maps = AttackMaps::compute(&pos);
    let params = CdceParams::default();

    // Bishops are excluded from FORK_TARGETS, so fork bonus is 0
    let forks_white = eval_forks(&pos, &maps, &params, White);
    assert_eq!(forks_white, p(0, 0));

    // One-way: OW[N][B] * 2 = (12 * 330 / 100) * 2 = 39 * 2 = 78
    let ow_white = eval_one_way::<_, false>(&pos, &maps, &params, White);
    assert_eq!(ow_white, p(78, 78));
}

#[test]
fn test_open_vs_semi_open_file() {
    let fen_open = "k7/8/8/8/8/8/8/3RK3 w - - 0 1";
    let pos_open = Board::from_fen(fen_open, Relaxed).unwrap();
    let maps_open = AttackMaps::compute(&pos_open);
    let params = CdceParams::default();

    let rook_open = eval_rook_activity(&pos_open, &maps_open, &params, White);
    assert_eq!(rook_open, p(25, 25));

    let fen_semi = "k7/3p4/8/8/8/8/8/3RK3 w - - 0 1";
    let pos_semi = Board::from_fen(fen_semi, Relaxed).unwrap();
    let maps_semi = AttackMaps::compute(&pos_semi);

    let rook_semi = eval_rook_activity(&pos_semi, &maps_semi, &params, White);
    assert_eq!(rook_semi, p(12, 12));
}

#[test]
fn test_rook_lift_rank3_full_and_zero_phase() {
    // Full phase
    let fen_full = "rnbqkbnr/pppppppp/8/8/8/R7/PPPPPPPP/1NBQKBNR w Kkq - 0 1";
    let pos_full = Board::from_fen(fen_full, Relaxed).unwrap();
    let maps_full = AttackMaps::compute(&pos_full);
    let params = CdceParams::default();

    assert_eq!(pos_full.phase(), 24);
    let rook_act_full = eval_rook_activity(&pos_full, &maps_full, &params, White);
    // off_home_rank(25) + rank3(20) = 45 mg only
    assert_eq!(rook_act_full, p(45, 0));
    assert_eq!(rook_act_full.taper(24, 24), Score(45));

    // Zero (near-zero) phase
    let fen_zero = "k7/p7/8/8/8/R7/P7/4K3 w - - 0 1";
    let pos_zero = Board::from_fen(fen_zero, Relaxed).unwrap();
    let maps_zero = AttackMaps::compute(&pos_zero);

    assert_eq!(pos_zero.phase(), 2);
    let rook_act_zero = eval_rook_activity(&pos_zero, &maps_zero, &params, White);
    assert_eq!(rook_act_zero, p(45, 0));
    // (45 * 2 + 0 * 22) / 24 = 90 / 24 = 3
    assert_eq!(rook_act_zero.taper(2, 24), Score(3));
}
