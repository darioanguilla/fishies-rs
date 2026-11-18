# fishies-rs

An aquarium/sea animation in ASCII art for the terminal, written in Rust.

This is a port of `fishies`, a Perl variant of [Asciiquarium](http://robobunny.com/projects/asciiquarium)
that draws on the terminal's own background instead of a black one. The Perl version
needs `Curses` and `Term::Animation`, so it doesn't run on Windows. This port has no curses
dependency (it uses [crossterm](https://crates.io/crates/crossterm)) and runs on Windows, Linux and macOS.

You get the water surface, a castle, swaying seaweed and colorful fish blowing bubbles. One
"random object" is always passing by: a ship, a whale with its water spout, a sea monster,
a big fish, a shark that eats fish, a fishhook that catches one, a swan, ducks, or dolphins.

## Install

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
