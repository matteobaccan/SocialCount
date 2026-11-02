// SocialCount - Asset Generator
// Generates logo, GitHub social preview, and screenshots for documentation
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

pub struct Fonts<'a> {
    pub bold: FontRef<'a>,
    pub regular: FontRef<'a>,
}

fn load_system_fonts() -> (Vec<u8>, Vec<u8>) {
    let bold_candidates = [
        "C:\\Windows\\Fonts\\segoeuib.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/System/Library/Fonts/SFNSBold.ttf",
    ];
    let reg_candidates = [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/SFNS.ttf",
    ];

    let mut bold_bytes = None;
    for path in &bold_candidates {
        if let Ok(b) = fs::read(path) {
            bold_bytes = Some(b);
            break;
        }
    }

    let mut reg_bytes = None;
    for path in &reg_candidates {
        if let Ok(b) = fs::read(path) {
            reg_bytes = Some(b);
            break;
        }
    }

    let bold_data = bold_bytes.or_else(|| reg_bytes.clone()).expect("System font required");
    let reg_data = reg_bytes.unwrap_or_else(|| bold_data.clone());
    (bold_data, reg_data)
}

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

    draw_rounded_rect(img, x, y, wi, hi, h * 0.28, Rgba([255, 35, 35, 255]));
    draw_rounded_rect(img, x, y, wi, hi, h * 0.28, Rgba([255, 255, 255, 45]));
    draw_rounded_rect(img, x + 1, y + 1, wi - 2, hi - 2, h * 0.28 - 1.0, Rgba([255, 35, 35, 255]));

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
    draw_circle_stroke(img, cx, head_cy, head_r, 1.5, stroke_color);

    let body_cy = cy + size * 0.42;
    let body_r = size * 0.38;
    draw_circle_stroke(img, cx, body_cy, body_r, 1.5, stroke_color);
}

fn draw_eye_icon(img: &mut RgbaImage, cx: f32, cy: f32, size: f32, color: Rgba<u8>) {
    let r = size * 0.36;
    draw_circle(img, cx, cy, r, color);

    let stroke_color = Rgba([color[0], color[1], color[2], 255]);
    draw_circle_stroke(img, cx, cy, r, 1.5, stroke_color);

    draw_circle(img, cx, cy, r * 0.45, Rgba([15, 23, 42, 255]));
    draw_circle(img, cx - r * 0.2, cy - r * 0.2, 1.5, Rgba([255, 255, 255, 255]));
}

fn draw_text(
    img: &mut RgbaImage,
    font: &impl Font,
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    color: Rgba<u8>,
) {
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let mut cursor = point(x, y + scaled_font.ascent());

    for ch in text.chars() {
        if ch == '\n' {
            cursor.x = x;
            cursor.y += scaled_font.height() + scaled_font.line_gap();
            continue;
        }
        let glyph = font.glyph_id(ch).with_scale_and_position(px_scale, cursor);
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, c| {
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                    let mut text_color = color;
                    text_color[3] = ((color[3] as f32) * c).round() as u8;
                    blend_pixel(img.get_pixel_mut(px as u32, py as u32), text_color);
                }
            });
        }
        cursor.x += scaled_font.h_advance(font.glyph_id(ch));
    }
}

fn measure_text(font: &impl Font, text: &str, scale: f32) -> (f32, f32) {
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let mut width = 0.0_f32;
    for ch in text.chars() {
        width += scaled_font.h_advance(font.glyph_id(ch));
    }
    (width, scaled_font.height())
}

fn draw_text_centered(
    img: &mut RgbaImage,
    font: &impl Font,
    text: &str,
    cx: f32,
    cy: f32,
    scale: f32,
    color: Rgba<u8>,
) {
    let (tw, th) = measure_text(font, text, scale);
    draw_text(img, font, text, cx - tw / 2.0, cy - th / 2.0, scale, color);
}

/// Generates square logo for repo and app icon
fn generate_logo(path: &Path, fonts: &Fonts) {
    let size = 512;
    let mut img: RgbaImage = ImageBuffer::new(size, size);
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;

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

    draw_rounded_rect(&mut img, 48, 48, 416, 416, 88.0, Rgba([18, 20, 28, 245]));
    draw_rounded_rect(&mut img, 50, 50, 412, 412, 86.0, Rgba([255, 255, 255, 28]));
    draw_rounded_rect(&mut img, 52, 52, 408, 408, 84.0, Rgba([15, 17, 24, 250]));

    draw_youtube_badge(&mut img, cx, 185.0, 180.0, 124.0);

    // Users counter box
    draw_rounded_rect(&mut img, 75, 285, 170, 72, 16.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, 75, 285, 170, 72, 16.0, Rgba([239, 68, 68, 220]));
    draw_rounded_rect(&mut img, 77, 287, 166, 68, 14.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, 108.0, 321.0, 24.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", 132.0, 308.0, 24.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, 174.0, 321.0, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1", 184.0, 310.0, 18.0, Rgba([252, 165, 165, 255]));

    // Views counter box
    draw_rounded_rect(&mut img, 267, 285, 170, 72, 16.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, 267, 285, 170, 72, 16.0, Rgba([239, 68, 68, 220]));
    draw_rounded_rect(&mut img, 269, 287, 166, 68, 14.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, 300.0, 321.0, 24.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.5K", 322.0, 308.0, 24.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, 386.0, 321.0, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15", 396.0, 310.0, 18.0, Rgba([252, 165, 165, 255]));

    // Live dot bar
    draw_circle(&mut img, cx - 80.0, 400.0, 5.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, cx - 80.0, 400.0, 9.0, 2.0, Rgba([34, 197, 94, 90]));
    draw_text(&mut img, &fonts.bold, "LIVE STATS WIDGET", cx - 60.0, 392.0, 15.0, Rgba([148, 163, 184, 240]));

    img.save(path).expect("Failed to save logo");
    println!("Saved: {}", path.display());
}

/// Generates screenshot for normal mode
fn generate_normal_screenshot(path: &Path, fonts: &Fonts) {
    let w = 480;
    let h = 580;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let wx = 55;
    let wy = 45;
    let ww = 370;
    let wh = 485;

    // Window shadow
    draw_rounded_rect(&mut img, wx - 6, wy + 6, ww + 12, wh + 12, 24.0, Rgba([0, 0, 0, 130]));

    // Window frame
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([18, 20, 28, 240]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, wx + 1, wy + 1, ww - 2, wh - 2, 17.0, Rgba([18, 20, 28, 245]));

    // Header: YouTube badge + Title + Opacity + Refresh + Close
    draw_youtube_badge(&mut img, (wx + 26) as f32, (wy + 24) as f32, 20.0, 14.0);
    draw_text(&mut img, &fonts.bold, "SocialCount", (wx + 44) as f32, (wy + 15) as f32, 16.0, Rgba([255, 255, 255, 240]));

    // Slider
    draw_rounded_rect(&mut img, wx + 215, wy + 21, 60, 6, 3.0, Rgba([70, 75, 90, 220]));
    draw_circle(&mut img, (wx + 258) as f32, (wy + 24) as f32, 6.0, Rgba([240, 245, 255, 240]));
    draw_text(&mut img, &fonts.regular, "🌓", (wx + 282) as f32, (wy + 16) as f32, 13.0, Rgba([175, 185, 200, 255]));

    // Refresh icon & Close icon
    draw_text(&mut img, &fonts.regular, "⟳", (wx + 312) as f32, (wy + 15) as f32, 16.0, Rgba([180, 205, 240, 255]));
    draw_text(&mut img, &fonts.regular, "✕", (wx + 342) as f32, (wy + 16) as f32, 13.0, Rgba([140, 145, 160, 255]));

    // Divider line
    draw_rounded_rect(&mut img, wx + 14, wy + 46, ww - 28, 1, 0.0, Rgba([255, 255, 255, 22]));

    // Channel 1: @matteobaccan
    let c1_y = wy + 62;
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([255, 255, 255, 25]));

    draw_circle(&mut img, (wx + 46) as f32, (c1_y + 46) as f32, 24.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (wx + 46) as f32, (c1_y + 46) as f32, 24.0, 1.5, Rgba([255, 255, 255, 70]));
    draw_text_centered(&mut img, &fonts.bold, "M", (wx + 46) as f32, (c1_y + 46) as f32, 20.0, Rgba([255, 255, 255, 255]));

    draw_text(&mut img, &fonts.bold, "Matteo Baccan", (wx + 82) as f32, (c1_y + 16) as f32, 15.0, Rgba([255, 255, 255, 255]));
    draw_text(&mut img, &fonts.regular, "@matteobaccan", (wx + 215) as f32, (c1_y + 19) as f32, 12.0, Rgba([148, 163, 184, 220]));

    // Subscribers box (with red border and delta)
    draw_rounded_rect(&mut img, wx + 82, c1_y + 46, 120, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 82, c1_y + 46, 120, 28, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, wx + 83, c1_y + 47, 118, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 98) as f32, (c1_y + 60) as f32, 13.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", (wx + 114) as f32, (c1_y + 53) as f32, 13.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (wx + 152) as f32, (c1_y + 60) as f32, 3.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1", (wx + 161) as f32, (c1_y + 54) as f32, 11.5, Rgba([252, 165, 165, 255]));

    // Views box (with red border and delta)
    draw_rounded_rect(&mut img, wx + 214, c1_y + 46, 128, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 214, c1_y + 46, 128, 28, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, wx + 215, c1_y + 47, 126, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 230) as f32, (c1_y + 60) as f32, 13.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.507", (wx + 246) as f32, (c1_y + 53) as f32, 13.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (wx + 295) as f32, (c1_y + 60) as f32, 3.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15", (wx + 304) as f32, (c1_y + 54) as f32, 11.5, Rgba([252, 165, 165, 255]));

    // Channel 2: @rustlang
    let c2_y = wy + 166;
    draw_rounded_rect(&mut img, wx + 14, c2_y, ww - 28, 92, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, wx + 14, c2_y, ww - 28, 92, 14.0, Rgba([255, 255, 255, 25]));

    draw_circle(&mut img, (wx + 46) as f32, (c2_y + 46) as f32, 24.0, Rgba([140, 60, 40, 255]));
    draw_circle_stroke(&mut img, (wx + 46) as f32, (c2_y + 46) as f32, 24.0, 1.5, Rgba([255, 255, 255, 70]));
    draw_text_centered(&mut img, &fonts.bold, "R", (wx + 46) as f32, (c2_y + 46) as f32, 20.0, Rgba([255, 255, 255, 255]));

    draw_text(&mut img, &fonts.bold, "Rust Foundation", (wx + 82) as f32, (c2_y + 16) as f32, 15.0, Rgba([255, 255, 255, 255]));
    draw_text(&mut img, &fonts.regular, "@rustlang", (wx + 225) as f32, (c2_y + 19) as f32, 12.0, Rgba([148, 163, 184, 220]));

    // Rust Subscribers
    draw_rounded_rect(&mut img, wx + 82, c2_y + 46, 120, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 82, c2_y + 46, 120, 28, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, wx + 83, c2_y + 47, 118, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 98) as f32, (c2_y + 60) as f32, 13.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "148K", (wx + 114) as f32, (c2_y + 53) as f32, 13.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (wx + 154) as f32, (c2_y + 60) as f32, 3.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+350", (wx + 163) as f32, (c2_y + 54) as f32, 11.0, Rgba([252, 165, 165, 255]));

    // Rust Views
    draw_rounded_rect(&mut img, wx + 214, c2_y + 46, 128, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 214, c2_y + 46, 128, 28, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, wx + 215, c2_y + 47, 126, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 230) as f32, (c2_y + 60) as f32, 13.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "1.8M", (wx + 246) as f32, (c2_y + 53) as f32, 13.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (wx + 288) as f32, (c2_y + 60) as f32, 3.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+4.2K", (wx + 297) as f32, (c2_y + 54) as f32, 11.0, Rgba([252, 165, 165, 255]));

    // Sticky Footer
    draw_rounded_rect(&mut img, wx + 14, wy + wh - 38, ww - 28, 1, 0.0, Rgba([255, 255, 255, 18]));
    draw_circle(&mut img, (wx + 125) as f32, (wy + wh - 20) as f32, 3.5, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (wx + 125) as f32, (wy + wh - 20) as f32, 6.0, 1.5, Rgba([34, 197, 94, 90]));
    draw_text(&mut img, &fonts.regular, "Updated: 16:45:00", (wx + 140) as f32, (wy + wh - 27) as f32, 11.5, Rgba([148, 163, 184, 220]));

    img.save(path).expect("Failed to save screenshot");
    println!("Saved: {}", path.display());
}

/// Generates screenshot for compact mode
fn generate_compact_screenshot(path: &Path, fonts: &Fonts) {
    let w = 420;
    let h = 320;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let wx = 95;
    let wy = 80;
    let ww = 230;
    let wh = 145;

    // Window shadow
    draw_rounded_rect(&mut img, wx - 4, wy + 4, ww + 8, wh + 8, 16.0, Rgba([0, 0, 0, 130]));

    // Window frame
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 10.0, Rgba([18, 20, 28, 240]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 10.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, wx + 1, wy + 1, ww - 2, wh - 2, 9.0, Rgba([18, 20, 28, 245]));

    // Header handle
    let cx = (wx + ww / 2) as f32;
    draw_rounded_rect(&mut img, (cx - 14.0) as i32, wy + 5, 28, 3, 1.5, Rgba([255, 255, 255, 120]));
    draw_text(&mut img, &fonts.regular, "✕", (wx + ww - 12) as f32, (wy + 2) as f32, 9.0, Rgba([140, 145, 160, 255]));

    // Compact Row 1: @matteobaccan
    let r1_y = wy + 16;
    draw_rounded_rect(&mut img, wx + 6, r1_y, ww - 12, 38, 6.0, Rgba([22, 25, 34, 235]));

    // Left pill
    draw_rounded_rect(&mut img, wx + 10, r1_y + 7, 76, 24, 5.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 10, r1_y + 7, 76, 24, 5.0, Rgba([239, 68, 68, 220]));
    draw_user_icon(&mut img, (wx + 22) as f32, (r1_y + 19) as f32, 11.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", (wx + 34) as f32, (r1_y + 13) as f32, 11.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (wx + 56) as f32, (r1_y + 19) as f32, 2.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1", (wx + 62) as f32, (r1_y + 14) as f32, 9.5, Rgba([252, 165, 165, 255]));

    // Center avatar
    draw_circle(&mut img, cx, (r1_y + 19) as f32, 14.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, cx, (r1_y + 19) as f32, 14.0, 1.5, Rgba([255, 255, 255, 80]));
    draw_text_centered(&mut img, &fonts.bold, "M", cx, (r1_y + 19) as f32, 13.0, Rgba([255, 255, 255, 255]));

    // Right pill
    draw_rounded_rect(&mut img, wx + ww - 86, r1_y + 7, 76, 24, 5.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + ww - 86, r1_y + 7, 76, 24, 5.0, Rgba([239, 68, 68, 220]));
    draw_eye_icon(&mut img, (wx + ww - 75) as f32, (r1_y + 19) as f32, 12.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.5K", (wx + ww - 63) as f32, (r1_y + 13) as f32, 11.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (wx + ww - 31) as f32, (r1_y + 19) as f32, 2.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15", (wx + ww - 25) as f32, (r1_y + 14) as f32, 9.5, Rgba([252, 165, 165, 255]));

    // Compact Row 2: @rustlang
    let r2_y = wy + 62;
    draw_rounded_rect(&mut img, wx + 6, r2_y, ww - 12, 38, 6.0, Rgba([22, 25, 34, 235]));

    // Left pill
    draw_rounded_rect(&mut img, wx + 10, r2_y + 7, 76, 24, 5.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 10, r2_y + 7, 76, 24, 5.0, Rgba([239, 68, 68, 220]));
    draw_user_icon(&mut img, (wx + 22) as f32, (r2_y + 19) as f32, 11.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "148K", (wx + 34) as f32, (r2_y + 13) as f32, 10.5, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (wx + 62) as f32, (r2_y + 19) as f32, 2.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+350", (wx + 67) as f32, (r2_y + 14) as f32, 8.5, Rgba([252, 165, 165, 255]));

    // Center avatar
    draw_circle(&mut img, cx, (r2_y + 19) as f32, 14.0, Rgba([140, 60, 40, 255]));
    draw_circle_stroke(&mut img, cx, (r2_y + 19) as f32, 14.0, 1.5, Rgba([255, 255, 255, 80]));
    draw_text_centered(&mut img, &fonts.bold, "R", cx, (r2_y + 19) as f32, 13.0, Rgba([255, 255, 255, 255]));

    // Right pill
    draw_rounded_rect(&mut img, wx + ww - 86, r2_y + 7, 76, 24, 5.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + ww - 86, r2_y + 7, 76, 24, 5.0, Rgba([239, 68, 68, 220]));
    draw_eye_icon(&mut img, (wx + ww - 75) as f32, (r2_y + 19) as f32, 12.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "1.8M", (wx + ww - 63) as f32, (r2_y + 13) as f32, 11.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (wx + ww - 31) as f32, (r2_y + 19) as f32, 2.5, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+4K", (wx + ww - 25) as f32, (r2_y + 14) as f32, 9.0, Rgba([252, 165, 165, 255]));

    img.save(path).expect("Failed to save compact screenshot");
    println!("Saved: {}", path.display());
}

/// Generates banner image for GitHub preview (1280x640) with real typography and live delta indicators
fn generate_banner(path: &Path, fonts: &Fonts) {
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
        (260.0, 180.0, 320.0, Rgba([239, 68, 68, 48])),
        (1020.0, 360.0, 360.0, Rgba([52, 211, 153, 34])),
        (640.0, 540.0, 280.0, Rgba([96, 165, 250, 32])),
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

    // ==========================================
    // LEFT SIDE: HERO BRANDING CARD
    // ==========================================
    let hero_x = 75;
    let hero_y = 65;
    let hero_w = 490;
    let hero_h = 510;

    // Soft deep card shadow
    draw_rounded_rect(&mut img, hero_x - 6, hero_y + 8, hero_w + 12, hero_h + 12, 28.0, Rgba([0, 0, 0, 150]));
    // Card body (frosted glass)
    draw_rounded_rect(&mut img, hero_x, hero_y, hero_w, hero_h, 24.0, Rgba([18, 22, 32, 245]));
    draw_rounded_rect(&mut img, hero_x, hero_y, hero_w, hero_h, 24.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, hero_x + 1, hero_y + 1, hero_w - 2, hero_h - 2, 23.0, Rgba([16, 19, 28, 250]));

    // Large YouTube badge in Hero
    draw_youtube_badge(&mut img, (hero_x + hero_w / 2) as f32, (hero_y + 85) as f32, 130.0, 88.0);

    // Title: SocialCount
    draw_text_centered(&mut img, &fonts.bold, "SocialCount", (hero_x + hero_w / 2) as f32, (hero_y + 155) as f32, 44.0, Rgba([255, 255, 255, 255]));

    // Subtitle
    draw_text_centered(&mut img, &fonts.bold, "Real-Time YouTube Statistics Widget", (hero_x + hero_w / 2) as f32, (hero_y + 205) as f32, 17.5, Rgba([224, 231, 255, 240]));
    draw_text_centered(&mut img, &fonts.regular, "Desktop overlay for creators with live delta tracking", (hero_x + hero_w / 2) as f32, (hero_y + 230) as f32, 14.0, Rgba([148, 163, 184, 220]));

    // Feature tags pills
    let tags = [
        ("🦀 Rust & egui", Rgba([249, 115, 22, 35]), Rgba([251, 146, 60, 255])),
        ("🪟 Windows Borderless", Rgba([59, 130, 246, 35]), Rgba([96, 165, 250, 255])),
        ("🔴 Live Delta ±", Rgba([239, 68, 68, 40]), Rgba([252, 165, 165, 255])),
        ("⚡ Ultra-Compact", Rgba([16, 185, 129, 35]), Rgba([52, 211, 153, 255])),
    ];

    let row1_y = hero_y + 268;
    // Tag 1 & 2
    draw_rounded_rect(&mut img, hero_x + 40, row1_y, 195, 32, 8.0, tags[0].1);
    draw_rounded_rect(&mut img, hero_x + 40, row1_y, 195, 32, 8.0, Rgba([tags[0].2[0], tags[0].2[1], tags[0].2[2], 90]));
    draw_text_centered(&mut img, &fonts.bold, tags[0].0, (hero_x + 40 + 97) as f32, (row1_y + 16) as f32, 13.0, tags[0].2);

    draw_rounded_rect(&mut img, hero_x + 250, row1_y, 200, 32, 8.0, tags[1].1);
    draw_rounded_rect(&mut img, hero_x + 250, row1_y, 200, 32, 8.0, Rgba([tags[1].2[0], tags[1].2[1], tags[1].2[2], 90]));
    draw_text_centered(&mut img, &fonts.bold, tags[1].0, (hero_x + 250 + 100) as f32, (row1_y + 16) as f32, 13.0, tags[1].2);

    let row2_y = hero_y + 310;
    // Tag 3 & 4
    draw_rounded_rect(&mut img, hero_x + 40, row2_y, 195, 32, 8.0, tags[2].1);
    draw_rounded_rect(&mut img, hero_x + 40, row2_y, 195, 32, 8.0, Rgba([tags[2].2[0], tags[2].2[1], tags[2].2[2], 90]));
    draw_text_centered(&mut img, &fonts.bold, tags[2].0, (hero_x + 40 + 97) as f32, (row2_y + 16) as f32, 13.0, tags[2].2);

    draw_rounded_rect(&mut img, hero_x + 250, row2_y, 200, 32, 8.0, tags[3].1);
    draw_rounded_rect(&mut img, hero_x + 250, row2_y, 200, 32, 8.0, Rgba([tags[3].2[0], tags[3].2[1], tags[3].2[2], 90]));
    draw_text_centered(&mut img, &fonts.bold, tags[3].0, (hero_x + 250 + 100) as f32, (row2_y + 16) as f32, 13.0, tags[3].2);

    // Mini preview pill inside Hero
    let hero_stat_y = hero_y + 365;
    draw_rounded_rect(&mut img, hero_x + 40, hero_stat_y, 195, 52, 10.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, hero_x + 40, hero_stat_y, 195, 52, 10.0, Rgba([239, 68, 68, 220]));
    draw_user_icon(&mut img, (hero_x + 65) as f32, (hero_stat_y + 26) as f32, 18.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", (hero_x + 85) as f32, (hero_stat_y + 17) as f32, 18.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (hero_x + 130) as f32, (hero_stat_y + 26) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1 since launch", (hero_x + 140) as f32, (hero_stat_y + 19) as f32, 12.0, Rgba([252, 165, 165, 255]));

    draw_rounded_rect(&mut img, hero_x + 250, hero_stat_y, 200, 52, 10.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, hero_x + 250, hero_stat_y, 200, 52, 10.0, Rgba([239, 68, 68, 220]));
    draw_eye_icon(&mut img, (hero_x + 275) as f32, (hero_stat_y + 26) as f32, 18.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.5K", (hero_x + 295) as f32, (hero_stat_y + 17) as f32, 18.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (hero_x + 355) as f32, (hero_stat_y + 26) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15 views", (hero_x + 365) as f32, (hero_stat_y + 19) as f32, 12.0, Rgba([252, 165, 165, 255]));

    // Author line & repo link
    let footer_y = hero_y + 440;
    draw_rounded_rect(&mut img, hero_x + 40, footer_y, hero_w - 80, 1, 0.0, Rgba([255, 255, 255, 20]));
    draw_circle(&mut img, (hero_x + 60) as f32, (footer_y + 28) as f32, 4.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (hero_x + 60) as f32, (footer_y + 28) as f32, 8.0, 1.5, Rgba([34, 197, 94, 90]));
    draw_text(&mut img, &fonts.bold, "by Matteo Baccan", (hero_x + 76) as f32, (footer_y + 21) as f32, 13.5, Rgba([241, 245, 249, 255]));
    draw_text(&mut img, &fonts.regular, "·  github.com/matteobaccan/SocialCount", (hero_x + 205) as f32, (footer_y + 22) as f32, 13.0, Rgba([148, 163, 184, 210]));

    // ==========================================
    // RIGHT SIDE: REALISTIC APP MOCKUP (Normal Mode)
    // ==========================================
    let mock_x = 610;
    let mock_y = 65;
    let mock_w = 460;
    let mock_h = 510;

    // Window shadow
    draw_rounded_rect(&mut img, mock_x - 8, mock_y + 12, mock_w + 16, mock_h + 16, 26.0, Rgba([0, 0, 0, 150]));
    // Window body
    draw_rounded_rect(&mut img, mock_x, mock_y, mock_w, mock_h, 20.0, Rgba([18, 20, 28, 240]));
    draw_rounded_rect(&mut img, mock_x, mock_y, mock_w, mock_h, 20.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, mock_x + 1, mock_y + 1, mock_w - 2, mock_h - 2, 19.0, Rgba([18, 20, 28, 245]));

    // Header mockup
    draw_youtube_badge(&mut img, (mock_x + 28) as f32, (mock_y + 26) as f32, 22.0, 15.0);
    draw_text(&mut img, &fonts.bold, "SocialCount", (mock_x + 48) as f32, (mock_y + 17) as f32, 16.0, Rgba([255, 255, 255, 240]));

    draw_rounded_rect(&mut img, mock_x + 290, mock_y + 23, 60, 6, 3.0, Rgba([70, 75, 90, 220]));
    draw_circle(&mut img, (mock_x + 330) as f32, (mock_y + 26) as f32, 6.0, Rgba([240, 245, 255, 240]));
    draw_text(&mut img, &fonts.regular, "🌓", (mock_x + 356) as f32, (mock_y + 18) as f32, 13.0, Rgba([175, 185, 200, 255]));

    draw_text(&mut img, &fonts.regular, "⟳", (mock_x + 386) as f32, (mock_y + 17) as f32, 16.0, Rgba([180, 205, 240, 255]));
    draw_text(&mut img, &fonts.regular, "✕", (mock_x + 420) as f32, (mock_y + 18) as f32, 13.0, Rgba([140, 145, 160, 255]));

    // Divider
    draw_rounded_rect(&mut img, mock_x + 16, mock_y + 50, mock_w - 32, 1, 0.0, Rgba([255, 255, 255, 22]));

    // Channel Row 1: @matteobaccan
    let row1_top = mock_y + 66;
    draw_rounded_rect(&mut img, mock_x + 16, row1_top, mock_w - 32, 98, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, mock_x + 16, row1_top, mock_w - 32, 98, 14.0, Rgba([255, 255, 255, 25]));

    // Avatar
    draw_circle(&mut img, (mock_x + 52) as f32, (row1_top + 49) as f32, 26.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (mock_x + 52) as f32, (row1_top + 49) as f32, 26.0, 1.5, Rgba([255, 255, 255, 70]));
    draw_text_centered(&mut img, &fonts.bold, "M", (mock_x + 52) as f32, (row1_top + 49) as f32, 22.0, Rgba([255, 255, 255, 255]));

    // Title and Handle
    draw_text(&mut img, &fonts.bold, "Matteo Baccan", (mock_x + 92) as f32, (row1_top + 16) as f32, 16.0, Rgba([255, 255, 255, 255]));
    draw_text(&mut img, &fonts.regular, "@matteobaccan", (mock_x + 235) as f32, (row1_top + 19) as f32, 13.0, Rgba([148, 163, 184, 220]));

    // Users counter box
    draw_rounded_rect(&mut img, mock_x + 92, row1_top + 48, 155, 32, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, mock_x + 92, row1_top + 48, 155, 32, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, mock_x + 93, row1_top + 49, 153, 30, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (mock_x + 112) as f32, (row1_top + 64) as f32, 15.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", (mock_x + 130) as f32, (row1_top + 56) as f32, 15.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (mock_x + 175) as f32, (row1_top + 64) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1", (mock_x + 186) as f32, (row1_top + 57) as f32, 13.0, Rgba([252, 165, 165, 255]));

    // Views counter box
    draw_rounded_rect(&mut img, mock_x + 260, row1_top + 48, 155, 32, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, mock_x + 260, row1_top + 48, 155, 32, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, mock_x + 261, row1_top + 49, 153, 30, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (mock_x + 280) as f32, (row1_top + 64) as f32, 15.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.507", (mock_x + 298) as f32, (row1_top + 56) as f32, 15.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (mock_x + 355) as f32, (row1_top + 64) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15", (mock_x + 366) as f32, (row1_top + 57) as f32, 13.0, Rgba([252, 165, 165, 255]));

    // Channel Row 2: @rustlang
    let row2_top = mock_y + 176;
    draw_rounded_rect(&mut img, mock_x + 16, row2_top, mock_w - 32, 98, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, mock_x + 16, row2_top, mock_w - 32, 98, 14.0, Rgba([255, 255, 255, 25]));

    draw_circle(&mut img, (mock_x + 52) as f32, (row2_top + 49) as f32, 26.0, Rgba([140, 60, 40, 255]));
    draw_circle_stroke(&mut img, (mock_x + 52) as f32, (row2_top + 49) as f32, 26.0, 1.5, Rgba([255, 255, 255, 70]));
    draw_text_centered(&mut img, &fonts.bold, "R", (mock_x + 52) as f32, (row2_top + 49) as f32, 22.0, Rgba([255, 255, 255, 255]));

    draw_text(&mut img, &fonts.bold, "Rust Foundation", (mock_x + 92) as f32, (row2_top + 16) as f32, 16.0, Rgba([255, 255, 255, 255]));
    draw_text(&mut img, &fonts.regular, "@rustlang", (mock_x + 250) as f32, (row2_top + 19) as f32, 13.0, Rgba([148, 163, 184, 220]));

    // Rust Subscribers
    draw_rounded_rect(&mut img, mock_x + 92, row2_top + 48, 155, 32, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, mock_x + 92, row2_top + 48, 155, 32, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, mock_x + 93, row2_top + 49, 153, 30, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (mock_x + 112) as f32, (row2_top + 64) as f32, 15.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "148K", (mock_x + 130) as f32, (row2_top + 56) as f32, 15.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (mock_x + 178) as f32, (row2_top + 64) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+350", (mock_x + 189) as f32, (row2_top + 57) as f32, 12.0, Rgba([252, 165, 165, 255]));

    // Rust Views
    draw_rounded_rect(&mut img, mock_x + 260, row2_top + 48, 155, 32, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, mock_x + 260, row2_top + 48, 155, 32, 8.0, Rgba([239, 68, 68, 240]));
    draw_rounded_rect(&mut img, mock_x + 261, row2_top + 49, 153, 30, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (mock_x + 280) as f32, (row2_top + 64) as f32, 15.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "1.8M", (mock_x + 298) as f32, (row2_top + 56) as f32, 15.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (mock_x + 350) as f32, (row2_top + 64) as f32, 4.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+4.2K", (mock_x + 361) as f32, (row2_top + 57) as f32, 12.0, Rgba([252, 165, 165, 255]));

    // Footer
    draw_rounded_rect(&mut img, mock_x + 16, mock_y + mock_h - 40, mock_w - 32, 1, 0.0, Rgba([255, 255, 255, 18]));
    draw_circle(&mut img, (mock_x + 155) as f32, (mock_y + mock_h - 22) as f32, 4.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (mock_x + 155) as f32, (mock_y + mock_h - 22) as f32, 7.0, 1.5, Rgba([34, 197, 94, 90]));
    draw_text(&mut img, &fonts.regular, "Updated: 16:45:00", (mock_x + 172) as f32, (mock_y + mock_h - 29) as f32, 12.0, Rgba([148, 163, 184, 220]));

    // ==========================================
    // FLOATING ULTRA-COMPACT WIDGET (Bottom Right Overlay)
    // ==========================================
    let cw_x = mock_x + 145;
    let cw_y = mock_y + 300;
    let cw_w = 295;
    let cw_h = 175;

    // Deep shadow for floating look
    draw_rounded_rect(&mut img, cw_x - 10, cw_y + 12, cw_w + 20, cw_h + 20, 22.0, Rgba([0, 0, 0, 180]));
    // Compact widget frame
    draw_rounded_rect(&mut img, cw_x, cw_y, cw_w, cw_h, 16.0, Rgba([15, 18, 25, 248]));
    draw_rounded_rect(&mut img, cw_x, cw_y, cw_w, cw_h, 16.0, Rgba([255, 255, 255, 45]));
    draw_rounded_rect(&mut img, cw_x + 1, cw_y + 1, cw_w - 2, cw_h - 2, 15.0, Rgba([15, 18, 25, 250]));

    // Compact handle
    let cw_cx = (cw_x + cw_w / 2) as f32;
    draw_rounded_rect(&mut img, (cw_cx - 18.0) as i32, cw_y + 6, 36, 3, 1.5, Rgba([255, 255, 255, 140]));
    draw_text(&mut img, &fonts.bold, "ULTRA-COMPACT MODE", (cw_x + 16) as f32, (cw_y + 14) as f32, 9.5, Rgba([148, 163, 184, 200]));
    draw_text(&mut img, &fonts.regular, "✕", (cw_x + cw_w - 16) as f32, (cw_y + 5) as f32, 10.0, Rgba([140, 145, 160, 255]));

    // Compact row 1
    let cr1_y = cw_y + 30;
    draw_rounded_rect(&mut img, cw_x + 8, cr1_y, cw_w - 16, 40, 8.0, Rgba([24, 28, 38, 240]));

    // Left pill
    draw_rounded_rect(&mut img, cw_x + 14, cr1_y + 7, 95, 26, 6.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, cw_x + 14, cr1_y + 7, 95, 26, 6.0, Rgba([239, 68, 68, 220]));
    draw_user_icon(&mut img, (cw_x + 28) as f32, (cr1_y + 20) as f32, 12.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "26", (cw_x + 42) as f32, (cr1_y + 13) as f32, 12.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (cw_x + 72) as f32, (cr1_y + 20) as f32, 3.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+1", (cw_x + 80) as f32, (cr1_y + 14) as f32, 10.5, Rgba([252, 165, 165, 255]));

    // Center avatar
    draw_circle(&mut img, cw_cx, (cr1_y + 20) as f32, 15.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, cw_cx, (cr1_y + 20) as f32, 15.0, 1.5, Rgba([255, 255, 255, 80]));
    draw_text_centered(&mut img, &fonts.bold, "M", cw_cx, (cr1_y + 20) as f32, 13.0, Rgba([255, 255, 255, 255]));

    // Right pill
    draw_rounded_rect(&mut img, cw_x + cw_w - 109, cr1_y + 7, 95, 26, 6.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, cw_x + cw_w - 109, cr1_y + 7, 95, 26, 6.0, Rgba([239, 68, 68, 220]));
    draw_eye_icon(&mut img, (cw_x + cw_w - 95) as f32, (cr1_y + 20) as f32, 13.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "2.5K", (cw_x + cw_w - 81) as f32, (cr1_y + 13) as f32, 12.0, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (cw_x + cw_w - 43) as f32, (cr1_y + 20) as f32, 3.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+15", (cw_x + cw_w - 35) as f32, (cr1_y + 14) as f32, 10.5, Rgba([252, 165, 165, 255]));

    // Compact row 2
    let cr2_y = cw_y + 76;
    draw_rounded_rect(&mut img, cw_x + 8, cr2_y, cw_w - 16, 40, 8.0, Rgba([24, 28, 38, 240]));

    // Left pill
    draw_rounded_rect(&mut img, cw_x + 14, cr2_y + 7, 95, 26, 6.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, cw_x + 14, cr2_y + 7, 95, 26, 6.0, Rgba([239, 68, 68, 220]));
    draw_user_icon(&mut img, (cw_x + 28) as f32, (cr2_y + 20) as f32, 12.0, Rgba([52, 211, 153, 255]));
    draw_text(&mut img, &fonts.bold, "148K", (cw_x + 42) as f32, (cr2_y + 13) as f32, 11.0, Rgba([209, 250, 229, 255]));
    draw_circle(&mut img, (cw_x + 72) as f32, (cr2_y + 20) as f32, 3.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+350", (cw_x + 78) as f32, (cr2_y + 14) as f32, 9.5, Rgba([252, 165, 165, 255]));

    // Center avatar
    draw_circle(&mut img, cw_cx, (cr2_y + 20) as f32, 15.0, Rgba([140, 60, 40, 255]));
    draw_circle_stroke(&mut img, cw_cx, (cr2_y + 20) as f32, 15.0, 1.5, Rgba([255, 255, 255, 80]));
    draw_text_centered(&mut img, &fonts.bold, "R", cw_cx, (cr2_y + 20) as f32, 13.0, Rgba([255, 255, 255, 255]));

    // Right pill
    draw_rounded_rect(&mut img, cw_x + cw_w - 109, cr2_y + 7, 95, 26, 6.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, cw_x + cw_w - 109, cr2_y + 7, 95, 26, 6.0, Rgba([239, 68, 68, 220]));
    draw_eye_icon(&mut img, (cw_x + cw_w - 95) as f32, (cr2_y + 20) as f32, 13.0, Rgba([96, 165, 250, 255]));
    draw_text(&mut img, &fonts.bold, "1.8M", (cw_x + cw_w - 81) as f32, (cr2_y + 13) as f32, 11.5, Rgba([224, 242, 254, 255]));
    draw_circle(&mut img, (cw_x + cw_w - 43) as f32, (cr2_y + 20) as f32, 3.0, Rgba([239, 68, 68, 255]));
    draw_text(&mut img, &fonts.bold, "+4K", (cw_x + cw_w - 35) as f32, (cr2_y + 14) as f32, 10.0, Rgba([252, 165, 165, 255]));

    // Subtle compact tip
    draw_text_centered(&mut img, &fonts.regular, "Double-click header to switch mode", cw_cx, (cw_y + 142) as f32, 10.0, Rgba([148, 163, 184, 180]));

    img.save(path).expect("Failed to save banner");
    println!("Saved: {}", path.display());
}

fn main() {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).expect("Failed to create assets dir");
    }

    let (bold_data, reg_data) = load_system_fonts();
    let bold = FontRef::try_from_slice(&bold_data).expect("Invalid bold font");
    let regular = FontRef::try_from_slice(&reg_data).expect("Invalid regular font");
    let fonts = Fonts { bold, regular };

    generate_logo(&assets_dir.join("logo.png"), &fonts);
    generate_normal_screenshot(&assets_dir.join("screenshot-normal.png"), &fonts);
    generate_compact_screenshot(&assets_dir.join("screenshot-compact.png"), &fonts);
    generate_banner(&assets_dir.join("socialcount-banner.png"), &fonts);

    println!("All assets successfully generated with typography and delta indicators!");
}
