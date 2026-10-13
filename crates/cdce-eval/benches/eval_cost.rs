//! Per-call eval cost for every CDCE variant over the UHO positions (plan.md §5 gate input).
//! Run: `cargo bench -p cdce-eval --bench eval_cost`.

use std::hint::black_box;
use std::fs;

use cdce_eval::{
    CdceCore, CdceEval, CdceFull, CdceMaterial, CdceNoForks, CdceNoMobility, CdceNoOneWay,
    CdceNoRook, CdceOneWayDefended, TunedValues, CDCE_FULL_MASK,
};
use criterion::{criterion_group, criterion_main, Criterion};
use gears::games::chess::Board;
use gears::general::board::BoardHelpers;
use gears::general::board::BoardTrait;
use gears::general::board::Strictness::Relaxed;
use motors::eval::Eval;

fn load_positions() -> Vec<Board> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../bench/UHO_Lichess_4852_v1.epd");
    let text = fs::read_to_string(path).expect("UHO book");
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().take(4).collect();
            if fields.len() < 4 {
                return None;
            }
            let fen = format!("{} 0 1", fields.join(" "));
            Board::from_fen(&fen, Relaxed).ok()
        })
        .collect()
}

fn bench_eval<E: Eval<Board>>(c: &mut Criterion, name: &str, mut eval: E, positions: &[Board]) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for pos in positions {
                let score = eval.eval(black_box(pos), 0, pos.active_player());
                sum += score.0 as i64;
            }
            black_box(sum)
        })
    });
}

fn eval_cost(c: &mut Criterion) {
    let positions = load_positions();
    assert!(!positions.is_empty(), "no positions loaded");
    bench_eval(c, "MATERIAL", CdceMaterial::default(), &positions);
    bench_eval(c, "CORE", CdceCore::default(), &positions);
    bench_eval(c, "NO_ONE_WAY", CdceNoOneWay::default(), &positions);
    bench_eval(c, "NO_FORKS", CdceNoForks::default(), &positions);
    bench_eval(c, "NO_ROOK", CdceNoRook::default(), &positions);
    bench_eval(c, "NO_MOBILITY", CdceNoMobility::default(), &positions);
    bench_eval(c, "ONE_WAY_DEFENDED", CdceOneWayDefended::default(), &positions);
    bench_eval(c, "FULL", CdceFull::default(), &positions);
    bench_eval(c, "TUNED", CdceEval::<CDCE_FULL_MASK, TunedValues>::default(), &positions);
}

criterion_group!(benches, eval_cost);
criterion_main!(benches);
