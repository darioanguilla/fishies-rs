# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust port of `fishies`, a Perl variant of Asciiquarium that draws on the terminal's own (transparent) background. Single binary crate, edition 2024, using crossterm (no curses). It targets Windows, Linux and macOS, plus Linux RISC-V.

## Commands

CI runs these exact checks, so run them before pushing:

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

- Single test: `cargo test <name_substring>`, e.g. `cargo test hooked_fish_reels_everything_up`
- Run: `cargo run` (or `cargo run --release`). `fishies --detect` prints what graphics the terminal reported.
- Debug a specific visitor: `FISHIES_OBJECT=shark cargo run`. Valid names are in `objects::OBJECT_NAMES`. Other env vars: `FISHIES_GRAPHICS=auto|sixel|kitty|off`, `FISHIES_CELL=WxH`, `FISHIES_STATS=1`.
- Ignored tests that write files:
  - `scripts/screenshots.sh` regenerates `docs/screenshots/*.png`. It runs `cargo test screenshots -- --ignored` and then takes the screenshots with a headless Chromium/Edge (set `BROWSER` to pick one).
  - `cargo test --release graphics_frames -- --ignored --nocapture` writes the `graphics-*.png` files and prints Sixel/kitty frame size and encode-time stats.

## Commits and releases

Commit messages **must** follow Conventional Commits, because releases are automatic. After CI passes on `master`, `.github/workflows/release.yml` works out the next version from the commits since the last tag (`.github/scripts/next-version.sh`):

| Commit type | Version bump |
|---|---|
| `feat:` | minor |
| `fix:` or `perf:` | patch |
| `!` or `BREAKING CHANGE` | major (minor while the version is 0.x) |
| `docs:`, `ci:`, `test:`, `chore:`, `style:` | no release |

If `Cargo.toml` already has a version higher than the last tag, that version is released as is; after editing it, run `cargo build` so `Cargo.lock` is updated too. The workflow then commits `release: X.Y.Z` and rewrites the README section between `<!-- release:start -->` and `<!-- release:end -->`. Don't edit that section by hand.

## Architecture

There are three render modes, which the `a` key cycles through (`View` in `main.rs`):

- **ASCII**: the original art in 16 colors.
- **ANSI**: truecolor half-block pixel art, two pixels per cell.
- **Graphics**: real pixels sent over Sixel or the kitty protocol. Only available when the startup terminal query detects support.

The modes share one simulation. Only the drawing differs.

- **`engine/`**: a small reimplementation of the parts of Perl's `Term::Animation` the script uses. `Animation` holds the `Entity` list.
  - Entities added during a tick are queued in `pending` and only become active on `flush()`.
  - Perl closures became enums: `Behavior` (per-tick callback), `CollHandler` (collision handler), `DeathAction` (death callback).
  - Rendering sorts entities by z, back (high z) to front (low z). `render.rs` holds the cell `Buffer` and `Screen`, which writes to the terminal incrementally.
- **`objects/`**: the aquarium logic, i.e. what the Perl script does on top of the engine. `populate` builds the scene. `tick` runs four phases in order: movement/behaviors, collisions, automatic deaths (offscreen/time/frame), then death callbacks, repeated until nothing else dies. Some death actions spawn the next random object (`random.rs`).
  - The `depth` module defines the z layers.
  - `rand_int`/`rand_f` mimic Perl's `int(rand(n))`/`rand(n)`, including the rule that `rand(0)` behaves like `rand(1)`.
- **`art.rs`**: ASCII art copied verbatim from the Perl `q{...}` literals. The Perl quoting rules are applied at runtime by `engine::shape::perl_q`/`perl_lines`, so keep the literals byte-for-byte.
- **`color.rs`**: Term::Animation color-mask letters (`rgbcmywk`, uppercase = bright) plus RGB helpers.
- **`pixel_art/`**: original pixel art for ANSI and graphics mode. Each sprite is drawn once through a `pen::Pen` at a given `Res`:
  - `Res::Low` gives ANSI half blocks.
  - `Res::High` gives graphics mode, with `HIRES` = 4× the pixels each way.
  - `Res::upscale` (Scale2x) enlarges grid-based art.
  - Entities carry `pixels` (low) and `hires` (high) frames, set together by the `Art` trait in `objects/mod.rs`.
- **`graphics/`**:
  - `detect`/`query` ask the terminal for kitty, DA1 Sixel (attribute 4) and the cell size in pixels; kitty wins when both are available.
  - `canvas` composes the scene into an RGBA image at real pixel size.
  - `sixel`/`kitty` encode the frames, wrapped in synchronized-output escapes.
  - The image covers every row except the last. A Sixel image that touches the bottom row makes the terminal scroll, so the last row is drawn as ANSI and holds the status messages.

### Invariants to preserve

- **Pixel sprites match their ASCII footprint.** An ANSI frame is exactly as wide as the ASCII frame and twice as tall. Hires frames are `HIRES` times that. Collisions and gameplay use the ASCII frames, so behavior doesn't change between modes.
- **Empty cells stay transparent.** Empty cells and empty pixels (`None`, or `0` in the canvas) must never paint a background color. That is the point of this variant.
- **Pixel animation rate is decoupled.** `Entity::pixel_rate` lets pixel frames advance faster than the ASCII frames.
- **The terminal is restored on every exit.** `main.rs` restores it through `TermGuard` and a panic hook. Keep that working when touching startup and shutdown.
