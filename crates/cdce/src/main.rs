use cdce_eval::{CdceCore, CdceFull, CdceMaterial};
use gears::cli::ArgIter;
use gears::games::chess::Board;
use gears::general::common::anyhow::anyhow;
use gears::general::common::Res;
use gears::AbstractRun;
use motors::io::cli::parse_cli;
use motors::search::EvalBuilder;
use motors::{create_match_for_game, list_chess_evals, list_chess_outputs, list_chess_searchers};

fn version_string() -> String {
    let mut res = "CDCE-".to_string();
    res += std::env::consts::ARCH;
    if cfg!(target_feature = "avx512f") {
        res += "-avx512";
    } else if cfg!(target_feature = "avx2") {
        res += "-avx2";
    } else if cfg!(target_feature = "avx") {
        res += "-avx";
    } else if cfg!(target_feature = "ssse3") {
        res += "-ssse3";
    } else if cfg!(target_feature = "sse2") {
        res += "-sse2";
    } else {
        res += "-compat";
    }
    if cfg!(debug_assertions) {
        res += " (debug version)";
    }
    if !cfg!(feature = "unsafe") {
        res += " [unsafe features disabled]";
    }
    res
}

pub fn run_cdce_with_args(args: ArgIter) -> Res<()> {
    println!("{}", version_string());
    let args = parse_cli(args).map_err(|err| anyhow!("Failed to parse command line arguments: {err}"))?;
    let searchers = list_chess_searchers();
    let mut evals = list_chess_evals();
    evals.push(Box::new(EvalBuilder::<Board, CdceCore>::default()));
    evals.push(Box::new(EvalBuilder::<Board, CdceMaterial>::default()));
    // The last registered eval is the startup default. CdceFull (CDCE, which has the Val_* options)
    // must stay last so it is the default engine eval and its UCI options seed the option list.
    evals.push(Box::new(EvalBuilder::<Board, CdceFull>::default()));
    let outputs = list_chess_outputs();
    let mut engine = create_match_for_game(args, searchers, evals, outputs)?;
    let _ = engine.run();
    Ok(())
}

fn main() {
    let mut args = std::env::args().peekable();
    _ = args.next(); // remove the program name
    if let Err(err) = run_cdce_with_args(args) {
        eprintln!("{err}");
    }
}
