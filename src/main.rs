// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod icons;
mod youtube;

use app::SocialCountApp;
use config::Config;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let config = Config::load_or_create();
    let initial_size = if config.compact_mode {
        let compact_h = (20.0 + config.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
        egui::Vec2::new(205.0, compact_h)
    } else {
        egui::Vec2::new(config.window_width.max(280.0), config.window_height.max(160.0))
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SocialCount - YouTube Channels")
            .with_inner_size(initial_size)
            .with_min_inner_size(initial_size)
            .with_max_inner_size(initial_size)
            .with_resizable(true)
            .with_maximize_button(false)
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "SocialCount",
        options,
        Box::new(|cc| Ok(Box::new(SocialCountApp::new(cc)))),
    )
}
