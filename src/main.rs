// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod icons;
mod youtube;

use app::SocialCountApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SocialCount - YouTube Channels")
            .with_inner_size([360.0, 440.0])
            .with_resizable(false)
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
