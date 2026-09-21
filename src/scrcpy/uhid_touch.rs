// Self-contained state machine: also tested directly with `rustc --test`.
// All active contacts are sent in ONE parallel HID report, so adding a fire
// contact cannot temporarily remove the existing aim/joystick contacts.

pub const DEVICE_ID: u16 = 0x544d;
pub const DEVICE_NAME: &str = "scrcpy-mask touch";
pub const MAX_CONTACTS: usize = 10;
pub const REPORT_LEN: usize = 2 + 6 * MAX_CONTACTS;
const MAX_AXIS: u16 = 32767;

pub fn descriptor() -> Vec<u8> {
    let mut out = vec![
        0x05, 0x0d, // Digitizers
        0x09, 0x04, // Touch Screen
        0xa1, 0x01, // Application collection
        0x85, 0x01, // Report ID 1
    ];
    for _ in 0..MAX_CONTACTS {
        out.extend_from_slice(&[
            0x05, 0x0d, 0x09, 0x22, 0xa1, 0x02, // Finger, logical collection
            0x09, 0x42, 0x09, 0x32, 0x09, 0x47, // Tip, In Range, Confidence
            0x15, 0x00, 0x25, 0x01, 0x75, 0x01, 0x95, 0x03, 0x81, 0x02, 0x75, 0x05, 0x95, 0x01,
            0x81, 0x03, // padding
            0x09, 0x51, 0x25, 0x09, 0x75, 0x08, 0x95, 0x01, 0x81, 0x02, // contact ID
            0x05, 0x01, 0x09, 0x30, 0x09, 0x31, // X, Y
            0x16, 0x00, 0x00, 0x26, 0xff, 0x7f, // logical 0..32767
            0x75, 0x10, 0x95, 0x02, 0x81, 0x02, 0xc0,
        ]);
    }
    out.extend_from_slice(&[
        0x05,
        0x0d,
        0x09,
        0x54, // contact count
        0x15,
        0x00,
        0x25,
        MAX_CONTACTS as u8,
        0x75,
        0x08,
        0x95,
        0x01,
        0x81,
        0x02,
        0xc0,
    ]);
    out
}

pub fn create_message() -> Vec<u8> {
    let desc = descriptor();
    let mut data = vec![12]; // scrcpy UHID_CREATE
    data.extend_from_slice(&DEVICE_ID.to_be_bytes());
    data.extend_from_slice(&[0, 0, 0, 0]); // no impersonated vendor/product
    data.push(DEVICE_NAME.len() as u8);
    data.extend_from_slice(DEVICE_NAME.as_bytes());
    data.extend_from_slice(&(desc.len() as u16).to_be_bytes());
    data.extend_from_slice(&desc);
    data
}

pub fn input_message(report: &[u8; REPORT_LEN]) -> Vec<u8> {
    let mut data = vec![13]; // UHID_INPUT
    data.extend_from_slice(&DEVICE_ID.to_be_bytes());
    data.extend_from_slice(&(REPORT_LEN as u16).to_be_bytes());
    data.extend_from_slice(report);
    data
}

pub fn destroy_message() -> Vec<u8> {
    let mut data = vec![14]; // UHID_DESTROY
    data.extend_from_slice(&DEVICE_ID.to_be_bytes());
    data
}

#[derive(Clone, Copy, Debug)]
struct Contact {
    pointer: u64,
    x: u16,
    y: u16,
}

#[derive(Default)]
pub struct Touchscreen {
    contacts: [Option<Contact>; MAX_CONTACTS],
}

impl Touchscreen {
    /// action: Android DOWN=0, UP=1, MOVE=2. MOVE never resurrects a released
    /// pointer; this is important after focus loss, dropped events or rotation.
    pub fn touch(
        &mut self,
        action: u8,
        pointer: u64,
        x: i32,
        y: i32,
        width: u16,
        height: u16,
        rotation: u16,
    ) -> Result<Option<[u8; REPORT_LEN]>, &'static str> {
        if width < 2 || height < 2 || rotation > 3 {
            return Err("Invalid touchscreen dimensions or rotation");
        }
        let slot = self
            .contacts
            .iter()
            .position(|c| c.is_some_and(|c| c.pointer == pointer));
        let (x, y) = natural_coordinates(x, y, width, height, rotation);
        match action {
            0 => {
                let i = slot
                    .or_else(|| self.contacts.iter().position(Option::is_none))
                    .ok_or("UHID touchscreen supports at most 10 simultaneous contacts")?;
                self.contacts[i] = Some(Contact { pointer, x, y });
                Ok(Some(self.report(None)))
            }
            1 => {
                let Some(i) = slot else { return Ok(None) };
                // Include the lifted contact with tip=0 in this same frame.
                // Its old contact ID remains valid until the next DOWN.
                let report = self.report(Some(i));
                self.contacts[i] = None;
                Ok(Some(report))
            }
            2 => {
                let Some(i) = slot else { return Ok(None) };
                self.contacts[i] = Some(Contact { pointer, x, y });
                Ok(Some(self.report(None)))
            }
            _ => Err("Unsupported UHID touch action"),
        }
    }

    fn report(&self, lifted: Option<usize>) -> [u8; REPORT_LEN] {
        let mut report = [0; REPORT_LEN];
        report[0] = 1;
        let mut index = 1;
        let mut count = 0;
        for (id, contact) in self.contacts.iter().enumerate() {
            if let Some(c) = contact {
                report[index] = if lifted == Some(id) { 0 } else { 7 };
                report[index + 1] = id as u8;
                report[index + 2..index + 4].copy_from_slice(&c.x.to_le_bytes());
                report[index + 4..index + 6].copy_from_slice(&c.y.to_le_bytes());
                index += 6;
                // Contact count is the number of VALID records (includes UP),
                // not the number of fingers currently touching.
                count += 1;
            }
        }
        report[REPORT_LEN - 1] = count;
        report
    }

    pub fn release_all(&mut self) -> [u8; REPORT_LEN] {
        let mut report = self.report(None);
        for i in 0..MAX_CONTACTS {
            report[1 + i * 6] = 0;
        }
        self.contacts = [None; MAX_CONTACTS];
        report
    }
}

fn natural_coordinates(x: i32, y: i32, width: u16, height: u16, rotation: u16) -> (u16, u16) {
    let scale = |v: i32, size: u16| -> u16 {
        ((i64::from(v.clamp(0, i32::from(size) - 1)) * i64::from(MAX_AXIS)) / (i64::from(size) - 1))
            as u16
    };
    let x = scale(x, width);
    let y = scale(y, height);
    // Android rotates an absolute touch device from natural to display space.
    // Undo that rotation before submitting the raw HID axes.
    //
    // Empirically verified against the live device (dumpsys input, viewport
    // orientation=1, surface 1080x2340, display 2340x1080): injecting the
    // display point (300, 500) produced raw (15183, 28565), and Android cooked
    // it to display (2339 - 300, 1079 - 500). Fitting: display = (rawY, MAX -
    // rawX), so the inverse family is:
    //   rotation 1 (90°):  raw = (MAX - y, x)
    //   rotation 2 (180°): raw = (MAX - x, MAX - y)
    //   rotation 3 (270°): raw = (y, MAX - x)
    match rotation {
        1 => (MAX_AXIS - y, x),
        2 => (MAX_AXIS - x, MAX_AXIS - y),
        3 => (y, MAX_AXIS - x),
        _ => (x, y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(t: &mut Touchscreen, action: u8, id: u64) -> [u8; REPORT_LEN] {
        t.touch(action, id, 500, 250, 1001, 501, 0)
            .unwrap()
            .unwrap()
    }
    #[test]
    fn moving_aim_while_firing_preserves_both_contacts() {
        let mut t = Touchscreen::default();
        event(&mut t, 0, 6);
        let fire = event(&mut t, 0, 7);
        assert_eq!((fire[1], fire[7], fire[REPORT_LEN - 1]), (7, 7, 2));
        let motion = event(&mut t, 2, 6);
        assert_eq!(fire, motion);
        let up = event(&mut t, 1, 7);
        assert_eq!((up[1], up[7], up[REPORT_LEN - 1]), (7, 0, 2));
        let motion = event(&mut t, 2, 6);
        assert_eq!(motion[REPORT_LEN - 1], 1);
    }
    #[test]
    fn stable_ids_and_no_aliasing_or_resurrection() {
        let mut t = Touchscreen::default();
        for i in 0..10 {
            event(&mut t, 0, 100 + i);
        }
        assert!(t.touch(0, 999, 1, 1, 100, 100, 0).is_err());
        event(&mut t, 1, 104);
        let r = event(&mut t, 0, 900);
        assert_eq!(r[2 + 4 * 6], 4);
        assert_eq!(r[REPORT_LEN - 1], 10);
        let release = t.release_all();
        assert_eq!(release[REPORT_LEN - 1], 10);
        assert!((0..10).all(|i| release[1 + i * 6] == 0));
        assert!(t.touch(2, 900, 1, 1, 100, 100, 0).unwrap().is_none());
    }
    #[test]
    fn coordinates_clamp_and_unrotate() {
        assert_eq!(natural_coordinates(-3, 200, 101, 201, 0), (0, 32767));
        // rotation 1 (90°): raw = (MAX - y, x)
        assert_eq!(natural_coordinates(0, 0, 201, 101, 1), (32767, 0));
        assert_eq!(natural_coordinates(201, 101, 201, 101, 1), (0, 32767));
        // rotation 2 (180°): raw = (MAX - x, MAX - y)
        assert_eq!(natural_coordinates(0, 0, 101, 201, 2), (32767, 32767));
        // rotation 3 (270°): raw = (y, MAX - x)
        assert_eq!(natural_coordinates(0, 0, 201, 101, 3), (0, 32767));
        assert_eq!(natural_coordinates(201, 101, 201, 101, 3), (32767, 0));
        // Empirical anchor from the live device (dumpsys input, viewport
        // orientation=1): display (300, 500) of 2340x1080 must produce raw
        // (17584, 4202), which Android cooks back to (300, 500).
        assert_eq!(natural_coordinates(300, 500, 2340, 1080, 1), (17584, 4202));
        assert!(Touchscreen::default().touch(0, 1, 0, 0, 0, 0, 0).is_err());
    }
    #[test]
    fn protocol_frames_have_correct_endianness_and_lengths() {
        let create = create_message();
        assert_eq!(
            &create[..8],
            &[12, 0x54, 0x4d, 0, 0, 0, 0, DEVICE_NAME.len() as u8]
        );
        let offset = 8 + DEVICE_NAME.len();
        assert_eq!(
            u16::from_be_bytes([create[offset], create[offset + 1]]) as usize,
            descriptor().len()
        );
        assert_eq!(create.len(), offset + 2 + descriptor().len());
        let frame = input_message(&event(&mut Touchscreen::default(), 0, u64::MAX));
        assert_eq!(&frame[..6], &[13, 0x54, 0x4d, 0, REPORT_LEN as u8, 1]);
        assert_eq!(frame.len(), REPORT_LEN + 5);
        assert_eq!(destroy_message(), vec![14, 0x54, 0x4d]);
    }
    #[test]
    fn duplicate_down_updates_without_adding_a_contact() {
        let mut t = Touchscreen::default();
        event(&mut t, 0, 1);
        assert_eq!(event(&mut t, 0, 1)[REPORT_LEN - 1], 1);
        assert!(t.touch(1, 55, 1, 1, 100, 100, 0).unwrap().is_none());
    }
}
