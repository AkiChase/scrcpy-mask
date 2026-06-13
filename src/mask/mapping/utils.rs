use std::{
    ops::MulAssign,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use crate::tokio_tasks::TokioTasksRuntime;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use tokio::{sync::broadcast, time::sleep};

use crate::scrcpy::{
    constant::{self, MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

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
        if let Err(e) = cs_tx.send(ScrcpyControlMsg::InjectTouchEvent {
            action,
            pointer_id,
            x: pos.x as i32,
            y: pos.y as i32,
            w: size.x as u16,
            h: size.y as u16,
            pressure: half::f16::from_f32_const(1.0),
            action_button: MotionEventButtons::PRIMARY,
            buttons: MotionEventButtons::PRIMARY,
        }) {
            log::warn!("[Mapping] send_touch failed: {}", e);
        }
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
        if let Err(e) = cs_tx.send(ScrcpyControlMsg::InjectKeycode {
            action,
            keycode,
            repeat,
            metastate,
        }) {
            log::warn!("[Mapping] send_keycode failed: {}", e);
        }
    }

    pub fn set_clipboard(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        sequence: Option<u64>,
        text: String,
        paste: bool,
    ) {
        let sequence = sequence.unwrap_or_else(|| rand::random());
        if let Err(e) = cs_tx.send(ScrcpyControlMsg::SetClipboard {
            sequence,
            paste,
            text,
        }) {
            log::warn!("[Mapping] set_clipboard failed: {}", e);
        }
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

/// Spawns a background task that performs micro-jitter during `initial_duration_ms`,
/// then swipes from `start` to `target` over `swipe_duration_ms`.
/// Returns a flag set to true when both phases complete.
pub fn spawn_initial_swipe(
    runtime: &TokioTasksRuntime,
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    pointer_id: u64,
    original_size: Vec2,
    start: Vec2,
    target: Vec2,
    initial_duration_ms: u64,
    swipe_duration_ms: u64,
    swipe_strategy: SingleSwipeStrategy,
) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();
    let cs_tx = cs_tx.clone();

    runtime.spawn_background_task(move |_ctx| async move {
        if initial_duration_ms > 0 {
            let jitter_count = ((initial_duration_ms as f32 / 16.0).round() as usize).max(2);
            let jitter_interval = initial_duration_ms / jitter_count as u64;
            for _ in 0..jitter_count {
                let offset = Vec2::new(
                    (rand::random::<f32>() - 0.5) * 4.0,
                    (rand::random::<f32>() - 0.5) * 4.0,
                );
                ControlMsgHelper::send_touch(
                    &cs_tx,
                    MotionEventAction::Move,
                    pointer_id,
                    original_size,
                    start + offset,
                );
                sleep(Duration::from_millis(jitter_interval)).await;
            }
            // return to start before the swipe
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Move,
                pointer_id,
                original_size,
                start,
            );
        }

        let points = build_single_segment_swipe_intermediate_points(
            start,
            target,
            swipe_strategy,
            swipe_duration_ms,
        );
        for point in points {
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Move,
                pointer_id,
                original_size,
                point.pos,
            );
            sleep(Duration::from_millis(point.wait_ms)).await;
        }

        done_clone.store(true, Ordering::Relaxed);
    });

    done
}

pub fn default_random_offset() -> f32 {
    10.0
}

pub fn default_random_distance_min_scale() -> f32 {
    0.9
}

pub fn default_random_distance_max_scale() -> f32 {
    1.1
}

pub fn default_jitter_offset() -> f32 {
    5.0
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

    let point_count = ((distance / 14.0).round() as usize).clamp(5, 25);
    let steps = point_count.saturating_sub(1);
    if steps == 0 {
        return vec![];
    }

    match strategy {
        SingleSwipeStrategy::Linear => {
            build_linear_path(start, delta, point_count, steps, duration_ms)
        }
        SingleSwipeStrategy::ArcWithCubicEasing => build_arc_path(
            start,
            delta,
            distance,
            point_count,
            steps,
            duration_ms,
            cubic_easing_timing,
        ),
        SingleSwipeStrategy::ArcWithEaseOut => build_arc_path(
            start,
            delta,
            distance,
            point_count,
            steps,
            duration_ms,
            ease_out_timing,
        ),
        SingleSwipeStrategy::ArcWithEaseInOut => build_arc_path(
            start,
            delta,
            distance,
            point_count,
            steps,
            duration_ms,
            ease_in_out_timing,
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
    let per_step_wait: u64;
    let rem: usize;
    if duration_ms > 0 {
        let d = duration_ms as usize;
        if d < steps {
            per_step_wait = 1;
            rem = 0;
        } else {
            per_step_wait = (d / steps) as u64;
            rem = d % steps;
        }
    } else {
        per_step_wait = 20;
        rem = 0;
    };
    let mut points = Vec::with_capacity(steps);
    for step in 1..=steps {
        let t = step as f32 / point_count as f32;
        let wait_ms = if step <= rem {
            per_step_wait + 1
        } else {
            per_step_wait
        };
        points.push(SwipePointStep {
            pos: start + delta * t,
            wait_ms,
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
    // buffer pos/weight for user_duration path so we can normalize after the loop
    let mut user_steps: Vec<(Vec2, f32)> = Vec::with_capacity(steps);

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

        if user_duration {
            user_steps.push((pos, base_wait * spacing_bias));
        } else {
            let wait_ms = (base_wait * spacing_bias).round().clamp(4.0, 28.0) as u64;
            points.push(SwipePointStep { pos, wait_ms });
        }

        prev_target = pos;
        prev_time = target_time;
    }

    if user_duration {
        let total_weight: f32 = user_steps.iter().map(|(_, w)| w).sum();
        if total_weight > 0.0 {
            for (pos, weight) in &user_steps {
                let wait_ms = (weight / total_weight * travel_ms).round().max(1.0) as u64;
                points.push(SwipePointStep { pos: *pos, wait_ms });
            }
            // correct integer rounding overshoot/undershoot
            let total: u64 = points.iter().map(|p| p.wait_ms).sum();
            if total > duration_ms {
                let mut excess = total - duration_ms;
                for p in points.iter_mut().rev() {
                    if excess == 0 {
                        break;
                    }
                    let remove = (p.wait_ms - 1).min(excess);
                    p.wait_ms -= remove;
                    excess -= remove;
                }
            } else if total < duration_ms {
                let mut shortfall = duration_ms - total;
                for p in points.iter_mut().rev() {
                    if shortfall == 0 {
                        break;
                    }
                    p.wait_ms += 1;
                    shortfall -= 1;
                }
            }
        }
    }

    points
}

fn build_multisegment_linear(waypoints: &[Vec2], duration_ms: u64) -> Vec<SwipePointStep> {
    let total_distance: f32 = waypoints.windows(2).map(|w| w[0].distance(w[1])).sum();
    if total_distance <= 1.0 {
        return vec![];
    }

    let total_point_count = ((total_distance / 14.0).round() as usize).clamp(10, 80);
    let steps = total_point_count.saturating_sub(1);
    if steps == 0 {
        return vec![];
    }

    let segment_lengths: Vec<f32> = waypoints.windows(2).map(|w| w[0].distance(w[1])).collect();
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

        let point_count = ((distance / 14.0).round() as usize).clamp(5, 25);
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

    let total_straight: f32 = waypoints.windows(2).map(|w| w[0].distance(w[1])).sum();
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
        // correct integer rounding overshoot/undershoot
        let total: u64 = points.iter().map(|p| p.wait_ms).sum();
        if total > duration_ms {
            let mut excess = total - duration_ms;
            for p in points.iter_mut().rev() {
                if excess == 0 {
                    break;
                }
                let remove = (p.wait_ms - 1).min(excess);
                p.wait_ms -= remove;
                excess -= remove;
            }
        } else if total < duration_ms {
            let mut shortfall = duration_ms - total;
            for p in points.iter_mut().rev() {
                if shortfall == 0 {
                    break;
                }
                p.wait_ms += 1;
                shortfall -= 1;
            }
        }
    }

    points
}

pub fn anchor_random_offset(max_offset_x: f32, max_offset_y: f32) -> Vec2 {
    if max_offset_x >= max_offset_y {
        let x_rand = 10.0_f32.min(max_offset_x * 0.1);
        let ratio = if max_offset_x > 0.0 {
            max_offset_y / max_offset_x
        } else {
            1.0
        };
        Vec2::new(x_rand, x_rand * ratio)
    } else {
        let y_rand = 10.0_f32.min(max_offset_y * 0.1);
        let ratio = if max_offset_y > 0.0 {
            max_offset_x / max_offset_y
        } else {
            1.0
        };
        Vec2::new(y_rand * ratio, y_rand)
    }
}

pub fn micro_jitter(offset: Vec2) -> Vec2 {
    if offset == Vec2::ZERO {
        return Vec2::ZERO;
    }
    let s = 0.15;
    let x = (rand::random::<f32>() * 2.0 - 1.0) * offset.x * s;
    let y = (rand::random::<f32>() * 2.0 - 1.0) * offset.y * s;
    Vec2::new(x, y)
}

pub fn next_jitter_deadline() -> Instant {
    Instant::now() + Duration::from_millis(80 + rand::random::<u64>() % 41)
}

/// Handle a direction change with randomized swipe movement.
/// Returns true if a move was sent (either via background task or directly).
pub fn handle_direction_move_randomized(
    old_state: Vec2,
    new_state: Vec2,
    target_base: Vec2,
    current_jitter: &mut Vec2,
    next_jitter_at: &mut Instant,
    move_gen: &Arc<AtomicU64>,
    pointer_id: u64,
    original_size: Vec2,
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
    runtime: &TokioTasksRuntime,
    strategy: SingleSwipeStrategy,
) {
    let cur = target_base + old_state + *current_jitter;
    let target = target_base + new_state;
    let dist = cur.distance(target);

    if dist > 2.0 {
        let points = build_single_segment_swipe_intermediate_points(
            cur,
            target,
            strategy,
            DEFAULT_SWIPE_DURATION,
        );
        let cs_tx = cs_tx.clone();
        let expected_gen = move_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let move_gen = move_gen.clone();
        runtime.spawn_background_task(move |_ctx| async move {
            for point in points {
                if move_gen.load(Ordering::Relaxed) != expected_gen {
                    return;
                }
                ControlMsgHelper::send_touch(
                    &cs_tx,
                    MotionEventAction::Move,
                    pointer_id,
                    original_size,
                    point.pos,
                );
                sleep(Duration::from_millis(point.wait_ms)).await;
            }
        });
    } else {
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Move,
            pointer_id,
            original_size,
            target,
        );
    }
    *current_jitter = Vec2::ZERO;
    *next_jitter_at = next_jitter_deadline();
}

/// Apply micro-jitter to the current touch position for randomization.
pub fn handle_direction_jitter(
    state: Vec2,
    target_base: Vec2,
    current_jitter: &mut Vec2,
    next_jitter_at: &mut Instant,
    random_offset: Vec2,
    pointer_id: u64,
    original_size: Vec2,
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
) {
    let jitter = micro_jitter(random_offset);
    ControlMsgHelper::send_touch(
        cs_tx,
        MotionEventAction::Move,
        pointer_id,
        original_size,
        target_base + state + jitter,
    );
    *current_jitter = jitter;
    *next_jitter_at = next_jitter_deadline();
}

/// Apply jitter as a short synchronous move path without blocking direction changes.
pub fn handle_direction_jitter_path(
    state: Vec2,
    target_base: Vec2,
    current_jitter: &mut Vec2,
    next_jitter_at: &mut Instant,
    jitter_offset: Vec2,
    pointer_id: u64,
    original_size: Vec2,
    cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
) {
    if jitter_offset == Vec2::ZERO {
        *current_jitter = Vec2::ZERO;
        *next_jitter_at = next_jitter_deadline();
        return;
    }

    let next_jitter = Vec2::new(
        (rand::random::<f32>() * 2.0 - 1.0) * jitter_offset.x,
        (rand::random::<f32>() * 2.0 - 1.0) * jitter_offset.y,
    );
    let start = target_base + state + *current_jitter;
    let target = target_base + state + next_jitter;
    let steps = 2 + (rand::random::<u8>() % 3) as usize;

    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        ControlMsgHelper::send_touch(
            cs_tx,
            MotionEventAction::Move,
            pointer_id,
            original_size,
            start.lerp(target, t),
        );
    }

    *current_jitter = next_jitter;
    *next_jitter_at = next_jitter_deadline();
}
