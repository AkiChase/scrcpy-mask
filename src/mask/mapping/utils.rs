use std::ops::MulAssign;

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::scrcpy::{
    constant::{self, MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

pub const MIN_MOVE_STEP_LENGTH: f32 = 25.; // px
pub const MIN_MOVE_STEP_INTERVAL: u64 = 25; // ms

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<Size> for Vec2 {
    fn from(size: Size) -> Self {
        Vec2::new(size.width as f32, size.height as f32)
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size { width, height }
    }
}

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Size {
            width: vec.x as u32,
            height: vec.y as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Position { x, y }
    }
}

impl From<Position> for Vec2 {
    fn from(pos: Position) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}

impl MulAssign<Vec2> for Position {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x = ((self.x as f32) * rhs.x).round() as i32;
        self.y = ((self.y as f32) * rhs.y).round() as i32;
    }
}

pub struct ControlMsgHelper;

impl ControlMsgHelper {
    pub fn send_touch(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        action: MotionEventAction,
        pointer_id: u64,
        size: Vec2,
        pos: Vec2,
    ) {
        cs_tx
            .send(ScrcpyControlMsg::InjectTouchEvent {
                action,
                pointer_id,
                x: pos.x as i32,
                y: pos.y as i32,
                w: size.x as u16,
                h: size.y as u16,
                pressure: half::f16::from_f32_const(1.0),
                action_button: MotionEventButtons::PRIMARY,
                buttons: MotionEventButtons::PRIMARY,
            })
            .unwrap();
    }

    pub fn send_keycode(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        keycode: constant::Keycode,
        metastate: constant::MetaState,
        down: bool,
        repeat: u32,
    ) {
        let action = if down {
            constant::KeyEventAction::Down
        } else {
            constant::KeyEventAction::Up
        };
        cs_tx
            .send(ScrcpyControlMsg::InjectKeycode {
                action,
                keycode,
                repeat,
                metastate,
            })
            .unwrap();
    }

    pub fn set_clipboard(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        sequence: Option<u64>,
        text: String,
        paste: bool,
    ) {
        let sequence = sequence.unwrap_or_else(|| rand::random());
        cs_tx
            .send(ScrcpyControlMsg::SetClipboard {
                sequence,
                paste,
                text,
            })
            .unwrap();
    }
}

pub fn ease_sigmoid_like(t: f32) -> f32 {
    1.0 / (1.0 + (-12.0 * (t - 0.5)).exp())
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn smoothstep(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}

// Gaussian-like weight for local arc shaping.
fn bell_curve(t: f32, center: f32, width: f32) -> f32 {
    let normalized = (t - center) / width.max(0.001);
    (-0.5 * normalized * normalized).exp()
}

#[derive(Debug, Clone, Copy)]
pub struct SwipePointStep {
    pub pos: Vec2,
    pub wait_ms: u64,
}

// Apply a random offset within the max x/y range.
pub fn random_offset_vec2(pos: Vec2, max_offset: Vec2) -> Vec2 {
    let x_offset = (rand::random::<f32>() * 2.0 - 1.0) * max_offset.x.abs();
    let y_offset = (rand::random::<f32>() * 2.0 - 1.0) * max_offset.y.abs();
    pos + Vec2::new(x_offset, y_offset)
}

// Build a human-like swipe path between start and end, excluding both endpoints.
pub fn build_swipe_intermediate_points(start: Vec2, end: Vec2) -> Vec<SwipePointStep> {
    let delta = end - start;
    let distance = delta.length();
    if distance <= 1.0 {
        return vec![];
    }

    let direction = delta / distance;
    let normal = Vec2::new(-direction.y, direction.x);

    // Derive gesture duration and point density from travel distance.
    let travel_ms = (distance * 0.55).clamp(110.0, 420.0);
    let point_count = ((distance / 14.0).round() as usize).clamp(10, 34);
    if point_count <= 1 {
        return vec![];
    }

    // Add a small random arc near the start and cancel it near the end.
    let start_arc_sign = if rand::random::<bool>() { 1.0 } else { -1.0 };
    let arc_scale = (distance * 0.035).clamp(1.5, 10.0);
    let start_arc_strength = arc_scale * (0.6 + rand::random::<f32>() * 0.4) * start_arc_sign;
    let end_arc_strength = -start_arc_strength * (0.85 + rand::random::<f32>() * 0.3);
    let mid_drift = arc_scale * 0.35 * (rand::random::<f32>() * 2.0 - 1.0);

    let mut samples = Vec::with_capacity(point_count + 1);
    let mut prev_pos = start;
    let mut cumulative_length = 0.0;
    samples.push((0.0, start));

    for i in 1..=point_count {
        let t = i as f32 / point_count as f32;
        // Accelerate early and decelerate late.
        let progress = smoothstep(t);
        let base = start + delta * progress;

        let start_arc = bell_curve(t, 0.12, 0.09) * start_arc_strength;
        let end_arc = bell_curve(t, 0.88, 0.09) * end_arc_strength;
        let drift = bell_curve(t, 0.5, 0.22) * mid_drift;
        let lateral_offset = start_arc + end_arc + drift;

        let pos = base + normal * lateral_offset;
        cumulative_length += pos.distance(prev_pos);
        samples.push((cumulative_length, pos));
        prev_pos = pos;
    }

    if cumulative_length <= 0.0 {
        return vec![];
    }

    let mut points = Vec::with_capacity(point_count.saturating_sub(1));
    let mut prev_target = start;
    let mut prev_time = 0.0;

    for step in 1..point_count {
        // Resample by arc length so point spacing follows the curved path.
        let target_distance = cumulative_length * (step as f32 / point_count as f32);
        let sample_index = samples.partition_point(|(length, _)| *length < target_distance);
        let (left_length, left_pos) = samples[sample_index.saturating_sub(1)];
        let (right_length, right_pos) = samples[sample_index.min(samples.len() - 1)];
        let segment_length = (right_length - left_length).max(0.0001);
        let segment_t = clamp01((target_distance - left_length) / segment_length);
        let pos = left_pos.lerp(right_pos, segment_t);

        let progress = step as f32 / point_count as f32;
        let time_t = smoothstep(progress);
        let target_time = travel_ms * time_t;
        let spacing = pos.distance(prev_target);

        // Wait time follows both eased timing and local point spacing.
        let base_wait = (target_time - prev_time).max(4.0);
        let spacing_bias = (spacing / 18.0).clamp(0.7, 1.35);
        let wait_ms = (base_wait * spacing_bias).round().clamp(4.0, 28.0) as u64;

        points.push(SwipePointStep { pos, wait_ms });
        prev_target = pos;
        prev_time = target_time;
    }

    points
}
