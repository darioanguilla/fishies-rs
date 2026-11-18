//! fishies - an aquarium/sea animation in ASCII art.
//!
//! Rust port of the Perl `fishies` script, itself a variant of Asciiquarium
//! by Kirk Baucom (most ASCII art by Joan Stark), with a transparent
//! background. Licensed under the GNU GPL, version 2 or later.
//!
//! Keys: `q` quit, `p` pause, `r` redraw (recreates every object).

mod art;
mod color;
mod engine;
mod objects;

use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Attribute, SetAttribute},
    terminal,
};

use engine::{Animation, Buffer, render::Screen};

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
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "-v" | "--version" => println!("fishies {}", env!("CARGO_PKG_VERSION")),
            _ => println!(
                "fishies {}: an aquarium animation in ASCII art\n\n\
                 Keys: q quit, p pause, r redraw\n\
                 Set FISHIES_OBJECT to one of {} to always spawn that object.",
                env!("CARGO_PKG_VERSION"),
                objects::OBJECT_NAMES.join(", ")
            ),
        }
        return Ok(());
    }
    objects::force_object(std::env::var("FISHIES_OBJECT").ok().as_deref());

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let _guard = TermGuard::new()?;
    let mut out = BufWriter::with_capacity(1 << 16, io::stdout());
    loop {
        match run_scene(&mut out)? {
            Next::Quit => return Ok(()),
            Next::Rebuild => continue,
        }
    }
}

/// Build the scene for the current terminal size and animate it until the
/// user quits or asks for a redraw / resizes the terminal.
fn run_scene(out: &mut impl Write) -> io::Result<Next> {
    let (w, h) = terminal::size()?;
    let mut anim = Animation::new(w as i32, h as i32);
    objects::populate(&mut anim);
    let mut buf = Buffer::new(w as i32, h as i32);
    let mut screen = Screen::new();
    anim.render(&mut buf);
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
                    _ => {}
                },
                Event::Resize(..) => return Ok(Next::Rebuild),
                _ => {}
            }
            continue;
        }
        if !paused {
            objects::tick(&mut anim);
            anim.render(&mut buf);
            screen.draw(out, &buf)?;
        }
        next_tick += TICK;
        let now = Instant::now();
        if next_tick < now {
            next_tick = now + TICK;
        }
    }
}
