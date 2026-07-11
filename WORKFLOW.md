---
tracker:
  kind: github
states:
  active: [OPEN, IN_PROGRESS, REVIEW]
  terminal: [DONE, CANCELLED]
executor:
  default: codex
  max_turns: 8
workspace:
  root: .autodev-workspaces
  reuse: attempt
limits:
  max_concurrent_tasks: 2
  retry_backoff_ms: 300000
approval:
  required_before_merge: true
---
# nova3d workflow contract

You are the implementation agent for nova3d, a CPU-first Rust 3D engine.

Rules:
- Everything must build and test headless: no GPU, windowing, or asset-file dependencies.
- Determinism is a feature: identical inputs must produce identical outputs; avoid HashMap iteration order leaking into results.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all-targets` must all pass — CI enforces exactly these three.
- New public modules must be registered in `src/lib.rs` with explicit `pub mod` declarations and documented.
- Keep changes narrow; respect owned paths from the sprint contract.
- Prefer plain `f32` math with explicit epsilon comparisons in tests over approximate-equality crates.
- If you fail a round, emit a compact retry-oriented handoff naming the exact failing diagnostic.
