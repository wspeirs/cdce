//! Maps CDCE eval terms onto pliers feature indices.
//!
//! Every tunable CDCE parameter is one feature. The one-way table is expanded
//! to its full 6x6 grid, so each `(attacker, target)` pair has its own weight
//! (plan.md §7: the eval must stay linear in the weights).
//! `tempo` and `rook_pawn_phase_gate` are not features and stay at their defaults.

use std::fmt::{self, Formatter};

use cdce_eval::CdceValues;
use gears::games::chess::pieces::PieceType;
use motors::eval::{ScoreType, SingleFeatureScore};
use pliers::gd::Weights;
use pliers::trace::{FeatureSubSet, SingleFeature, SparseTrace};

use cdce_eval::layout;
use cdce_eval::CdceParams;

/// A contiguous block of feature indices sharing one name.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Subset {
    pub name: &'static str,
    pub start: usize,
    pub len: usize,
}

impl FeatureSubSet for Subset {
    fn num_features(self) -> usize {
        self.len
    }

    fn start_idx(self) -> usize {
        self.start
    }

    fn write(self, f: &mut Formatter, weights: &Weights, _special: &[bool]) -> fmt::Result {
        for i in 0..self.len {
            // pliers stores the phased pair at (2*feature, 2*feature+1) = (mg, eg).
            let feature = self.start + i;
            writeln!(f, "{}[{i}]: mg {} eg {}", self.name, weights[2 * feature], weights[2 * feature + 1])?;
        }
        Ok(())
    }
}

pub const MATERIAL: Subset = Subset { name: "material", start: layout::MATERIAL, len: 5 };
pub const ONE_WAY: Subset = Subset { name: "one_way", start: layout::ONE_WAY, len: 36 };
pub const KNIGHT_FORK: Subset = Subset { name: "knight_fork", start: layout::KNIGHT_FORK, len: 1 };
pub const ROOK: Subset = Subset { name: "rook", start: layout::ROOK, len: 6 };
pub const MOBILITY: Subset = Subset { name: "mobility", start: layout::MOBILITY, len: 4 };
pub const KING_ATTACKER: Subset = Subset { name: "king_attacker", start: layout::KING_ATTACKER, len: 4 };
pub const KING_SHIELD: Subset = Subset { name: "king_shield", start: layout::KING_SHIELD, len: 1 };
pub const PAWN_DOUBLED: Subset = Subset { name: "pawn_doubled", start: layout::PAWN_DOUBLED, len: 1 };
pub const PAWN_ISOLATED: Subset = Subset { name: "pawn_isolated", start: layout::PAWN_ISOLATED, len: 1 };
pub const PAWN_PASSED: Subset = Subset { name: "pawn_passed", start: layout::PAWN_PASSED, len: 8 };

/// Every subset in index order; used for weight display.
pub const ALL_SUBSETS: [Subset; 10] =
    [MATERIAL, ONE_WAY, KNIGHT_FORK, ROOK, MOBILITY, KING_ATTACKER, KING_SHIELD, PAWN_DOUBLED, PAWN_ISOLATED, PAWN_PASSED];

/// Total number of features (one weight per feature, phased pairs handled by pliers).
pub const NUM_FEATURES: usize = cdce_eval::layout::NUM_FEATURES;

/// Index into `MOBILITY` / `KING_ATTACKER` for a non-king piece (Knight..Queen).
fn minor_major_index(piece: PieceType) -> usize {
    match piece {
        PieceType::Knight => 0,
        PieceType::Bishop => 1,
        PieceType::Rook => 2,
        PieceType::Queen => 3,
        other => panic!("no mobility/king-attacker feature for {other:?}"),
    }
}

/// Feature-trace values for the CDCE eval. Each method emits one feature hit.
#[derive(Debug, Clone, Copy, Default)]
pub struct TuneValues;

impl CdceValues for TuneValues {
    type Score = SparseTrace;

    fn material(&self, piece: PieceType) -> SingleFeatureScore<SparseTrace> {
        match piece {
            PieceType::King | PieceType::Empty => panic!("no material feature for {piece:?}"),
            p => SingleFeature::new(MATERIAL, p as usize),
        }
    }

    fn one_way(&self, attacker: PieceType, target: PieceType) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ONE_WAY, attacker as usize * 6 + target as usize)
    }

    fn knight_fork(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(KNIGHT_FORK, 0)
    }

    fn rook_off_home_rank(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 0)
    }

    fn rook_relative_rank3(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 1)
    }

    fn rook_file_open(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 2)
    }

    fn rook_file_semi_open(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 3)
    }

    fn rook_connected(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 4)
    }

    fn rook_pawn_gate(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(ROOK, 5)
    }

    fn rook_pawn_phase_gate(&self) -> isize {
        CdceParams::default().rook_pawn_phase_gate as isize
    }

    fn mobility(&self, piece: PieceType) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(MOBILITY, minor_major_index(piece))
    }

    fn king_attacker(&self, attacker: PieceType) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(KING_ATTACKER, minor_major_index(attacker))
    }

    fn king_shield(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(KING_SHIELD, 0)
    }

    fn pawn_doubled(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(PAWN_DOUBLED, 0)
    }

    fn pawn_isolated(&self) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(PAWN_ISOLATED, 0)
    }

    fn pawn_passed(&self, relative_rank: usize) -> SingleFeatureScore<SparseTrace> {
        SingleFeature::new(PAWN_PASSED, relative_rank)
    }

    fn tempo(&self) -> <SparseTrace as ScoreType>::Finalized {
        SparseTrace::default()
    }
}
