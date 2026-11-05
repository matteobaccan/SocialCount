// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};
use eframe::epaint::QuadraticBezierShape;

/// Draws the vector YouTube badge logo
pub fn draw_youtube_icon(painter: &Painter, rect: Rect) {
    let center = rect.center();
    let width = 20.0_f32;
    let height = 14.0_f32;

    let badge_rect = Rect::from_center_size(center, Vec2::new(width, height));

    // Red badge with rounded corners
    let red_color = Color32::from_rgb(255, 35, 35);
    painter.rect_filled(badge_rect, 4.0_f32, red_color);

    // Subtle reflection stroke
    painter.rect_stroke(
        badge_rect,
        4.0_f32,
        Stroke::new(1.0_f32, Color32::from_white_alpha(40)),
    );

    // Centered white play triangle
    let tri_h = 3.5_f32;
    let tri_w = 4.0_f32;
    let p0 = Pos2::new(center.x - tri_w * 0.45, center.y - tri_h);
    let p1 = Pos2::new(center.x - tri_w * 0.45, center.y + tri_h);
    let p2 = Pos2::new(center.x + tri_w * 0.65, center.y);

    painter.add(egui::Shape::convex_polygon(
        vec![p0, p1, p2],
        Color32::WHITE,
        Stroke::NONE,
    ));
}

/// Draws modern minimalist vector user/subscriber icon
pub fn draw_user_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6_f32, color);
    let center = rect.center();
    let h = rect.height();

    // Head: circle with semitransparent fill and clean outline
    let head_center = Pos2::new(center.x, center.y - h * 0.17);
    let head_radius = h * 0.23;
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60);

    painter.circle_filled(head_center, head_radius, fill_color);
    painter.circle_stroke(head_center, head_radius, stroke);

    // Shoulders: curved arc
    let y_bottom = center.y + h * 0.38;
    let y_ctrl = center.y + h * 0.08;
    let half_w = rect.width() * 0.39;

    let p0 = Pos2::new(center.x - half_w, y_bottom);
    let p1 = Pos2::new(center.x, y_ctrl);
    let p2 = Pos2::new(center.x + half_w, y_bottom);

    painter.add(QuadraticBezierShape::from_points_stroke(
        [p0, p1, p2],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
}

/// Draws vector eye/views icon (with pupil and catchlight highlight)
pub fn draw_eye_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6_f32, color);
    let center = rect.center();
    let half_w = rect.width() * 0.44;
    let half_h = rect.height() * 0.28;

    let p_left = Pos2::new(center.x - half_w, center.y);
    let p_right = Pos2::new(center.x + half_w, center.y);

    // Upper eyelid
    let ctrl_top = Pos2::new(center.x, center.y - half_h * 1.55);
    painter.add(QuadraticBezierShape::from_points_stroke(
        [p_left, ctrl_top, p_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    // Lower eyelid
    let ctrl_bot = Pos2::new(center.x, center.y + half_h * 1.55);
    painter.add(QuadraticBezierShape::from_points_stroke(
        [p_left, ctrl_bot, p_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    // Iris / Pupil
    let pupil_radius = half_h * 0.70;
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 75);
    painter.circle_filled(center, pupil_radius, fill_color);
    painter.circle_stroke(center, pupil_radius, stroke);

    // Tiny white catchlight reflection
    let catchlight_center = Pos2::new(
        center.x - pupil_radius * 0.35,
        center.y - pupil_radius * 0.35,
    );
    painter.circle_filled(catchlight_center, 1.2_f32, Color32::WHITE);
}

/// Draws pulsing emerald green live status dot
pub fn draw_live_dot(painter: &Painter, center: Pos2, is_loading: bool, time: f64) {
    if is_loading {
        // Yellow pulsing dot when loading
        let pulse = ((time * 6.0).sin() * 0.5 + 0.5) as f32;
        let radius = 3.5_f32 + pulse * 1.5_f32;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(251, 191, 36, 120),
        );
        painter.circle_filled(center, 3.0_f32, Color32::from_rgb(251, 191, 36));
    } else {
        // Emerald pulsing dot when live
        let pulse = ((time * 2.8).sin() * 0.5 + 0.5) as f32;
        let halo_radius = 4.0_f32 + pulse * 2.5_f32;
        let alpha = (120.0 * (1.0 - pulse)).round() as u8;
        painter.circle_filled(
            center,
            halo_radius,
            Color32::from_rgba_unmultiplied(34, 197, 94, alpha),
        );
        painter.circle_filled(center, 3.2_f32, Color32::from_rgb(34, 197, 94));
    }
}

/// Draws vector half-moon opacity icon
pub fn draw_moon_icon(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    painter.circle_stroke(center, radius, Stroke::new(1.2_f32, color));
    // Fill left semicircle
    let steps = 16;
    let mut points = Vec::with_capacity(steps + 2);
    points.push(center);
    for i in 0..=steps {
        let angle = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (i as f32 / steps as f32);
        points.push(Pos2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
    points.push(center);
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

/// Draws vector gear / settings icon
pub fn draw_gear_icon(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let teeth = 6;
    let r_inner = radius * 0.70_f32;
    let r_outer = radius;
    let total_steps = teeth * 2;
    let mut points = Vec::with_capacity(total_steps);
    for i in 0..total_steps {
        let angle = std::f32::consts::TAU * (i as f32 / total_steps as f32);
        let r = if i % 2 == 0 { r_outer } else { r_inner };
        points.push(Pos2::new(
            center.x + r * angle.cos(),
            center.y + r * angle.sin(),
        ));
    }
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    painter.circle_filled(center, radius * 0.38_f32, Color32::from_rgb(18, 20, 28));
}

/// Draws vector video camera icon
pub fn draw_video_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.4_f32, color);
    let center = rect.center();
    let body_w = rect.width() * 0.54;
    let body_h = rect.height() * 0.56;
    let body_rect = Rect::from_center_size(
        Pos2::new(center.x - rect.width() * 0.12, center.y),
        Vec2::new(body_w, body_h),
    );
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 65);
    painter.rect_filled(body_rect, 3.0_f32, fill_color);
    painter.rect_stroke(body_rect, 3.0_f32, stroke);

    // Lens trapezoid on the right
    let p0 = Pos2::new(body_rect.right(), center.y - body_h * 0.28);
    let p1 = Pos2::new(center.x + rect.width() * 0.42, center.y - body_h * 0.48);
    let p2 = Pos2::new(center.x + rect.width() * 0.42, center.y + body_h * 0.48);
    let p3 = Pos2::new(body_rect.right(), center.y + body_h * 0.28);
    painter.add(egui::Shape::convex_polygon(
        vec![p0, p1, p2, p3],
        fill_color,
        stroke,
    ));
}

/// Draws vector speech / comment bubble icon
pub fn draw_comment_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.4_f32, color);
    let center = rect.center();
    let bubble_w = rect.width() * 0.76;
    let bubble_h = rect.height() * 0.56;
    let bubble_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y - rect.height() * 0.08),
        Vec2::new(bubble_w, bubble_h),
    );
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 65);
    painter.rect_filled(bubble_rect, 3.5_f32, fill_color);
    painter.rect_stroke(bubble_rect, 3.5_f32, stroke);

    // Tail at bottom left
    let t0 = Pos2::new(
        bubble_rect.left() + bubble_w * 0.22,
        bubble_rect.bottom() - 1.0,
    );
    let t1 = Pos2::new(
        bubble_rect.left() + bubble_w * 0.10,
        center.y + rect.height() * 0.44,
    );
    let t2 = Pos2::new(
        bubble_rect.left() + bubble_w * 0.48,
        bubble_rect.bottom() - 1.0,
    );
    painter.add(egui::Shape::convex_polygon(
        vec![t0, t1, t2],
        fill_color,
        stroke,
    ));
}

/// Draws vector sort arrow (chevron pointing up for Ascending, down for Descending)
pub fn draw_sort_arrow(painter: &Painter, center: Pos2, ascending: bool, color: Color32) {
    let w = 3.2_f32;
    let h = 2.2_f32;
    let stroke = Stroke::new(1.4_f32, color);
    if ascending {
        // Pointing up: ^
        let p0 = Pos2::new(center.x - w, center.y + h * 0.5);
        let p1 = Pos2::new(center.x, center.y - h * 0.5);
        let p2 = Pos2::new(center.x + w, center.y + h * 0.5);
        painter.line_segment([p0, p1], stroke);
        painter.line_segment([p1, p2], stroke);
    } else {
        // Pointing down: v
        let p0 = Pos2::new(center.x - w, center.y - h * 0.5);
        let p1 = Pos2::new(center.x, center.y + h * 0.5);
        let p2 = Pos2::new(center.x + w, center.y - h * 0.5);
        painter.line_segment([p0, p1], stroke);
        painter.line_segment([p1, p2], stroke);
    }
}

/// Draws vector heart / like icon
pub fn draw_like_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.3_f32, color);
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 75);
    let center = rect.center();
    let w = rect.width() * 0.72;
    let h = rect.height() * 0.68;

    let r = w * 0.25;
    let left_center = Pos2::new(center.x - r, center.y - h * 0.12);
    let right_center = Pos2::new(center.x + r, center.y - h * 0.12);
    painter.circle_filled(left_center, r, fill_color);
    painter.circle_filled(right_center, r, fill_color);
    painter.circle_stroke(left_center, r, stroke);
    painter.circle_stroke(right_center, r, stroke);

    let bottom = Pos2::new(center.x, center.y + h * 0.46);
    let p_left = Pos2::new(center.x - w * 0.48, center.y - h * 0.08);
    let p_right = Pos2::new(center.x + w * 0.48, center.y - h * 0.08);
    painter.add(egui::Shape::convex_polygon(
        vec![p_left, bottom, p_right],
        fill_color,
        Stroke::NONE,
    ));
    painter.line_segment([p_left, bottom], stroke);
    painter.line_segment([bottom, p_right], stroke);
}
