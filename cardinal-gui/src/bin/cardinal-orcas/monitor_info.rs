#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorTraversal {
    Sequential, // step through monitors in a fixed order
    Geometric,  // wrap to monitor in geometric direction, or farthest if none
}

// Only one static for monitor rects
use display_info::DisplayInfo;
use egui::{Pos2, Rect};
use once_cell::sync::Lazy;
use std::sync::Mutex;
pub static MONITOR_RECTS: Lazy<Mutex<Vec<Rect>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Clone, Debug)]
pub struct MonitorGrid {
    pub rects: Vec<Rect>,
    // Optionally, you could add adjacency info here for geometric traversal
}

pub fn fill_monitor_rects() {
    let mut rects = Vec::new();
    for display in DisplayInfo::all().unwrap_or_default() {
        let min = Pos2::new(display.x as f32, display.y as f32);
        let max = Pos2::new(
            (display.x + display.width as i32) as f32,
            (display.y + display.height as i32) as f32,
        );
        rects.push(Rect::from_min_max(min, max));
    }
    *MONITOR_RECTS.lock().unwrap() = rects;
}

/// Returns (new_position, new_monitor_idx) after wrapping, or None if no wrap.
pub fn wrap_sequential(
    pos: Pos2,
    current_idx: usize,
    dx: f32,
    dy: f32,
    rects: &[Rect],
) -> Option<(Pos2, usize)> {
    let n = rects.len();
    if n == 0 {
        return None;
    }
    let rect = rects[current_idx];
    let mut new_idx = current_idx;
    let mut new_pos = pos;
    let mut wrapped = false;
    if dx < 0.0 && pos.x < rect.left() {
        new_idx = (current_idx + n - 1) % n;
        let rel_y = (pos.y - rect.top()) / rect.height();
        let new_rect = rects[new_idx];
        new_pos.x = new_rect.right();
        new_pos.y = new_rect.top() + rel_y * new_rect.height();
        wrapped = true;
    } else if dx > 0.0 && pos.x > rect.right() {
        new_idx = (current_idx + 1) % n;
        let rel_y = (pos.y - rect.top()) / rect.height();
        let new_rect = rects[new_idx];
        new_pos.x = new_rect.left();
        new_pos.y = new_rect.top() + rel_y * new_rect.height();
        wrapped = true;
    } else if dy < 0.0 && pos.y < rect.top() {
        new_idx = (current_idx + n - 1) % n;
        let rel_x = (pos.x - rect.left()) / rect.width();
        let new_rect = rects[new_idx];
        new_pos.y = new_rect.bottom();
        new_pos.x = new_rect.left() + rel_x * new_rect.width();
        wrapped = true;
    } else if dy > 0.0 && pos.y > rect.bottom() {
        new_idx = (current_idx + 1) % n;
        let rel_x = (pos.x - rect.left()) / rect.width();
        let new_rect = rects[new_idx];
        new_pos.y = new_rect.top();
        new_pos.x = new_rect.left() + rel_x * new_rect.width();
        wrapped = true;
    }
    if wrapped {
        Some((new_pos, new_idx))
    } else {
        None
    }
}

/// Returns (new_position, new_monitor_idx) after geometric wrap, or None if no wrap.
pub fn wrap_geometric(
    pos: Pos2,
    dx: f32,
    dy: f32,
    rects: &[Rect],
) -> Option<(Pos2, usize)> {
    // Find the farthest monitor in the direction, or the one that overlaps in that direction
    if rects.is_empty() {
        return None;
    }
    let mut best_idx = None;
    let mut best_score = f32::NEG_INFINITY;
    for (i, r) in rects.iter().enumerate() {
        let score = if dx.abs() > dy.abs() {
            // Horizontal wrap
            if dx < 0.0 && pos.x < r.left() {
                (r.right() - pos.x).abs()
            } else if dx > 0.0 && pos.x > r.right() {
                (pos.x - r.left()).abs()
            } else {
                -1.0
            }
        } else {
            // Vertical wrap
            if dy < 0.0 && pos.y < r.top() {
                (r.bottom() - pos.y).abs()
            } else if dy > 0.0 && pos.y > r.bottom() {
                (pos.y - r.top()).abs()
            } else {
                -1.0
            }
        };
        if score > best_score {
            best_score = score;
            best_idx = Some(i);
        }
    }
    if let Some(idx) = best_idx {
        let r = rects[idx];
        let rel = Pos2::new(
            (pos.x - r.left()) / r.width(),
            (pos.y - r.top()) / r.height(),
        );
        let new_pos = Pos2::new(
            r.left() + rel.x * r.width(),
            r.top() + rel.y * r.height(),
        );
        Some((new_pos, idx))
    } else {
        None
    }
}
