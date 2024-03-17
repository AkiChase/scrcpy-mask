use scrcpy_mask::control_msg::{gen_ctrl_msg, ControlMsgType};
use serde_json::json;

// https://github.com/Genymobile/scrcpy/blob/master/app/tests/test_control_msg_serialize.c
// Thansk for Copilot

#[test]
fn test_serialize_inject_keycode() {
    let msg = json!(
        {
            "action": 1,
            "keycode": 66,
            "repeat": 5,
            "metastate": 65
        });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeInjectKeycode, &msg);

    let expected: [u8; 14] = [
        ControlMsgType::ControlMsgTypeInjectKeycode as u8,
        0x01, // AKEY_EVENT_ACTION_UP
        0x00, 0x00, 0x00, 0x42, // AKEYCODE_ENTER
        0x00, 0x00, 0x00, 0x05, // repeat
        0x00, 0x00, 0x00, 0x41, // AMETA_SHIFT_ON | AMETA_SHIFT_LEFT_ON
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_inject_text() {
    let msg = json!(
        {
            "text": "hello, world!"
        });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeInjectText, &msg);

    let expected: [u8; 18] = [
        ControlMsgType::ControlMsgTypeInjectText as u8,
        0x00, 0x00, 0x00, 0x0d, // text length
        'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ',' as u8, ' ' as u8, 'w' as u8, 'o' as u8, 'r' as u8, 'l' as u8, 'd' as u8, '!' as u8, // text
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_inject_text_long() {
    let msg = json!({
        "text": "a".repeat(scrcpy_mask::control_msg::SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH)
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeInjectText, &msg);

    let mut expected: Vec<u8> = vec![
        ControlMsgType::ControlMsgTypeInjectText as u8,
        0x00, 0x00, 0x01, 0x2c, // text length (32 bits)
    ];
    expected.extend(vec!['a' as u8; scrcpy_mask::control_msg::SC_CONTROL_MSG_INJECT_TEXT_MAX_LENGTH]);

    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_inject_touch_event() {
    let msg = json!(
        {
            "action": 0,
            "pointerId": 0x1234567887654321 as u64,
            "position":{
                "x": 100,
                "y": 200,
                "w": 1080,
                "h": 1920
            },
            "pressure": 1.0,
            "actionButton": 1,
            "buttons": 1
        });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeInjectTouchEvent, &msg);

    let expected: [u8; 32] = [
        ControlMsgType::ControlMsgTypeInjectTouchEvent as u8,
        0x00, // AKEY_EVENT_ACTION_DOWN
        0x12, 0x34, 0x56, 0x78, 0x87, 0x65, 0x43, 0x21, // pointer id
        0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0xc8, // 100 200
        0x04, 0x38, 0x07, 0x80, // 1080 1920
        0xff, 0xff, // pressure
        0x00, 0x00, 0x00, 0x01, // AMOTION_EVENT_BUTTON_PRIMARY (action button)
        0x00, 0x00, 0x00, 0x01, // AMOTION_EVENT_BUTTON_PRIMARY (buttons)
    ];
    assert_eq!(buf, expected);
}


#[test]
fn test_serialize_inject_scroll_event() {
    let msg = json!(
        {
            "position":{
                "x": 260,
                "y": 1026,
                "w": 1080,
                "h": 1920
            },
            "hscroll": 1.0,
            "vscroll": -1.0,
            "buttons": 1
        });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeInjectScrollEvent, &msg);

    let expected: [u8; 21] = [
        ControlMsgType::ControlMsgTypeInjectScrollEvent as u8,
        0x00, 0x00, 0x01, 0x04, 0x00, 0x00, 0x04, 0x02, // 260 1026
        0x04, 0x38, 0x07, 0x80, // 1080 1920
        0x7F, 0xFF, // 1 (float encoded as i16)
        0x80, 0x00, // -1 (float encoded as i16)
        0x00, 0x00, 0x00, 0x01, // 1
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_back_or_screen_on() {
    let msg = json!({
        "action": 1
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeBackOrScreenOn, &msg);

    let expected: [u8; 2] = [
        ControlMsgType::ControlMsgTypeBackOrScreenOn as u8,
        0x01, // AKEY_EVENT_ACTION_UP
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_get_clipboard() {
    let msg = json!({
        "copyKey": 1
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeGetClipboard, &msg);

    let expected: [u8; 2] = [
        ControlMsgType::ControlMsgTypeGetClipboard as u8,
        0x01, // SC_COPY_KEY_COPY
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_set_clipboard() {
    let msg = json!({
        "sequence": 0x0102030405060708 as u64,
        "paste": true,
        "text": "hello, world!"
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeSetClipboard, &msg);

    let expected: Vec<u8> = vec![
        ControlMsgType::ControlMsgTypeSetClipboard as u8,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // sequence
        1, // paste
        0x00, 0x00, 0x00, 0x0d, // text length
        'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ',' as u8, ' ' as u8, 'w' as u8, 'o' as u8, 'r' as u8, 'l' as u8, 'd' as u8, '!' as u8, // text
    ];

    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_set_clipboard_long() {
    let long_text = "a".repeat(scrcpy_mask::control_msg::SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH);
    let msg = json!({
        "sequence": 0x0102030405060708 as u64,
        "paste": true,
        "text": long_text
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeSetClipboard, &msg);

    let mut expected: Vec<u8> = vec![
        ControlMsgType::ControlMsgTypeSetClipboard as u8,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // sequence
        1, // paste
        (scrcpy_mask::control_msg::SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH >> 24) as u8,
        ((scrcpy_mask::control_msg::SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH >> 16) & 0xff) as u8,
        ((scrcpy_mask::control_msg::SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH >> 8) & 0xff) as u8,
        (scrcpy_mask::control_msg::SC_CONTROL_MSG_CLIPBOARD_TEXT_MAX_LENGTH & 0xff) as u8,
    ];
    expected.extend_from_slice(long_text.as_bytes());

    assert_eq!(buf, expected);
}


#[test]
fn test_serialize_set_screen_power_mode() {
    let msg = json!({
        "mode": 2
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeSetScreenPowerMode, &msg);

    let expected: [u8; 2] = [
        ControlMsgType::ControlMsgTypeSetScreenPowerMode as u8,
        0x02, // SC_SCREEN_POWER_MODE_NORMAL
    ];
    assert_eq!(buf, expected);
}


#[test]
fn test_serialize_uhid_create() {
    let msg = json!({
        "id": 42,
        "reportDescSize": 11,
        "reportDesc": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeUhidCreate, &msg);

    let expected: [u8; 16] = [
        ControlMsgType::ControlMsgTypeUhidCreate as u8,
        0, 42, // id
        0, 11, // size
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, // data
    ];
    assert_eq!(buf, expected);
}

#[test]
fn test_serialize_uhid_input() {
    let msg = json!({
        "id": 42,
        "size": 5,
        "data": [1, 2, 3, 4, 5]
    });

    let buf = gen_ctrl_msg(ControlMsgType::ControlMsgTypeUhidInput, &msg);

    let expected: [u8; 10] = [
        ControlMsgType::ControlMsgTypeUhidInput as u8,
        0, 42, // id
        0, 5, // size
        1, 2, 3, 4, 5, // data
    ];
    assert_eq!(buf, expected);
}

