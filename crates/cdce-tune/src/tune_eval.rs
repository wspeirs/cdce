//! pliers `Eval` for the CDCE eval: the feature trace is the full CDCE eval
//! computed with `TuneValues`, so pliers fits exactly the terms the engine uses.

use std::fmt::Formatter;

use cdce_eval::{CdceEval, CDCE_FULL_MASK};
use gears::games::chess::Board;
use pliers::eval::{Eval, EvalScale, WeightsInterpretation};
use pliers::gd::{Weight, Weights};
use pliers::load_data::NoFilter;
use pliers::trace::{FeatureSubSet, SparseTrace};

use crate::features::{
    TuneValues, ALL_SUBSETS, NUM_FEATURES,
};

#[derive(Debug, Default)]
pub struct CdceTuneEval {
    /// Start the fit from `CdceParams::standard()` instead of zero.
    pub seed_standard: bool,
}

impl WeightsInterpretation for CdceTuneEval {
    /// Fixed scale, identical to the zero-start fit. The trait default would search a scale from `initial_weights`, which fails to converge.
    fn eval_scale(&self) -> EvalScale {
        EvalScale::Scale(110.0)
    }

    fn retune_from_zero(&self) -> bool {
        !self.seed_standard
    }

    fn initial_weights(&self) -> Option<Weights> {
        self.seed_standard.then(crate::standard_weights)
    }

    fn display(&self) -> fn(&mut Formatter, &Weights, &[Weight]) -> std::fmt::Result {
        |f: &mut Formatter<'_>, weights: &Weights, _old_weights: &[Weight]| {
            for subset in ALL_SUBSETS {
                subset.write(f, weights, &[])?;
            }
            Ok(())
        }
    }
}

impl Eval<Board> for CdceTuneEval {
    fn num_features() -> usize {
        NUM_FEATURES
    }

    type Filter = NoFilter;

    fn feature_trace(pos: &Board) -> impl pliers::trace::TraceTrait {
        let eval = CdceEval::<CDCE_FULL_MASK, TuneValues>::new(TuneValues);
        let trace: SparseTrace = eval.do_eval_white(pos);
        trace
    }
}
