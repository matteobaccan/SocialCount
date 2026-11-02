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

            // Distanza dal bordo arrotondato
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
    let outer = radius + stroke_w / 2.0;
    let inner = radius - stroke_w / 2.0;

    let min_x = (cx - outer - 1.0).max(0.0) as u32;
    let max_x = (cx + outer + 1.0).min(iw as f32 - 1.0) as u32;
    let min_y = (cy - outer - 1.0).max(0.0) as u32;
    let max_y = (cy + outer + 1.0).min(ih as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist >= inner - 0.5 && dist <= outer + 0.5 {
                let factor = if dist < inner + 0.5 {
                    (dist - (inner - 0.5)).clamp(0.0, 1.0)
                } else if dist > outer - 0.5 {
                    ((outer + 0.5) - dist).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let mut aa = color;
                aa[3] = ((color[3] as f32) * factor).round() as u8;
                blend_pixel(img.get_pixel_mut(x, y), aa);
            }
        }
    }
}

fn draw_triangle(img: &mut RgbaImage, p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), color: Rgba<u8>) {
    let (iw, ih) = img.dimensions();
    let min_x = p1.0.min(p2.0).min(p3.0).max(0.0) as u32;
    let max_x = p1.0.max(p2.0).max(p3.0).min(iw as f32 - 1.0) as u32;
    let min_y = p1.1.min(p2.1).min(p3.1).max(0.0) as u32;
    let max_y = p1.1.max(p2.1).max(p3.1).min(ih as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w1 = ((p2.1 - p3.1) * (px - p3.0) + (p3.0 - p2.0) * (py - p3.1))
                / ((p2.1 - p3.1) * (p1.0 - p3.0) + (p3.0 - p2.0) * (p1.1 - p3.1));
            let w2 = ((p3.1 - p1.1) * (px - p3.0) + (p1.0 - p3.0) * (py - p3.1))
                / ((p2.1 - p3.1) * (p1.0 - p3.0) + (p3.0 - p2.0) * (p1.1 - p3.1));
            let w3 = 1.0 - w1 - w2;

            if w1 >= 0.0 && w2 >= 0.0 && w3 >= 0.0 {
                blend_pixel(img.get_pixel_mut(x, y), color);
            }
        }
    }
}

fn draw_youtube_badge(img: &mut RgbaImage, cx: f32, cy: f32, w: f32, h: f32) {
    let rx = (cx - w / 2.0).round() as i32;
    let ry = (cy - h / 2.0).round() as i32;
    draw_rounded_rect(img, rx, ry, w as i32, h as i32, h * 0.28, Rgba([255, 33, 33, 255]));

    let tw = w * 0.32;
    let th = h * 0.40;
    let p1 = (cx - tw * 0.4, cy - th * 0.5);
    let p2 = (cx + tw * 0.6, cy);
    let p3 = (cx - tw * 0.4, cy + th * 0.5);
    draw_triangle(img, p1, p2, p3, Rgba([255, 255, 255, 255]));
}

fn draw_user_icon(img: &mut RgbaImage, cx: f32, cy: f32, size: f32, color: Rgba<u8>) {
    let head_r = size * 0.28;
    let head_cy = cy - size * 0.20;
    draw_circle(img, cx, head_cy, head_r, color);

    let body_r = size * 0.45;
    let body_cy = cy + size * 0.50;
    draw_circle(img, cx, body_cy, body_r, color);
}

fn draw_eye_icon(img: &mut RgbaImage, cx: f32, cy: f32, size: f32, color: Rgba<u8>) {
    let _w = size * 1.1;
    let h = size * 0.75;
    draw_circle_stroke(img, cx, cy, h * 0.5, 2.0, color);
    draw_circle(img, cx, cy, h * 0.30, color);
    draw_circle(img, cx + h * 0.12, cy - h * 0.12, h * 0.10, Rgba([255, 255, 255, 240]));
}

/// Genera il logo moderno ad alta risoluzione (512x512)
fn generate_logo(path: &Path) {
    let mut img: RgbaImage = ImageBuffer::new(512, 512);

    // Sfondo dark glass arrotondato con bagliore
    let cx = 256.0_f32;
    let cy = 256.0_f32;

    // Bagliore radiale
    for y in 0..512 {
        for x in 0..512 {
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

    // Shield arrotondato effetto vetro
    draw_rounded_rect(&mut img, 48, 48, 416, 416, 88.0, Rgba([18, 20, 28, 245]));
    draw_rounded_rect(&mut img, 50, 50, 412, 412, 86.0, Rgba([255, 255, 255, 28]));
    draw_rounded_rect(&mut img, 52, 52, 408, 408, 84.0, Rgba([15, 17, 24, 250]));

    // Icona YouTube badge centrale grande
    draw_youtube_badge(&mut img, cx, 195.0, 190.0, 130.0);

    // Box indicatori stilizzati sotto
    // Box verde (utenti)
    draw_rounded_rect(&mut img, 80, 295, 160, 68, 16.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, 82, 297, 156, 64, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, 84, 299, 152, 60, 12.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, 120.0, 329.0, 28.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, 145, 323, 75, 12, 6.0, Rgba([209, 250, 229, 255]));

    // Box blu (viste)
    draw_rounded_rect(&mut img, 272, 295, 160, 68, 16.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, 274, 297, 156, 64, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, 276, 299, 152, 60, 12.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, 312.0, 329.0, 28.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, 337, 323, 75, 12, 6.0, Rgba([224, 242, 254, 255]));

    // Barra Live Dot in basso
    draw_circle(&mut img, cx - 70.0, 395.0, 6.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, cx - 70.0, 395.0, 10.0, 2.0, Rgba([34, 197, 94, 90]));
    draw_rounded_rect(&mut img, (cx - 45.0) as i32, 390, 120, 10, 5.0, Rgba([148, 163, 184, 200]));

    img.save(path).expect("Failed to save logo");
    println!("Saved: {}", path.display());
}

/// Genera screenshot della modalità normale
fn generate_normal_screenshot(path: &Path) {
    let w = 480;
    let h = 580;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Sfondo desktop gradiente elegante (dark wallpaper style)
    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Shadow e widget SocialCount
    let wx = 60;
    let wy = 50;
    let ww = 360;
    let wh = 460;

    // Ombra finestra
    draw_rounded_rect(&mut img, wx - 6, wy + 4, ww + 12, wh + 12, 24.0, Rgba([0, 0, 0, 120]));

    // Finestra frosted glass SocialCount
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([18, 20, 28, 235]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([255, 255, 255, 30]));
    draw_rounded_rect(&mut img, wx + 1, wy + 1, ww - 2, wh - 2, 17.0, Rgba([18, 20, 28, 240]));

    // Header: YouTube icon + Titolo "SocialCount" + slider + refresh + close
    draw_youtube_badge(&mut img, (wx + 26) as f32, (wy + 24) as f32, 20.0, 14.0);
    draw_rounded_rect(&mut img, wx + 42, wy + 18, 90, 12, 4.0, Rgba([255, 255, 255, 230])); // "SocialCount"

    // Slider
    draw_rounded_rect(&mut img, wx + 215, wy + 21, 60, 6, 3.0, Rgba([80, 85, 100, 220]));
    draw_circle(&mut img, (wx + 255) as f32, (wy + 24) as f32, 6.0, Rgba([255, 255, 255, 240]));

    // Refresh icon & Close icon
    draw_circle_stroke(&mut img, (wx + 295) as f32, (wy + 24) as f32, 6.0, 2.0, Rgba([147, 197, 253, 240]));
    draw_circle(&mut img, (wx + 328) as f32, (wy + 24) as f32, 8.0, Rgba([239, 68, 68, 80]));

    // Linea divisoria
    draw_rounded_rect(&mut img, wx + 14, wy + 46, ww - 28, 1, 0.0, Rgba([255, 255, 255, 22]));

    // Canale 1: @matteobaccan
    let c1_y = wy + 60;
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, wx + 14, c1_y, ww - 28, 92, 14.0, Rgba([255, 255, 255, 25]));

    // Avatar rotondo con riflesso
    draw_circle(&mut img, (wx + 44) as f32, (c1_y + 46) as f32, 24.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (wx + 44) as f32, (c1_y + 46) as f32, 24.0, 2.0, Rgba([255, 255, 255, 70]));

    // Nome canale & handle
    draw_rounded_rect(&mut img, wx + 80, c1_y + 16, 120, 12, 4.0, Rgba([255, 255, 255, 240]));
    draw_rounded_rect(&mut img, wx + 80, c1_y + 34, 80, 9, 3.0, Rgba([148, 163, 184, 210]));

    // Box Utenti (sinistra) e Viste (destra) - 100% solidi
    draw_rounded_rect(&mut img, wx + 80, c1_y + 52, 112, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 80, c1_y + 52, 112, 28, 8.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 81, c1_y + 53, 110, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 98) as f32, (c1_y + 66) as f32, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 115, c1_y + 60, 60, 11, 4.0, Rgba([209, 250, 229, 255]));

    draw_rounded_rect(&mut img, wx + 200, c1_y + 52, 112, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 200, c1_y + 52, 112, 28, 8.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 201, c1_y + 53, 110, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 218) as f32, (c1_y + 66) as f32, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 235, c1_y + 60, 60, 11, 4.0, Rgba([224, 242, 254, 255]));

    // Canale 2 (esempio)
    let c2_y = wy + 164;
    draw_rounded_rect(&mut img, wx + 14, c2_y, ww - 28, 92, 14.0, Rgba([26, 29, 39, 235]));
    draw_rounded_rect(&mut img, wx + 14, c2_y, ww - 28, 92, 14.0, Rgba([255, 255, 255, 25]));

    draw_circle(&mut img, (wx + 44) as f32, (c2_y + 46) as f32, 24.0, Rgba([90, 45, 90, 255]));
    draw_circle_stroke(&mut img, (wx + 44) as f32, (c2_y + 46) as f32, 24.0, 2.0, Rgba([255, 255, 255, 70]));

    draw_rounded_rect(&mut img, wx + 80, c2_y + 16, 100, 12, 4.0, Rgba([255, 255, 255, 240]));
    draw_rounded_rect(&mut img, wx + 80, c2_y + 34, 70, 9, 3.0, Rgba([148, 163, 184, 210]));

    draw_rounded_rect(&mut img, wx + 80, c2_y + 52, 112, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 80, c2_y + 52, 112, 28, 8.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 81, c2_y + 53, 110, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 98) as f32, (c2_y + 66) as f32, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 115, c2_y + 60, 50, 11, 4.0, Rgba([209, 250, 229, 255]));

    draw_rounded_rect(&mut img, wx + 200, c2_y + 52, 112, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 200, c2_y + 52, 112, 28, 8.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 201, c2_y + 53, 110, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 218) as f32, (c2_y + 66) as f32, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 235, c2_y + 60, 65, 11, 4.0, Rgba([224, 242, 254, 255]));

    // Footer sticky
    let fy = wy + wh - 36;
    draw_rounded_rect(&mut img, wx + 14, fy, ww - 28, 1, 0.0, Rgba([255, 255, 255, 22]));
    draw_circle(&mut img, (wx + 115) as f32, (fy + 18) as f32, 4.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (wx + 115) as f32, (fy + 18) as f32, 7.0, 1.5, Rgba([34, 197, 94, 110]));
    draw_rounded_rect(&mut img, wx + 130, fy + 13, 140, 9, 3.0, Rgba([148, 163, 184, 220]));

    // Resize grip puntini in basso a destra
    for i in 0..3 {
        draw_circle(&mut img, (wx + ww - 14 + i * 4) as f32, (wy + wh - 6) as f32, 1.5, Rgba([147, 197, 253, 180]));
        draw_circle(&mut img, (wx + ww - 10 + i * 4) as f32, (wy + wh - 10) as f32, 1.5, Rgba([147, 197, 253, 180]));
        draw_circle(&mut img, (wx + ww - 6 + i * 4) as f32, (wy + wh - 14) as f32, 1.5, Rgba([147, 197, 253, 180]));
    }

    img.save(path).expect("Failed to save normal screenshot");
    println!("Saved: {}", path.display());
}

/// Genera screenshot della modalità ultra-compatta
fn generate_compact_screenshot(path: &Path) {
    let w = 400;
    let h = 260;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Desktop background
    for y in 0..h {
        let f = y as f32 / h as f32;
        let r = (14.0 + f * 12.0) as u8;
        let g = (16.0 + f * 18.0) as u8;
        let b = (26.0 + f * 34.0) as u8;
        for x in 0..w {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Ultra-compact widget
    let wx = 95;
    let wy = 80;
    let ww = 210;
    let wh = 96;

    // Ombra
    draw_rounded_rect(&mut img, wx - 4, wy + 3, ww + 8, wh + 8, 12.0, Rgba([0, 0, 0, 110]));

    // Container
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 8.0, Rgba([18, 20, 28, 240]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 8.0, Rgba([255, 255, 255, 30]));

    // Mini header con drag handle e close
    draw_youtube_badge(&mut img, (wx + 14) as f32, (wy + 9) as f32, 12.0, 8.5);
    draw_rounded_rect(&mut img, wx + (ww / 2) - 15, wy + 7, 30, 3, 1.5, Rgba([255, 255, 255, 80]));
    draw_circle(&mut img, (wx + ww - 12) as f32, (wy + 9) as f32, 4.0, Rgba([239, 68, 68, 120]));

    // Canale 1 compatto: [ 👤 26 ]  ( Avatar 28px )  [ 👁 2.507 ]
    let c1_y = wy + 18;
    draw_rounded_rect(&mut img, wx + 6, c1_y, ww - 12, 34, 6.0, Rgba([24, 27, 36, 230]));

    // Box Utenti a sinistra
    draw_rounded_rect(&mut img, wx + 10, c1_y + 6, 68, 22, 5.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 10, c1_y + 6, 68, 22, 5.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 11, c1_y + 7, 66, 20, 4.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 22) as f32, (c1_y + 17) as f32, 10.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 34, c1_y + 13, 36, 8, 3.0, Rgba([209, 250, 229, 255]));

    // Avatar rotondo al centro
    let acx = (wx + ww / 2) as f32;
    draw_circle(&mut img, acx, (c1_y + 17) as f32, 14.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, acx, (c1_y + 17) as f32, 14.0, 1.2, Rgba([255, 255, 255, 90]));

    // Box Viste a destra
    draw_rounded_rect(&mut img, wx + ww - 78, c1_y + 6, 68, 22, 5.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + ww - 78, c1_y + 6, 68, 22, 5.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + ww - 77, c1_y + 7, 66, 20, 4.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + ww - 66) as f32, (c1_y + 17) as f32, 11.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + ww - 54, c1_y + 13, 38, 8, 3.0, Rgba([224, 242, 254, 255]));

    // Canale 2 compatto
    let c2_y = wy + 55;
    draw_rounded_rect(&mut img, wx + 6, c2_y, ww - 12, 34, 6.0, Rgba([24, 27, 36, 230]));

    draw_rounded_rect(&mut img, wx + 10, c2_y + 6, 68, 22, 5.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 10, c2_y + 6, 68, 22, 5.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 11, c2_y + 7, 66, 20, 4.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 22) as f32, (c2_y + 17) as f32, 10.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 34, c2_y + 13, 30, 8, 3.0, Rgba([209, 250, 229, 255]));

    draw_circle(&mut img, acx, (c2_y + 17) as f32, 14.0, Rgba([90, 45, 90, 255]));
    draw_circle_stroke(&mut img, acx, (c2_y + 17) as f32, 14.0, 1.2, Rgba([255, 255, 255, 90]));

    draw_rounded_rect(&mut img, wx + ww - 78, c2_y + 6, 68, 22, 5.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + ww - 78, c2_y + 6, 68, 22, 5.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + ww - 77, c2_y + 7, 66, 20, 4.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + ww - 66) as f32, (c2_y + 17) as f32, 11.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + ww - 54, c2_y + 13, 40, 8, 3.0, Rgba([224, 242, 254, 255]));

    img.save(path).expect("Failed to save compact screenshot");
    println!("Saved: {}", path.display());
}

/// Genera il banner GitHub Social Preview (1280x640)
fn generate_banner(path: &Path) {
    let w = 1280;
    let h = 640;
    let mut img: RgbaImage = ImageBuffer::new(w, h);

    // Sfondo gradiente scuro premium con sfumature rosse e blu/smeraldo
    for y in 0..h {
        let fy = y as f32 / h as f32;
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let d_red = (((fx - 0.2).powi(2) + (fy - 0.4).powi(2)).sqrt() * 2.5).clamp(0.0, 1.0);
            let d_blue = (((fx - 0.8).powi(2) + (fy - 0.6).powi(2)).sqrt() * 2.2).clamp(0.0, 1.0);

            let r = (16.0 + (1.0 - d_red) * 45.0) as u8;
            let g = (18.0 + (1.0 - d_blue) * 20.0) as u8;
            let b = (28.0 + (1.0 - d_blue) * 48.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Pattern a griglia sottile sullo sfondo
    for y in (0..h).step_by(32) {
        for x in 0..w {
            blend_pixel(img.get_pixel_mut(x, y), Rgba([255, 255, 255, 6]));
        }
    }
    for x in (0..w).step_by(32) {
        for y in 0..h {
            blend_pixel(img.get_pixel_mut(x, y), Rgba([255, 255, 255, 6]));
        }
    }

    // LATO SINISTRO: Testi e Branding
    // Logo YouTube grande
    draw_youtube_badge(&mut img, 150.0, 160.0, 110.0, 76.0);

    // Titolo "SocialCount"
    draw_rounded_rect(&mut img, 230, 125, 360, 40, 8.0, Rgba([255, 255, 255, 255]));

    // Sottotitolo "Real-time YouTube Channel Statistics Widget in Rust"
    draw_rounded_rect(&mut img, 100, 235, 480, 16, 5.0, Rgba([220, 228, 245, 220]));
    draw_rounded_rect(&mut img, 100, 265, 420, 14, 4.0, Rgba([148, 163, 184, 200]));

    // Feature Badges
    // 1. Rust Powered
    draw_rounded_rect(&mut img, 100, 315, 140, 36, 18.0, Rgba([249, 115, 22, 40]));
    draw_rounded_rect(&mut img, 100, 315, 140, 36, 18.0, Rgba([249, 115, 22, 180]));
    draw_rounded_rect(&mut img, 125, 328, 90, 10, 3.0, Rgba([255, 237, 213, 255]));

    // 2. Transparent Window
    draw_rounded_rect(&mut img, 255, 315, 165, 36, 18.0, Rgba([147, 197, 253, 35]));
    draw_rounded_rect(&mut img, 255, 315, 165, 36, 18.0, Rgba([147, 197, 253, 180]));
    draw_rounded_rect(&mut img, 280, 328, 115, 10, 3.0, Rgba([239, 246, 255, 255]));

    // 3. Ultra-compact
    draw_rounded_rect(&mut img, 435, 315, 145, 36, 18.0, Rgba([52, 211, 153, 35]));
    draw_rounded_rect(&mut img, 435, 315, 145, 36, 18.0, Rgba([52, 211, 153, 180]));
    draw_rounded_rect(&mut img, 460, 328, 95, 10, 3.0, Rgba([236, 253, 245, 255]));

    // Autore e Repo in basso a sinistra
    draw_rounded_rect(&mut img, 100, 480, 180, 14, 4.0, Rgba([255, 255, 255, 240])); // by Matteo Baccan
    draw_rounded_rect(&mut img, 100, 505, 320, 12, 4.0, Rgba([148, 163, 184, 180])); // github.com/matteobaccan/SocialCount

    // LATO DESTRO: Anteprima mock del widget SocialCount
    let wx = 760;
    let wy = 85;
    let ww = 380;
    let wh = 470;

    // Ombra
    draw_rounded_rect(&mut img, wx - 12, wy + 8, ww + 24, wh + 24, 26.0, Rgba([0, 0, 0, 140]));

    // Widget Window
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([18, 20, 28, 245]));
    draw_rounded_rect(&mut img, wx, wy, ww, wh, 18.0, Rgba([255, 255, 255, 35]));

    // Header
    draw_youtube_badge(&mut img, (wx + 28) as f32, (wy + 26) as f32, 22.0, 15.0);
    draw_rounded_rect(&mut img, wx + 48, wy + 20, 95, 13, 4.0, Rgba([255, 255, 255, 240]));
    draw_rounded_rect(&mut img, wx + 230, wy + 23, 65, 6, 3.0, Rgba([80, 85, 100, 220]));
    draw_circle(&mut img, (wx + 270) as f32, (wy + 26) as f32, 6.0, Rgba([255, 255, 255, 240]));
    draw_circle(&mut img, (wx + 348) as f32, (wy + 26) as f32, 9.0, Rgba([239, 68, 68, 80]));

    // Card 1
    let c1_y = wy + 65;
    draw_rounded_rect(&mut img, wx + 16, c1_y, ww - 32, 95, 14.0, Rgba([26, 29, 39, 245]));
    draw_rounded_rect(&mut img, wx + 16, c1_y, ww - 32, 95, 14.0, Rgba([255, 255, 255, 25]));
    draw_circle(&mut img, (wx + 48) as f32, (c1_y + 48) as f32, 25.0, Rgba([40, 70, 110, 255]));
    draw_circle_stroke(&mut img, (wx + 48) as f32, (c1_y + 48) as f32, 25.0, 2.0, Rgba([255, 255, 255, 80]));
    draw_rounded_rect(&mut img, wx + 86, c1_y + 18, 130, 13, 4.0, Rgba([255, 255, 255, 250]));
    draw_rounded_rect(&mut img, wx + 86, c1_y + 36, 85, 10, 3.0, Rgba([148, 163, 184, 210]));

    draw_rounded_rect(&mut img, wx + 86, c1_y + 54, 116, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 86, c1_y + 54, 116, 28, 8.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 87, c1_y + 55, 114, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 104) as f32, (c1_y + 68) as f32, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 122, c1_y + 62, 60, 11, 4.0, Rgba([209, 250, 229, 255]));

    draw_rounded_rect(&mut img, wx + 212, c1_y + 54, 116, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 212, c1_y + 54, 116, 28, 8.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 213, c1_y + 55, 114, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 230) as f32, (c1_y + 68) as f32, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 248, c1_y + 62, 62, 11, 4.0, Rgba([224, 242, 254, 255]));

    // Card 2
    let c2_y = wy + 175;
    draw_rounded_rect(&mut img, wx + 16, c2_y, ww - 32, 95, 14.0, Rgba([26, 29, 39, 245]));
    draw_rounded_rect(&mut img, wx + 16, c2_y, ww - 32, 95, 14.0, Rgba([255, 255, 255, 25]));
    draw_circle(&mut img, (wx + 48) as f32, (c2_y + 48) as f32, 25.0, Rgba([90, 45, 90, 255]));
    draw_circle_stroke(&mut img, (wx + 48) as f32, (c2_y + 48) as f32, 25.0, 2.0, Rgba([255, 255, 255, 80]));
    draw_rounded_rect(&mut img, wx + 86, c2_y + 18, 110, 13, 4.0, Rgba([255, 255, 255, 250]));
    draw_rounded_rect(&mut img, wx + 86, c2_y + 36, 75, 10, 3.0, Rgba([148, 163, 184, 210]));

    draw_rounded_rect(&mut img, wx + 86, c2_y + 54, 116, 28, 8.0, Rgba([16, 44, 28, 255]));
    draw_rounded_rect(&mut img, wx + 86, c2_y + 54, 116, 28, 8.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 87, c2_y + 55, 114, 26, 7.0, Rgba([16, 44, 28, 255]));
    draw_user_icon(&mut img, (wx + 104) as f32, (c2_y + 68) as f32, 14.0, Rgba([52, 211, 153, 255]));
    draw_rounded_rect(&mut img, wx + 122, c2_y + 62, 52, 11, 4.0, Rgba([209, 250, 229, 255]));

    draw_rounded_rect(&mut img, wx + 212, c2_y + 54, 116, 28, 8.0, Rgba([18, 36, 58, 255]));
    draw_rounded_rect(&mut img, wx + 212, c2_y + 54, 116, 28, 8.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 213, c2_y + 55, 114, 26, 7.0, Rgba([18, 36, 58, 255]));
    draw_eye_icon(&mut img, (wx + 230) as f32, (c2_y + 68) as f32, 14.0, Rgba([96, 165, 250, 255]));
    draw_rounded_rect(&mut img, wx + 248, c2_y + 62, 68, 11, 4.0, Rgba([224, 242, 254, 255]));

    // Footer
    let fy = wy + wh - 40;
    draw_circle(&mut img, (wx + 130) as f32, (fy + 18) as f32, 4.0, Rgba([34, 197, 94, 255]));
    draw_circle_stroke(&mut img, (wx + 130) as f32, (fy + 18) as f32, 7.0, 1.5, Rgba([34, 197, 94, 110]));
    draw_rounded_rect(&mut img, wx + 146, fy + 13, 140, 10, 3.0, Rgba([148, 163, 184, 220]));

    img.save(path).expect("Failed to save banner");
    println!("Saved: {}", path.display());
}

fn main() {
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir).expect("Failed to create assets directory");
    }

    println!("Generating assets...");
    generate_logo(&assets_dir.join("logo.png"));
    generate_banner(&assets_dir.join("socialcount-banner.png"));
    generate_normal_screenshot(&assets_dir.join("screenshot-normal.png"));
    generate_compact_screenshot(&assets_dir.join("screenshot-compact.png"));
    println!("All assets successfully generated in assets/");
}
