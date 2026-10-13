//! Reader's guide to the CDCE evaluation function.
//!
//! CDCE evaluates chess positions using a sum of terms parameterized by compile-time masks.
//! Active terms are evaluated in the following order (as implemented in `eval_white_relative`):
//!
//! 1. **Material**: Piece counts per side multiplied by piece values.
//!    - Pawn: 100 (`Val_Pawn`)
//!    - Knight: 320 (`Val_Knight`)
//!    - Bishop: 330 (`Val_Bishop`)
//!    - Rook: 500 (`Val_Rook`)
//!    - Queen: 900 (`Val_Queen`)
//!
//! 2. **One-Way Attacks**: Asymmetric attacks where a friendly piece attacks an enemy piece that does
//!    not counter-attack it (`!att[target].has(attacker)`). Adds `(attacker_weight * target_value) / 100`.
//!    - Knight: 12% (`OW_Weight_Knight`)
//!    - Bishop: 6% (`OW_Weight_Bishop`)
//!    - Rook: 6% (`OW_Weight_Rook`)
//!    - Queen: 3% (`OW_Weight_Queen`)
//!    - Pawn: 4% (`OW_Weight_Pawn`)
//!
//! 3. **Knight Forks**: Tactical double attack bonus when a knight attacks two or more enemy targets
//!    among {Knight, Rook, Queen, King}.
//!    - Fork bonus: 40 (`Fork_Bonus`)
//!
//! 4. **Rook Activity**: Positional mobility and file/rank control bonuses for rooks.
//!    - Rook off home rank: 25 (`Rook_OffHomeRank`)
//!    - Rook on relative rank 3: 20 (`Rook_Rank3`)
//!    - Rook on open file: 25 (`Rook_FileOpen`)
//!    - Rook on semi-open file: 12 (`Rook_FileSemiOpen`)
//!    - Rooks connected: 10 (`Rook_Connected`)
//!    - Rook pawn gate (a/h pawn on relative rank 3 or 4 while phase > phase gate): 8 (`Rook_PawnGate`)
//!    - Rook pawn phase gate threshold: 14 (`Rook_PawnPhaseGate`)
//!
//! 5. **Mobility**: Count of safe reachable squares (not occupied by friendly pieces and not attacked
//!    by enemy pawns) per piece type.
//!    - Knight: 4 (`Mobility_Knight`)
//!    - Bishop: 5 (`Mobility_Bishop`)
//!    - Rook: 3 (`Mobility_Rook`)
//!    - Queen: 1 (`Mobility_Queen`)
//!
//! **Tempo**:
//! - Side-to-move bonus applied during finalization: 10 (`Tempo`).
//!
//! King safety and pawn structure code exists but is compiled out of CDCE (mask), and scores are side-to-move relative after finalize.

use std::fmt::Display;
use gears::games::chess::Color::{Black, White};
use gears::games::chess::{Board, Color};
use gears::general::board::BoardTrait;
use gears::general::common::{Res, StaticallyNamedEntity};
use gears::score::{Score, ScoreT};
use gears::ugi::EngineOption;
use motors::eval::{Eval, ScoreType};

use crate::attack_maps::AttackMaps;
use crate::params::CdceParams;
use crate::terms::*;
use crate::values::CdceValues;

// Term bitmask flags
pub const TERM_MATERIAL: u32 = 1 << 0;
pub const TERM_ONE_WAY: u32 = 1 << 1;
pub const TERM_FORKS: u32 = 1 << 2;
pub const TERM_ROOK: u32 = 1 << 3;
pub const TERM_MOBILITY: u32 = 1 << 4;
pub const TERM_KING_SAFETY: u32 = 1 << 5;
pub const TERM_PAWN_STRUCTURE: u32 = 1 << 6;
pub const TERM_ONE_WAY_DEFENDED: u32 = 1 << 7;

/// Full CDCE evaluation with all terms enabled.
pub const CDCE_FULL_MASK: u32 = TERM_MATERIAL
    | TERM_ONE_WAY
    | TERM_FORKS
    | TERM_ROOK
    | TERM_MOBILITY;

/// Core CDCE evaluation (material + one-way + forks + rook; no standard terms).
pub const CDCE_CORE_MASK: u32 = TERM_MATERIAL | TERM_ONE_WAY | TERM_FORKS | TERM_ROOK;

// Variant type aliases
pub type CdceFull = CdceEval<CDCE_FULL_MASK>;
pub type CdceCore = CdceEval<CDCE_CORE_MASK>;
pub type CdceNoOneWay = CdceEval<{ CDCE_FULL_MASK & !TERM_ONE_WAY }>;
pub type CdceNoForks = CdceEval<{ CDCE_FULL_MASK & !TERM_FORKS }>;
pub type CdceNoRook = CdceEval<{ CDCE_FULL_MASK & !TERM_ROOK }>;
pub type CdceNoMobility = CdceEval<{ CDCE_FULL_MASK & !TERM_MOBILITY }>;
pub type CdceOneWayDefended = CdceEval<{ (CDCE_FULL_MASK & !TERM_ONE_WAY) | TERM_ONE_WAY_DEFENDED }>;
pub type CdceMaterial = CdceEval<TERM_MATERIAL>;

/// CDCE evaluation function, parameterized by a compile-time term bitmask.
/// Disabled terms compile out completely with zero runtime cost.
#[derive(Debug, Clone)]
pub struct CdceEval<const TERMS: u32, V: CdceValues = CdceParams> {
    pub values: V,
}

impl<const TERMS: u32> Default for CdceEval<TERMS, CdceParams> {
    fn default() -> Self {
        Self {
            values: CdceParams::default(),
        }
    }
}

impl<const TERMS: u32> Default for CdceEval<TERMS, crate::TunedValues> {
    fn default() -> Self {
        Self::new(crate::TunedValues::tuned())
    }
}

impl<const TERMS: u32, V: CdceValues> CdceEval<TERMS, V> {
    pub fn new(values: V) -> Self {
        Self { values }
    }

    /// Evaluates the board position from White's perspective.
    pub fn eval_white_relative(&self, pos: &Board, maps: &AttackMaps) -> V::Score {
        let mut total_score = V::Score::default();

        for side in [White, Black] {
            let mut side_score = V::Score::default();

            if (TERMS & TERM_MATERIAL) != 0 {
                side_score += eval_material(pos, &self.values, side);
            }
            if (TERMS & TERM_ONE_WAY) != 0 {
                side_score += eval_one_way::<V, false>(pos, maps, &self.values, side);
            } else if (TERMS & TERM_ONE_WAY_DEFENDED) != 0 {
                side_score += eval_one_way::<V, true>(pos, maps, &self.values, side);
            }
            if (TERMS & TERM_FORKS) != 0 {
                side_score += eval_forks(pos, maps, &self.values, side);
            }
            if (TERMS & TERM_ROOK) != 0 {
                side_score += eval_rook_activity(pos, maps, &self.values, side);
            }
            if (TERMS & TERM_MOBILITY) != 0 {
                side_score += eval_mobility(pos, maps, &self.values, side);
            }
            if (TERMS & TERM_KING_SAFETY) != 0 {
                side_score += eval_king_safety(pos, maps, &self.values, side);
            }
            if (TERMS & TERM_PAWN_STRUCTURE) != 0 {
                side_score += eval_pawn_structure(pos, &self.values, side);
            }

            total_score += if side == White { side_score } else { -side_score };
        }

        total_score
    }

    /// Side-to-move relative score, the convention the engine's `Eval::eval` uses.
    pub fn do_eval(&self, pos: &Board) -> <V::Score as ScoreType>::Finalized {
        self.do_eval_for(pos, pos.active_player())
    }

    /// White-relative score. The tuner needs this: EPD labels are white-relative (`1-0`).
    pub fn do_eval_white(&self, pos: &Board) -> <V::Score as ScoreType>::Finalized {
        self.do_eval_for(pos, White)
    }

    /// Computes attack maps only when a map-dependent term is enabled; material and
    /// pawn-structure-only configs skip the per-node slider lookups entirely.
    fn do_eval_for(&self, pos: &Board, color: Color) -> <V::Score as ScoreType>::Finalized {
        const MAP_TERMS: u32 = TERM_ONE_WAY | TERM_ONE_WAY_DEFENDED | TERM_FORKS | TERM_ROOK | TERM_MOBILITY | TERM_KING_SAFETY;
        let maps = if (TERMS & MAP_TERMS) != 0 { AttackMaps::compute(pos) } else { AttackMaps::empty() };
        let s = self.eval_white_relative(pos, &maps);
        s.finalize(pos.phase(), 24, color, self.values.tempo(), &[V::Score::default(), V::Score::default()])
    }
}

impl<const TERMS: u32> Eval<Board> for CdceEval<TERMS, CdceParams> {
    fn eval(&mut self, pos: &Board, _ply: usize, _engine: Color) -> Score {
        self.do_eval(pos)
    }

    fn piece_scale(&self) -> ScoreT {
        4
    }

    fn ugi_options(&self) -> Vec<EngineOption> {
        self.values.ugi_options()
    }

    fn set_option(&mut self, name: &str, value: &str) -> Res<()> {
        self.values.set_option(name, value)
    }
}

impl<const TERMS: u32> Eval<Board> for CdceEval<TERMS, crate::TunedValues> {
    fn eval(&mut self, pos: &Board, _ply: usize, _engine: Color) -> Score {
        self.do_eval(pos)
    }

    fn piece_scale(&self) -> ScoreT {
        4
    }
}

impl<const TERMS: u32, V: CdceValues> StaticallyNamedEntity for CdceEval<TERMS, V> {
    fn static_short_name() -> impl Display {
        let base = match TERMS {
            CDCE_FULL_MASK => "CDCE",
            CDCE_CORE_MASK => "CDCE-Core",
            _ if TERMS == (CDCE_FULL_MASK & !TERM_ONE_WAY) => "CDCE-NoOneWay",
            _ if TERMS == (CDCE_FULL_MASK & !TERM_FORKS) => "CDCE-NoForks",
            _ if TERMS == (CDCE_FULL_MASK & !TERM_ROOK) => "CDCE-NoRook",
            _ if TERMS == (CDCE_FULL_MASK & !TERM_MOBILITY) => "CDCE-NoMobility",
            _ if TERMS == ((CDCE_FULL_MASK & !TERM_ONE_WAY) | TERM_ONE_WAY_DEFENDED) => "CDCE-OneWayDefended",
            _ if TERMS == TERM_MATERIAL => "CDCE-Material",
            _ => "CDCE-Custom",
        };
        format!("{base}{}", V::NAME_SUFFIX)
    }

    fn static_long_name() -> String {
        format!("CDCE Chess Eval [{}]", Self::static_short_name())
    }

    fn static_description() -> String {
        format!("Cheap & Deep Chess Engine Eval variant: {}", Self::static_short_name())
    }
}
