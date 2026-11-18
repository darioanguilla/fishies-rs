# fishies-rs

[![CI](https://github.com/darioanguilla/fishies-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/darioanguilla/fishies-rs/actions/workflows/ci.yml)
[![Release](https://github.com/darioanguilla/fishies-rs/actions/workflows/release.yml/badge.svg)](https://github.com/darioanguilla/fishies-rs/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/darioanguilla/fishies-rs)](https://github.com/darioanguilla/fishies-rs/releases/latest)
[![License: GPL v2+](https://img.shields.io/badge/license-GPL--2.0--or--later-blue)](LICENSE)

An aquarium/sea animation in ASCII art for the terminal, written in Rust.

This is a port of `fishies`, a Perl variant of [Asciiquarium](http://robobunny.com/projects/asciiquarium)
that draws on the terminal's own background instead of a black one. The Perl version
needs `Curses` and `Term::Animation`, so it doesn't run on Windows. This port has no curses
dependency (it uses [crossterm](https://crates.io/crates/crossterm)) and runs on Windows, Linux and macOS.

You get the water surface, a castle, swaying seaweed and colorful fish blowing bubbles. One
"random object" is always passing by: a ship, a whale with its water spout, a sea monster,
a big fish, a shark that eats fish, a fishhook that catches one, a swan, ducks, or dolphins.

## Download

Prebuilt binaries are attached to every [release](https://github.com/darioanguilla/fishies-rs/releases/latest):

| OS      | x86_64                         | arm64                           | RISC-V                          |
|---------|--------------------------------|---------------------------------|---------------------------------|
| Linux   | `x86_64-unknown-linux-gnu`     | `aarch64-unknown-linux-gnu`     | `riscv64gc-unknown-linux-gnu`   |
| Windows | `x86_64-pc-windows-msvc`       | `aarch64-pc-windows-msvc`       | n/a                             |
| macOS   | `x86_64-apple-darwin`          | `aarch64-apple-darwin`          | n/a                             |

Windows archives are `.zip`, all others are `.tar.gz`. Check them against `SHA256SUMS.txt`.
Rust has no Windows or macOS target for RISC-V, so RISC-V builds are Linux only.

On macOS, binaries downloaded with a browser are not notarized. Clear the quarantine flag once:

```sh
xattr -d com.apple.quarantine ./fishies
```

## Install with Cargo

```sh
cargo install --git https://github.com/darioanguilla/fishies-rs
```

Or build from source:

```sh
cargo build --release
./target/release/fishies
```

## Keys

| Key            | Action                                       |
|----------------|----------------------------------------------|
| `q`, `Esc`, `Ctrl+C` | quit                                   |
| `p`            | pause / resume                               |
| `r`            | redraw (recreates every object)              |

Resizing the terminal also rebuilds the scene.

To always spawn the same random object (handy for debugging), set `FISHIES_OBJECT` to one of
`ship`, `whale`, `monster`, `big_fish`, `shark`, `fishhook`, `swan`, `ducks`, `dolphins`.

## Development and releases

- **CI** (`.github/workflows/ci.yml`) runs `cargo fmt`, `clippy` and the tests on Linux (x86_64,
  arm64), Windows and macOS for every push and pull request. It also cross-compiles the other
  release targets. Each run's summary lists the test results for each platform, and the CI badge
  above shows the state of `master`.
- **Release** (`.github/workflows/release.yml`) runs after CI succeeds on `master`. If the version
  in `Cargo.toml` has no release yet, it creates the `v<version>` tag, builds all the targets and
  publishes the GitHub release.

To publish a new version, bump `version` in `Cargo.toml`, run `cargo build` to update
`Cargo.lock`, then commit and push to `master`.

## Differences from the Perl original

- The engine is a small reimplementation of the parts of `Term::Animation` the script uses
  (`src/engine`). Entity callbacks are enums instead of closures.
- When the fishhook leaves the screen, its hook point is removed too. The original left it behind.
- `rand_color` can now pick every color in its list. The original never picked bright magenta.

## Credits and license

- Asciiquarium by Kirk Baucom, most of the ASCII art by Joan Stark.
- `fishies` transparency by UndeadLeech, the rwx fishy by qmacro.

Licensed under the GNU General Public License, version 2 or (at your option) any later version,
like the original. See [LICENSE](LICENSE).
