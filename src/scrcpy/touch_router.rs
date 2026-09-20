// Each control connection owns its own contacts.
use std::collections::BTreeMap;

use super::{
    constant::MotionEventAction,
    control_msg::ScrcpyControlMsg,
    uhid_touch::{self, Touchscreen},
};
use crate::config::TouchBackend;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Geometry {
    pub width: u32,
    pub height: u32,
    pub rotation: u16,
}

impl Geometry {
    pub fn valid(self) -> bool {
        (2..=65535).contains(&self.width)
            && (2..=65535).contains(&self.height)
            && self.rotation <= 3
    }
}

pub struct TouchRouter {
    backend: TouchBackend,
    geometry: Geometry,
    uhid: Touchscreen,
    created: bool,
    sdk_contacts: BTreeMap<u64, ScrcpyControlMsg>,
}

impl TouchRouter {
    pub fn new(backend: TouchBackend) -> Self {
        Self {
            backend,
            geometry: Geometry::default(),
            uhid: Touchscreen::default(),
            created: false,
            sdk_contacts: BTreeMap::new(),
        }
    }

    pub fn set_geometry(&mut self, geometry: Geometry) -> Vec<u8> {
        if self.geometry == geometry {
            return Vec::new();
        }
        let mut data = self.release_all();
        self.geometry = geometry;
        if self.backend == TouchBackend::Uhid && geometry.valid() && !self.created {
            // Register on connection, before the user starts mapping. The server
            // has no create ACK, so this is a request, not a success assertion.
            data.extend(uhid_touch::create_message());
            self.created = true;
            log::info!(
                "[UHID] Virtual touchscreen creation requested; verify Android device registration"
            );
        }
        data
    }

    pub fn route(&mut self, mut msg: ScrcpyControlMsg) -> Result<Vec<u8>, &'static str> {
        if matches!(msg, ScrcpyControlMsg::ReleaseAllTouches) {
            return Ok(self.release_all());
        }
        match &mut msg {
            ScrcpyControlMsg::InjectTouchEvent {
                action,
                pointer_id,
                x,
                y,
                w,
                h,
                ..
            } => {
                if !self.geometry.valid() || *w < 2 || *h < 2 {
                    // No guessed coordinates before the phone announces its display.
                    return Ok(Vec::new());
                }
                if self.backend == TouchBackend::Uhid {
                    let report = self.uhid.touch(
                        *action as u8,
                        *pointer_id,
                        *x,
                        *y,
                        *w,
                        *h,
                        self.geometry.rotation,
                    )?;
                    let Some(report) = report else {
                        return Ok(Vec::new());
                    };
                    let mut data = Vec::new();
                    if !self.created {
                        data.extend(uhid_touch::create_message());
                        self.created = true;
                        // scrcpy's UHID_CREATE has no acknowledgement. This only
                        // proves the request was queued, not that Android accepted it.
                        log::info!(
                            "[UHID] Virtual touchscreen creation requested; verify Android device registration"
                        );
                    }
                    data.extend(uhid_touch::input_message(&report));
                    return Ok(data);
                }
                let id = *pointer_id;
                if *action == MotionEventAction::Move && !self.sdk_contacts.contains_key(&id) {
                    return Ok(Vec::new());
                }
                scale(x, y, w, h, self.geometry);
                match action {
                    MotionEventAction::Down | MotionEventAction::Move => {
                        self.sdk_contacts.insert(id, msg.clone());
                    }
                    MotionEventAction::Up => {
                        self.sdk_contacts.remove(&id);
                    }
                }
            }
            ScrcpyControlMsg::InjectScrollEvent { x, y, w, h, .. } => {
                if !self.geometry.valid() || *w < 2 || *h < 2 {
                    return Ok(Vec::new());
                }
                // Scroll and keyboard/text retain SDK semantics, even in UHID touch mode.
                scale(x, y, w, h, self.geometry);
            }
            _ => {}
        }
        Ok(msg.into())
    }

    pub fn release_all(&mut self) -> Vec<u8> {
        if self.backend == TouchBackend::Uhid {
            let report = self.uhid.release_all();
            return if self.created {
                uhid_touch::input_message(&report)
            } else {
                Vec::new()
            };
        }
        let mut data = Vec::new();
        for (_, mut msg) in std::mem::take(&mut self.sdk_contacts) {
            if let ScrcpyControlMsg::InjectTouchEvent {
                action, pressure, ..
            } = &mut msg
            {
                *action = MotionEventAction::Up;
                *pressure = half::f16::ZERO;
            }
            data.extend(Vec::<u8>::from(msg));
        }
        data
    }

    pub fn close(&mut self) -> Vec<u8> {
        let mut data = self.release_all();
        if self.created {
            data.extend(uhid_touch::destroy_message());
            self.created = false;
        }
        data
    }
}

fn scale(x: &mut i32, y: &mut i32, w: &mut u16, h: &mut u16, geometry: Geometry) {
    *x = (i64::from(*x) * i64::from(geometry.width) / i64::from(*w)) as i32;
    *y = (i64::from(*y) * i64::from(geometry.height) / i64::from(*h)) as i32;
    *w = geometry.width as u16;
    *h = geometry.height as u16;
}

#[cfg(test)]
mod tests {
    use super::super::constant::MotionEventButtons;
    use super::*;

    fn touch(action: MotionEventAction, pointer_id: u64) -> ScrcpyControlMsg {
        ScrcpyControlMsg::InjectTouchEvent {
            action,
            pointer_id,
            x: 25,
            y: 50,
            w: 101,
            h: 201,
            pressure: half::f16::ONE,
            action_button: MotionEventButtons::empty(),
            buttons: MotionEventButtons::empty(),
        }
    }
    fn geometry(rotation: u16) -> Geometry {
        Geometry {
            width: 1080,
            height: 2340,
            rotation,
        }
    }

    #[test]
    fn uhid_creates_once_and_destroys_after_release() {
        let mut router = TouchRouter::new(TouchBackend::Uhid);
        assert!(
            router
                .route(touch(MotionEventAction::Down, 6))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            router.set_geometry(geometry(0)),
            uhid_touch::create_message()
        );
        let first = router.route(touch(MotionEventAction::Down, 6)).unwrap();
        assert_eq!(first[0], 13);
        let second = router.route(touch(MotionEventAction::Down, 7)).unwrap();
        assert_eq!(second[0], 13);
        assert_eq!(*second.last().unwrap(), 2);
        let close = router.close();
        assert_eq!(close[6], 0); // first contact lifted
        assert_eq!(close[12], 0); // second contact lifted
        assert!(close.ends_with(&uhid_touch::destroy_message()));
        assert!(router.close().is_empty());
    }

    #[test]
    fn rotation_and_local_barrier_drop_stale_moves() {
        let mut router = TouchRouter::new(TouchBackend::Uhid);
        router.set_geometry(geometry(0));
        router.route(touch(MotionEventAction::Down, 6)).unwrap();
        assert_eq!(router.set_geometry(geometry(1))[6], 0);
        assert!(
            router
                .route(touch(MotionEventAction::Move, 6))
                .unwrap()
                .is_empty()
        );
        router.route(touch(MotionEventAction::Down, 7)).unwrap();
        assert_eq!(
            router.route(ScrcpyControlMsg::ReleaseAllTouches).unwrap()[6],
            0
        );
        assert!(
            router
                .route(touch(MotionEventAction::Move, 7))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn sdk_still_uses_sdk_packets_and_releases_every_pointer() {
        let mut router = TouchRouter::new(TouchBackend::Sdk);
        router.set_geometry(geometry(0));
        for id in [6, 7] {
            let data = router.route(touch(MotionEventAction::Down, id)).unwrap();
            assert_eq!(&data[..2], &[2, 0]);
        }
        let release = router.close();
        assert_eq!(release.len(), 64);
        assert_eq!(&release[..2], &[2, 1]);
        assert_eq!(&release[32..34], &[2, 1]);
        assert!(
            router
                .route(touch(MotionEventAction::Move, 6))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn non_touch_messages_keep_their_existing_protocol() {
        let mut router = TouchRouter::new(TouchBackend::Uhid);
        let msg = ScrcpyControlMsg::InjectText {
            text: "test".into(),
        };
        let expected: Vec<u8> = msg.clone().into();
        assert_eq!(router.route(msg).unwrap(), expected);
        assert!(router.close().is_empty());
    }
}
