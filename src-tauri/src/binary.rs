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

pub fn float_to_u16fp(f: f32) -> u16 {
    assert!(f >= 0.0 && f <= 1.0);
    let mut u: u32 = (f * (1 << 16) as f32) as u32;
    if u >= 0xffff {
        assert_eq!(u, 0x10000); // for f == 1.0
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
