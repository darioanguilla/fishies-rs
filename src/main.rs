//! fishies - an aquarium/sea animation in ASCII art.
//!
//! Rust port of the Perl `fishies` script, itself a variant of Asciiquarium
//! by Kirk Baucom (most ASCII art by Joan Stark), with a transparent
//! background. Licensed under the GNU GPL, version 2 or later.
//!
//! Keys: `q` quit, `p` pause, `r` redraw (recreates every object),
//! `a` cycle through ASCII, ANSI (truecolor half-block pixel art) and, when
//! the terminal supports Sixel or the kitty graphics protocol, graphics mode.

mod art;
mod color;
mod engine;
mod graphics;
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
use graphics::{Graphics, Support};

/// Same pace as curses `halfdelay(1)` in the original.
const TICK: Duration = Duration::from_millis(100);
/// How long a status message stays on the last row.
const MESSAGE_TIME: Duration = Duration::from_secs(4);

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

/// What the `a` key cycles through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum View {
    #[default]
    Ascii,
    Ansi,
    Graphics,
}

impl View {
    /// The next view; graphics only when the terminal supports it.
    fn next(self, graphics: bool) -> View {
        match self {
            View::Ascii => View::Ansi,
            View::Ansi if graphics => View::Graphics,
            View::Ansi | View::Graphics => View::Ascii,
        }
    }
}

/// State that survives redraws and resizes.
struct App {
    view: View,
    support: Support,
    message: Option<(String, Instant)>,
    show_stats: bool,
}

enum Next {
    Quit,
    Rebuild,
}

fn usage() -> String {
    format!(
        "fishies {}: an aquarium animation in ASCII art\n\n\
         Usage: fishies [-h | --help] [-v | --version] [--detect]\n\n\
         Keys: q quit, p pause, r redraw, a ASCII / ANSI / graphics mode\n\n\
         --detect                 show what graphics the terminal supports\n\
         FISHIES_OBJECT=<name>    always spawn that object: {}\n\
         FISHIES_GRAPHICS=<mode>  auto (default), sixel, kitty or off\n\
         FISHIES_CELL=<w>x<h>     cell size in pixels, if the terminal doesn't tell\n\
         FISHIES_STATS=1          show the size and encoding time of graphics frames",
        env!("CARGO_PKG_VERSION"),
        objects::OBJECT_NAMES.join(", ")
    )
}

/// Ask the terminal about graphics; it must be in raw mode.
fn detect() -> Support {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    graphics::detect::detect(cols, rows)
}

fn main() -> io::Result<()> {
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
        Some("--detect") if args.len() == 1 => {
            terminal::enable_raw_mode()?;
            let support = detect();
            terminal::disable_raw_mode()?;
            println!("{}", graphics::detect::report(&support));
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
    let mut app = App {
        view: View::default(),
        support: detect(),
        message: None,
        show_stats: std::env::var_os("FISHIES_STATS").is_some_and(|v| v != "0"),
    };
    let mut out = BufWriter::with_capacity(1 << 20, io::stdout());
    loop {
        match run_scene(&mut out, &mut app)? {
            Next::Quit => return Ok(()),
            Next::Rebuild => continue,
        }
    }
}

/// Build the scene for the current terminal size and animate it until the
/// user quits or asks for a redraw / resizes the terminal.
fn run_scene(out: &mut impl Write, app: &mut App) -> io::Result<Next> {
    let (w, h) = terminal::size()?;
    let mut anim = Animation::new(w as i32, h as i32);
    objects::populate(&mut anim);
    let mut buf = Buffer::new(w as i32, h as i32);
    let mut screen = Screen::new();
    let mut gfx = app
        .support
        .protocol
        .map(|p| Graphics::new(p, app.support.cell, w, h));
    draw(out, &anim, &mut buf, &mut screen, gfx.as_mut(), app)?;

    let mut paused = false;
    let mut next_tick = Instant::now() + TICK;
    let next = loop {
        let timeout = next_tick.saturating_duration_since(Instant::now());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(k) if k.kind != KeyEventKind::Release => match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Next::Quit,
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        break Next::Quit;
                    }
                    KeyCode::Char('r') => break Next::Rebuild,
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('a') => {
                        switch_view(out, app, gfx.as_ref())?;
                        // redraw at once, even while paused
                        draw(out, &anim, &mut buf, &mut screen, gfx.as_mut(), app)?;
                    }
                    _ => {}
                },
                Event::Resize(..) => break Next::Rebuild,
                _ => {}
            }
            continue;
        }
        if !paused {
            objects::tick(&mut anim);
        }
        if !paused || app.message.is_some() {
            draw(out, &anim, &mut buf, &mut screen, gfx.as_mut(), app)?;
        }
        next_tick += TICK;
        let now = Instant::now();
        if next_tick < now {
            next_tick = now + TICK;
        }
    };
    // the next scene, or the shell, starts without our image
    if app.view == View::Graphics
        && let Some(g) = &gfx
    {
        g.clear(out)?;
    }
    Ok(next)
}

/// Move to the next view. When graphics are not available the `a` key skips
/// them, and says why on the status row.
fn switch_view(out: &mut impl Write, app: &mut App, gfx: Option<&Graphics>) -> io::Result<()> {
    let available = app.support.protocol.is_some();
    let next = app.view.next(available);
    if app.view == View::Graphics
        && let Some(g) = gfx
    {
        g.clear(out)?;
    }
    if app.view == View::Ansi && next == View::Ascii && !available {
        app.message = Some((
            format!(
                "Graphics mode not available: {}. Run `fishies --detect` for details.",
                app.support.reason
            ),
            Instant::now(),
        ));
    } else if next == View::Graphics {
        let how = app.support.protocol.map_or("", graphics::Protocol::name);
        app.message = Some((format!("Graphics mode ({how})"), Instant::now()));
    } else {
        app.message = None;
    }
    app.view = next;
    Ok(())
}

fn draw(
    out: &mut impl Write,
    anim: &Animation,
    buf: &mut Buffer,
    screen: &mut Screen,
    gfx: Option<&mut Graphics>,
    app: &mut App,
) -> io::Result<()> {
    let graphics = app.view == View::Graphics;
    anim.render(
        buf,
        if app.view == View::Ascii {
            Mode::Ascii
        } else {
            Mode::Ansi
        },
    );
    if graphics {
        // the image covers everything but the last row
        buf.clear_rows(0..buf.height.saturating_sub(1));
    }
    if app
        .message
        .as_ref()
        .is_some_and(|(_, t)| t.elapsed() > MESSAGE_TIME)
    {
        app.message = None;
    }
    let status = match (&app.message, graphics && app.show_stats) {
        (Some((m, _)), _) => Some(m.clone()),
        (None, true) => gfx.as_ref().and_then(|g| g.stats()),
        (None, false) => None,
    };
    if let Some(s) = status {
        let row = buf.height.saturating_sub(1);
        buf.text(0, row, &format!(" {s} "));
    }
    screen.draw(out, buf)?;
    if graphics && let Some(g) = gfx {
        g.draw(out, anim)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::View;

    #[test]
    fn the_a_key_skips_graphics_when_unavailable() {
        assert_eq!(View::Ascii.next(false), View::Ansi);
        assert_eq!(View::Ansi.next(false), View::Ascii);
        assert_eq!(View::Ansi.next(true), View::Graphics);
        assert_eq!(View::Graphics.next(true), View::Ascii);
    }
}
