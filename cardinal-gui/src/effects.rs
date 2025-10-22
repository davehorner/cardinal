/// Map effect name (case-insensitive) to effect index. Returns Some(index) or None if not found.
pub fn effect_name_to_index(name: &str) -> Option<usize> {
    let name = name.to_ascii_lowercase();
    (0..EFFECT_COUNT).find(|&idx| effect_name(idx).to_ascii_lowercase() == name)
}
// Effect module for cardinal-gui
// Provides effect count, names, and application logic

use egui::Color32;

pub const EFFECT_COUNT: usize = 38;

pub fn effect_name(index: usize) -> &'static str {
    match index {
        0 => "Normal",
        1 => "Invert",
        2 => "Grayscale",
        3 => "Plasma (horizontal)",
        4 => "Plasma (vertical)",
        5 => "Plasma (circle)",
        6 => "Plasma (diagonal)",
        7 => "Rainbow (horizontal)",
        8 => "Rainbow (vertical)",
        9 => "Rainbow (circle)",
        10 => "Rainbow (diagonal)",
        11 => "Waves (horizontal)",
        12 => "Waves (vertical)",
        13 => "Waves (circle)",
        14 => "Waves (diagonal)",
        15 => "Noise (horizontal)",
        16 => "Noise (vertical)",
        17 => "Noise (circle)",
        18 => "Noise (diagonal)",
        // Inverted input versions
        19 => "Normal (inverted input)",
        20 => "Invert (inverted input)",
        21 => "Grayscale (inverted input)",
        22 => "Plasma (horizontal, inverted input)",
        23 => "Plasma (vertical, inverted input)",
        24 => "Plasma (circle, inverted input)",
        25 => "Plasma (diagonal, inverted input)",
        26 => "Rainbow (horizontal, inverted input)",
        27 => "Rainbow (vertical, inverted input)",
        28 => "Rainbow (circle, inverted input)",
        29 => "Rainbow (diagonal, inverted input)",
        30 => "Waves (horizontal, inverted input)",
        31 => "Waves (vertical, inverted input)",
        32 => "Waves (circle, inverted input)",
        33 => "Waves (diagonal, inverted input)",
        34 => "Noise (horizontal, inverted input)",
        35 => "Noise (vertical, inverted input)",
        36 => "Noise (circle, inverted input)",
        37 => "Noise (diagonal, inverted input)",
        _ => "Unknown",
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_effect(
    index: usize,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    t: f32,
    zebra_offset: usize,
) -> Color32 {
    let fx = x as f32;
    let fy = y as f32;
    let cx = (w as f32) / 2.0;
    let cy = (h as f32) / 2.0;
    let dist = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
    let diag = (fx + fy) * 0.5;
    // Debug print: show which effect is being applied for the first pixel each frame
    // t is now passed as-is; dynamic scaling is handled in apply_effect_blend for FullDyn
    if index >= 19 {
        // Only affect nearly black: threshold 32
        if r <= 32 && g <= 32 && b <= 32 {
            // Use original color, but apply the effect as if the input was white (255,255,255)
            let (r, g, b) = (255, 255, 255);
            return match index % 19 {
                0 => Color32::from_rgba_unmultiplied(r, g, b, a),
                1 => Color32::from_rgba_unmultiplied(255 - r, 255 - g, 255 - b, a),
                2 => {
                    let avg = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                    Color32::from_rgba_unmultiplied(avg, avg, avg, a)
                }
                3 => {
                    let v = (fx * 0.08 + t * 0.12).sin() + (fy * 0.08 + t * 0.15).cos();
                    let v = v * 0.5;
                    let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                    let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                    let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                }
                4 => {
                    let v = (fy * 0.08 + t * 0.12).sin() + (fx * 0.08 + t * 0.15).cos();
                    let v = v * 0.5;
                    let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                    let g = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                    let b = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                }
                5 => {
                    let theta = (fy - cy).atan2(fx - cx);
                    let v = (dist * 0.15 + t * 0.12).sin() + (theta * 6.0 + t * 0.2).cos();
                    let v = v * 0.5;
                    let r = ((v * 127.0 + 128.0) as u8)
                        .saturating_add((theta.sin() * 127.0 + 128.0) as u8);
                    let g = ((v * 127.0 + 128.0) as u8)
                        .saturating_add((theta.cos() * 127.0 + 128.0) as u8);
                    let b = ((v * 127.0 + 128.0) as u8)
                        .saturating_add((dist.sin() * 127.0 + 128.0) as u8);
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                }
                6 => {
                    let v = (diag * 0.08 + t * 0.12).sin() + (t * 0.15).cos();
                    let v = v * 0.5;
                    let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                    let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                    let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                }
                7 => {
                    let r = (fx * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
                    let g = (fy * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
                    let b = ((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
                }
                8 => {
                    let r = (fy * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
                    let g = (fx * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
                    let b = ((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
                }
                9 => {
                    let theta = (fy - cy).atan2(fx - cx);
                    let r = (dist * 0.18 + t * 0.2).sin() * 127.0 + 128.0;
                    let g = (theta * 4.0 + t * 0.3).cos() * 127.0 + 128.0;
                    let b = (dist * 0.09 + theta + t * 0.4).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
                }
                10 => {
                    let r = (diag * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
                    let g = (diag * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
                    let b = (diag * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
                }
                11 => {
                    let v = (fx * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
                }
                12 => {
                    let v = (fy * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
                }
                13 => {
                    let v = (dist * 0.25 + t * 0.25).sin() * 127.0 + 128.0;
                    let w = (dist * 0.12 + t * 0.1).cos() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(v as u8, 255 - w as u8, (w as u8) / 2, a)
                }
                14 => {
                    let v = (diag * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
                    Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
                }
                15 => {
                    let seed = (x as u32).wrapping_mul(73856093)
                        ^ (zebra_offset as u32).wrapping_mul(83492791);
                    let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                    Color32::from_rgba_unmultiplied(
                        val,
                        val.wrapping_mul(3),
                        val.wrapping_mul(7),
                        a,
                    )
                }
                16 => {
                    let seed = (y as u32).wrapping_mul(19349663)
                        ^ (zebra_offset as u32).wrapping_mul(83492791);
                    let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                    Color32::from_rgba_unmultiplied(
                        val,
                        val.wrapping_mul(3),
                        val.wrapping_mul(7),
                        a,
                    )
                }
                17 => {
                    let theta = (fy - cy).atan2(fx - cx);
                    let seed = (dist as u32).wrapping_mul(1234567)
                        ^ (theta.to_bits().wrapping_mul(314159))
                        ^ (zebra_offset as u32).wrapping_mul(83492791);
                    let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                    Color32::from_rgba_unmultiplied(val, val.rotate_left(2), val.rotate_left(4), a)
                }
                18 => {
                    let seed = (diag as u32).wrapping_mul(9876543)
                        ^ (zebra_offset as u32).wrapping_mul(83492791);
                    let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                    Color32::from_rgba_unmultiplied(
                        val,
                        val.wrapping_mul(3),
                        val.wrapping_mul(7),
                        a,
                    )
                }
                _ => Color32::from_rgba_unmultiplied(255, 255, 255, a),
            };
        } else {
            // Not nearly black: leave unchanged
            return Color32::from_rgba_unmultiplied(r, g, b, a);
        }
    }
    // Non-inverted effects
    let (r, g, b) = (r, g, b);
    match index % 19 {
        1 => Color32::from_rgba_unmultiplied(255 - r, 255 - g, 255 - b, a),
        2 => {
            let avg = ((r as u16 + g as u16 + b as u16) / 3) as u8;
            Color32::from_rgba_unmultiplied(avg, avg, avg, a)
        }
        3 => {
            let v = (fx * 0.08 + t * 0.12).sin() + (fy * 0.08 + t * 0.15).cos();
            let v = v * 0.5;
            let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
            let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
            let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        4 => {
            let v = (fy * 0.08 + t * 0.12).sin() + (fx * 0.08 + t * 0.15).cos();
            let v = v * 0.5;
            let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
            let g = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
            let b = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        5 => {
            let theta = (fy - cy).atan2(fx - cx);
            let v = (dist * 0.15 + t * 0.12).sin() + (theta * 6.0 + t * 0.2).cos();
            let v = v * 0.5;
            let r = ((v * 127.0 + 128.0) as u8).saturating_add((theta.sin() * 127.0 + 128.0) as u8);
            let g = ((v * 127.0 + 128.0) as u8).saturating_add((theta.cos() * 127.0 + 128.0) as u8);
            let b = ((v * 127.0 + 128.0) as u8).saturating_add((dist.sin() * 127.0 + 128.0) as u8);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        6 => {
            let v = (diag * 0.08 + t * 0.12).sin() + (t * 0.15).cos();
            let v = v * 0.5;
            let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
            let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
            let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        7 => {
            let r = (fx * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
            let g = (fy * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
            let b = ((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
        }
        8 => {
            let r = (fy * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
            let g = (fx * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
            let b = ((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
        }
        9 => {
            let theta = (fy - cy).atan2(fx - cx);
            let r = (dist * 0.18 + t * 0.2).sin() * 127.0 + 128.0;
            let g = (theta * 4.0 + t * 0.3).cos() * 127.0 + 128.0;
            let b = (dist * 0.09 + theta + t * 0.4).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
        }
        10 => {
            let r = (diag * 0.12 + t * 0.2).sin() * 127.0 + 128.0;
            let g = (diag * 0.12 + t * 0.3).cos() * 127.0 + 128.0;
            let b = (diag * 0.07 + t * 0.4).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(r as u8, g as u8, b as u8, a)
        }
        11 => {
            let v = (fx * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
        }
        12 => {
            let v = (fy * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
        }
        13 => {
            let v = (dist * 0.25 + t * 0.25).sin() * 127.0 + 128.0;
            let w = (dist * 0.12 + t * 0.1).cos() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(v as u8, 255 - w as u8, (w as u8) / 2, a)
        }
        14 => {
            let v = (diag * 0.2 + t * 0.25).sin() * 127.0 + 128.0;
            Color32::from_rgba_unmultiplied(v as u8, 255 - v as u8, (v as u8) / 2, a)
        }
        15 => {
            let seed =
                (x as u32).wrapping_mul(73856093) ^ (zebra_offset as u32).wrapping_mul(83492791);
            let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
            Color32::from_rgba_unmultiplied(val, val.wrapping_mul(3), val.wrapping_mul(7), a)
        }
        16 => {
            let seed =
                (y as u32).wrapping_mul(19349663) ^ (zebra_offset as u32).wrapping_mul(83492791);
            let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
            Color32::from_rgba_unmultiplied(val, val.wrapping_mul(3), val.wrapping_mul(7), a)
        }
        17 => {
            let theta = (fy - cy).atan2(fx - cx);
            let seed = (dist as u32).wrapping_mul(1234567)
                ^ (theta.to_bits().wrapping_mul(314159))
                ^ (zebra_offset as u32).wrapping_mul(83492791);
            let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
            Color32::from_rgba_unmultiplied(val, val.rotate_left(2), val.rotate_left(4), a)
        }
        18 => {
            let seed =
                (diag as u32).wrapping_mul(9876543) ^ (zebra_offset as u32).wrapping_mul(83492791);
            let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
            Color32::from_rgba_unmultiplied(val, val.wrapping_mul(3), val.wrapping_mul(7), a)
        }
        _ => Color32::from_rgba_unmultiplied(r, g, b, a),
    }
}

/// Blend effect and original pixel using various blend modes
#[derive(Clone, Copy, PartialEq)]
pub enum BlendMode {
    None,
    FullMode,
    FullDyn,
    BlendMode,
    Random,
    Diagonal,
    Checker,
    Threshold,
    Intensity,
    AnyLit,
    Optimal,
    OptimalPerFrame,
    RandomMode,
    RandomModeMode,
}

#[allow(clippy::too_many_arguments)]
pub fn apply_effect_blend(
    index: usize,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    t: f32,
    zebra_offset: usize,
    mode: BlendMode,
) -> Color32 {
    use getrandom::fill;

    /// Returns a random f32 in [0, 1)
    pub fn random_f32() -> f32 {
        let mut bytes = [0u8; 4];
        fill(&mut bytes).expect("getrandom failed");
        let val = u32::from_le_bytes(bytes);
        (val as f32) / (u32::MAX as f32)
    }

    /// Returns a random usize in [start, end)
    pub fn random_range(start: usize, end: usize) -> usize {
        let mut bytes = [0u8; 8];
        fill(&mut bytes).expect("getrandom failed");
        let val = u64::from_le_bytes(bytes);
        let range = end - start;
        start + (val as usize % range)
    }
    // For FullDyn, vary t over time for dynamic effect
    let effect = match mode {
        BlendMode::FullDyn => {
            // Multiply t by 10000 for dynamic animation
            apply_effect(index, r, g, b, a, x, y, w, h, t * 10000.0, zebra_offset)
        }
        _ => apply_effect(index, r, g, b, a, x, y, w, h, t, zebra_offset),
    };
    let intensity = (r as u16 + g as u16 + b as u16) as f32 / (3.0 * 255.0);
    let mut alpha = match mode {
        BlendMode::None => 0.0,
        BlendMode::FullMode => 1.0, // Always apply effect color
        BlendMode::FullDyn => 1.0,
        BlendMode::BlendMode => intensity.clamp(0.0, 1.0),
        BlendMode::Random => random_f32(),
        BlendMode::Diagonal => ((x as f32 + y as f32) / (w as f32 + h as f32)).clamp(0.0, 1.0),
        BlendMode::Checker => {
            if (x / 8 + y / 8).is_multiple_of(2) {
                1.0
            } else {
                0.0
            }
        }
        BlendMode::Threshold => {
            if intensity > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        BlendMode::Intensity => intensity.clamp(0.0, 1.0),
        BlendMode::AnyLit => {
            if intensity > 0.0 {
                1.0
            } else {
                0.0
            }
        }
        BlendMode::Optimal | BlendMode::OptimalPerFrame => 1.0, // fallback, should not be used directly
        BlendMode::RandomMode => match random_range(0, 5) {
            0 => random_f32(),
            1 => ((x as f32 + y as f32) / (w as f32 + h as f32)).clamp(0.0, 1.0),
            2 => {
                if (x / 8 + y / 8).is_multiple_of(2) {
                    1.0
                } else {
                    0.0
                }
            }
            3 => {
                if intensity > 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => intensity.clamp(0.0, 1.0),
        },
        BlendMode::RandomModeMode => {
            // Randomly select any blend mode (including RandomMode) for this pixel
            let blend_modes = [
                BlendMode::FullMode,
                BlendMode::FullDyn,
                BlendMode::BlendMode,
                BlendMode::Random,
                BlendMode::Diagonal,
                BlendMode::Checker,
                BlendMode::Threshold,
                BlendMode::Intensity,
                BlendMode::AnyLit,
                BlendMode::RandomMode,
                // Exclude RandomModeMode to prevent deep recursion
            ];
            let selected = blend_modes[random_range(0, blend_modes.len())];
            // Recurse with selected mode
            return apply_effect_blend(index, r, g, b, a, x, y, w, h, t, zebra_offset, selected);
        }
    };
    // For any invert effect, if pixel is black, use full effect color (white)
    if (index == 1 || index == 20) && r == 0 && g == 0 && b == 0 {
        alpha = 1.0;
    }
    let blend = |orig: u8, eff: u8| {
        ((eff as f32) * alpha + (orig as f32) * (1.0 - alpha))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_unmultiplied(
        blend(r, effect.r()),
        blend(g, effect.g()),
        blend(b, effect.b()),
        a,
    )
}
