use std::fmt::Debug;
use gears::games::chess::pieces::PieceType;
use gears::score::{p, PhasedScore, Score, ScoreT};
use motors::eval::{ScoreType, SingleFeatureScore};
use crate::params::CdceParams;

/// Trait abstracting over eval score values.
/// In search, this evaluates to `PhasedScore`. In tuning (`pliers`), this evaluates to feature traces.
pub trait CdceValues: Debug + Clone + Send + 'static {
    type Score: ScoreType;
    /// Appended to the engine name, e.g. `CDCE-Tuned`.
    const NAME_SUFFIX: &'static str = "";

    fn material(&self, piece: PieceType) -> SingleFeatureScore<Self::Score>;
    fn one_way(&self, attacker: PieceType, target: PieceType) -> SingleFeatureScore<Self::Score>;
    fn knight_fork(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_off_home_rank(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_relative_rank3(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_file_open(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_file_semi_open(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_connected(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_pawn_gate(&self) -> SingleFeatureScore<Self::Score>;
    fn rook_pawn_phase_gate(&self) -> isize;
    fn mobility(&self, piece: PieceType) -> SingleFeatureScore<Self::Score>;
    fn king_attacker(&self, attacker: PieceType) -> SingleFeatureScore<Self::Score>;
    fn king_shield(&self) -> SingleFeatureScore<Self::Score>;
    fn pawn_doubled(&self) -> SingleFeatureScore<Self::Score>;
    fn pawn_isolated(&self) -> SingleFeatureScore<Self::Score>;
    fn pawn_passed(&self, relative_rank: usize) -> SingleFeatureScore<Self::Score>;
    fn tempo(&self) -> <Self::Score as ScoreType>::Finalized;
}

impl CdceValues for CdceParams {
    type Score = PhasedScore;

    #[inline]
    fn material(&self, piece: PieceType) -> PhasedScore {
        let v = self.piece_value(piece) as i16;
        p(v, v)
    }

    #[inline]
    fn one_way(&self, attacker: PieceType, target: PieceType) -> PhasedScore {
        let v = self.ow_table[attacker as usize][target as usize] as i16;
        p(v, v)
    }

    #[inline]
    fn knight_fork(&self) -> PhasedScore {
        let v = self.fork_bonus as i16;
        p(v, v)
    }

    #[inline]
    fn rook_off_home_rank(&self) -> PhasedScore {
        p(self.rook_off_home_rank as i16, 0)
    }

    #[inline]
    fn rook_relative_rank3(&self) -> PhasedScore {
        p(self.rook_rank3 as i16, 0)
    }

    #[inline]
    fn rook_file_open(&self) -> PhasedScore {
        let v = self.rook_file_open as i16;
        p(v, v)
    }

    #[inline]
    fn rook_file_semi_open(&self) -> PhasedScore {
        let v = self.rook_file_semi_open as i16;
        p(v, v)
    }

    #[inline]
    fn rook_connected(&self) -> PhasedScore {
        let v = self.rook_connected as i16;
        p(v, v)
    }

    #[inline]
    fn rook_pawn_gate(&self) -> PhasedScore {
        p(self.rook_pawn_gate as i16, 0)
    }

    #[inline]
    fn rook_pawn_phase_gate(&self) -> isize {
        self.rook_pawn_phase_gate as isize
    }

    #[inline]
    fn mobility(&self, piece: PieceType) -> PhasedScore {
        let v = match piece {
            PieceType::Knight => self.mobility_knight,
            PieceType::Bishop => self.mobility_bishop,
            PieceType::Rook => self.mobility_rook,
            PieceType::Queen => self.mobility_queen,
            _ => 0,
        } as i16;
        p(v, v)
    }

    #[inline]
    fn king_attacker(&self, attacker: PieceType) -> PhasedScore {
        let v = match attacker {
            PieceType::Knight => self.king_attacker_knight,
            PieceType::Bishop => self.king_attacker_bishop,
            PieceType::Rook => self.king_attacker_rook,
            PieceType::Queen => self.king_attacker_queen,
            _ => 0,
        } as i16;
        p(v, 0)
    }

    #[inline]
    fn king_shield(&self) -> PhasedScore {
        p(self.king_shield as i16, 0)
    }

    #[inline]
    fn pawn_doubled(&self) -> PhasedScore {
        let v = self.pawn_doubled as i16;
        p(v, v)
    }

    #[inline]
    fn pawn_isolated(&self) -> PhasedScore {
        let v = self.pawn_isolated as i16;
        p(v, v)
    }

    #[inline]
    fn pawn_passed(&self, relative_rank: usize) -> PhasedScore {
        let v = self.pawn_passed_rank.get(relative_rank).copied().unwrap_or(0) as i16;
        p(v, v)
    }

    #[inline]
    fn tempo(&self) -> Score {
        Score(self.tempo as ScoreT)
    }
}
