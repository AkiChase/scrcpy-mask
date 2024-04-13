pub fn write_16be(buf: &mut [u8], val: u16) {
    buf[0] = (val >> 8) as u8;
    buf[1] = val as u8;
}

pub fn write_32be(buf: &mut [u8], val: u32) {
    buf[0] = (val >> 24) as u8;
    buf[1] = (val >> 16) as u8;
    buf[2] = (val >> 8) as u8;
    buf[3] = val as u8;
}

pub fn write_64be(buf: &mut [u8], val: u64) {
    buf[0] = (val >> 56) as u8;
    buf[1] = (val >> 48) as u8;
    buf[2] = (val >> 40) as u8;
    buf[3] = (val >> 32) as u8;
    buf[4] = (val >> 24) as u8;
    buf[5] = (val >> 16) as u8;
    buf[6] = (val >> 8) as u8;
    buf[7] = val as u8;
}

pub fn float_to_u16fp(mut f: f32) -> u16 {
    if f < 0.0 || f > 1.0 {
        f = 1.0;
    }
    let mut u: u32 = (f * (1 << 16) as f32) as u32;
    if u >= 0xffff {
        u = 0xffff;
    }
    u as u16
}

pub fn float_to_i16fp(f: f32) -> i16 {
    assert!(f >= -1.0 && f <= 1.0);
    let mut i: i32 = (f * (1 << 15) as f32) as i32;
    assert!(i >= -0x8000);
    if i >= 0x7fff {
        assert_eq!(i, 0x8000); // for f == 1.0
        i = 0x7fff;
    }
    i as i16
}

pub fn write_posion(buf: &mut [u8], x: i32, y: i32, w: u16, h: u16) {
    write_32be(buf, x as u32);
    write_32be(&mut buf[4..8], y as u32);
    write_16be(&mut buf[8..10], w);
    write_16be(&mut buf[10..12], h);
}

pub fn write_string(utf8: &str, max_len: usize, buf: &mut Vec<u8>) {
    let len = str_utf8_truncation_index(utf8, max_len) as u32;
    // first 4 bytes for length
    let len_bytes = len.to_be_bytes();
    buf.extend_from_slice(&len_bytes);
    // then [len] bytes for the string
    buf.extend_from_slice(utf8.as_bytes())
}

// truncate utf8 string to max_len bytes
fn str_utf8_truncation_index(utf8: &str, max_len: usize) -> usize {
    let len = utf8.len();
    if len <= max_len {
        return len;
    }
    let mut len = max_len;
    while utf8.is_char_boundary(len) {
        len -= 1;
    }
    len
}
