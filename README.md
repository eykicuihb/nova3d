# nova3d

A CPU-first 3D engine in Rust, developed **entirely by an autonomous multi-agent
harness** ([autodev-temporal-symphony](https://github.com/eykicuihb/autodev-temporal-symphony-v23))
as a live drill: every feature wave is planned, implemented, tested, reviewed, and
merged by AI agents under human approval gates.

## Roadmap

| Wave | Module | Contents |
|------|--------|----------|
| 1 | `math` | `Vec3`, `Mat4`, `Quat` — pure functions, unit tested |
| 2 | `scene` | Transform hierarchy, scene graph |
| 3 | `raster` | Software rasterizer with deterministic framebuffers |
| 4 | `io` | PPM export, golden-image tests |

## Constraints

- Headless: no GPU, no window, tests run on bare CI runners
- Deterministic: identical inputs → byte-identical outputs
- Gate: `cargo fmt` + `cargo clippy -D warnings` + `cargo test` (enforced by CI)

## License

MIT
