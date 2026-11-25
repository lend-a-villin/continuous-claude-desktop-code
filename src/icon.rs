// The CCDC icon: a filled circle with a white "C" (a ring with a gap on the right),
// drawn in code so no image files or image libraries are needed.
// Also included by build.rs to write the .ico / .png files for the installers.

pub const ORANGE: [u8; 3] = [217, 119, 87];
#[allow(dead_code)]
pub const GREY: [u8; 3] = [140, 140, 140];

/// RGBA pixels, `size` x `size`, anti-aliased by 4x4 supersampling.
pub fn draw(size: u32, color: [u8; 3]) -> Vec<u8> {
    let mut px = vec![0u8; (size * size * 4) as usize];
    let s = size as f32;
    let c = s / 2.0;
    let r_disc = s * 0.47;
    let r_out = s * 0.30;
    let r_in = s * 0.17;
    const N: u32 = 4;
    for y in 0..size {
        for x in 0..size {
            let (mut disc, mut ring) = (0u32, 0u32);
            for sy in 0..N {
                for sx in 0..N {
                    let fx = x as f32 + (sx as f32 + 0.5) / N as f32 - c;
                    let fy = y as f32 + (sy as f32 + 0.5) / N as f32 - c;
                    let d = (fx * fx + fy * fy).sqrt();
                    if d <= r_disc {
                        disc += 1;
                        // gap of the "C": the wedge opening to the right (about ±40°)
                        let in_gap = fx > 0.0 && fy.abs() < fx * 0.84;
                        if d <= r_out && d >= r_in && !in_gap {
                            ring += 1;
                        }
                    }
                }
            }
            let total = (N * N) as f32;
            let a_disc = disc as f32 / total;
            let a_ring = ring as f32 / total;
            let i = ((y * size + x) * 4) as usize;
            if disc > 0 {
                // white over the colour, by how much of the pixel the ring covers
                let t = if disc > 0 { a_ring / a_disc } else { 0.0 };
                for k in 0..3 {
                    px[i + k] = (color[k] as f32 * (1.0 - t) + 255.0 * t).round() as u8;
                }
                px[i + 3] = (a_disc * 255.0).round() as u8;
            }
        }
    }
    px
}

/// Minimal PNG encoder (stored deflate blocks, no compression): good enough for icons.
#[allow(dead_code)]
pub fn png(size: u32, rgba: &[u8]) -> Vec<u8> {
    fn crc32(data: &[u8]) -> u32 {
        let mut c = 0xffff_ffffu32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }
    fn adler32(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &x in data {
            a = (a + x as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    }
    let mut raw = Vec::with_capacity(rgba.len() + size as usize);
    for row in rgba.chunks(size as usize * 4) {
        raw.push(0); // filter: none
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65535).peekable();
    while let Some(b) = blocks.next() {
        z.push(if blocks.peek().is_none() { 1 } else { 0 });
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&size.to_be_bytes());
    ihdr.extend_from_slice(&size.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

/// .ico holding PNG images of the given sizes.
#[allow(dead_code)]
pub fn ico(sizes: &[u32], color: [u8; 3]) -> Vec<u8> {
    let pngs: Vec<Vec<u8>> = sizes.iter().map(|&s| png(s, &draw(s, color))).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (i, &s) in sizes.iter().enumerate() {
        let b = if s >= 256 { 0 } else { s as u8 };
        out.extend_from_slice(&[b, b, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(pngs[i].len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += pngs[i].len() as u32;
    }
    for p in &pngs {
        out.extend_from_slice(p);
    }
    out
}
