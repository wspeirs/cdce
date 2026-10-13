# CDCE — Plan (trimmed)

## 1. Goal

A UCI chess engine that plays well, runs fast, and has an evaluation that is easy to read and change.

**First experiment (run later, not part of this plan's deliverables):** self-play with different piece-value sets, for example `Val_Knight=320` against `Val_Knight=420`.

**Non-goals** (cut because they took days): Texel tuning, SPRT matrices, CCRL/Stockfish anchors, search-toggle studies, per-term ablation variants, dyn-vs-generic gating, multi-thread/ponder audits, the 1000-game acceptance run. Code that already exists for these stays in the repo, is unreferenced by this plan, and is not maintained.

## 2. Decisions

- [x] **Base the engine on CAPS** (toanth/motors, GPL-3.0). It already has movegen, a strong search, UCI and an `Eval` trait, so we write only the eval.
- [x] **Toolchain** is pinned to Rust 1.92.0 in `rust-toolchain.toml`. `.cargo/config.toml` sets `target-cpu=native`.
- [x] **GPL-3.0 `LICENSE`.**
- [x] **Motors fork** is a submodule at `vendor/motors`, used through path dependencies.
- [x] **Eval-owned UCI options** (fork patch 2). This is how `Val_*` piece values are set. It is already built, so it is kept; it is the cheapest way to run different values in self-play.
- [x] Commit the work: `.gitmodules`, `vendor/motors` (the pinned fork commit), `crates/`, `bench/`, `Cargo.*`. Currently untracked or staged only.

Out of scope, not needed: search toggles (fork patch 1, already built, left on by default), the generic `Caps<E>` patch (the engine keeps `dyn`).

## 3. Layout

```
cdce/
├── crates/cdce-eval/   eval: AttackMaps, terms, params
├── crates/cdce/        UCI binary
├── vendor/motors/      CAPS fork
└── bench/              fastchess scripts (used later)
```

`crates/cdce/src/main.rs` registers the upstream evals plus the CDCE evals, and `CDCE` is the default eval. `SetEval` must be sent before any `Val_*` or other eval option, because it rebuilds the eval from defaults.

## 4. Evaluation (keep it readable)

Rule: one small function per term, summed in one place, so reading `eval()` shows everything the engine thinks. No term exists without a test.

- [x] `AttackMaps`: per-square attack bitboards, computed once per eval and shared by all terms.
- [x] **Material**, with UCI options `Val_Pawn`, `Val_Knight`, `Val_Bishop`, `Val_Rook`, `Val_Queen`.
- [x] **One-way attacks**: an attack on a piece that cannot attack back scores `weight[attacker] * value[target]`.
- [x] **Knight forks**: a bonus when a knight attacks 2 or more of N/R/Q/K.
- [x] **Rook activity**: off the home rank, on relative rank 3, open and semi-open file, connected rooks, and the A/H-pawn term.
- [x] **Mobility**, linear weight per piece type.
- [x] Eval variants registered for `SetEval`: `CDCE`, `CDCE-Core`, `CDCE-Material`, and the upstream `MateOnCE`, `PiSTOn`, `LiTE`.
- [x] **Make the default piece values sane.** The defaults now are R=330, N=420, which is the weak pseudocode set. Change them to P100 N320 B330 R500 Q900, so the baseline for self-play is a normal set.
- [x] **Simplify.** King safety and pawn structure are no longer part of the plan. Remove them from `CDCE` and `CDCE-Full`, or leave them off by default. The full variant is then material + one-way + forks + rook + mobility. Also remove the `CDCE-Tuned` registration and the `tuned*` modules from the default path. Keep the `cdce-tune` crate untouched.
- [x] Add a short comment block at the top of `eval.rs` that lists every term, its default weight and its UCI option name.

## 5. Tests (permanent, fast)

- [x] Color-flip symmetry: `eval(pos) == eval(flip(pos))` over a fixed position set.
- [x] Hand-built positions with exact term values: one-way, mutual, pawn-attacks-rook, royal fork, two-bishop fork (no bonus), open vs semi-open file, rook lift by phase.
- [x] Option round trip: `setoption Val_Rook 500` changes the material term.
- [x] Update the tests after the simplification in §4. `cargo test --release -p cdce-eval` must pass.

## 6. Verify (short only)

- [x] `cargo build --release` succeeds.
- [x] `cdce` answers `uci`, lists `SetEval` and `Val_*`, and `go movetime` returns a legal move (checked: `bestmove a2a4`).
- [x] After §4: 20-game self-play smoke run, about a minute at a tiny time control, with no crashes, illegal moves or time losses.

## 7. Self-play harness (built now, run later)

- [x] `bench/selfplay.sh <name-A> <opt-A…> -- <name-B> <opt-B…> [games]`. It runs fastchess with the same binary on both sides and passes each side its own `setoption` lines, such as `Val_Knight=320`. Defaults: `Threads=1`, the existing opening book, color-reversed pairs, a small game count and a short time control. It prints the score and Elo ± error.
- [x] Document the one-line usage in `README.md`.

The script is checked only with a tiny dry run. The actual piece-value experiment is a separate session.
