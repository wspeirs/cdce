//! `TunedValues::from_params` must evaluate exactly like `CdceParams` for every term.
//! This checks the shared feature layout (`layout.rs`) against the hand-written `CdceParams` impl.

use cdce_eval::{CdceEval, CdceParams, TunedValues, CDCE_FULL_MASK};
use gears::games::chess::Board;
use gears::general::board::BoardHelpers;
use gears::general::board::Strictness::Relaxed;

const FENS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPpP/R3K2R w KQkq - 0 1",
    "8/2k5/4p3/1P2P1p1/2K3P1/8/8/8 w - - 0 1",
    "4k3/8/8/3pP3/8/8/8/R3K2R b - - 0 1",
];

#[test]
fn test_tuned_from_params_matches_params_eval() {
    for params in [CdceParams::default(), CdceParams::standard()] {
        let concrete = CdceEval::<CDCE_FULL_MASK, CdceParams>::new(params.clone());
        let tuned = CdceEval::<CDCE_FULL_MASK, TunedValues>::new(TunedValues::from_params(&params));
        for fen in FENS {
            let pos = Board::from_fen(fen, Relaxed).expect("valid FEN");
            assert_eq!(concrete.do_eval(&pos), tuned.do_eval(&pos), "mismatch on {fen}");
        }
    }
}
