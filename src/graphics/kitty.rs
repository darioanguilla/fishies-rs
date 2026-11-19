//! Kitty graphics protocol encoder.
//!
//! The frame is sent as zlib-compressed RGBA (with a real alpha channel),
//! base64-encoded in chunks of at most 4096 bytes. Every frame reuses image
//! id 1 and placement id 1, so the terminal replaces the previous frame
//! instead of stacking a new one.

use std::io::Write as _;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use super::canvas::Canvas;

const CHUNK: usize = 4096;

fn base64(data: &[u8]) -> Vec<u8> {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        out.push(ABC[(n >> 18) as usize & 63]);
        out.push(ABC[(n >> 12) as usize & 63]);
        out.push(if c.len() > 1 {
            ABC[(n >> 6) as usize & 63]
        } else {
            b'='
        });
        out.push(if c.len() > 2 {
            ABC[n as usize & 63]
        } else {
            b'='
        });
    }
    out
}

pub fn encode(canvas: &Canvas) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(canvas.px.len() * 4);
    for &p in &canvas.px {
        rgba.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8, (p >> 24) as u8]);
    }
    let mut z = ZlibEncoder::new(Vec::new(), Compression::fast());
    z.write_all(&rgba).expect("writing to memory");
    let data = base64(&z.finish().expect("writing to memory"));

    let mut out = Vec::with_capacity(data.len() + data.len() / CHUNK * 16 + 96);
    let chunks: Vec<&[u8]> = data.chunks(CHUNK).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        if i == 0 {
            // transmit and display, RGBA, zlib, quiet, don't move the cursor
            let _ = write!(
                out,
                "\x1b_Ga=T,f=32,o=z,s={},v={},i=1,p=1,q=2,C=1,m={more};",
                canvas.width, canvas.height
            );
        } else {
            let _ = write!(out, "\x1b_Gm={more};");
        }
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}

/// Remove our image from the screen.
pub const DELETE: &[u8] = b"\x1b_Ga=d,d=I,i=1,q=2\x1b\\";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b""), b"");
        assert_eq!(base64(b"f"), b"Zg==");
        assert_eq!(base64(b"fo"), b"Zm8=");
        assert_eq!(base64(b"foobar"), b"Zm9vYmFy");
    }

    #[test]
    fn frames_are_chunked() {
        // noise compresses badly, so this needs several chunks
        let mut c = Canvas::new(100, 100);
        let mut s = 12345u32;
        for p in &mut c.px {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            *p = 0xFF00_0000 | (s >> 8);
        }
        let out = String::from_utf8(encode(&c)).unwrap();
        assert!(out.starts_with("\x1b_Ga=T,f=32,o=z,s=100,v=100,i=1,p=1,q=2,C=1,m=1;"));
        assert!(out.matches("\x1b_G").count() > 2);
        assert!(out.ends_with("\x1b\\"));
        assert!(out.contains("\x1b_Gm=0;"));
    }
}
