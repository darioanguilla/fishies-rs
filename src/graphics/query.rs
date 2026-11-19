//! Ask the terminal questions and read its answers, with a timeout.
//!
//! Terminals answer queries such as "which features do you have?" (DA1) by
//! typing the reply into our input. crossterm drops replies it doesn't know,
//! so this reads the input directly, before the event loop starts. The
//! terminal must already be in raw mode, or the reply would be echoed and
//! line-buffered.

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Write `request`, then collect the reply until `done` says it's complete
/// or `timeout` expires. Returns whatever arrived.
pub fn ask(request: &[u8], timeout: Duration, done: impl Fn(&[u8]) -> bool) -> io::Result<Vec<u8>> {
    let mut input = sys::Input::open()?;
    let mut out = io::stdout();
    out.write_all(request)?;
    out.flush()?;
    let deadline = Instant::now() + timeout;
    let mut reply = Vec::new();
    while !done(&reply) {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        input.read(&mut reply, left)?;
    }
    Ok(reply)
}

#[cfg(unix)]
mod sys {
    use std::fs::File;
    use std::io;
    use std::os::fd::{AsRawFd, RawFd};
    use std::time::Duration;

    pub struct Input {
        fd: RawFd,
        _tty: Option<File>,
    }

    impl Input {
        pub fn open() -> io::Result<Self> {
            // SAFETY: isatty only inspects the descriptor.
            if unsafe { libc::isatty(0) } == 1 {
                return Ok(Input { fd: 0, _tty: None });
            }
            let tty = File::options().read(true).open("/dev/tty")?;
            Ok(Input {
                fd: tty.as_raw_fd(),
                _tty: Some(tty),
            })
        }

        /// Append what is available, waiting at most `timeout`.
        pub fn read(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> io::Result<()> {
            let mut pfd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ms = timeout.as_millis().clamp(1, i32::MAX as u128) as i32;
            // SAFETY: one valid pollfd.
            let ready = unsafe { libc::poll(&mut pfd, 1, ms) };
            if ready < 0 {
                let err = io::Error::last_os_error();
                return if err.kind() == io::ErrorKind::Interrupted {
                    Ok(())
                } else {
                    Err(err)
                };
            }
            if ready == 0 {
                return Ok(());
            }
            let mut chunk = [0u8; 256];
            // SAFETY: the buffer is valid for chunk.len() bytes.
            let n = unsafe { libc::read(self.fd, chunk.as_mut_ptr().cast(), chunk.len()) };
            if n < 0 {
                return Err(io::Error::last_os_error());
            }
            buf.extend_from_slice(&chunk[..n as usize]);
            Ok(())
        }
    }
}

#[cfg(windows)]
mod sys {
    use std::io;
    use std::time::Duration;

    use windows_sys::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_INPUT, GetConsoleMode, GetStdHandle, INPUT_RECORD, KEY_EVENT,
        ReadConsoleInputW, STD_INPUT_HANDLE, SetConsoleMode,
    };
    use windows_sys::Win32::System::Threading::WaitForSingleObject;

    /// The console input, switched to VT input so that replies arrive as
    /// plain characters; the previous mode is restored on drop.
    pub struct Input {
        handle: HANDLE,
        mode: u32,
    }

    impl Input {
        pub fn open() -> io::Result<Self> {
            // SAFETY: plain Win32 calls on the process' own console handle.
            unsafe {
                let handle = GetStdHandle(STD_INPUT_HANDLE);
                let mut mode = 0;
                if GetConsoleMode(handle, &mut mode) == 0 {
                    return Err(io::Error::last_os_error());
                }
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_INPUT);
                Ok(Input { handle, mode })
            }
        }

        /// Append what is available, waiting at most `timeout`.
        pub fn read(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> io::Result<()> {
            let ms = timeout.as_millis().clamp(1, u32::MAX as u128) as u32;
            // SAFETY: valid handle; the record buffer outlives the call.
            unsafe {
                if WaitForSingleObject(self.handle, ms) != WAIT_OBJECT_0 {
                    return Ok(());
                }
                let mut records: [INPUT_RECORD; 64] = std::mem::zeroed();
                let mut n = 0;
                if ReadConsoleInputW(self.handle, records.as_mut_ptr(), 64, &mut n) == 0 {
                    return Err(io::Error::last_os_error());
                }
                for r in &records[..n as usize] {
                    if u32::from(r.EventType) != KEY_EVENT {
                        continue;
                    }
                    let key = r.Event.KeyEvent;
                    let unit = key.uChar.UnicodeChar;
                    if key.bKeyDown != 0
                        && unit != 0
                        && let Some(c) = char::from_u32(u32::from(unit))
                    {
                        let mut utf8 = [0; 4];
                        buf.extend_from_slice(c.encode_utf8(&mut utf8).as_bytes());
                    }
                }
            }
            Ok(())
        }
    }

    impl Drop for Input {
        fn drop(&mut self) {
            // SAFETY: restores the mode read in `open`.
            unsafe {
                SetConsoleMode(self.handle, self.mode);
            }
        }
    }
}
