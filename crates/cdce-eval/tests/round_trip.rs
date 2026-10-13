use cdce_eval::{CdceFull, CdceParams};
use gears::games::chess::pieces::PieceType::{Knight, Pawn, Rook};
use gears::games::chess::Color::White;
use gears::games::chess::Board;
use gears::general::board::BoardHelpers;
use gears::general::board::Strictness::Relaxed;
use gears::score::Score;
use motors::eval::Eval;

#[test]
fn test_param_round_trip() {
    let mut params = CdceParams::default();

    // Default values
    assert_eq!(params.val_rook, 500);
    assert_eq!(params.val_knight, 320);
    assert_eq!(params.ow_weight_knight, 12);

    // OW[Knight][Rook] = 12 * 500 / 100 = 60
    assert_eq!(params.ow_table[Knight as usize][Rook as usize], 60);
    // OW[Pawn][Rook] = 4 * 500 / 100 = 20
    assert_eq!(params.ow_table[Pawn as usize][Rook as usize], 20);

    // Update Val_Rook to 600
    params.set_option("Val_Rook", "600").expect("Failed to set Val_Rook");
    assert_eq!(params.val_rook, 600);

    // Recomputed OW table:
    // OW[Knight][Rook] = 12 * 600 / 100 = 72
    assert_eq!(params.ow_table[Knight as usize][Rook as usize], 72);
    // OW[Pawn][Rook] = 4 * 600 / 100 = 24
    assert_eq!(params.ow_table[Pawn as usize][Rook as usize], 24);

    // Update OW_Weight_Knight to 20
    params.set_option("OW_Weight_Knight", "20").expect("Failed to set OW_Weight_Knight");
    assert_eq!(params.ow_weight_knight, 20);
    // OW[Knight][Rook] = 20 * 600 / 100 = 120
    assert_eq!(params.ow_table[Knight as usize][Rook as usize], 120);

    // Test on CdceFull via Eval::set_option
    let mut eval = CdceFull::default();
    let pos = Board::from_fen("k7/8/8/8/8/8/8/K2R4 w - - 0 1", Relaxed).unwrap();
    let score_default = eval.eval(&pos, 0, White);

    eval.set_option("Val_Rook", "600").expect("Failed to set option on eval");
    let score_new = eval.eval(&pos, 0, White);

    // Increasing rook value from 500 to 600 should increase score by 100
    assert_eq!(score_new - score_default, Score(100));
}

#[test]
fn test_standard_set_applies_recomputed_ow_table() {
    let params = CdceParams::standard();
    assert_eq!(params.val_rook, 500);
    assert_eq!(params.val_knight, 320);
    assert_eq!(params.val_queen, 900);
    // OW[Knight][Rook] = 12 * 500 / 100 = 60
    assert_eq!(params.ow_table[Knight as usize][Rook as usize], 60);
}
