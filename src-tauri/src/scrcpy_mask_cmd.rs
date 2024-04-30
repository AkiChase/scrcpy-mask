use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

use crate::{
    binary,
    control_msg::{gen_inject_key_ctrl_msg, gen_inject_touch_ctrl_msg, ControlMsgType},
};

pub async fn handle_sm_cmd(
    cmd_type: ScrcpyMaskCmdType,
    payload: &serde_json::Value,
    writer: &mut OwnedWriteHalf,
) {
    match cmd_type {
        ScrcpyMaskCmdType::SendKey => {
            let ctrl_msg_type = ControlMsgType::ControlMsgTypeInjectKeycode as u8;
            let keycode = payload["keycode"].as_u64().unwrap() as u32;
            let metastate = match payload.get("metastate") {
                Some(metastate) => metastate.as_u64().unwrap() as u32,
                None => 0, // AMETA_NONE
            };
            match payload["action"].as_u64().unwrap() {
                // default
                0 => {
                    // down
                    let buf = gen_inject_key_ctrl_msg(
                        ctrl_msg_type,
                        0, // AKEY_EVENT_ACTION_DOWN
                        keycode,
                        0,
                        metastate,
                    );
                    writer.write_all(&buf).await.unwrap();
                    writer.flush().await.unwrap();
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    // up
                    let buf = gen_inject_key_ctrl_msg(
                        ctrl_msg_type,
                        0, // AKEY_EVENT_ACTION_DOWN
                        keycode,
                        0,
                        metastate,
                    );
                    writer.write_all(&buf).await.unwrap();
                    writer.flush().await.unwrap();
                }
                // down
                1 => {
                    let buf = gen_inject_key_ctrl_msg(
                        ctrl_msg_type,
                        1, // AKEY_EVENT_ACTION_UP
                        keycode,
                        0,
                        metastate,
                    );
                    writer.write_all(&buf).await.unwrap();
                    writer.flush().await.unwrap();
                }
                // up
                2 => {
                    let buf = gen_inject_key_ctrl_msg(
                        ctrl_msg_type,
                        1, // AKEY_EVENT_ACTION_UP
                        keycode,
                        0,
                        metastate,
                    );
                    writer.write_all(&buf).await.unwrap();
                    writer.flush().await.unwrap();
                }
                _ => {}
            };
        }
        ScrcpyMaskCmdType::Touch => {
            let ctrl_msg_type = ControlMsgType::ControlMsgTypeInjectTouchEvent as u8;
            let pointer_id = payload["pointerId"].as_u64().unwrap();
            let w = payload["screen"]["w"].as_u64().unwrap() as u16;
            let h = payload["screen"]["h"].as_u64().unwrap() as u16;
            let x = payload["pos"]["x"].as_i64().unwrap() as i32;
            let y = payload["pos"]["y"].as_i64().unwrap() as i32;
            let time = payload["time"].as_u64().unwrap();
            match payload["action"].as_u64().unwrap() {
                // default
                0 => {
                    // down
                    touch(ctrl_msg_type, pointer_id, x, y, w, h, 0, writer).await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(time)).await;
                    // up
                    touch(ctrl_msg_type, pointer_id, x, y, w, h, 1, writer).await;
                }
                // down
                1 => {
                    touch(ctrl_msg_type, pointer_id, x, y, w, h, 0, writer).await;
                }
                // up
                2 => {
                    touch(ctrl_msg_type, pointer_id, x, y, w, h, 1, writer).await;
                }
                // move
                3 => {
                    touch(ctrl_msg_type, pointer_id, x, y, w, h, 2, writer).await;
                }
                _ => {}
            }
        }
        ScrcpyMaskCmdType::Swipe => {
            let ctrl_msg_type = ControlMsgType::ControlMsgTypeInjectTouchEvent as u8;
            let pointer_id = payload["pointerId"].as_u64().unwrap();
            let w = payload["screen"]["w"].as_u64().unwrap() as u16;
            let h = payload["screen"]["h"].as_u64().unwrap() as u16;
            let pos_arr = payload["pos"].as_array().unwrap();
            let pos_arr: Vec<(i32, i32)> = pos_arr
                .iter()
                .map(|pos| {
                    (
                        pos["x"].as_i64().unwrap() as i32,
                        pos["y"].as_i64().unwrap() as i32,
                    )
                })
                .collect();
            let interval_between_pos = payload["intervalBetweenPos"].as_u64().unwrap();
            match payload["action"].as_u64().unwrap() {
                // default
                0 => {
                    swipe(
                        ctrl_msg_type,
                        pointer_id,
                        w,
                        h,
                        pos_arr,
                        interval_between_pos,
                        writer,
                        true,
                        true,
                    )
                    .await;
                }
                // no up
                1 => {
                    swipe(
                        ctrl_msg_type,
                        pointer_id,
                        w,
                        h,
                        pos_arr,
                        interval_between_pos,
                        writer,
                        true,
                        false,
                    )
                    .await;
                }
                // no down
                2 => {
                    swipe(
                        ctrl_msg_type,
                        pointer_id,
                        w,
                        h,
                        pos_arr,
                        interval_between_pos,
                        writer,
                        false,
                        true,
                    )
                    .await;
                }
                _ => {}
            };
        }
        ScrcpyMaskCmdType::Shutdown => {}
    }
}

pub async fn touch(
    ctrl_msg_type: u8,
    pointer_id: u64,
    x: i32,
    y: i32,
    w: u16,
    h: u16,
    action: u8, // 0: down, 1: up, 2: move
    writer: &mut OwnedWriteHalf,
) {
    let pressure = binary::float_to_u16fp(0.8);
    let action_button: u32 = 1;
    let buttons: u32 = 1;
    let buf = gen_inject_touch_ctrl_msg(
        ctrl_msg_type,
        action,
        pointer_id,
        x,
        y,
        w,
        h,
        pressure,
        action_button,
        buttons,
    );
    writer.write_all(&buf).await.unwrap();
    writer.flush().await.unwrap();
}

/// Determine the number of segments based on the distance between two points
fn get_divide_num(x1: i32, y1: i32, x2: i32, y2: i32, segment_length: i32) -> i32 {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let d = (dx.pow(2) + dy.pow(2)) as f64;
    let d = d.sqrt();
    let divide_num = (d / segment_length as f64).ceil() as i32;
    divide_num
}

pub async fn swipe(
    ctrl_msg_type: u8,
    pointer_id: u64,
    w: u16,
    h: u16,
    pos_arr: Vec<(i32, i32)>,
    interval_between_pos: u64,
    writer: &mut OwnedWriteHalf,
    down_flag: bool,
    up_flag: bool,
) {
    // down
    if down_flag {
        touch(
            ctrl_msg_type,
            pointer_id,
            pos_arr[0].0,
            pos_arr[0].1,
            w,
            h,
            0,
            writer,
        )
        .await;
    }

    // move
    let mut cur_index = 1;
    while cur_index < pos_arr.len() {
        let (x, y) = pos_arr[cur_index];
        let (prev_x, prev_y) = pos_arr[cur_index - 1];
        // divide it into several segments
        let segment_length = 100;
        let divide_num = get_divide_num(prev_x, prev_y, x, y, segment_length);
        let dx = (x - prev_x) / divide_num;
        let dy = (y - prev_y) / divide_num;
        let d_interval = interval_between_pos / (divide_num as u64);

        for i in 1..divide_num + 1 {
            let nx = prev_x + dx * i;
            let ny = prev_y + dy * i;
            touch(ctrl_msg_type, pointer_id, nx, ny, w, h, 2, writer).await;
            if d_interval > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(d_interval)).await;
            }
        }

        cur_index += 1;
    }

    // up
    if up_flag {
        touch(
            ctrl_msg_type,
            pointer_id,
            pos_arr[pos_arr.len() - 1].0,
            pos_arr[pos_arr.len() - 1].1,
            w,
            h,
            1,
            writer,
        )
        .await;
    }
}

#[derive(Debug)]
pub enum ScrcpyMaskCmdType {
    SendKey,
    Touch,
    Swipe,
    Shutdown,
}

impl ScrcpyMaskCmdType {
    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            15 => Some(Self::SendKey),
            16 => Some(Self::Touch),
            17 => Some(Self::Swipe),
            18 => Some(Self::Shutdown),
            _ => None,
        }
    }
}
