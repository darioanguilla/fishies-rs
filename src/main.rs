//! fishies - an aquarium/sea animation in ASCII art.
//!
//! Rust port of the Perl `fishies` script, itself a variant of Asciiquarium
//! by Kirk Baucom (most ASCII art by Joan Stark), with a transparent
//! background. Licensed under the GNU GPL, version 2 or later.
//!
//! Keys: `q` quit, `p` pause, `r` redraw (recreates every object),
//! `a` switch between ASCII and ANSI mode (truecolor and pixel art).

mod art;
mod color;
mod engine;
mod objects;
mod pixel_art;

use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Attribute, SetAttribute},
    terminal,
};

use engine::{Animation, Buffer, Mode, render::Screen};

/// Same pace as curses `halfdelay(1)` in the original.
const TICK: Duration = Duration::from_millis(100);

/// Puts the terminal back the way it was, even on panic.
struct TermGuard;

impl TermGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(TermGuard)
    }
}

fn restore_terminal() {
    let _ = execute!(
        io::stdout(),
        SetAttribute(Attribute::Reset),
        cursor::Show,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

enum Next {
    Quit,
    Rebuild,
}

fn main() -> io::Result<()> {
    let usage = || {
        format!(
            "fishies {}: an aquarium animation in ASCII art\n\n\
             Usage: fishies [-h | --help] [-v | --version]\n\n\
             Keys: q quit, p pause, r redraw, a ASCII/ANSI mode\n\
             Set FISHIES_OBJECT to one of {} to always spawn that object.",
            env!("CARGO_PKG_VERSION"),
            objects::OBJECT_NAMES.join(", ")
        )
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => {}
        Some("-v" | "--version") if args.len() == 1 => {
            println!("fishies {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("-h" | "--help") if args.len() == 1 => {
            println!("{}", usage());
            return Ok(());
        }
        Some(_) => {
            eprintln!(
                "fishies: unexpected argument(s): {}\n\n{}",
                args.join(" "),
                usage()
            );
            std::process::exit(2);
        }
    }
    objects::force_object(std::env::var("FISHIES_OBJECT").ok().as_deref());

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let _guard = TermGuard::new()?;
    let mut out = BufWriter::with_capacity(1 << 16, io::stdout());
    // the mode survives redraws and resizes
    let mut mode = Mode::default();
    loop {
        match run_scene(&mut out, &mut mode)? {
            Next::Quit => return Ok(()),
            Next::Rebuild => continue,
        }
    }
}

/// Build the scene for the current terminal size and animate it until the
/// user quits or asks for a redraw / resizes the terminal.
fn run_scene(out: &mut impl Write, mode: &mut Mode) -> io::Result<Next> {
    let (w, h) = terminal::size()?;
    let mut anim = Animation::new(w as i32, h as i32);
    objects::populate(&mut anim);
    let mut buf = Buffer::new(w as i32, h as i32);
    let mut screen = Screen::new();
    anim.render(&mut buf, *mode);
    screen.draw(out, &buf)?;

    let mut paused = false;
    let mut next_tick = Instant::now() + TICK;
    loop {
        let timeout = next_tick.saturating_duration_since(Instant::now());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(k) if k.kind != KeyEventKind::Release => match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(Next::Quit),
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Next::Quit);
                    }
                    KeyCode::Char('r') => return Ok(Next::Rebuild),
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('a') => {
                        // redraw at once, even while paused
                        *mode = mode.toggled();
                        anim.render(&mut buf, *mode);
                        screen.draw(out, &buf)?;
                    }
                    _ => {}
                },
                Event::Resize(..) => return Ok(Next::Rebuild),
                _ => {}
            }
            continue;
        }
        if !paused {
            objects::tick(&mut anim);
            anim.render(&mut buf, *mode);
            screen.draw(out, &buf)?;
        }
        next_tick += TICK;
        let now = Instant::now();
        if next_tick < now {
            next_tick = now + TICK;
        }
    }
}
