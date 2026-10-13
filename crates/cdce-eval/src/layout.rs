//! Feature layout shared by the tuner (`cdce-tune`) and the engine (`TunedValues`).
//!
//! Every tunable term is one feature index. A phased weight is stored at `2 * idx` (middlegame) and
//! `2 * idx + 1` (endgame) by pliers. Keep these constants in sync with `TunedValues` and `TuneValues`;
//! both read them from here.

use gears::games::chess::pieces::PieceType;

/// Material by piece, Pawn..Queen (5 features).
pub const MATERIAL: usize = 0;
/// One-way attacks, `attacker * 6 + target` over all 36 piece pairs.
pub const ONE_WAY: usize = 5;
pub const KNIGHT_FORK: usize = 41;
/// Rook activity: off home rank, relative rank 3, open file, semi-open file, connected, pawn gate.
pub const ROOK: usize = 42;
pub const ROOK_OFF_HOME_RANK: usize = ROOK;
pub const ROOK_RANK3: usize = ROOK + 1;
pub const ROOK_FILE_OPEN: usize = ROOK + 2;
pub const ROOK_FILE_SEMI_OPEN: usize = ROOK + 3;
pub const ROOK_CONNECTED: usize = ROOK + 4;
pub const ROOK_PAWN_GATE: usize = ROOK + 5;
/// Mobility by minor/major piece, Knight..Queen (4 features).
pub const MOBILITY: usize = 48;
/// King-zone attacks by minor/major piece, Knight..Queen (4 features).
pub const KING_ATTACKER: usize = 52;
pub const KING_SHIELD: usize = 56;
pub const PAWN_DOUBLED: usize = 57;
pub const PAWN_ISOLATED: usize = 58;
/// Passed pawn by relative rank, 0..8 (8 features).
pub const PAWN_PASSED: usize = 59;

/// Total number of tunable features.
pub const NUM_FEATURES: usize = 67;

/// Index within `MOBILITY` / `KING_ATTACKER` for Knight..Queen; `None` for every other piece.
pub fn minor_major(piece: PieceType) -> Option<usize> {
    match piece {
        PieceType::Knight => Some(0),
        PieceType::Bishop => Some(1),
        PieceType::Rook => Some(2),
        PieceType::Queen => Some(3),
        _ => None,
    }
}
