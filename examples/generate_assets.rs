// SocialCount - Asset Generator
// Generates logo, GitHub social preview, and screenshots for documentation
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

fn blend_pixel(bg: &mut Rgba<u8>, fg: Rgba<u8>) {
    let alpha = fg[3] as f32 / 255.0;
    if alpha <= 0.0 {
        return;
    }
    let inv_alpha = 1.0 - alpha;
    bg[0] = (fg[0] as f32 * alpha + bg[0] as f32 * inv_alpha).round().clamp(0.0, 255.0) as u8;
    bg[1] = (fg[1] as f32 * alpha + bg[1] as f32 * inv_alpha).round().clamp(0.0, 255.0) as u8;
    bg[2] = (fg[2] as f32 * alpha + bg[2] as f32 * inv_alpha).round().clamp(0.0, 255.0) as u8;
    bg[3] = ((bg[3] as f32) + (fg[3] as f32) * (1.0 - bg[3] as f32 / 255.0)).clamp(0.0, 255.0) as u8;
}

fn draw_rounded_rect(
    img: &mut RgbaImage,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: f32,
    color: Rgba<u8>,
) {
    let (iw, ih) = img.dimensions();
    for py in y..(y + h) {
        if py < 0 || py >= ih as i32 {
            continue;
        }
        for px in x..(x + w) {
            if px < 0 || px >= iw as i32 {
                continue;
            }

            // Distance from rounded corner
            let dx = if px < x + radius as i32 {
                (x + radius as i32) - px
            } else if px >= x + w - radius as i32 {
                px - (x + w - radius as i32 - 1)
            } else {
                0
            };

            let dy = if py < y + radius as i32 {
                (y + radius as i32) - py
            } else if py >= y + h - radius as i32 {
                py - (y + h - radius as i32 - 1)
            } else {
                0
            };

            if dx > 0 && dy > 0 {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius - 0.5 {
                    blend_pixel(img.get_pixel_mut(px as u32, py as u32), color);
                } else if dist < radius + 0.5 {
                    let mut aa = color;
                    let factor = (radius + 0.5 - dist).clamp(0.0, 1.0);
                    aa[3] = ((color[3] as f32) * factor).round() as u8;
                    blend_pixel(img.get_pixel_mut(px as u32, py as u32), aa);
                }
            } else {
                blend_pixel(img.get_pixel_mut(px as u32, py as u32), color);
            }
        }
    }
}

fn draw_circle(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    let (iw, ih) = img.dimensions();
    let min_x = (cx - radius - 1.0).max(0.0) as u32;
    let max_x = (cx + radius + 1.0).min(iw as f32 - 1.0) as u32;
    let min_y = (cy - radius - 1.0).max(0.0) as u32;
    let max_y = (cy + radius + 1.0).min(ih as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius - 0.5 {
                blend_pixel(img.get_pixel_mut(x, y), color);
            } else if dist < radius + 0.5 {
                let mut aa = color;
                let factor = (radius + 0.5 - dist).clamp(0.0, 1.0);
                aa[3] = ((color[3] as f32) * factor).round() as u8;
                blend_pixel(img.get_pixel_mut(x, y), aa);
            }
        }
    }
}

fn draw_circle_stroke(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, stroke_w: f32, color: Rgba<u8>) {
    let (iw, ih) = img.dimensions();
    let outer = radius + stroke_w * 0.5;
    let inner = radius - stroke_w * 0.5;
    let min_x = (cx - outer - 1.0).max(0.0) as u32;
    let max_x = (cx + outer + 1.0).min(iw as f32 - 1.0) as u32;
    let min_y = (cy - outer - 1.0).max(0.0) as u32;
    let max_y = (cy + outer + 1.0).min(ih as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let d_outer = outer - dist;
            let d_inner = dist - inner;
            let d = d_outer.min(d_inner);

            if d >= 0.5 {
                blend_pixel(img.get_pixel_mut(x, y), color);
            } else if d > -0.5 {
                let factor = (d + 0.5).clamp(0.0, 1.0);
                let mut aa = color;
                aa[3] = ((color[3] as f32) * factor).round() as u8;
                blend_pixel(img.get_pixel_mut(x, y), aa);
            }
        }
    }
}

fn draw_triangle(img: &mut RgbaImage, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), color: Rgba<u8>) {
    let (iw, ih) = img.dimensions();
    let min_x = p0.0.min(p1.0).min(p2.0).max(0.0) as u32;
    let max_x = (p0.0.max(p1.0).max(p2.0) + 1.0).min(iw as f32 - 1.0) as u32;
    let min_y = p0.1.min(p1.1).min(p2.1).max(0.0) as u32;
    let max_y = (p0.1.max(p1.1).max(p2.1) + 1.0).min(ih as f32 - 1.0) as u32;

    fn edge(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
        (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(p1, p2, p);
            let w1 = edge(p2, p0, p);
            let w2 = edge(p0, p1, p);
            if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                blend_pixel(img.get_pixel_mut(x, y), color);
            }
        }
    }
}

fn draw_youtube_badge(img: &mut RgbaImage, cx: f32, cy: f32, w: f32, h: f32) {
    let x = (cx - w / 2.0).round() as i32;
    let y = (cy - h / 2.0).round() as i32;
    let wi = w.round() as i32;
    let hi = h.round() as i32;

    // Red badge
    draw_rounded_rect(img, x, y, wi, hi, h * 0.28, Rgba([255, 35, 35, 255]));

    // Reflection border
    draw_rounded_rect(img, x, y, wi, hi, h * 0.28, Rgba([255, 255, 255, 45]));
    draw_rounded_rect(img, x + 1, y + 1, wi - 2, hi - 2, h * 0.28 - 1.0, Rgba([255, 35, 35, 255]));

    // Centered white play triangle
    let tri_w = w * 0.24;
    let tri_h = h * 0.32;
    let p0 = (cx - tri_w * 0.45, cy - tri_h);
    let p1 = (cx - tri_w * 0.45, cy + tri_h);
    let p2 = (cx + tri_w * 0.65, cy);
    draw_triangle(img, p0, p1, p2, Rgba([255, 255, 255, 255]));
}

fn draw_user_icon(img: &mut RgbaImage, cx: f32, cy: f32, size: f32, color: Rgba<u8>) {
    let head_cy = cy - size * 0.20;
    let head_r = size * 0.26;
    draw_circle(img, cx, head_cy, head_r, color);

    let stroke_color = Rgba([color[0], color[1], color[2], 255]);
    draw_circle_stroke(img, cx, head_cy, head_r, 2.0, stroke_color);

    // Body arc
    let body_cy = cy + size * 0.42;
    let body_r = size * 0.38;
    draw_circle_stroke(img, cx, body_cy, body_r, 2.0, stroke_color);
}

fn draw_eye_icon(img: &mut RgbaImage, cx: f32, cy: f32, size: f32, color: Rgba<u8>) {
    let r = size * 0.36;
    draw_circle(img, cx, cy, r, color);

    let stroke_color = Rgba([color[0], color[1], color[2], 255]);
    draw_circle_stroke(img, cx, cy, r, 2.0, stroke_color);

    // Pupil
    draw_circle(img, cx, cy, r * 0.45, Rgba([15, 23, 42, 255]));
    // Catchlight highlight
    draw_circle(img, cx - r * 0.2, cy - r * 0.2, 1.8, Rgba([255, 255, 255, 255]));
}

/// Generates square logo for repo and app icon
fn generate_logo(path: &Path) {
    let size = 512;
    let mut img: RgbaImage = ImageBuffer::new(size, size);
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;

    // Glowing radial ambient background
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 240.0 {
                let factor = (1.0 - dist / 240.0).powf(1.8);
                let r = (240.0 * factor * 0.25) as u8;
                let g = (40.0 * factor * 0.2) as u8;
                let b = (70.0 * factor * 0.3) as u8;
                let a = (220.0 * factor) as u8;
                blend_pixel(img.get_pixel_mut(x, y), Rgba([r, g, b, a]));
            }
        }
    }

    // Rounded frosted glass shield
    draw_rounded_rect(&mut img, 48, 48, 416, 416, 88.0, Rgba([18, 20, 28, 245]));
    draw_rounded_rect(&mut img, 50, 50, 412, 412, 86.0, Rgba([255, 255, 255, 28]));
    draw_rounded_rect(&mut img, 52, 52, 408, 408, 84.0, Rgba([15, 17, 24, 250]));

    // Centered YouTube badge icon
    draw_youtube_badge(&mut img, cx, 195.0, 190.0, 130.0);

    // Stylized indicator boxes below
    // Emerald box (subscribers)
    draw_rounded_rect(&mut img, 80, 295, 160, 68, 16.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, 82, 297, 156, 64, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, 84, 299, 152, 60, 12.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, 120.0, 329.0, 28.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, 145, 323, 75, 12, 6.0, Rgba([209, 250, 229, 255]));

    // Sapphire box (views)
    draw_rounded_rect(&mut img, 272, 295, 160, 68, 16.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, 274, 297, 156, 64, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, 276, 299, 152, 60, 12.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, 312.0, 329.0, 28.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, 337, 323, 75, 12, 6.0, Rgba([224, 242, 254, 255]));

    // Live Dot bar at the bottom
    draw_circle(&mut img, cx - 70.0, 395.0, 6.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, cx - 70.0, 395.0, 10.0, 2.0, Rgba([34, 197, 94, 90]));
    draw_rounded_rect(&mut img, (cx - 45.0) as i32, 390, 120, 10, 5.0, Rgba([148, 163, 184, 200]));

    img.save(path).expect("Failed to save logo");
    println!("Saved: {}", path.display());
}

/// Generates screenshot for normal mode
fn generate_normal_screenshot(path: &Path) {
    let w = 480;
    let h = 580;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Desktop gradient background (dark wallpaper style)
    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Shadow and SocialCount widget
    let wx = 60;
    let wy = 50;
    let ww = 360;
    let wh = 460;

    // Window shadow
    draw_rounded_rect(&mut img, wx - 6, wy + 4, ww + 12, wh + 12, 24.0, Rgba([0, 0, 0, 120]));

    // Frosted glass SocialCount window
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([18, 20, 28, 235]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, wx + 1, wy + 1, ww - 2, wh - 2, 17.0, Rgba([18, 20, 28, 240]));

    // Header: YouTube icon + Title + slider + refresh + close
    draw_youtube_badge(&mut img, (wx + 26) as f32, (wy + 24) as f32, 20.0, 14.0);
    draw_rounded_rect(&mut img, wx + 42, wy + 18, 90, 12, 4.0, Rgba([255, 255, 255, 230])); // "SocialCount"

    // Slider
    draw_rounded_rect(&mut img, wx + 215, wy + 21, 60, 6, 3.0, Rgba([80, 85, 100, 220]));
    draw_circle(&mut img, (wx + 255) as f32, (wy + 24) as f32, 6.0, Rgba([255, 255, 255, 240]));

    // Refresh icon & Close icon
    draw_circle_stroke(&mut img, (wx + 295) as f32, (wy + 24) as f32, 6.0, 2.0, Rgba([147, 197, 253, 240]));
    draw_circle(&mut img, (wx + 328) as f32, (wy + 24) as f32, 8.0, Rgba([239, 68, 68, 80]));

    // Divider line
    draw_rounded_rect(&mut img, wx + 14, wy + 46, ww - 28, 1, 0.0, Rgba([255, 255, 255, 22]));

    // Channel 1: @matteobaccan
    let c1_y = wy + 60;
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([255, 255, 255, 25]));

    // Circular avatar with reflection
    draw_circle(&mut img, (wx + 44) as f32, (c1_y + 46) as f32, 24.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (wx + 44) as f32, (c1_y + 46) as f32, 24.0, 2.0, Rgba([255, 255, 255, 70]));

    // Channel name & handle
    draw_rounded_rect(&mut img, wx + 80, c1_y + 16, 120, 14, 4.0, Rgba([255, 255, 255, 240]));
    draw_rounded_rect(&mut img, wx + 210, c1_y + 18, 90, 10, 3.0, Rgba([148, 163, 184, 180]));

    // Stat boxes
    // Emerald box
    draw_rounded_rect(&mut img, wx + 80, c1_y + 46, 116, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 80, c1_y + 46, 116, 28, 8.0, Rgba([52, 211, 153, 200]));
    draw_rounded_rect(&mut img, wx + 81, c1_y + 47, 114, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 100) as f32, (c1_y + 60) as f32, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 118, c1_y + 54, 56, 11, 3.0, Rgba([209, 250, 229, 255]));

    // Sapphire box
    draw_rounded_rect(&mut img, wx + 204, c1_y + 46, 116, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 204, c1_y + 46, 116, 28, 8.0, Rgba([96, 165, 250, 200]));
    draw_rounded_rect(&mut img, wx + 205, c1_y + 47, 114, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 224) as f32, (c1_y + 60) as f32, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 242, c1_y + 54, 60, 11, 3.0, Rgba([224, 242, 254, 255]));

    // Sticky Footer
    draw_rounded_rect(&mut img, wx + 14, wy + wh - 36, ww - 28, 1, 0.0, Rgba([255, 255, 255, 18]));
    draw_circle(&mut img, (wx + 130) as f32, (wy + wh - 18) as f32, 3.5, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (wx + 130) as f32, (wy + wh - 18) as f32, 6.0, 1.5, Rgba([34, 197, 94, 90]));
    draw_rounded_rect(&mut img, wx + 145, wy + wh - 23, 110, 10, 3.0, Rgba([148, 163, 184, 190]));

    img.save(path).expect("Failed to save screenshot");
    println!("Saved: {}", path.display());
}

/// Generates screenshot for compact mode
fn generate_compact_screenshot(path: &Path) {
    let w = 420;
    let h = 320;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Desktop gradient background (dark wallpaper style)
    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let wx = 100;
    let wy = 80;
    let ww = 220;
    let wh = 140;

    // Window shadow
    draw_rounded_rect(&mut img, wx - 4, wy + 4, ww + 8, wh + 8, 16.0, Rgba([0, 0, 0, 130]));

    // Frosted glass SocialCount compact window
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 10.0, Rgba([18, 20, 28, 235]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 10.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, wx + 1, wy + 1, ww - 2, wh - 2, 9.0, Rgba([18, 20, 28, 240]));

    // Compact Header: tiny central drag pill and close button
    let cx = (wx + ww / 2) as f32;
    draw_rounded_rect(&mut img, (cx - 14.0) as i32, wy + 5, 28, 3, 1.5, Rgba([255, 255, 255, 120]));
    draw_circle(&mut img, (wx + ww - 10) as f32, (wy + 7) as f32, 4.0, Rgba([239, 68, 68, 100]));

    // Compact Row 1
    let r1_y = wy + 16;
    draw_rounded_rect(&mut img, wx + 6, r1_y, ww - 12, 34, 6.0, Rgba([22, 25, 34, 235]));

    // Left pill
    draw_rounded_rect(&mut img, wx + 10, r1_y + 6, 70, 22, 5.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 10, r1_y + 6, 70, 22, 5.0, Rgba([52, 211, 153, 200]));
    draw_user_icon(&mut img, (wx + 22) as f32, (r1_y + 17) as f32, 11.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 34, r1_y + 12, 36, 9, 3.0, Rgba([209, 250, 229, 255]));

    // Center avatar
    draw_circle(&mut img, cx, (r1_y + 17) as f32, 14.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, cx, (r1_y + 17) as f32, 14.0, 1.5, Rgba([255, 255, 255, 80]));

    // Right pill
    draw_rounded_rect(&mut img, wx + ww - 80, r1_y + 6, 70, 22, 5.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + ww - 80, r1_y + 6, 70, 22, 5.0, Rgba([96, 165, 250, 200]));
    draw_eye_icon(&mut img, (wx + ww - 68) as f32, (r1_y + 17) as f32, 12.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + ww - 56, r1_y + 12, 38, 9, 3.0, Rgba([224, 242, 254, 255]));

    img.save(path).expect("Failed to save compact screenshot");
    println!("Saved: {}", path.display());
}

/// Generates banner image for GitHub preview
fn generate_banner(path: &Path) {
    let w = 1280;
    let h = 640;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Deep modern dark gradient
    for y in 0..h {
        let fy = y as f32 / h as f32;
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let r = (11.0 + fy * 12.0 + fx * 4.0) as u8;
            let g = (14.0 + fy * 14.0 + fx * 6.0) as u8;
            let b = (22.0 + fy * 26.0 + fx * 18.0) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Glowing accents in background
    let glows = [
        (250.0, 200.0, 280.0, Rgba([239, 68, 68, 45])),
        (980.0, 380.0, 320.0, Rgba([52, 211, 153, 30])),
        (640.0, 520.0, 250.0, Rgba([96, 165, 250, 28])),
    ];

    for (gx, gy, gr, color) in glows {
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - gx;
                let dy = y as f32 - gy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < gr {
                    let factor = (1.0 - dist / gr).powf(2.0);
                    let mut aa = color;
                    aa[3] = ((color[3] as f32) * factor) as u8;
                    blend_pixel(img.get_pixel_mut(x, y), aa);
                }
            }
        }
    }

    // Left side: Large Hero Card with App Logo & Branding
    let card_x = 100;
    let card_y = 100;
    let card_w = 460;
    let card_h = 440;

    // Card shadow
    draw_rounded_rect(&mut img, card_x - 6, card_y + 8, card_w + 12, card_h + 12, 28.0, Rgba([0, 0, 0, 140]));
    // Card body
    draw_rounded_rect(&mut img, card_x, card_y, card_w, card_h, 24.0, Rgba([18, 22, 32, 245]));
    draw_rounded_rect(&mut img, card_x, card_y, card_w, card_h, 24.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, card_x + 1, card_y + 1, card_w - 2, card_h - 2, 23.0, Rgba([16, 19, 28, 250]));

    // Large YouTube badge in Hero
    draw_youtube_badge(&mut img, (card_x + card_w / 2) as f32, (card_y + 120) as f32, 170.0, 116.0);

    // Title placeholder line
    draw_rounded_rect(&mut img, card_x + 80, card_y + 205, card_w - 160, 20, 6.0, Rgba([255, 255, 255, 240]));

    // Subtitle placeholder line
    draw_rounded_rect(&mut img, card_x + 60, card_y + 240, card_w - 120, 12, 4.0, Rgba([148, 163, 184, 210]));
    draw_rounded_rect(&mut img, card_x + 90, card_y + 260, card_w - 180, 10, 4.0, Rgba([100, 116, 139, 180]));

    // Stat boxes in Hero
    let box_y = card_y + 300;
    // Users
    draw_rounded_rect(&mut img, card_x + 50, box_y, 160, 62, 14.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, card_x + 50, box_y, 160, 62, 14.0, Rgba([52, 211, 153, 220]));
    draw_rounded_rect(&mut img, card_x + 51, box_y + 1, 158, 60, 13.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (card_x + 85) as f32, (box_y + 31) as f32, 22.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, card_x + 105, box_y + 25, 80, 13, 4.0, Rgba([209, 250, 229, 255]));

    // Views
    draw_rounded_rect(&mut img, card_x + 245, box_y, 160, 62, 14.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, card_x + 245, box_y, 160, 62, 14.0, Rgba([96, 165, 250, 220]));
    draw_rounded_rect(&mut img, card_x + 246, box_y + 1, 158, 60, 13.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (card_x + 280) as f32, (box_y + 31) as f32, 22.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, card_x + 300, box_y + 25, 80, 13, 4.0, Rgba([224, 242, 254, 255]));

    // Pulsing live indicator
    draw_circle(&mut img, (card_x + 160) as f32, (card_y + 395) as f32, 5.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (card_x + 160) as f32, (card_y + 395) as f32, 9.0, 2.0, Rgba([34, 197, 94, 100]));
    draw_rounded_rect(&mut img, card_x + 180, card_y + 390, 120, 10, 4.0, Rgba([148, 163, 184, 210]));

    // Right side: Floating Realistic App Mockup
    let mock_x = 640;
    let mock_y = 120;
    let mock_w = 480;
    let mock_h = 400;

    // Window shadow
    draw_rounded_rect(&mut img, mock_x - 8, mock_y + 12, mock_w + 16, mock_h + 16, 26.0, Rgba([0, 0, 0, 150]));
    // Window body
    draw_rounded_rect(&mut img, mock_x, mock_y, mock_w, mock_h, 20.0, Rgba([18, 20, 28, 240]));
    draw_rounded_rect(&mut img, mock_x, mock_y, mock_w, mock_h, 20.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, mock_x + 1, mock_y + 1, mock_w - 2, mock_h - 2, 19.0, Rgba([18, 20, 28, 245]));

    // Header mockup
    draw_youtube_badge(&mut img, (mock_x + 30) as f32, (mock_y + 26) as f32, 22.0, 15.0);
    draw_rounded_rect(&mut img, mock_x + 50, mock_y + 20, 110, 13, 4.0, Rgba([255, 255, 255, 230]));
    draw_rounded_rect(&mut img, mock_x + 300, mock_y + 23, 70, 7, 3.5, Rgba([80, 85, 100, 220]));
    draw_circle(&mut img, (mock_x + 345) as f32, (mock_y + 26) as f32, 7.0, Rgba([255, 255, 255, 240]));
    draw_circle_stroke(&mut img, (mock_x + 395) as f32, (mock_y + 26) as f32, 7.0, 2.0, Rgba([147, 197, 253, 240]));
    draw_circle(&mut img, (mock_x + 435) as f32, (mock_y + 26) as f32, 9.0, Rgba([239, 68, 68, 90]));

    // Divider
    draw_rounded_rect(&mut img, mock_x + 16, mock_y + 50, mock_w - 32, 1, 0.0, Rgba([255, 255, 255, 22]));

    // Channel Row in Mockup
    let row_y = mock_y + 68;
    draw_rounded_rect(&mut img, mock_x + 16, row_y, mock_w - 32, 100, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, mock_x + 16, row_y, mock_w - 32, 100, 14.0, Rgba([255, 255, 255, 25]));

    // Avatar
    draw_circle(&mut img, (mock_x + 52) as f32, (row_y + 50) as f32, 28.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (mock_x + 52) as f32, (row_y + 50) as f32, 28.0, 2.0, Rgba([255, 255, 255, 70]));

    // Handle and name
    draw_rounded_rect(&mut img, mock_x + 94, row_y + 18, 140, 15, 4.0, Rgba([255, 255, 255, 240]));
    draw_rounded_rect(&mut img, mock_x + 245, row_y + 20, 100, 11, 3.0, Rgba([148, 163, 184, 180]));

    // Counter pills in row
    // Users
    draw_rounded_rect(&mut img, mock_x + 94, row_y + 50, 160, 32, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, mock_x + 94, row_y + 50, 160, 32, 8.0, Rgba([52, 211, 153, 200]));
    draw_rounded_rect(&mut img, mock_x + 95, row_y + 51, 158, 30, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (mock_x + 120) as f32, (row_y + 66) as f32, 16.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, mock_x + 140, row_y + 59, 90, 14, 4.0, Rgba([209, 250, 229, 255]));

    // Views
    draw_rounded_rect(&mut img, mock_x + 270, row_y + 50, 160, 32, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, mock_x + 270, row_y + 50, 160, 32, 8.0, Rgba([96, 165, 250, 200]));
    draw_rounded_rect(&mut img, mock_x + 271, row_y + 51, 158, 30, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (mock_x + 296) as f32, (row_y + 66) as f32, 16.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, mock_x + 316, row_y + 59, 90, 14, 4.0, Rgba([224, 242, 254, 255]));

    // Compact floating widget overlayed on bottom right
    let cw_x = mock_x + 180;
    let cw_y = mock_y + 220;
    let cw_w = 260;
    let cw_h = 140;

    draw_rounded_rect(&mut img, cw_x - 6, cw_y + 8, cw_w + 12, cw_h + 12, 18.0, Rgba([0, 0, 0, 160]));
    draw_rounded_rect(&mut img, cw_x, cw_y, cw_w, cw_h, 14.0, Rgba([15, 18, 25, 245]));
    draw_rounded_rect(&mut img, cw_x, cw_y, cw_w, cw_h, 14.0, Rgba([255, 255, 255, 40]));

    // Compact pill handle
    let cw_cx = (cw_x + cw_w / 2) as f32;
    draw_rounded_rect(&mut img, (cw_cx - 16.0) as i32, cw_y + 6, 32, 3, 1.5, Rgba([255, 255, 255, 130]));

    // Compact row
    let cr_y = cw_y + 20;
    draw_rounded_rect(&mut img, cw_x + 8, cr_y, cw_w - 16, 42, 8.0, Rgba([24, 28, 38, 240]));
    draw_rounded_rect(&mut img, cw_x + 14, cr_y + 8, 80, 26, 6.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (cw_x + 28) as f32, (cr_y + 21) as f32, 12.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, cw_x + 42, cr_y + 16, 44, 10, 3.0, Rgba([209, 250, 229, 255]));

    draw_circle(&mut img, cw_cx, (cr_y + 21) as f32, 16.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, cw_cx, (cr_y + 21) as f32, 16.0, 1.5, Rgba([255, 255, 255, 80]));

    draw_rounded_rect(&mut img, cw_x + cw_w - 94, cr_y + 8, 80, 26, 6.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (cw_x + cw_w - 80) as f32, (cr_y + 21) as f32, 13.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, cw_x + cw_w - 66, cr_y + 16, 44, 10, 3.0, Rgba([224, 242, 254, 255]));

    img.save(path).expect("Failed to save banner");
    println!("Saved: {}", path.display());
}

fn main() {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).expect("Failed to create assets dir");
    }

    generate_logo(&assets_dir.join("logo.png"));
    generate_normal_screenshot(&assets_dir.join("screenshot-normal.png"));
    generate_compact_screenshot(&assets_dir.join("screenshot-compact.png"));
    generate_banner(&assets_dir.join("socialcount-banner.png"));

    println!("All assets successfully generated!");
}
