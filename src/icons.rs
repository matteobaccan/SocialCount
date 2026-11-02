// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};
use eframe::epaint::QuadraticBezierShape;

/// Disegna il logo YouTube vettoriale
pub fn draw_youtube_icon(painter: &Painter, rect: Rect) {
    let center = rect.center();
    let width = 20.0_f32;
    let height = 14.0_f32;

    let badge_rect = Rect::from_center_size(center, Vec2::new(width, height));

    // Badge rosso con angoli arrotondati
    let red_color = Color32::from_rgb(255, 35, 35);
    painter.rect_filled(badge_rect, 4.0_f32, red_color);

    // Bordo sottile di riflesso
    painter.rect_stroke(
        badge_rect,
        4.0_f32,
        Stroke::new(1.0_f32, Color32::from_white_alpha(40)),
    );

    // Triangolo play bianco al centro
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

/// Disegna l'icona utente/iscritti vettoriale (stile minimalista moderno)
pub fn draw_user_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6_f32, color);
    let center = rect.center();
    let h = rect.height();

    // Testa: cerchio con riempimento semitrasparente e contorno netto
    let head_center = Pos2::new(center.x, center.y - h * 0.17);
    let head_radius = h * 0.23;
    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60);

    painter.circle_filled(head_center, head_radius, fill_color);
    painter.circle_stroke(head_center, head_radius, stroke);

    // Spalle: arco curvo
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

/// Disegna l'icona occhio/visualizzazioni vettoriale (con pupilla e punto luce)
pub fn draw_eye_icon(painter: &Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.6_f32, color);
    let center = rect.center();
    let half_w = rect.width() * 0.44;
    let half_h = rect.height() * 0.28;

    let p_left = Pos2::new(center.x - half_w, center.y);
    let p_right = Pos2::new(center.x + half_w, center.y);

    // Palpebra superiore
    let ctrl_top = Pos2::new(center.x, center.y - half_h * 1.55);
    painter.add(QuadraticBezierShape::from_points_stroke(
        [p_left, ctrl_top, p_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    // Palpebra inferiore
    let ctrl_bot = Pos2::new(center.x, center.y + half_h * 1.55);
    painter.add(QuadraticBezierShape::from_points_stroke(
        [p_left, ctrl_bot, p_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    // Iride / Pupilla
    let pupil_radius = half_h * 0.85;
    painter.circle_filled(center, pupil_radius, color);

    // Punto luce (catchlight) bianco per profondità visiva
    let light_center = Pos2::new(center.x - pupil_radius * 0.35, center.y - pupil_radius * 0.35);
    painter.circle_filled(light_center, pupil_radius * 0.35, Color32::WHITE);
}

/// Disegna un indicatore di stato live pulsante (dot verde o ambra)
pub fn draw_live_dot(painter: &Painter, center: Pos2, is_loading: bool, time: f64) {
    let (base_color, halo_color) = if is_loading {
        (Color32::from_rgb(251, 191, 36), Color32::from_rgba_unmultiplied(251, 191, 36, 70))
    } else {
        (Color32::from_rgb(52, 211, 153), Color32::from_rgba_unmultiplied(52, 211, 153, 70))
    };

    // Pulsazione periodica
    let pulse = ((time * 3.0).sin() * 0.5 + 0.5) as f32;
    let halo_radius = 4.0_f32 + pulse * 2.0_f32;

    painter.circle_filled(center, halo_radius, halo_color);
    painter.circle_filled(center, 3.0_f32, base_color);
    painter.circle_filled(center, 1.2_f32, Color32::WHITE);
}
