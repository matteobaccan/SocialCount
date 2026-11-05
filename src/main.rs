// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod db;
mod icons;
mod youtube;

use app::SocialCountApp;
use config::Config;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let config = Config::load_or_create();
    let initial_size = if config.compact_mode {
        let compact_h = (22.0 + config.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
        egui::Vec2::new(215.0, compact_h)
    } else {
        egui::Vec2::new(
            config.window_width.max(280.0),
            config.window_height.max(160.0),
        )
    };

    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../assets/logo.png")).ok();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("SocialCount - YouTube Channels")
        .with_inner_size(initial_size)
        .with_resizable(false)
        .with_maximize_button(false)
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top();

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SocialCount",
        options,
        Box::new(|cc| Ok(Box::new(SocialCountApp::new(cc)))),
    )
}
