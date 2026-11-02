// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use crate::config::Config;
use crate::youtube::{ChannelData, YouTubeFetcher};
use eframe::egui::{self, Color32, Margin, Pos2, Rect, Rounding, Stroke, Vec2};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

enum WorkerMessage {
    ChannelUpdated(Box<ChannelData>),
    RefreshFinished,
}

#[cfg(windows)]
mod win32_drag {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicIsize, Ordering};

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn GetCursorPos(lpPoint: *mut Point) -> i32;
        pub fn GetActiveWindow() -> isize;
        pub fn GetWindowRect(hWnd: isize, lpRect: *mut Rect) -> i32;
        pub fn SetWindowPos(
            hWnd: isize,
            hWndInsertAfter: isize,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: u32,
        ) -> i32;
        pub fn GetAsyncKeyState(vKey: i32) -> i16;
        pub fn GetWindowLongW(hWnd: isize, nIndex: i32) -> i32;
        pub fn SetWindowLongW(hWnd: isize, nIndex: i32, dwNewLong: i32) -> i32;
        pub fn GetWindowLongPtrW(hWnd: isize, nIndex: i32) -> isize;
        pub fn SetWindowLongPtrW(hWnd: isize, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn CallWindowProcW(
            lpPrevWndFunc: isize,
            hWnd: isize,
            Msg: u32,
            wParam: usize,
            lParam: isize,
        ) -> isize;
        pub fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
    }

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        pub fn DwmSetWindowAttribute(
            hwnd: isize,
            dwAttribute: u32,
            pvAttribute: *const c_void,
            cbAttribute: u32,
        ) -> i32;
    }

    pub const VK_LBUTTON: i32 = 0x01;
    pub const SWP_NOSIZE: u32 = 0x0001;
    pub const SWP_NOZORDER: u32 = 0x0004;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_FRAMECHANGED: u32 = 0x0020;

    const GWLP_WNDPROC: i32 = -4;
    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;
    const WM_NCCALCSIZE: u32 = 0x0083;
    const WM_NCPAINT: u32 = 0x0085;

    const WS_POPUP: u32 = 0x8000_0000;
    const WS_CAPTION: u32 = 0x00C0_0000;
    const WS_BORDER: u32 = 0x0080_0000;
    const WS_DLGFRAME: u32 = 0x0040_0000;
    const WS_THICKFRAME: u32 = 0x0004_0000;
    const WS_EX_DLGMODALFRAME: u32 = 0x0000_0001;
    const WS_EX_WINDOWEDGE: u32 = 0x0000_0100;
    const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
    const WS_EX_STATICEDGE: u32 = 0x0002_0000;

    const DWMWA_NCRENDERING_POLICY: u32 = 2;
    const DWMNCRP_DISABLED: u32 = 1;
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_DONOTROUND: u32 = 1;
    const DWMWA_BORDER_COLOR: u32 = 34;
    const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;

    static OLD_WNDPROC: AtomicIsize = AtomicIsize::new(0);

    unsafe extern "system" fn borderless_wndproc(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        if msg == WM_NCCALCSIZE || msg == WM_NCPAINT {
            return 0;
        }
        let old = OLD_WNDPROC.load(Ordering::Relaxed);
        if old != 0 {
            unsafe { CallWindowProcW(old, hwnd, msg, wparam, lparam) }
        } else {
            0
        }
    }

    pub fn remove_window_borders(hwnd: isize) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            let our_proc = borderless_wndproc as *const () as isize;
            let prev = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            let has_border_style = (style & (WS_CAPTION | WS_BORDER | WS_DLGFRAME | WS_THICKFRAME)) != 0
                || (style & WS_POPUP) == 0;

            if prev == our_proc && !has_border_style {
                return;
            }

            let mut rect = Rect::default();
            let has_rect = GetWindowRect(hwnd, &mut rect) != 0;

            if prev != 0 && prev != our_proc {
                OLD_WNDPROC.store(prev, Ordering::Relaxed);
                SetWindowLongPtrW(hwnd, GWLP_WNDPROC, our_proc);
            }

            let ncrp: u32 = DWMNCRP_DISABLED;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_NCRENDERING_POLICY,
                (&ncrp as *const u32).cast::<c_void>(),
                std::mem::size_of::<u32>() as u32,
            );
            let corner: u32 = DWMWCP_DONOTROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                (&corner as *const u32).cast::<c_void>(),
                std::mem::size_of::<u32>() as u32,
            );
            let border_none: u32 = DWMWA_COLOR_NONE;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                (&border_none as *const u32).cast::<c_void>(),
                std::mem::size_of::<u32>() as u32,
            );

            let new_style = (style
                & !(WS_CAPTION | WS_BORDER | WS_DLGFRAME | WS_THICKFRAME))
                | WS_POPUP;
            SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);

            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            let new_ex = ex_style
                & !(WS_EX_DLGMODALFRAME
                    | WS_EX_WINDOWEDGE
                    | WS_EX_CLIENTEDGE
                    | WS_EX_STATICEDGE);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex as i32);

            if has_rect {
                SetWindowPos(
                    hwnd,
                    0,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
    }
}

#[cfg(windows)]
#[derive(Clone, Copy)]
struct Win32DragState {
    hwnd: isize,
    offset_cx: i32,
    offset_cy: i32,
    start_x: i32,
    start_y: i32,
    moved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResizeMode {
    SouthEast,
    East,
    South,
}

#[derive(Clone, Copy, Debug)]
struct ResizeDragState {
    mode: ResizeMode,
    start_window_size: Vec2,
    start_pointer_pos: Pos2,
}

pub struct SocialCountApp {
    config: Config,
    channels: Vec<ChannelData>,
    textures: HashMap<String, egui::TextureHandle>,
    tx: Sender<WorkerMessage>,
    rx: Receiver<WorkerMessage>,
    is_loading: bool,
    last_refresh: Instant,
    last_refresh_str: String,
    last_channel_fetch: HashMap<String, Instant>,
    card_opacity: u8,
    is_compact: bool,
    pre_compact_size: Option<Vec2>,
    resize_drag: Option<ResizeDragState>,
    show_profile_manager: bool,
    new_channel_input: String,
    editing_channel_idx: Option<usize>,
    editing_channel_input: String,
    profile_error_msg: Option<String>,
    #[cfg(windows)]
    main_hwnd: isize,
    #[cfg(windows)]
    border_removed: bool,
    #[cfg(windows)]
    win32_drag_state: Option<Win32DragState>,
}

impl SocialCountApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::TRANSPARENT;
        visuals.override_text_color = Some(Color32::from_rgb(245, 247, 250));
        cc.egui_ctx.set_visuals(visuals);

        let config = Config::load_or_create();
        let (tx, rx) = channel();

        let initial_channels: Vec<ChannelData> = config
            .channels
            .iter()
            .map(|h| ChannelData::new_loading(h))
            .collect();

        let card_opacity = config.card_opacity;
        let is_compact = config.compact_mode;
        let pre_compact_size = Some(Vec2::new(
            config.window_width.max(280.0),
            config.window_height.max(160.0),
        ));

        let mut app = Self {
            config,
            channels: initial_channels,
            textures: HashMap::new(),
            tx,
            rx,
            is_loading: false,
            last_refresh: Instant::now(),
            last_refresh_str: chrono_or_fallback_time(),
            last_channel_fetch: HashMap::new(),
            card_opacity,
            is_compact,
            pre_compact_size,
            resize_drag: None,
            show_profile_manager: false,
            new_channel_input: String::new(),
            editing_channel_idx: None,
            editing_channel_input: String::new(),
            profile_error_msg: None,
            #[cfg(windows)]
            main_hwnd: 0,
            #[cfg(windows)]
            border_removed: false,
            #[cfg(windows)]
            win32_drag_state: None,
        };

        app.trigger_refresh();
        app
    }

    fn toggle_compact(&mut self, ctx: &egui::Context) {
        self.is_compact = !self.is_compact;
        self.config.compact_mode = self.is_compact;

        let target_size = if self.is_compact {
            let current_size = ctx.screen_rect().size();
            self.pre_compact_size = Some(current_size);
            self.config.window_width = current_size.x;
            self.config.window_height = current_size.y;
            // Calculate ultra-compact height: ~36px per channel + compact header space
            let compact_h = (20.0 + self.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
            let compact_w = 205.0_f32;
            Vec2::new(compact_w, compact_h)
        } else {
            self.pre_compact_size.unwrap_or_else(|| {
                Vec2::new(self.config.window_width.max(280.0), self.config.window_height.max(160.0))
            })
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(target_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(target_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));

        let _ = self.config.save();
    }

    fn trigger_refresh(&mut self) {
        self.config = Config::load_or_create();
        let handles = self.config.channels.clone();
        self.trigger_refresh_channels(handles);
    }

    fn trigger_refresh_channels(&mut self, handles: Vec<String>) {
        if self.is_loading || handles.is_empty() {
            return;
        }

        self.config = Config::load_or_create();

        for handle in &handles {
            if !self.channels.iter().any(|c| &c.handle == handle) {
                self.channels.push(ChannelData::new_loading(handle));
            }
        }

        for ch in &mut self.channels {
            if handles.contains(&ch.handle) && ch.subscriber_count == 0 {
                ch.subscriber_text = "Loading...".to_string();
            }
        }

        self.is_loading = true;
        let tx = self.tx.clone();

        thread::spawn(move || {
            let fetcher = Arc::new(YouTubeFetcher::new());

            for handle in handles {
                let data = fetcher.fetch_channel(&handle);
                let _ = tx.send(WorkerMessage::ChannelUpdated(Box::new(data)));
            }

            let _ = tx.send(WorkerMessage::RefreshFinished);
        });
    }

    fn update_textures(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::ChannelUpdated(new_data_box) => {
                    let mut new_data = *new_data_box;
                    self.last_channel_fetch.insert(new_data.handle.clone(), Instant::now());
                    if let Some(bytes) = &new_data.avatar_bytes {
                        if let Ok(img) = image::load_from_memory(bytes) {
                            let mut rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            let cx = w as f32 / 2.0;
                            let cy = h as f32 / 2.0;
                            let radius = (w.min(h) as f32 / 2.0) - 0.5;

                            for y in 0..h {
                                for x in 0..w {
                                    let dx = x as f32 + 0.5 - cx;
                                    let dy = y as f32 + 0.5 - cy;
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    let pixel = rgba.get_pixel_mut(x, y);
                                    if dist > radius {
                                        pixel[3] = 0;
                                    } else if dist > radius - 1.0 {
                                        let factor = (radius - dist).clamp(0.0, 1.0);
                                        pixel[3] = ((pixel[3] as f32) * factor) as u8;
                                    }
                                }
                            }

                            let size = [w as usize, h as usize];
                            let pixels = rgba.as_flat_samples();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                pixels.as_slice(),
                            );
                            let texture = ctx.load_texture(
                                format!("avatar_{}", new_data.handle),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.textures.insert(new_data.handle.clone(), texture);
                        }
                    }

                    if let Some(pos) = self.channels.iter().position(|c| c.handle == new_data.handle) {
                        let old = &self.channels[pos];

                        // Preserve avatar and title if new fetch failed or lacked image
                        if new_data.avatar_bytes.is_none() && old.avatar_bytes.is_some() {
                            new_data.avatar_bytes = old.avatar_bytes.clone();
                        }
                        if (new_data.title.is_empty() || new_data.title == new_data.handle.trim_start_matches('@'))
                            && !old.title.is_empty()
                        {
                            new_data.title = old.title.clone();
                        }

                        // Detect if subscriber count changed compared to previously known valid value
                        if old.subscriber_count > 0 && new_data.subscriber_count > 0 && new_data.subscriber_count != old.subscriber_count {
                            new_data.sub_changed = true;
                            new_data.sub_changed_at = Some(Instant::now());
                        } else {
                            new_data.sub_changed = old.sub_changed;
                            new_data.sub_changed_at = old.sub_changed_at;
                        }

                        // Detect if view count changed compared to previous value
                        let old_views_clean = crate::youtube::clean_stat_number(&old.views_text);
                        let new_views_clean = crate::youtube::clean_stat_number(&new_data.views_text);
                        if !old_views_clean.is_empty() && old_views_clean != "-" && !new_views_clean.is_empty() && new_views_clean != "-" && old_views_clean != new_views_clean {
                            new_data.views_changed = true;
                            new_data.views_changed_at = Some(Instant::now());
                        } else {
                            new_data.views_changed = old.views_changed;
                            new_data.views_changed_at = old.views_changed_at;
                        }

                        self.channels[pos] = new_data;
                    } else {
                        self.channels.push(new_data);
                    }

                    self.channels.sort_by_key(|a| std::cmp::Reverse(a.subscriber_count));
                }
                WorkerMessage::RefreshFinished => {
                    self.is_loading = false;
                    self.last_refresh = Instant::now();
                    self.last_refresh_str = chrono_or_fallback_time();
                }
            }
        }
    }
}

fn chrono_or_fallback_time() -> String {
    #[cfg(windows)]
    {
        use std::mem::MaybeUninit;
        #[repr(C)]
        struct SystemTime {
            w_year: u16,
            w_month: u16,
            w_day_of_week: u16,
            w_day: u16,
            w_hour: u16,
            w_minute: u16,
            w_second: u16,
            w_milliseconds: u16,
        }
        extern "system" {
            fn GetLocalTime(lpSystemTime: *mut SystemTime);
        }
        unsafe {
            let mut st = MaybeUninit::<SystemTime>::uninit();
            GetLocalTime(st.as_mut_ptr());
            let st = st.assume_init();
            format!("{:02}/{:02}/{} {:02}:{:02}:{:02}", st.w_day, st.w_month, st.w_year, st.w_hour, st.w_minute, st.w_second)
        }
    }
    #[cfg(not(windows))]
    {
        "Just now".to_string()
    }
}

impl eframe::App for SocialCountApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(windows)]
        self.update_hwnd_and_border_win32();
        #[cfg(windows)]
        self.update_win32_drag(ctx);

        self.update_textures(ctx);

        // Periodic refresh check:
        // If a channel had an error or reported N/A / N/D, retry 1 minute (60s) later.
        // Otherwise, refresh according to refresh_interval_secs (default: 300s = 5m).
        if !self.is_loading {
            let normal_interval = Duration::from_secs(self.config.refresh_interval_secs.max(30));
            let retry_interval = Duration::from_secs(60);

            let mut to_refresh = Vec::new();
            for ch in &self.channels {
                let last = self
                    .last_channel_fetch
                    .get(&ch.handle)
                    .copied()
                    .unwrap_or(self.last_refresh);

                let interval = if ch.has_error() {
                    retry_interval
                } else {
                    normal_interval
                };

                if last.elapsed() >= interval {
                    to_refresh.push(ch.handle.clone());
                }
            }

            // Also check for newly configured channels not yet in memory
            for handle in &self.config.channels {
                if !self.channels.iter().any(|c| &c.handle == handle) && !to_refresh.contains(handle) {
                    to_refresh.push(handle.clone());
                }
            }

            if !to_refresh.is_empty() {
                self.trigger_refresh_channels(to_refresh);
            }
        }

        if self.is_loading {
            ctx.request_repaint_after(Duration::from_millis(150));
        } else {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        // Secondary / Right-click detection to open profile manager (CRUD)
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary)) {
            self.show_profile_manager = !self.show_profile_manager;
            self.profile_error_msg = None;
            self.editing_channel_idx = None;
            if self.show_profile_manager {
                if self.is_compact {
                    let manager_size = Vec2::new(350.0, 440.0);
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(manager_size));
                }
            } else if self.is_compact {
                let compact_h = (20.0 + self.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(205.0, compact_h)));
            }
        }

        // Target size calculation and anti-ballooning stabilization across monitors
        let target_size = if self.show_profile_manager {
            if self.is_compact {
                Vec2::new(350.0, 440.0)
            } else {
                Vec2::new(self.config.window_width.max(340.0), self.config.window_height.max(420.0))
            }
        } else if self.is_compact {
            let compact_h = (20.0 + self.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
            Vec2::new(205.0, compact_h)
        } else {
            Vec2::new(self.config.window_width.max(280.0), self.config.window_height.max(160.0))
        };

        let pointer_down = ctx.input(|i| i.pointer.any_down());
        if self.resize_drag.is_none() && !pointer_down {
            let current_size = ctx.screen_rect().size();
            if (current_size.x - target_size.x).abs() > 4.0 || (current_size.y - target_size.y).abs() > 4.0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT).inner_margin(Margin::same(4.0)))
            .show(ctx, |ui| {
                let full_rect = ui.max_rect();
                let alpha_factor = self.card_opacity as f32 / 255.0;

                // Main frosted glass acrylic container
                let container_bg = if self.card_opacity == 0 {
                    Color32::TRANSPARENT
                } else {
                    Color32::from_rgba_unmultiplied(18, 20, 28, self.card_opacity.min(245))
                };

                let container_stroke = if self.card_opacity == 0 {
                    Stroke::NONE
                } else {
                    Stroke::new(1.0_f32, Color32::from_white_alpha((35.0 * alpha_factor).round() as u8))
                };

                let container_shadow = if self.card_opacity == 0 {
                    egui::epaint::Shadow {
                        offset: Vec2::ZERO,
                        blur: 0.0_f32,
                        spread: 0.0_f32,
                        color: Color32::TRANSPARENT,
                    }
                } else {
                    egui::epaint::Shadow {
                        offset: [0.0_f32, 6.0_f32].into(),
                        blur: 22.0_f32,
                        spread: 1.0_f32,
                        color: Color32::from_black_alpha((160.0 * alpha_factor).round() as u8),
                    }
                };

                let container_margin = if self.is_compact && !self.show_profile_manager {
                    Margin::symmetric(5.0, 3.0)
                } else {
                    Margin::symmetric(14.0, 12.0)
                };

                let _container_response = egui::Frame::none()
                    .fill(container_bg)
                    .stroke(container_stroke)
                    .shadow(container_shadow)
                    .rounding(Rounding::same(if self.is_compact && !self.show_profile_manager { 8.0 } else { 18.0 }))
                    .inner_margin(container_margin)
                    .show(ui, |ui| {
                        if self.show_profile_manager {
                            self.render_profile_manager(ui, ctx, alpha_factor);
                        } else {
                            // 1. HEADER (distinct between normal and ultra-compact view)
                            if self.is_compact {
                                self.render_header_compact(ui, ctx);
                            } else {
                                self.render_header(ui, ctx);
                            }

                            // Top divider line (visible only in normal view when opacity > 0)
                            let sep_alpha = (18.0 * alpha_factor).round() as u8;
                            if !self.is_compact {
                                ui.add_space(8.0);
                                let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
                                if sep_alpha > 0 {
                                    ui.painter().hline(sep_rect.x_range(), sep_rect.center().y, Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)));
                                }
                                ui.add_space(8.0);
                            } else {
                                ui.add_space(2.0);
                            }

                            // 2. CHANNELS LIST
                            if self.is_compact {
                                // In compact mode scrollbar is unnecessary: direct compact card layout
                                for channel in &self.channels {
                                    self.render_channel_card_compact(ui, channel);
                                    ui.add_space(2.0);
                                }
                            } else {
                                let footer_height = 26.0_f32;
                                let scroll_height = (ui.available_height() - footer_height - 12.0_f32).max(60.0_f32);

                                egui::ScrollArea::vertical()
                                    .max_height(scroll_height)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for channel in &self.channels {
                                            self.render_channel_card(ui, channel);
                                            ui.add_space(8.0);
                                        }
                                    });

                                ui.add_space(4.0);

                                // Bottom divider line above footer
                                let (sep2_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
                                if sep_alpha > 0 {
                                    ui.painter().hline(sep2_rect.x_range(), sep2_rect.center().y, Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)));
                                }

                                ui.add_space(4.0);

                                // 3. STICKY FOOTER (visible in normal view)
                                self.render_footer(ui);
                            }
                        }
                    });

                // Resize handles and grip
                self.render_resize_handles(ui, ctx, full_rect);
            });
    }
}

impl SocialCountApp {
    /// Header for normal view: Logo, Title, Opacity Slider, Refresh, Settings, Close
    fn render_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let total_w = ui.available_width();
        let header_h = 26.0_f32;

        let (header_rect, header_resp) = ui.allocate_exact_size(
            Vec2::new(total_w, header_h),
            egui::Sense::click_and_drag(),
        );

        if header_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if header_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if header_resp.double_clicked() {
            self.toggle_compact(ctx);
        }
        if header_resp.drag_started_by(egui::PointerButton::Primary) || header_resp.dragged_by(egui::PointerButton::Primary) {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Sub-UI for interactive buttons on the right
        let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect).layout(*ui.layout()));

        // Draw YouTube logo and title on the left
        let logo_rect = Rect::from_center_size(
            Pos2::new(header_rect.min.x + 12.0, header_rect.center().y),
            Vec2::new(18.0, 13.0),
        );
        crate::icons::draw_youtube_icon(ui.painter(), logo_rect);

        ui.painter().text(
            Pos2::new(header_rect.min.x + 26.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "SocialCount",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(255, 255, 255),
        );

        // Controls on the right
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_resp = ui.allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            if close_hover {
                ui.painter().circle_filled(close_resp.rect.center(), 11.0, Color32::from_rgba_unmultiplied(239, 68, 68, 45));
            }
            let close_color = if close_hover { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(175, 180, 195) };
            ui.painter().text(
                close_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                egui::FontId::proportional(13.0),
                close_color,
            );
            if close_resp.clicked() {
                if !self.is_compact {
                    let current_size = ctx.screen_rect().size();
                    self.config.window_width = current_size.x;
                    self.config.window_height = current_size.y;
                }
                self.config.compact_mode = self.is_compact;
                self.config.card_opacity = self.card_opacity;
                let _ = self.config.save();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.add_space(2.0);

            let settings_resp = ui.allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Manage profiles (or right-click)");
            let settings_hover = settings_resp.hovered();
            if settings_hover {
                ui.painter().circle_filled(settings_resp.rect.center(), 11.0, Color32::from_white_alpha(25));
            }
            let settings_color = if self.show_profile_manager {
                Color32::from_rgb(147, 197, 253)
            } else if settings_hover {
                Color32::from_rgb(220, 230, 245)
            } else {
                Color32::from_rgb(175, 180, 195)
            };
            ui.painter().text(
                settings_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                "⚙",
                egui::FontId::proportional(13.0),
                settings_color,
            );
            if settings_resp.clicked() {
                self.show_profile_manager = !self.show_profile_manager;
                self.profile_error_msg = None;
                self.editing_channel_idx = None;
            }

            ui.add_space(2.0);

            let refresh_tooltip = if self.is_loading { "Refreshing..." } else { "Refresh now" };
            let refresh_resp = ui.allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text(refresh_tooltip);
            let refresh_hover = refresh_resp.hovered();
            if refresh_hover {
                ui.painter().circle_filled(refresh_resp.rect.center(), 11.0, Color32::from_white_alpha(25));
            }
            let refresh_color = if self.is_loading {
                Color32::from_rgb(251, 191, 36)
            } else if refresh_hover {
                Color32::from_rgb(147, 197, 253)
            } else {
                Color32::from_rgb(180, 205, 240)
            };
            let refresh_char = if self.is_loading { "⏳" } else { "⟳" };
            ui.painter().text(
                refresh_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                refresh_char,
                egui::FontId::proportional(14.0),
                refresh_color,
            );
            if refresh_resp.clicked() && !self.is_loading {
                self.trigger_refresh();
            }

            ui.add_space(4.0);

            // Background Opacity Slider
            let op_percent = ((self.card_opacity as f32 / 255.0) * 100.0).round() as u32;
            ui.horizontal(|ui| {
                ui.spacing_mut().slider_width = 70.0_f32;
                let slider = egui::Slider::new(&mut self.card_opacity, 0..=255)
                    .show_value(false);
                let slider_resp = ui.add(slider)
                    .on_hover_text(format!("Background opacity: {}%", op_percent));

                if slider_resp.drag_stopped() || (slider_resp.changed() && !slider_resp.dragged()) {
                    self.config.card_opacity = self.card_opacity;
                    let _ = self.config.save();
                }

                ui.label(
                    egui::RichText::new("🌓")
                        .size(12.0)
                        .color(Color32::from_rgb(175, 185, 200)),
                );
            });
        });
    }

    /// Header for ultra-compact view: minimal drag handle and close button
    fn render_header_compact(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let total_w = ui.available_width();
        let header_h = 13.0_f32;

        let (header_rect, header_resp) = ui.allocate_exact_size(
            Vec2::new(total_w, header_h),
            egui::Sense::click_and_drag(),
        );

        if header_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if header_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if header_resp.double_clicked() {
            self.toggle_compact(ctx);
        }
        if header_resp.drag_started_by(egui::PointerButton::Primary) || header_resp.dragged_by(egui::PointerButton::Primary) {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Minimalist central drag pill
        let handle_color = if header_resp.hovered() || header_resp.dragged() {
            Color32::from_white_alpha(120)
        } else {
            Color32::from_white_alpha(45)
        };
        let handle_rect = Rect::from_center_size(
            header_rect.center(),
            Vec2::new(28.0, 3.0),
        );
        ui.painter().rect_filled(handle_rect, Rounding::same(1.5), handle_color);

        // Micro logo on the left
        let logo_rect = Rect::from_center_size(
            Pos2::new(header_rect.min.x + 8.0, header_rect.center().y),
            Vec2::new(12.0, 8.5),
        );
        crate::icons::draw_youtube_icon(ui.painter(), logo_rect);

        // Tiny close button on the right
        let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect).layout(*ui.layout()));
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_resp = ui.allocate_response(Vec2::splat(13.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            let close_color = if close_hover { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(140, 145, 160) };
            ui.painter().text(
                close_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                egui::FontId::proportional(10.0),
                close_color,
            );
            if close_resp.clicked() {
                self.config.compact_mode = self.is_compact;
                self.config.card_opacity = self.card_opacity;
                let _ = self.config.save();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    /// Sticky footer for normal view
    fn render_footer(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 16.0), egui::Sense::hover());
                    let time_now = ui.input(|i| i.time);
                    crate::icons::draw_live_dot(ui.painter(), dot_rect.center(), self.is_loading, time_now);

                    ui.add_space(4.0);

                    ui.label(
                        egui::RichText::new(format!("Updated: {}", self.last_refresh_str))
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
            });
        });
    }

    fn render_resize_handles(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, rect: Rect) {
        let edge_thickness = 8.0_f32;
        let corner_size = 18.0_f32;

        let se_rect = Rect::from_min_max(
            Pos2::new(rect.max.x - corner_size, rect.max.y - corner_size),
            rect.max,
        );
        let east_rect = Rect::from_min_max(
            Pos2::new(rect.max.x - edge_thickness, rect.min.y + corner_size),
            Pos2::new(rect.max.x, rect.max.y - corner_size),
        );
        let south_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + corner_size, rect.max.y - edge_thickness),
            Pos2::new(rect.max.x - corner_size, rect.max.y),
        );

        let se_resp = ui.interact(se_rect, egui::Id::new("resize_se"), egui::Sense::click_and_drag());
        let east_resp = ui.interact(east_rect, egui::Id::new("resize_east"), egui::Sense::click_and_drag());
        let south_resp = ui.interact(south_rect, egui::Id::new("resize_south"), egui::Sense::click_and_drag());

        let min_w = if self.is_compact { 175.0_f32 } else { 280.0_f32 };
        let min_h = if self.is_compact { 50.0_f32 } else { 160.0_f32 };
        let max_w = 1200.0_f32;
        let max_h = 1600.0_f32;

        // Start drag: capture initial window size and initial pointer position
        if se_resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(min_w, min_h)));
                ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(Vec2::new(max_w, max_h)));
                self.resize_drag = Some(ResizeDragState {
                    mode: ResizeMode::SouthEast,
                    start_window_size: ctx.screen_rect().size(),
                    start_pointer_pos: pos,
                });
            }
        } else if east_resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(min_w, min_h)));
                ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(Vec2::new(max_w, max_h)));
                self.resize_drag = Some(ResizeDragState {
                    mode: ResizeMode::East,
                    start_window_size: ctx.screen_rect().size(),
                    start_pointer_pos: pos,
                });
            }
        } else if south_resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(min_w, min_h)));
                ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(Vec2::new(max_w, max_h)));
                self.resize_drag = Some(ResizeDragState {
                    mode: ResizeMode::South,
                    start_window_size: ctx.screen_rect().size(),
                    start_pointer_pos: pos,
                });
            }
        }

        // Handle continuous deterministic resizing
        let pointer_down = ui.input(|i| i.pointer.primary_down());
        if !pointer_down {
            if let Some(_drag_state) = self.resize_drag.take() {
                let current_size = ctx.screen_rect().size();
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(current_size));
                ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(current_size));
                if !self.is_compact {
                    self.config.window_width = current_size.x;
                    self.config.window_height = current_size.y;
                    self.pre_compact_size = Some(current_size);
                    let _ = self.config.save();
                }
            }
        } else if let Some(drag_state) = self.resize_drag {
            if let Some(curr_pos) = ui.input(|i| i.pointer.latest_pos()) {
                let diff = curr_pos - drag_state.start_pointer_pos;

                let (new_w, new_h) = match drag_state.mode {
                    ResizeMode::SouthEast => (
                        (drag_state.start_window_size.x + diff.x).clamp(min_w, max_w),
                        (drag_state.start_window_size.y + diff.y).clamp(min_h, max_h),
                    ),
                    ResizeMode::East => (
                        (drag_state.start_window_size.x + diff.x).clamp(min_w, max_w),
                        drag_state.start_window_size.y,
                    ),
                    ResizeMode::South => (
                        drag_state.start_window_size.x,
                        (drag_state.start_window_size.y + diff.y).clamp(min_h, max_h),
                    ),
                };

                let new_size = Vec2::new(new_w, new_h);
                if !self.is_compact {
                    self.pre_compact_size = Some(new_size);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
            }
        }

        // Cursors
        let is_se_active = se_resp.hovered()
            || matches!(self.resize_drag, Some(ResizeDragState { mode: ResizeMode::SouthEast, .. }));
        let is_east_active = east_resp.hovered()
            || matches!(self.resize_drag, Some(ResizeDragState { mode: ResizeMode::East, .. }));
        let is_south_active = south_resp.hovered()
            || matches!(self.resize_drag, Some(ResizeDragState { mode: ResizeMode::South, .. }));

        if is_se_active {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNwSe);
        } else if is_east_active {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
        } else if is_south_active {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
        }

        // Draw dotted corner grip in bottom right
        let painter = ui.painter();
        let alpha_factor = self.card_opacity as f32 / 255.0;
        let grip_alpha = (70.0 * alpha_factor).round() as u8;

        let grip_color = if is_se_active {
            Color32::from_rgb(147, 197, 253)
        } else if grip_alpha > 0 {
            Color32::from_white_alpha(grip_alpha)
        } else {
            Color32::TRANSPARENT
        };

        if grip_color != Color32::TRANSPARENT {
            let p = rect.max - Vec2::new(6.0, 6.0);
            painter.line_segment([p, p - Vec2::new(3.0, 0.0)], Stroke::new(1.6_f32, grip_color));
            painter.line_segment([p - Vec2::new(5.0, 0.0), p - Vec2::new(0.0, 5.0)], Stroke::new(1.6_f32, grip_color));
            painter.line_segment([p - Vec2::new(9.0, 0.0), p - Vec2::new(0.0, 9.0)], Stroke::new(1.6_f32, grip_color));
        }
    }

    /// Normal view: full channel card with avatar, title, handle and solid stat pills
    fn render_channel_card(&self, ui: &mut egui::Ui, channel: &ChannelData) {
        let opacity = self.card_opacity;
        let alpha_factor = opacity as f32 / 255.0;

        let card_frame = egui::Frame::none()
            .fill(if opacity == 0 {
                Color32::TRANSPARENT
            } else {
                Color32::from_rgba_unmultiplied(26, 29, 39, (opacity as f32 * 0.95).round() as u8)
            })
            .stroke(if opacity == 0 {
                Stroke::NONE
            } else {
                Stroke::new(1.0_f32, Color32::from_white_alpha((22.0 * alpha_factor).round() as u8))
            })
            .rounding(Rounding::same(14.0))
            .inner_margin(Margin::symmetric(12.0, 10.0));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let avatar_size = 48.0;
                let (avatar_rect, _resp) = ui.allocate_exact_size(
                    Vec2::splat(avatar_size),
                    egui::Sense::hover(),
                );

                if let Some(texture) = self.textures.get(&channel.handle) {
                    let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                    ui.painter().image(
                        texture.id(),
                        avatar_rect,
                        uv,
                        Color32::WHITE,
                    );

                    ui.painter().circle_stroke(
                        avatar_rect.center(),
                        avatar_size / 2.0,
                        Stroke::new(1.5_f32, Color32::from_white_alpha(55)),
                    );
                    ui.painter().circle_stroke(
                        avatar_rect.center(),
                        avatar_size / 2.0 - 1.0,
                        Stroke::new(0.7_f32, Color32::from_black_alpha(80)),
                    );
                } else {
                    ui.painter().circle_filled(
                        avatar_rect.center(),
                        avatar_size / 2.0,
                        Color32::from_rgb(45, 52, 70),
                    );
                    ui.painter().circle_stroke(
                        avatar_rect.center(),
                        avatar_size / 2.0,
                        Stroke::new(1.5_f32, Color32::from_white_alpha(40)),
                    );

                    let initial = channel
                        .title
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();

                    ui.painter().text(
                        avatar_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initial,
                        egui::FontId::proportional(19.0),
                        Color32::WHITE,
                    );
                }

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let mut title_resp = ui.add(
                            egui::Link::new(
                                egui::RichText::new(&channel.title)
                                    .strong()
                                    .size(15.0)
                                    .color(Color32::from_rgb(255, 255, 255)),
                            )
                        );
                        if let Some(err) = &channel.error {
                            title_resp = title_resp.on_hover_text(format!("Error: {} (Retrying in 1m)", err));
                        } else if channel.has_error() {
                            title_resp = title_resp.on_hover_text("Stat unavailable (Retrying in 1m)");
                        }
                        if title_resp.clicked() {
                            let _ = open::that(&channel.channel_url);
                        }
                        if title_resp.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&channel.handle)
                                .size(11.5)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });

                    ui.add_space(6.0);

                    // Counter pills: Equal width, responsive to window width, always solid & opaque
                    let avail_w = ui.available_width();
                    let gap = 8.0_f32;
                    let box_w = ((avail_w - gap) / 2.0_f32).max(70.0_f32);
                    let box_h = 28.0_f32;
                    let box_size = Vec2::new(box_w, box_h);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;

                        // Users/Subscribers Box
                        let (box1_rect, _) = ui.allocate_exact_size(box_size, egui::Sense::hover());
                        ui.painter().rect_filled(box1_rect, Rounding::same(8.0), Color32::from_rgb(16, 44, 28));
                        let stroke1_color = if channel.sub_changed {
                            Color32::from_rgb(239, 68, 68) // Red border if number changed
                        } else {
                            Color32::from_rgb(52, 211, 153)
                        };
                        let stroke1_width = if channel.sub_changed { 1.8_f32 } else { 1.0_f32 };
                        ui.painter().rect_stroke(box1_rect, Rounding::same(8.0), Stroke::new(stroke1_width, stroke1_color));

                        let sub_num = crate::youtube::clean_stat_number(&channel.subscriber_text);
                        let font_id = egui::FontId::proportional(13.0);
                        let sub_galley = ui.fonts(|f| f.layout_no_wrap(sub_num, font_id.clone(), Color32::from_rgb(209, 250, 229)));
                        let icon1_w = 14.0_f32;
                        let item_gap = 5.0_f32;
                        let total1_w = icon1_w + item_gap + sub_galley.size().x;
                        let start1_x = box1_rect.center().x - total1_w / 2.0;

                        let icon1_rect = Rect::from_center_size(
                            Pos2::new(start1_x + icon1_w / 2.0, box1_rect.center().y),
                            Vec2::splat(14.0),
                        );
                        crate::icons::draw_user_icon(ui.painter(), icon1_rect, Color32::from_rgb(52, 211, 153));

                        ui.painter().galley(
                            Pos2::new(start1_x + icon1_w + item_gap, box1_rect.center().y - sub_galley.size().y / 2.0),
                            sub_galley,
                            Color32::from_rgb(209, 250, 229),
                        );

                        // Views Box
                        let (box2_rect, _) = ui.allocate_exact_size(box_size, egui::Sense::hover());
                        ui.painter().rect_filled(box2_rect, Rounding::same(8.0), Color32::from_rgb(18, 36, 58));
                        let stroke2_color = if channel.views_changed {
                            Color32::from_rgb(239, 68, 68) // Red border if number changed
                        } else {
                            Color32::from_rgb(96, 165, 250)
                        };
                        let stroke2_width = if channel.views_changed { 1.8_f32 } else { 1.0_f32 };
                        ui.painter().rect_stroke(box2_rect, Rounding::same(8.0), Stroke::new(stroke2_width, stroke2_color));

                        let views_num = crate::youtube::clean_stat_number(&channel.views_text);
                        let views_galley = ui.fonts(|f| f.layout_no_wrap(views_num, font_id, Color32::from_rgb(224, 242, 254)));
                        let icon2_w = 16.0_f32;
                        let total2_w = icon2_w + item_gap + views_galley.size().x;
                        let start2_x = box2_rect.center().x - total2_w / 2.0;

                        let icon2_rect = Rect::from_center_size(
                            Pos2::new(start2_x + icon2_w / 2.0, box2_rect.center().y),
                            Vec2::new(16.0, 14.0),
                        );
                        crate::icons::draw_eye_icon(ui.painter(), icon2_rect, Color32::from_rgb(96, 165, 250));

                        ui.painter().galley(
                            Pos2::new(start2_x + icon2_w + item_gap, box2_rect.center().y - views_galley.size().y / 2.0),
                            views_galley,
                            Color32::from_rgb(224, 242, 254),
                        );
                    });
                });
            });
        });
    }

    /// ULTRA-COMPACT view: Circular channel icon in center flanked by solid stat pills at the corners
    fn render_channel_card_compact(&self, ui: &mut egui::Ui, channel: &ChannelData) {
        let opacity = self.card_opacity;
        let alpha_factor = opacity as f32 / 255.0;

        let card_frame = egui::Frame::none()
            .fill(if opacity == 0 {
                Color32::TRANSPARENT
            } else {
                Color32::from_rgba_unmultiplied(22, 25, 34, (opacity as f32 * 0.95).round() as u8)
            })
            .stroke(if opacity == 0 {
                Stroke::NONE
            } else {
                Stroke::new(1.0_f32, Color32::from_white_alpha((18.0 * alpha_factor).round() as u8))
            })
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(4.0, 2.0));

        card_frame.show(ui, |ui| {
            let total_w = ui.available_width();
            let avatar_size = 28.0_f32; // Ultra-compact avatar 28px
            let gap = 4.0_f32;
            let box_w = ((total_w - avatar_size - gap * 2.0) / 2.0_f32).max(48.0_f32);
            let box_h = 21.0_f32; // Compact box 21px high
            let box_size = Vec2::new(box_w, box_h);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;

                // --- LEFT CORNER: USERS / SUBSCRIBERS COUNTER ---
                let (box1_rect, _) = ui.allocate_exact_size(box_size, egui::Sense::hover());
                ui.painter().rect_filled(box1_rect, Rounding::same(5.0), Color32::from_rgb(16, 44, 28));
                let stroke1_color = if channel.sub_changed {
                    Color32::from_rgb(239, 68, 68) // Red border if number changed
                } else {
                    Color32::from_rgb(52, 211, 153)
                };
                let stroke1_width = if channel.sub_changed { 1.5_f32 } else { 1.0_f32 };
                ui.painter().rect_stroke(box1_rect, Rounding::same(5.0), Stroke::new(stroke1_width, stroke1_color));

                let sub_num = crate::youtube::clean_stat_number(&channel.subscriber_text);
                let font_id = egui::FontId::proportional(11.0);
                let sub_galley = ui.fonts(|f| f.layout_no_wrap(sub_num, font_id.clone(), Color32::from_rgb(209, 250, 229)));
                let icon1_w = 11.0_f32;
                let item_gap = 3.0_f32;
                let total1_w = icon1_w + item_gap + sub_galley.size().x;
                let start1_x = box1_rect.center().x - total1_w / 2.0;

                let icon1_rect = Rect::from_center_size(
                    Pos2::new(start1_x + icon1_w / 2.0, box1_rect.center().y),
                    Vec2::splat(11.0),
                );
                crate::icons::draw_user_icon(ui.painter(), icon1_rect, Color32::from_rgb(52, 211, 153));

                ui.painter().galley(
                    Pos2::new(start1_x + icon1_w + item_gap, box1_rect.center().y - sub_galley.size().y / 2.0),
                    sub_galley,
                    Color32::from_rgb(209, 250, 229),
                );

                // --- CENTER: CIRCULAR CHANNEL AVATAR ---
                let (avatar_rect, avatar_resp) = ui.allocate_exact_size(
                    Vec2::splat(avatar_size),
                    egui::Sense::click(),
                );

                if let Some(texture) = self.textures.get(&channel.handle) {
                    let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                    ui.painter().image(
                        texture.id(),
                        avatar_rect,
                        uv,
                        Color32::WHITE,
                    );

                    ui.painter().circle_stroke(
                        avatar_rect.center(),
                        avatar_size / 2.0,
                        Stroke::new(1.0_f32, Color32::from_white_alpha(65)),
                    );
                } else {
                    ui.painter().circle_filled(
                        avatar_rect.center(),
                        avatar_size / 2.0,
                        Color32::from_rgb(45, 52, 70),
                    );
                    let initial = channel.title.chars().next().unwrap_or('?').to_uppercase().to_string();
                    ui.painter().text(
                        avatar_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initial,
                        egui::FontId::proportional(13.0),
                        Color32::WHITE,
                    );
                }

                if avatar_resp.clicked() {
                    let _ = open::that(&channel.channel_url);
                }
                avatar_resp.on_hover_ui(|ui| {
                    ui.label(egui::RichText::new(&channel.title).strong().color(Color32::WHITE));
                    ui.label(egui::RichText::new(&channel.handle).color(Color32::from_rgb(148, 163, 184)));
                    if let Some(err) = &channel.error {
                        ui.label(egui::RichText::new(format!("⚠️ {} (Retrying in 1m)", err)).size(10.0).color(Color32::from_rgb(239, 68, 68)));
                    } else if channel.has_error() {
                        ui.label(egui::RichText::new("⚠️ Stat unavailable (Retrying in 1m)").size(10.0).color(Color32::from_rgb(239, 68, 68)));
                    }
                });

                // --- RIGHT CORNER: VIEWS COUNTER ---
                let (box2_rect, _) = ui.allocate_exact_size(box_size, egui::Sense::hover());
                ui.painter().rect_filled(box2_rect, Rounding::same(5.0), Color32::from_rgb(18, 36, 58));
                let stroke2_color = if channel.views_changed {
                    Color32::from_rgb(239, 68, 68) // Red border if number changed
                } else {
                    Color32::from_rgb(96, 165, 250)
                };
                let stroke2_width = if channel.views_changed { 1.5_f32 } else { 1.0_f32 };
                ui.painter().rect_stroke(box2_rect, Rounding::same(5.0), Stroke::new(stroke2_width, stroke2_color));

                let views_num = crate::youtube::clean_stat_number(&channel.views_text);
                let views_galley = ui.fonts(|f| f.layout_no_wrap(views_num, font_id, Color32::from_rgb(224, 242, 254)));
                let icon2_w = 13.0_f32;
                let total2_w = icon2_w + item_gap + views_galley.size().x;
                let start2_x = box2_rect.center().x - total2_w / 2.0;

                let icon2_rect = Rect::from_center_size(
                    Pos2::new(start2_x + icon2_w / 2.0, box2_rect.center().y),
                    Vec2::new(13.0, 10.0),
                );
                crate::icons::draw_eye_icon(ui.painter(), icon2_rect, Color32::from_rgb(96, 165, 250));

                ui.painter().galley(
                    Pos2::new(start2_x + icon2_w + item_gap, box2_rect.center().y - views_galley.size().y / 2.0),
                    views_galley,
                    Color32::from_rgb(224, 242, 254),
                );
            });
        });
    }

    /// Complete Profile Manager (CRUD) modal accessible via right-click or ⚙ button
    fn render_profile_manager(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, alpha_factor: f32) {
        let total_w = ui.available_width();
        let header_h = 26.0_f32;

        // 1. Header with drag handle and close button
        let (header_rect, header_resp) = ui.allocate_exact_size(
            Vec2::new(total_w, header_h),
            egui::Sense::click_and_drag(),
        );

        if header_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if header_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if header_resp.drag_started_by(egui::PointerButton::Primary) || header_resp.dragged_by(egui::PointerButton::Primary) {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.painter().text(
            Pos2::new(header_rect.min.x + 4.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "⚙ Manage YouTube Channels",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(255, 255, 255),
        );

        let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect).layout(*ui.layout()));
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_resp = ui.allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            if close_hover {
                ui.painter().circle_filled(close_resp.rect.center(), 11.0, Color32::from_rgba_unmultiplied(239, 68, 68, 45));
            }
            let close_color = if close_hover { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(175, 180, 195) };
            ui.painter().text(
                close_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                egui::FontId::proportional(13.0),
                close_color,
            );
            if close_resp.clicked() {
                self.show_profile_manager = false;
                self.editing_channel_idx = None;
                if self.is_compact {
                    let compact_h = (20.0 + self.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(205.0, compact_h)));
                }
            }
        });

        // Divider line
        ui.add_space(8.0);
        let sep_alpha = (25.0 * alpha_factor).round() as u8;
        let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
        if sep_alpha > 0 {
            ui.painter().hline(sep_rect.x_range(), sep_rect.center().y, Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)));
        }
        ui.add_space(8.0);

        // 2. ADD CHANNEL FORM (CREATE)
        ui.horizontal(|ui| {
            let input_width = (ui.available_width() - 88.0).max(120.0);
            let input_resp = ui.add_sized(
                [input_width, 24.0],
                egui::TextEdit::singleline(&mut self.new_channel_input)
                    .hint_text("e.g. @matteobaccan")
                    .margin(Margin::symmetric(6.0, 4.0)),
            );

            let enter_pressed = input_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let add_btn = ui.add_sized(
                [80.0, 24.0],
                egui::Button::new(egui::RichText::new("+ Add").size(12.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(37, 99, 235))
                    .rounding(Rounding::same(5.0)),
            );

            if add_btn.clicked() || enter_pressed {
                let trimmed = self.new_channel_input.trim();
                if !trimmed.is_empty() {
                    let formatted = if !trimmed.starts_with('@') && !trimmed.starts_with("UC") {
                        format!("@{}", trimmed)
                    } else {
                        trimmed.to_string()
                    };

                    if self.config.channels.iter().any(|h| h.eq_ignore_ascii_case(&formatted)) {
                        self.profile_error_msg = Some("Channel is already in the list".to_string());
                    } else {
                        self.profile_error_msg = None;
                        self.config.channels.push(formatted.clone());
                        self.channels.push(ChannelData::new_loading(&formatted));
                        let _ = self.config.save();
                        self.new_channel_input.clear();

                        // Start background fetch for newly added channel
                        let tx = self.tx.clone();
                        let h = formatted.clone();
                        std::thread::spawn(move || {
                            let fetcher = std::sync::Arc::new(YouTubeFetcher::new());
                            let data = fetcher.fetch_channel(&h);
                            let _ = tx.send(WorkerMessage::ChannelUpdated(Box::new(data)));
                        });
                    }
                }
            }
        });

        if let Some(err) = &self.profile_error_msg {
            ui.add_space(2.0);
            ui.label(egui::RichText::new(format!("⚠️ {}", err)).size(11.0).color(Color32::from_rgb(239, 68, 68)));
        }

        ui.add_space(6.0);

        // 3. CHANNELS LIST (READ, UPDATE, DELETE)
        let list_h = (ui.available_height() - 36.0).max(100.0);
        let mut to_delete_idx = None;
        let mut to_move_up_idx = None;
        let mut to_move_down_idx = None;
        let mut to_save_edit = None;

        egui::ScrollArea::vertical()
            .max_height(list_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.config.channels.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No channels configured")
                                .size(13.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new("Add your first channel above!")
                                .size(11.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });
                } else {
                    for idx in 0..self.config.channels.len() {
                        let handle = &self.config.channels[idx];
                        let ch_data = self.channels.iter().find(|c| c.handle == *handle);

                        let card_bg = Color32::from_rgba_unmultiplied(255, 255, 255, 12);
                        let card_stroke = Stroke::new(1.0_f32, Color32::from_white_alpha(20));

                        egui::Frame::none()
                            .fill(card_bg)
                            .stroke(card_stroke)
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                if self.editing_channel_idx == Some(idx) {
                                    // Inline update mode (UPDATE)
                                    ui.horizontal(|ui| {
                                        ui.add_sized(
                                            [(ui.available_width() - 70.0).max(100.0), 22.0],
                                            egui::TextEdit::singleline(&mut self.editing_channel_input),
                                        );

                                        if ui.small_button("✓").on_hover_text("Save").clicked() {
                                            to_save_edit = Some((idx, self.editing_channel_input.trim().to_string()));
                                        }
                                        if ui.small_button("✕").on_hover_text("Cancel").clicked() {
                                            self.editing_channel_idx = None;
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        // Mini avatar or icon
                                        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), egui::Sense::hover());
                                        let mut drawn = false;
                                        if let Some(c) = ch_data {
                                            if let Some(tex) = self.textures.get(&c.handle) {
                                                ui.painter().image(
                                                    tex.id(),
                                                    avatar_rect,
                                                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                                    Color32::WHITE,
                                                );
                                                drawn = true;
                                            }
                                        }
                                        if !drawn {
                                            crate::icons::draw_youtube_icon(ui.painter(), Rect::from_center_size(avatar_rect.center(), Vec2::new(16.0, 11.5)));
                                        }

                                        ui.add_space(4.0);

                                        // Title and handle
                                        let title = ch_data.map(|c| c.title.as_str()).unwrap_or(handle.as_str());
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(title)
                                                    .size(12.0)
                                                    .strong()
                                                    .color(Color32::from_rgb(240, 243, 248)),
                                            );
                                            ui.label(
                                                egui::RichText::new(handle)
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(148, 163, 184)),
                                            );
                                        });

                                        // CRUD controls on the right: Move up, Move down, Edit, Delete
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Delete (D)
                                            let del_btn = ui.add(
                                                egui::Button::new(egui::RichText::new("🗑").size(11.0).color(Color32::from_rgb(248, 113, 113)))
                                                    .fill(Color32::from_rgba_unmultiplied(239, 68, 68, 25))
                                                    .rounding(Rounding::same(4.0)),
                                            ).on_hover_text("Delete channel");
                                            if del_btn.clicked() {
                                                to_delete_idx = Some(idx);
                                            }

                                            // Edit (U)
                                            let edit_btn = ui.add(
                                                egui::Button::new(egui::RichText::new("✏").size(11.0).color(Color32::from_rgb(147, 197, 253)))
                                                    .fill(Color32::from_white_alpha(15))
                                                    .rounding(Rounding::same(4.0)),
                                            ).on_hover_text("Edit handle");
                                            if edit_btn.clicked() {
                                                self.editing_channel_idx = Some(idx);
                                                self.editing_channel_input = handle.clone();
                                            }

                                            // Move down (U - Reorder)
                                            if idx + 1 < self.config.channels.len()
                                                && ui.small_button("▼").on_hover_text("Move down").clicked()
                                            {
                                                to_move_down_idx = Some(idx);
                                            }

                                            // Move up (U - Reorder)
                                            if idx > 0
                                                && ui.small_button("▲").on_hover_text("Move up").clicked()
                                            {
                                                to_move_up_idx = Some(idx);
                                            }
                                        });
                                    });
                                }
                            });
                        ui.add_space(4.0);
                    }
                }
            });

        // Apply requested updates
        if let Some((idx, new_handle)) = to_save_edit {
            if !new_handle.is_empty() {
                let formatted = if !new_handle.starts_with('@') && !new_handle.starts_with("UC") {
                    format!("@{}", new_handle)
                } else {
                    new_handle
                };
                self.config.channels[idx] = formatted.clone();
                let _ = self.config.save();
                self.editing_channel_idx = None;

                // Update or reload
                if let Some(pos) = self.channels.iter().position(|c| c.handle == formatted) {
                    self.channels[pos] = ChannelData::new_loading(&formatted);
                } else {
                    self.channels.push(ChannelData::new_loading(&formatted));
                }
                let tx = self.tx.clone();
                let h = formatted.clone();
                std::thread::spawn(move || {
                    let fetcher = std::sync::Arc::new(YouTubeFetcher::new());
                    let data = fetcher.fetch_channel(&h);
                    let _ = tx.send(WorkerMessage::ChannelUpdated(Box::new(data)));
                });
            }
        }

        if let Some(idx) = to_move_up_idx {
            self.config.channels.swap(idx, idx - 1);
            let _ = self.config.save();
        }

        if let Some(idx) = to_move_down_idx {
            self.config.channels.swap(idx, idx + 1);
            let _ = self.config.save();
        }

        if let Some(idx) = to_delete_idx {
            let removed = self.config.channels.remove(idx);
            self.channels.retain(|c| c.handle != removed);
            let _ = self.config.save();
        }

        ui.add_space(6.0);

        // 4. FOOTER
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Total channels: {}", self.config.channels.len()))
                    .size(11.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let done_btn = ui.add_sized(
                    [80.0, 22.0],
                    egui::Button::new(egui::RichText::new("✓ Done").size(12.0).color(Color32::WHITE))
                        .fill(Color32::from_rgb(59, 130, 246))
                        .rounding(Rounding::same(5.0)),
                );
                if done_btn.clicked() {
                    self.show_profile_manager = false;
                    self.editing_channel_idx = None;
                    if self.is_compact {
                        let compact_h = (20.0 + self.channels.len() as f32 * 36.0).clamp(54.0, 450.0);
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(205.0, compact_h)));
                    }
                }
            });
        });
    }

    #[cfg(windows)]
    fn update_hwnd_and_border_win32(&mut self) {
        if self.main_hwnd == 0 {
            let active_hwnd = unsafe { win32_drag::GetActiveWindow() };
            if active_hwnd != 0 {
                self.main_hwnd = active_hwnd;
            } else {
                let mut wide: Vec<u16> = "SocialCount - YouTube Channels".encode_utf16().collect();
                wide.push(0);
                let found = unsafe { win32_drag::FindWindowW(std::ptr::null(), wide.as_ptr()) };
                if found != 0 {
                    self.main_hwnd = found;
                }
            }
        }
        if self.main_hwnd != 0 && !self.border_removed {
            win32_drag::remove_window_borders(self.main_hwnd);
            self.border_removed = true;
        }
    }

    #[cfg(windows)]
    fn start_win32_drag(&mut self) {
        if self.main_hwnd == 0 {
            self.update_hwnd_and_border_win32();
        }
        if self.main_hwnd != 0 && self.win32_drag_state.is_none() {
            let hwnd = self.main_hwnd;
            let mut cur = win32_drag::Point::default();
            let mut rect = win32_drag::Rect::default();
            unsafe {
                if win32_drag::GetCursorPos(&mut cur) != 0
                    && win32_drag::GetWindowRect(hwnd, &mut rect) != 0
                {
                    let cx = (rect.left + rect.right) / 2;
                    let cy = (rect.top + rect.bottom) / 2;
                    self.win32_drag_state = Some(Win32DragState {
                        hwnd,
                        offset_cx: cur.x - cx,
                        offset_cy: cur.y - cy,
                        start_x: cur.x,
                        start_y: cur.y,
                        moved: false,
                    });
                }
            }
        }
    }

    #[cfg(windows)]
    fn update_win32_drag(&mut self, ctx: &egui::Context) {
        if let Some(mut tr) = self.win32_drag_state {
            let down = unsafe {
                (win32_drag::GetAsyncKeyState(win32_drag::VK_LBUTTON) as u16 & 0x8000) != 0
            };
            if !down {
                self.win32_drag_state = None;
            } else {
                let mut cur = win32_drag::Point::default();
                let mut rect = win32_drag::Rect::default();
                unsafe {
                    if win32_drag::GetCursorPos(&mut cur) != 0
                        && win32_drag::GetWindowRect(tr.hwnd, &mut rect) != 0
                    {
                        if (cur.x - tr.start_x).abs() > 2 || (cur.y - tr.start_y).abs() > 2 {
                            tr.moved = true;
                        }
                        if tr.moved {
                            let w = rect.right - rect.left;
                            let h = rect.bottom - rect.top;
                            let new_cx = cur.x - tr.offset_cx;
                            let new_cy = cur.y - tr.offset_cy;
                            let x = new_cx - w / 2;
                            let y = new_cy - h / 2;
                            win32_drag::SetWindowPos(
                                tr.hwnd,
                                0,
                                x,
                                y,
                                0,
                                0,
                                win32_drag::SWP_NOSIZE
                                    | win32_drag::SWP_NOZORDER
                                    | win32_drag::SWP_NOACTIVATE,
                            );
                        }
                    }
                }
                self.win32_drag_state = Some(tr);
                ctx.request_repaint();
            }
        }
    }
}
