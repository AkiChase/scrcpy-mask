use std::ops::MulAssign;

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::scrcpy::{
    constant::{self, MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

// TODO 移除这个常量, 后续应该用更合理的
pub const MIN_MOVE_STEP_LENGTH: f32 = 25.; // px

pub const DEFAULT_SWIPE_DURATION: u64 = 25; // ms

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

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn smoothstep(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}

// Timing functions for arc path strategies.
fn cubic_easing_timing(t: f32) -> f32 {
    -1.08 * t * t * t + 1.78 * t * t + 0.30 * t
}
fn ease_out_timing(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
fn ease_in_out_timing(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Selects both path shape and timing curve for single-segment swipe generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SingleSwipeStrategy {
    /// Straight line from start to end, constant speed, no curvature.
    #[default]
    Linear,

    /// Curved path with cubic polynomial timing (fast initial contact,
    /// friction cruise, mild lift-off acceleration). Best for finger
    /// swipes that end with the finger lifting off the screen.
    ArcWithCubicEasing,

    /// Curved path with ease-out timing (fast start, gradual deceleration
    /// to a stop). Best for directional pad initial slides where the
    /// finger comes to rest at the target.
    ArcWithEaseOut,

    /// Curved path with ease-in-out timing (gentle start, speed up, then
    /// decelerate to a stop). Best for directional pad direction changes
    /// where the finger is already touching the screen.
    ArcWithEaseInOut,
}

/// Selects both path shape and timing curve for multi-segment swipe generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiSwipeStrategy {
    Linear,
    ArcWithCubicEasing,
}

impl From<SingleSwipeStrategy> for MultiSwipeStrategy {
    fn from(s: SingleSwipeStrategy) -> Self {
        match s {
            SingleSwipeStrategy::Linear => MultiSwipeStrategy::Linear,
            SingleSwipeStrategy::ArcWithCubicEasing
            | SingleSwipeStrategy::ArcWithEaseOut
            | SingleSwipeStrategy::ArcWithEaseInOut => MultiSwipeStrategy::ArcWithCubicEasing,
        }
    }
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

pub fn default_random_offset() -> f32 {
    10.0
}

// Apply a random offset within a fixed human-touch error range.
pub fn random_offset_vec2(pos: Vec2, offset: Vec2) -> Vec2 {
    if offset == Vec2::ZERO {
        return pos;
    }
    let x_offset = (rand::random::<f32>() * 2.0 - 1.0) * offset.x;
    let y_offset = (rand::random::<f32>() * 2.0 - 1.0) * offset.y;
    pos + Vec2::new(x_offset, y_offset)
}

pub fn build_multisegment_swipe_intermediate_points(
    waypoints: &[Vec2],
    strategy: MultiSwipeStrategy,
    duration_ms: u64,
) -> Vec<SwipePointStep> {
    if waypoints.len() < 2 {
        return vec![];
    }

    match strategy {
        MultiSwipeStrategy::Linear => build_multisegment_linear(waypoints, duration_ms),
        MultiSwipeStrategy::ArcWithCubicEasing => {
            build_multisegment_arc_cubic(waypoints, duration_ms, cubic_easing_timing)
        }
    }
}

// `duration_ms`: 0 = auto-calculate from distance, >0 = user-specified.
pub fn build_single_segment_swipe_intermediate_points(
    start: Vec2,
    end: Vec2,
    strategy: SingleSwipeStrategy,
    duration_ms: u64,
) -> Vec<SwipePointStep> {
    let delta = end - start;
    let distance = delta.length();
    if distance <= 1.0 {
        return vec![];
    }

    let point_count = ((distance / 14.0).round() as usize).clamp(10, 34);
    let steps = point_count.saturating_sub(1);
    if steps == 0 {
        return vec![];
    }

    match strategy {
        SingleSwipeStrategy::Linear => {
            build_linear_path(start, delta, point_count, steps, duration_ms)
        }
        SingleSwipeStrategy::ArcWithCubicEasing => build_arc_path(
            start, delta, distance, point_count, steps, duration_ms, cubic_easing_timing,
        ),
        SingleSwipeStrategy::ArcWithEaseOut => build_arc_path(
            start, delta, distance, point_count, steps, duration_ms, ease_out_timing,
        ),
        SingleSwipeStrategy::ArcWithEaseInOut => build_arc_path(
            start, delta, distance, point_count, steps, duration_ms, ease_in_out_timing,
        ),
    }
}

fn build_linear_path(
    start: Vec2,
    delta: Vec2,
    point_count: usize,
    steps: usize,
    duration_ms: u64,
) -> Vec<SwipePointStep> {
    let per_step_wait = if duration_ms > 0 {
        (duration_ms as f32 / steps as f32).round().max(1.0) as u64
    } else {
        20
    };
    let mut points = Vec::with_capacity(steps);
    for step in 1..=steps {
        let t = step as f32 / point_count as f32;
        points.push(SwipePointStep {
            pos: start + delta * t,
            wait_ms: per_step_wait,
        });
    }
    points
}

fn build_arc_path(
    start: Vec2,
    delta: Vec2,
    distance: f32,
    point_count: usize,
    steps: usize,
    duration_ms: u64,
    timing_fn: fn(f32) -> f32,
) -> Vec<SwipePointStep> {
    let user_duration = duration_ms > 0;
    let travel_ms = if user_duration {
        duration_ms as f32
    } else {
        (distance * 0.55).clamp(110.0, 420.0)
    };

    let direction = delta / distance;
    let normal = Vec2::new(-direction.y, direction.x);

    let arc_scale = (distance * 0.035).clamp(1.5, 10.0);
    let sign = if rand::random::<bool>() { 1.0 } else { -1.0 };
    let start_arc = arc_scale * (0.6 + rand::random::<f32>() * 0.4) * sign;
    let end_arc = -start_arc * (0.85 + rand::random::<f32>() * 0.3);
    let mid_drift = arc_scale * 0.35 * (rand::random::<f32>() * 2.0 - 1.0);

    let mut samples = Vec::with_capacity(point_count + 1);
    let mut prev_pos = start;
    let mut cumulative_length = 0.0;
    samples.push((0.0, start));

    for i in 1..=point_count {
        let t = i as f32 / point_count as f32;
        let progress = smoothstep(t);
        let lateral = bell_curve(t, 0.12, 0.09) * start_arc
            + bell_curve(t, 0.88, 0.09) * end_arc
            + bell_curve(t, 0.5, 0.22) * mid_drift;
        let pos = start + delta * progress + normal * lateral;
        cumulative_length += pos.distance(prev_pos);
        samples.push((cumulative_length, pos));
        prev_pos = pos;
    }

    if cumulative_length <= 0.0 {
        return vec![];
    }

    let mut points = Vec::with_capacity(steps);
    let mut prev_target = start;
    let mut prev_time = 0.0;

    for step in 1..point_count {
        let target_distance = cumulative_length * (step as f32 / point_count as f32);
        let idx = samples.partition_point(|(len, _)| *len < target_distance);
        let (l_len, l_pos) = samples[idx.saturating_sub(1)];
        let (r_len, r_pos) = samples[idx.min(samples.len() - 1)];
        let seg_len = (r_len - l_len).max(0.0001);
        let seg_t = clamp01((target_distance - l_len) / seg_len);
        let pos = l_pos.lerp(r_pos, seg_t);

        let progress = step as f32 / point_count as f32;
        let t = clamp01(progress);
        let time_t = timing_fn(t);
        let target_time = travel_ms * time_t;
        let spacing = pos.distance(prev_target);

        let base_wait = if user_duration {
            (target_time - prev_time).max(0.0)
        } else {
            (target_time - prev_time).max(4.0)
        };
        let spacing_bias = (spacing / 18.0).clamp(0.7, 1.35);
        let wait_ms = if user_duration {
            (base_wait * spacing_bias).round().max(1.0)
        } else {
            (base_wait * spacing_bias).round().clamp(4.0, 28.0)
        } as u64;

        points.push(SwipePointStep { pos, wait_ms });
        prev_target = pos;
        prev_time = target_time;
    }

    if user_duration {
        let raw_total: u64 = points.iter().map(|p| p.wait_ms).sum();
        if raw_total > 0 && raw_total != duration_ms {
            let scale = duration_ms as f64 / raw_total as f64;
            for p in &mut points {
                p.wait_ms = (p.wait_ms as f64 * scale).round().max(1.0) as u64;
            }
        }
    }

    points
}

fn build_multisegment_linear(
    waypoints: &[Vec2],
    duration_ms: u64,
) -> Vec<SwipePointStep> {
    let total_distance: f32 = waypoints
        .windows(2)
        .map(|w| w[0].distance(w[1]))
        .sum();
    if total_distance <= 1.0 {
        return vec![];
    }

    let total_point_count = ((total_distance / 14.0).round() as usize).clamp(10, 80);
    let steps = total_point_count.saturating_sub(1);
    if steps == 0 {
        return vec![];
    }

    let segment_lengths: Vec<f32> = waypoints
        .windows(2)
        .map(|w| w[0].distance(w[1]))
        .collect();
    let mut cumulative_seg = vec![0.0f32];
    for &len in &segment_lengths {
        cumulative_seg.push(cumulative_seg.last().unwrap() + len);
    }

    let per_step_wait = if duration_ms > 0 {
        (duration_ms as f32 / steps as f32).round().max(1.0) as u64
    } else {
        20
    };

    let mut points = Vec::with_capacity(steps);
    for step in 1..=steps {
        let t = step as f32 / total_point_count as f32;
        let target_dist = total_distance * t;
        let seg_idx = cumulative_seg
            .partition_point(|&d| d < target_dist)
            .saturating_sub(1)
            .min(segment_lengths.len().saturating_sub(1));
        let seg_start_dist = cumulative_seg[seg_idx];
        let seg_len = segment_lengths[seg_idx];
        let local_t = if seg_len > 0.0 {
            clamp01((target_dist - seg_start_dist) / seg_len)
        } else {
            0.0
        };
        let pos = waypoints[seg_idx].lerp(waypoints[seg_idx + 1], local_t);
        points.push(SwipePointStep {
            pos,
            wait_ms: per_step_wait,
        });
    }
    points
}

fn build_multisegment_arc_cubic(
    waypoints: &[Vec2],
    duration_ms: u64,
    timing_fn: fn(f32) -> f32,
) -> Vec<SwipePointStep> {
    let n_segments = waypoints.len() - 1;
    let user_duration = duration_ms > 0;
    let sign = if rand::random::<bool>() { 1.0 } else { -1.0 };

    struct SegmentSamples {
        samples: Vec<(f32, Vec2)>,
        total_len: f32,
    }

    let mut all_segments: Vec<SegmentSamples> = Vec::new();
    let mut first_start_arc = 0.0f32;

    for seg_idx in 0..n_segments {
        let start = waypoints[seg_idx];
        let end = waypoints[seg_idx + 1];
        let delta = end - start;
        let distance = delta.length();
        if distance <= 1.0 {
            continue;
        }

        let point_count = ((distance / 14.0).round() as usize).clamp(10, 34);
        let direction = delta / distance;
        let normal = Vec2::new(-direction.y, direction.x);
        let arc_scale = (distance * 0.035).clamp(1.5, 10.0);

        let start_arc = if seg_idx == 0 {
            let val = arc_scale * (0.6 + rand::random::<f32>() * 0.4) * sign;
            first_start_arc = val;
            val
        } else {
            0.0
        };

        let end_arc = if seg_idx == n_segments - 1 {
            let base = if first_start_arc != 0.0 {
                first_start_arc
            } else {
                arc_scale * (0.6 + rand::random::<f32>() * 0.4) * sign
            };
            -base * (0.85 + rand::random::<f32>() * 0.3)
        } else {
            0.0
        };

        let mid_drift = arc_scale * 0.35 * (rand::random::<f32>() * 2.0 - 1.0);

        let mut samples = Vec::with_capacity(point_count + 1);
        let mut prev = start;
        let mut cum_len = 0.0;
        samples.push((0.0, start));

        for i in 1..=point_count {
            let t = i as f32 / point_count as f32;
            let progress = smoothstep(t);
            let lateral = bell_curve(t, 0.12, 0.09) * start_arc
                + bell_curve(t, 0.88, 0.09) * end_arc
                + bell_curve(t, 0.5, 0.22) * mid_drift;
            let pos = start + delta * progress + normal * lateral;
            cum_len += pos.distance(prev);
            samples.push((cum_len, pos));
            prev = pos;
        }

        if cum_len > 0.0 {
            all_segments.push(SegmentSamples {
                samples,
                total_len: cum_len,
            });
        }
    }

    if all_segments.is_empty() {
        return vec![];
    }

    let mut global_samples: Vec<(f32, Vec2)> = Vec::new();
    let mut offset = 0.0f32;
    for seg in &all_segments {
        for &(len, pos) in &seg.samples {
            global_samples.push((offset + len, pos));
        }
        offset += seg.total_len;
    }
    let total_length = offset;

    if total_length <= 0.0 {
        return vec![];
    }

    let total_straight: f32 = waypoints
        .windows(2)
        .map(|w| w[0].distance(w[1]))
        .sum();
    let total_point_count = ((total_straight / 14.0).round() as usize).clamp(10, 80);
    let steps = total_point_count.saturating_sub(1);
    if steps == 0 {
        return vec![];
    }

    let travel_ms = if user_duration {
        duration_ms as f32
    } else {
        (total_straight * 0.55).clamp(110.0, 420.0)
    };

    // Detect sharp corners: (waypoint_pos, severity 0..1)
    let corners: Vec<(Vec2, f32)> = waypoints
        .windows(3)
        .filter_map(|w| {
            let prev_len = w[0].distance(w[1]);
            let next_len = w[1].distance(w[2]);
            if prev_len <= 0.0 || next_len <= 0.0 {
                return None;
            }
            let prev = (w[1] - w[0]) / prev_len;
            let next = (w[2] - w[1]) / next_len;
            let dot = prev.dot(next).clamp(-1.0, 1.0);
            let angle = dot.acos();
            if angle > 0.3 {
                Some((w[1], angle / std::f32::consts::PI))
            } else {
                None
            }
        })
        .collect();

    let mut points = Vec::with_capacity(steps);
    let mut prev_target = waypoints[0];
    let mut prev_time = 0.0f32;

    for step in 1..=steps {
        let target_distance = total_length * (step as f32 / total_point_count as f32);
        let idx = global_samples.partition_point(|(len, _)| *len < target_distance);
        let (l_len, l_pos) = global_samples[idx.saturating_sub(1)];
        let (r_len, r_pos) = global_samples[idx.min(global_samples.len() - 1)];
        let seg_len = (r_len - l_len).max(0.0001);
        let seg_t = clamp01((target_distance - l_len) / seg_len);
        let pos = l_pos.lerp(r_pos, seg_t);

        let progress = step as f32 / total_point_count as f32;
        let t = clamp01(progress);
        let time_t = timing_fn(t);
        let target_time = travel_ms * time_t;
        let spacing = pos.distance(prev_target);

        let base_wait = if user_duration {
            (target_time - prev_time).max(0.0)
        } else {
            (target_time - prev_time).max(4.0)
        };
        let spacing_bias = (spacing / 18.0).clamp(0.7, 1.35);
        let mut wait_ms = if user_duration {
            (base_wait * spacing_bias).round().max(1.0)
        } else {
            (base_wait * spacing_bias).round().clamp(4.0, 28.0)
        } as u64;

        // Slow down near sharp corners: proximity-weighted by turn severity.
        if !corners.is_empty() {
            let mut slowdown = 1.0f32;
            for &(corner_pos, severity) in &corners {
                let dist = pos.distance(corner_pos);
                let radius = 36.0;
                if dist < radius {
                    let proximity = (1.0 - dist / radius).powi(2);
                    slowdown += severity * proximity * 3.0;
                }
            }
            wait_ms = (wait_ms as f32 * slowdown).round().max(1.0) as u64;
        }

        points.push(SwipePointStep { pos, wait_ms });
        prev_target = pos;
        prev_time = target_time;
    }

    if user_duration {
        let raw_total: u64 = points.iter().map(|p| p.wait_ms).sum();
        if raw_total > 0 && raw_total != duration_ms {
            let scale = duration_ms as f64 / raw_total as f64;
            for p in &mut points {
                p.wait_ms = (p.wait_ms as f64 * scale).round().max(1.0) as u64;
            }
        }
    }

    points
}
