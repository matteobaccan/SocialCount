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
            let has_border_style = (style & (WS_CAPTION | WS_BORDER | WS_DLGFRAME | WS_THICKFRAME))
                != 0
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

            let new_style =
                (style & !(WS_CAPTION | WS_BORDER | WS_DLGFRAME | WS_THICKFRAME)) | WS_POPUP;
            SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);

            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            let new_ex = ex_style
                & !(WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSortColumn {
    Title,
    Type,
    Age,
    Views,
    Likes,
    Comments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
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
    selected_channel_details: Option<String>,
    video_sort_column: VideoSortColumn,
    video_sort_direction: SortDirection,
    export_notification: Option<(String, Instant)>,
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

        let mut initial_channels: Vec<ChannelData> = Vec::new();
        let db_opt = crate::db::Database::open().ok();
        for h in &config.channels {
            let mut data = ChannelData::new_loading(h);
            if let Some(db) = &db_opt {
                if let Ok(Some(cached)) = db.get_channel_by_handle(h) {
                    data.title = cached.title;
                    if cached.subscriber_count > 0 {
                        data.subscriber_text = format!("{} subscribers", cached.subscriber_count);
                        data.subscriber_count = cached.subscriber_count;
                        data.initial_subscriber_count = Some(cached.subscriber_count);
                    }
                    if cached.view_count > 0 {
                        data.views_text = format!("{} views", cached.view_count);
                        data.views_count = cached.view_count;
                        data.initial_views_count = Some(cached.view_count);
                    }
                    if cached.video_count > 0 {
                        data.video_count = cached.video_count;
                        data.video_count_text = format!("{} videos", cached.video_count);
                        data.initial_video_count = Some(cached.video_count);
                    }
                    data.channel_id = Some(cached.channel_id.clone());
                    if !cached.uploads_playlist_id.is_empty() {
                        data.uploads_playlist_id = Some(cached.uploads_playlist_id);
                    }
                    if let Ok(comments) = db.get_channel_total_comments(&cached.channel_id) {
                        data.total_comments = comments;
                        data.initial_total_comments = Some(comments);
                    }
                    if let Ok(likes) = db.get_channel_total_likes(&cached.channel_id) {
                        data.total_likes = likes;
                        data.initial_total_likes = Some(likes);
                    }
                }
            }
            initial_channels.push(data);
        }

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
            selected_channel_details: None,
            video_sort_column: VideoSortColumn::Age,
            video_sort_direction: SortDirection::Descending,
            export_notification: None,
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
            // Calculate ultra-compact height: ~50px per channel + compact header space
            let compact_h = (22.0 + self.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
            let compact_w = 215.0_f32;
            Vec2::new(compact_w, compact_h)
        } else {
            self.pre_compact_size.unwrap_or_else(|| {
                Vec2::new(
                    self.config.window_width.max(280.0),
                    self.config.window_height.max(160.0),
                )
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
        let api_key = self.config.effective_api_key.clone();

        thread::spawn(move || {
            let fetcher = Arc::new(YouTubeFetcher::with_api_key(api_key));

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
                    self.last_channel_fetch
                        .insert(new_data.handle.clone(), Instant::now());
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
                            let color_image =
                                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                            let texture = ctx.load_texture(
                                format!("avatar_{}", new_data.handle),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.textures.insert(new_data.handle.clone(), texture);
                        }
                    }

                    if let Some(pos) = self
                        .channels
                        .iter()
                        .position(|c| c.handle == new_data.handle)
                    {
                        let old = &self.channels[pos];

                        // Retain initial baseline counts recorded at launch
                        new_data.initial_subscriber_count = old.initial_subscriber_count;
                        new_data.initial_views_count = old.initial_views_count;
                        new_data.initial_video_count = old.initial_video_count;
                        new_data.initial_total_comments = old.initial_total_comments;
                        new_data.initial_total_likes = old.initial_total_likes;

                        // Set initial baseline if not yet recorded and current value > 0
                        if new_data.initial_subscriber_count.is_none()
                            && new_data.subscriber_count > 0
                        {
                            new_data.initial_subscriber_count = Some(new_data.subscriber_count);
                        }
                        if new_data.initial_views_count.is_none() && new_data.views_count > 0 {
                            new_data.initial_views_count = Some(new_data.views_count);
                        }
                        if new_data.initial_video_count.is_none() && new_data.video_count > 0 {
                            new_data.initial_video_count = Some(new_data.video_count);
                        }
                        if new_data.initial_total_comments.is_none() && new_data.total_comments > 0
                        {
                            new_data.initial_total_comments = Some(new_data.total_comments);
                        }
                        if new_data.initial_total_likes.is_none() && new_data.total_likes > 0 {
                            new_data.initial_total_likes = Some(new_data.total_likes);
                        }

                        // Preserve avatar and title if new fetch failed or lacked image
                        if new_data.avatar_bytes.is_none() && old.avatar_bytes.is_some() {
                            new_data.avatar_bytes = old.avatar_bytes.clone();
                        }
                        if (new_data.title.is_empty()
                            || new_data.title == new_data.handle.trim_start_matches('@'))
                            && !old.title.is_empty()
                        {
                            new_data.title = old.title.clone();
                        }
                        if new_data.channel_id.is_none() && old.channel_id.is_some() {
                            new_data.channel_id = old.channel_id.clone();
                        }
                        if new_data.uploads_playlist_id.is_none()
                            && old.uploads_playlist_id.is_some()
                        {
                            new_data.uploads_playlist_id = old.uploads_playlist_id.clone();
                        }
                        if new_data.latest_comment.is_none() && old.latest_comment.is_some() {
                            new_data.latest_comment = old.latest_comment.clone();
                        }

                        // Detect if subscriber count changed compared to previously known valid value
                        if old.subscriber_count > 0
                            && new_data.subscriber_count > 0
                            && new_data.subscriber_count != old.subscriber_count
                        {
                            new_data.sub_changed = true;
                            new_data.sub_changed_at = Some(Instant::now());
                        } else {
                            new_data.sub_changed = old.sub_changed;
                            new_data.sub_changed_at = old.sub_changed_at;
                        }

                        // Detect if view count changed compared to previous value
                        if old.views_count > 0
                            && new_data.views_count > 0
                            && new_data.views_count != old.views_count
                        {
                            new_data.views_changed = true;
                            new_data.views_changed_at = Some(Instant::now());
                        } else {
                            let old_views_clean =
                                crate::youtube::clean_stat_number(&old.views_text);
                            let new_views_clean =
                                crate::youtube::clean_stat_number(&new_data.views_text);
                            if !old_views_clean.is_empty()
                                && old_views_clean != "-"
                                && !new_views_clean.is_empty()
                                && new_views_clean != "-"
                                && old_views_clean != new_views_clean
                            {
                                new_data.views_changed = true;
                                new_data.views_changed_at = Some(Instant::now());
                            } else {
                                new_data.views_changed = old.views_changed;
                                new_data.views_changed_at = old.views_changed_at;
                            }
                        }

                        // Detect if video count changed compared to previous value
                        if old.video_count > 0
                            && new_data.video_count > 0
                            && new_data.video_count != old.video_count
                        {
                            new_data.video_changed = true;
                            new_data.video_changed_at = Some(Instant::now());
                        } else {
                            new_data.video_changed = old.video_changed;
                            new_data.video_changed_at = old.video_changed_at;
                        }

                        // Detect if comments count changed compared to previous value
                        if old.total_comments > 0
                            && new_data.total_comments > 0
                            && new_data.total_comments != old.total_comments
                        {
                            new_data.comments_changed = true;
                            new_data.comments_changed_at = Some(Instant::now());
                        } else {
                            new_data.comments_changed = old.comments_changed;
                            new_data.comments_changed_at = old.comments_changed_at;
                        }

                        // Detect if likes count changed compared to previous value
                        if old.total_likes > 0
                            && new_data.total_likes > 0
                            && new_data.total_likes != old.total_likes
                        {
                            new_data.likes_changed = true;
                            new_data.likes_changed_at = Some(Instant::now());
                        } else {
                            new_data.likes_changed = old.likes_changed;
                            new_data.likes_changed_at = old.likes_changed_at;
                        }

                        self.channels[pos] = new_data;
                    } else {
                        if new_data.initial_subscriber_count.is_none()
                            && new_data.subscriber_count > 0
                        {
                            new_data.initial_subscriber_count = Some(new_data.subscriber_count);
                        }
                        if new_data.initial_views_count.is_none() && new_data.views_count > 0 {
                            new_data.initial_views_count = Some(new_data.views_count);
                        }
                        if new_data.initial_video_count.is_none() && new_data.video_count > 0 {
                            new_data.initial_video_count = Some(new_data.video_count);
                        }
                        if new_data.initial_total_comments.is_none() && new_data.total_comments > 0
                        {
                            new_data.initial_total_comments = Some(new_data.total_comments);
                        }
                        if new_data.initial_total_likes.is_none() && new_data.total_likes > 0 {
                            new_data.initial_total_likes = Some(new_data.total_likes);
                        }
                        self.channels.push(new_data);
                    }

                    self.channels
                        .sort_by_key(|a| std::cmp::Reverse(a.subscriber_count));
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
            format!(
                "{:02}/{:02}/{} {:02}:{:02}:{:02}",
                st.w_day, st.w_month, st.w_year, st.w_hour, st.w_minute, st.w_second
            )
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
                if !self.channels.iter().any(|c| &c.handle == handle)
                    && !to_refresh.contains(handle)
                {
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
                let compact_h = (22.0 + self.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                    215.0, compact_h,
                )));
            }
        }

        // Target size calculation and anti-ballooning stabilization across monitors
        let target_size = if self.show_profile_manager {
            if self.is_compact {
                Vec2::new(350.0, 440.0)
            } else {
                Vec2::new(
                    self.config.window_width.max(340.0),
                    self.config.window_height.max(420.0),
                )
            }
        } else if self.is_compact {
            let compact_h = (22.0 + self.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
            Vec2::new(215.0, compact_h)
        } else {
            Vec2::new(
                self.config.window_width.max(280.0),
                self.config.window_height.max(160.0),
            )
        };

        let pointer_down = ctx.input(|i| i.pointer.any_down());
        if self.resize_drag.is_none() && !pointer_down {
            let current_size = ctx.screen_rect().size();
            if (current_size.x - target_size.x).abs() > 4.0
                || (current_size.y - target_size.y).abs() > 4.0
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
            }
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin::same(4.0)),
            )
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
                    Stroke::new(
                        1.0_f32,
                        Color32::from_white_alpha((35.0 * alpha_factor).round() as u8),
                    )
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
                    .rounding(Rounding::same(
                        if self.is_compact && !self.show_profile_manager {
                            8.0
                        } else {
                            18.0
                        },
                    ))
                    .inner_margin(container_margin)
                    .show(ui, |ui| {
                        if self.show_profile_manager {
                            self.render_profile_manager(ui, ctx, alpha_factor);
                        } else if self.selected_channel_details.is_some() {
                            self.render_channel_details_view(ui, ctx, alpha_factor);
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
                                let (sep_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 1.0),
                                    egui::Sense::hover(),
                                );
                                if sep_alpha > 0 {
                                    ui.painter().hline(
                                        sep_rect.x_range(),
                                        sep_rect.center().y,
                                        Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)),
                                    );
                                }
                                ui.add_space(8.0);
                            } else {
                                ui.add_space(2.0);
                            }

                            // 2. CHANNELS LIST
                            if self.is_compact {
                                // In compact mode scrollbar is unnecessary: direct compact card layout
                                let mut to_approve: Option<(String, CardMetric)> = None;
                                for channel in &self.channels {
                                    if let Some(m) = self.render_channel_card_compact(ui, channel) {
                                        to_approve = Some((channel.handle.clone(), m));
                                    }
                                    ui.add_space(2.0);
                                }
                                if let Some((handle, metric)) = to_approve {
                                    if let Some(ch) =
                                        self.channels.iter_mut().find(|c| c.handle == handle)
                                    {
                                        ch.approve_metric(metric);
                                    }
                                }
                            } else {
                                let footer_height = 26.0_f32;
                                let scroll_height =
                                    (ui.available_height() - footer_height - 12.0_f32)
                                        .max(60.0_f32);

                                egui::ScrollArea::vertical()
                                    .max_height(scroll_height)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        let mut clicked = None;
                                        let mut to_approve: Option<(String, CardMetric)> = None;
                                        for channel in &self.channels {
                                            let (h, m) = self.render_channel_card(ui, channel);
                                            if let Some(h) = h {
                                                clicked = Some(h);
                                            }
                                            if let Some(m) = m {
                                                to_approve = Some((channel.handle.clone(), m));
                                            }
                                            ui.add_space(8.0);
                                        }
                                        if let Some((handle, metric)) = to_approve {
                                            if let Some(ch) = self
                                                .channels
                                                .iter_mut()
                                                .find(|c| c.handle == handle)
                                            {
                                                ch.approve_metric(metric);
                                            }
                                        }
                                        if let Some(h) = clicked {
                                            self.selected_channel_details = Some(h);
                                        }
                                    });

                                ui.add_space(4.0);

                                // Bottom divider line above footer
                                let (sep2_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 1.0),
                                    egui::Sense::hover(),
                                );
                                if sep_alpha > 0 {
                                    ui.painter().hline(
                                        sep2_rect.x_range(),
                                        sep2_rect.center().y,
                                        Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)),
                                    );
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

        let (header_rect, _) =
            ui.allocate_exact_size(Vec2::new(total_w, header_h), egui::Sense::hover());

        // Dedicated drag region for the title bar that does not overlap right-aligned controls
        let controls_w = 195.0_f32;
        let drag_w = (total_w - controls_w).max(40.0);
        let drag_rect = Rect::from_min_size(header_rect.min, Vec2::new(drag_w, header_h));
        let drag_resp = ui.interact(
            drag_rect,
            ui.id().with("header_drag"),
            egui::Sense::click_and_drag(),
        );

        if drag_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if drag_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if drag_resp.double_clicked() {
            self.toggle_compact(ctx);
        }
        if drag_resp.drag_started_by(egui::PointerButton::Primary)
            || drag_resp.dragged_by(egui::PointerButton::Primary)
        {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Sub-UI for interactive buttons on the right
        let mut header_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect)
                .layout(*ui.layout()),
        );

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
            let close_resp = ui
                .allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            if close_hover {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(
                    close_resp.rect.center(),
                    11.0,
                    Color32::from_rgba_unmultiplied(239, 68, 68, 45),
                );
            }
            let close_color = if close_hover {
                Color32::from_rgb(248, 113, 113)
            } else {
                Color32::from_rgb(175, 180, 195)
            };
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

            let settings_resp = ui
                .allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Manage profiles (or right-click)");
            let settings_hover = settings_resp.hovered();
            if settings_hover {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(
                    settings_resp.rect.center(),
                    11.0,
                    Color32::from_white_alpha(25),
                );
            }
            let settings_color = if self.show_profile_manager {
                Color32::from_rgb(147, 197, 253)
            } else if settings_hover {
                Color32::from_rgb(220, 230, 245)
            } else {
                Color32::from_rgb(175, 180, 195)
            };
            crate::icons::draw_gear_icon(
                ui.painter(),
                settings_resp.rect.center(),
                5.5,
                settings_color,
            );
            if settings_resp.clicked() {
                self.show_profile_manager = !self.show_profile_manager;
                self.profile_error_msg = None;
                self.editing_channel_idx = None;
            }

            ui.add_space(2.0);

            let refresh_tooltip = if self.is_loading {
                "Refreshing..."
            } else {
                "Refresh now"
            };
            let refresh_resp = ui
                .allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text(refresh_tooltip);
            let refresh_hover = refresh_resp.hovered();
            if refresh_hover {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(
                    refresh_resp.rect.center(),
                    11.0,
                    Color32::from_white_alpha(25),
                );
            }
            let refresh_color = if self.is_loading {
                Color32::from_rgb(251, 191, 36)
            } else if refresh_hover {
                Color32::from_rgb(147, 197, 253)
            } else {
                Color32::from_rgb(180, 205, 240)
            };
            let refresh_char = if self.is_loading { "..." } else { "⟳" };
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
                let slider = egui::Slider::new(&mut self.card_opacity, 0..=255).show_value(false);
                let slider_resp = ui
                    .add(slider)
                    .on_hover_text(format!("Background opacity: {}%", op_percent));

                if slider_resp.hovered() || slider_resp.dragged() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default);
                }

                if slider_resp.drag_stopped() || (slider_resp.changed() && !slider_resp.dragged()) {
                    self.config.card_opacity = self.card_opacity;
                    let _ = self.config.save();
                }

                let (moon_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                crate::icons::draw_moon_icon(
                    ui.painter(),
                    moon_rect.center(),
                    5.0,
                    Color32::from_rgb(175, 185, 200),
                );
            });
        });
    }

    /// Header for ultra-compact view: minimal drag handle and close button
    fn render_header_compact(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let total_w = ui.available_width();
        let header_h = 13.0_f32;

        let (header_rect, _) =
            ui.allocate_exact_size(Vec2::new(total_w, header_h), egui::Sense::hover());

        let close_w = 20.0_f32;
        let drag_w = (total_w - close_w).max(20.0);
        let drag_rect = Rect::from_min_size(header_rect.min, Vec2::new(drag_w, header_h));
        let drag_resp = ui.interact(
            drag_rect,
            ui.id().with("compact_drag"),
            egui::Sense::click_and_drag(),
        );

        if drag_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if drag_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if drag_resp.double_clicked() {
            self.toggle_compact(ctx);
        }
        if drag_resp.drag_started_by(egui::PointerButton::Primary)
            || drag_resp.dragged_by(egui::PointerButton::Primary)
        {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        // Minimalist central drag pill
        let handle_color = if drag_resp.hovered() || drag_resp.dragged() {
            Color32::from_white_alpha(120)
        } else {
            Color32::from_white_alpha(45)
        };
        let handle_rect = Rect::from_center_size(header_rect.center(), Vec2::new(28.0, 3.0));
        ui.painter()
            .rect_filled(handle_rect, Rounding::same(1.5), handle_color);

        // Micro logo on the left
        let logo_rect = Rect::from_center_size(
            Pos2::new(header_rect.min.x + 8.0, header_rect.center().y),
            Vec2::new(12.0, 8.5),
        );
        crate::icons::draw_youtube_icon(ui.painter(), logo_rect);

        // Tiny close button on the right
        let mut header_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect)
                .layout(*ui.layout()),
        );
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_resp = ui
                .allocate_response(Vec2::splat(13.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            if close_hover {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
            let close_color = if close_hover {
                Color32::from_rgb(248, 113, 113)
            } else {
                Color32::from_rgb(140, 145, 160)
            };
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
        let total_w = ui.available_width();
        let footer_h = 18.0_f32;
        let (footer_rect, _) =
            ui.allocate_exact_size(Vec2::new(total_w, footer_h), egui::Sense::hover());

        let painter = ui.painter();
        let time_now = ui.input(|i| i.time);

        // Center content: live indicator dot + last updated timestamp
        let updated_str = format!("Updated: {}", self.last_refresh_str);
        let updated_font = egui::FontId::proportional(11.0);
        let updated_color = Color32::from_rgb(148, 163, 184);

        let text_galley = ui
            .fonts(|f| f.layout_no_wrap(updated_str.clone(), updated_font.clone(), updated_color));
        let dot_spacing = 6.0_f32;
        let dot_radius = 3.5_f32;
        let center_content_w = (dot_radius * 2.0) + dot_spacing + text_galley.size().x;

        let start_x = footer_rect.center().x - (center_content_w / 2.0);
        let dot_pos = Pos2::new(start_x + dot_radius, footer_rect.center().y);
        crate::icons::draw_live_dot(painter, dot_pos, self.is_loading, time_now);

        let text_pos = Pos2::new(
            start_x + (dot_radius * 2.0) + dot_spacing,
            footer_rect.center().y,
        );
        painter.text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            updated_str,
            updated_font,
            updated_color,
        );

        // Bottom right: version indication
        let version_str = concat!("v", env!("CARGO_PKG_VERSION"));
        let version_font = egui::FontId::proportional(10.5);
        let version_color = Color32::from_rgb(100, 116, 139);

        let version_galley = ui.fonts(|f| {
            f.layout_no_wrap(version_str.to_string(), version_font.clone(), version_color)
        });
        let version_pos = Pos2::new(footer_rect.max.x - 10.0, footer_rect.center().y);

        let version_rect = Rect::from_min_size(
            Pos2::new(
                footer_rect.max.x - 10.0 - version_galley.size().x,
                footer_rect.center().y - version_galley.size().y / 2.0,
            ),
            version_galley.size(),
        );

        ui.interact(
            version_rect,
            egui::Id::new("app_version"),
            egui::Sense::hover(),
        )
        .on_hover_text(format!("SocialCount {}", version_str));

        painter.text(
            version_pos,
            egui::Align2::RIGHT_CENTER,
            version_str,
            version_font,
            version_color,
        );
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

        let se_resp = ui.interact(
            se_rect,
            egui::Id::new("resize_se"),
            egui::Sense::click_and_drag(),
        );
        let east_resp = ui.interact(
            east_rect,
            egui::Id::new("resize_east"),
            egui::Sense::click_and_drag(),
        );
        let south_resp = ui.interact(
            south_rect,
            egui::Id::new("resize_south"),
            egui::Sense::click_and_drag(),
        );

        let min_w = if self.is_compact {
            175.0_f32
        } else {
            280.0_f32
        };
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
            || matches!(
                self.resize_drag,
                Some(ResizeDragState {
                    mode: ResizeMode::SouthEast,
                    ..
                })
            );
        let is_east_active = east_resp.hovered()
            || matches!(
                self.resize_drag,
                Some(ResizeDragState {
                    mode: ResizeMode::East,
                    ..
                })
            );
        let is_south_active = south_resp.hovered()
            || matches!(
                self.resize_drag,
                Some(ResizeDragState {
                    mode: ResizeMode::South,
                    ..
                })
            );

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
            painter.line_segment(
                [p, p - Vec2::new(3.0, 0.0)],
                Stroke::new(1.6_f32, grip_color),
            );
            painter.line_segment(
                [p - Vec2::new(5.0, 0.0), p - Vec2::new(0.0, 5.0)],
                Stroke::new(1.6_f32, grip_color),
            );
            painter.line_segment(
                [p - Vec2::new(9.0, 0.0), p - Vec2::new(0.0, 9.0)],
                Stroke::new(1.6_f32, grip_color),
            );
        }
    }

    /// Normal view: full channel card with avatar, title, handle and solid stat pills
    fn render_channel_card(
        &self,
        ui: &mut egui::Ui,
        channel: &ChannelData,
    ) -> (Option<String>, Option<CardMetric>) {
        let opacity = self.card_opacity;
        let alpha_factor = opacity as f32 / 255.0;
        let mut open_details = None;
        let mut approve_metric = None;

        let card_frame = egui::Frame::none()
            .fill(if opacity == 0 {
                Color32::TRANSPARENT
            } else {
                Color32::from_rgba_unmultiplied(26, 29, 39, (opacity as f32 * 0.95).round() as u8)
            })
            .stroke(if opacity == 0 {
                Stroke::NONE
            } else {
                Stroke::new(
                    1.0_f32,
                    Color32::from_white_alpha((22.0 * alpha_factor).round() as u8),
                )
            })
            .rounding(Rounding::same(14.0))
            .inner_margin(Margin::symmetric(12.0, 10.0));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let avatar_size = 48.0;
                let (avatar_rect, avatar_resp) =
                    ui.allocate_exact_size(Vec2::splat(avatar_size), egui::Sense::click());

                if avatar_resp.clicked() {
                    open_details = Some(channel.handle.clone());
                }
                if avatar_resp.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                avatar_resp.on_hover_text("Click to view full channel stats and videos");

                if let Some(texture) = self.textures.get(&channel.handle) {
                    let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                    ui.painter()
                        .image(texture.id(), avatar_rect, uv, Color32::WHITE);

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
                        let mut title_resp = ui.add(egui::Link::new(
                            egui::RichText::new(&channel.title)
                                .strong()
                                .size(15.0)
                                .color(Color32::from_rgb(255, 255, 255)),
                        ));
                        if let Some(err) = &channel.error {
                            title_resp = title_resp
                                .on_hover_text(format!("Error: {} (Retrying in 1m)", err));
                        } else if channel.has_error() {
                            title_resp =
                                title_resp.on_hover_text("Stat unavailable (Retrying in 1m)");
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

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let details_btn = ui.add_sized(
                                [60.0, 22.0],
                                egui::Button::new(
                                    egui::RichText::new("Details")
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::from_rgb(226, 232, 240)),
                                )
                                .fill(Color32::from_rgba_unmultiplied(42, 48, 64, 190))
                                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(35)))
                                .rounding(Rounding::same(6.0)),
                            );
                            if details_btn.hovered() {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                            }
                            if details_btn.clicked() {
                                open_details = Some(channel.handle.clone());
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Counter pills: Responsive to window width, customizable visibility
                    let avail_w = ui.available_width();
                    let gap = 8.0_f32;
                    let box_h = 28.0_f32;

                    let mut visible_metrics = Vec::new();
                    if self.config.show_subscribers {
                        visible_metrics.push(CardMetric::Subscribers);
                    }
                    if self.config.show_views {
                        visible_metrics.push(CardMetric::Views);
                    }
                    if self.config.show_videos {
                        visible_metrics.push(CardMetric::Videos);
                    }
                    if self.config.show_comments {
                        visible_metrics.push(CardMetric::Comments);
                    }
                    if self.config.show_likes {
                        visible_metrics.push(CardMetric::Likes);
                    }

                    for (row_idx, chunk) in visible_metrics.chunks(2).enumerate() {
                        if row_idx > 0 {
                            ui.add_space(5.0);
                        }
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            if chunk.len() == 2 {
                                let box_w = ((avail_w - gap) / 2.0_f32).max(70.0_f32);
                                let box_size = Vec2::new(box_w, box_h);
                                if let Some(m) =
                                    render_card_metric_box(ui, box_size, chunk[0], channel)
                                {
                                    approve_metric = Some(m);
                                }
                                if let Some(m) =
                                    render_card_metric_box(ui, box_size, chunk[1], channel)
                                {
                                    approve_metric = Some(m);
                                }
                            } else {
                                let box_w = avail_w;
                                let box_size = Vec2::new(box_w, box_h);
                                if let Some(m) =
                                    render_card_metric_box(ui, box_size, chunk[0], channel)
                                {
                                    approve_metric = Some(m);
                                }
                            }
                        });
                    }
                });
            });
        });

        (open_details, approve_metric)
    }

    /// ULTRA-COMPACT view: Circular channel icon on left flanked by 2x2 grid of stat pills (Subs, Views, Videos, Comments)
    fn render_channel_card_compact(
        &self,
        ui: &mut egui::Ui,
        channel: &ChannelData,
    ) -> Option<CardMetric> {
        let opacity = self.card_opacity;
        let alpha_factor = opacity as f32 / 255.0;
        let mut approve_metric = None;

        let card_frame = egui::Frame::none()
            .fill(if opacity == 0 {
                Color32::TRANSPARENT
            } else {
                Color32::from_rgba_unmultiplied(22, 25, 34, (opacity as f32 * 0.95).round() as u8)
            })
            .stroke(if opacity == 0 {
                Stroke::NONE
            } else {
                Stroke::new(
                    1.0_f32,
                    Color32::from_white_alpha((18.0 * alpha_factor).round() as u8),
                )
            })
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(4.0, 3.0));

        card_frame.show(ui, |ui| {
            let avatar_size = 32.0_f32;
            let gap_x = 5.0_f32;
            let gap_y = 2.5_f32;

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;

                // --- AVATAR ON THE LEFT ---
                let (avatar_rect, avatar_resp) =
                    ui.allocate_exact_size(Vec2::splat(avatar_size), egui::Sense::click());

                if avatar_resp.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                if let Some(texture) = self.textures.get(&channel.handle) {
                    let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                    ui.painter()
                        .image(texture.id(), avatar_rect, uv, Color32::WHITE);

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
                        egui::FontId::proportional(14.0),
                        Color32::WHITE,
                    );
                }

                if avatar_resp.clicked() {
                    let _ = open::that(&channel.channel_url);
                }
                avatar_resp.on_hover_ui(|ui| {
                    ui.label(
                        egui::RichText::new(&channel.title)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new(&channel.handle)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    if let Some(err) = &channel.error {
                        ui.label(
                            egui::RichText::new(format!("Error: {} (Retrying in 1m)", err))
                                .size(10.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                    } else if channel.has_error() {
                        ui.label(
                            egui::RichText::new("Stat unavailable (Retrying in 1m)")
                                .size(10.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                    }
                });

                // --- STATS ON THE RIGHT (CONFIGURABLE) ---
                let mut visible_metrics = Vec::new();
                if self.config.show_subscribers {
                    visible_metrics.push(CardMetric::Subscribers);
                }
                if self.config.show_views {
                    visible_metrics.push(CardMetric::Views);
                }
                if self.config.show_videos {
                    visible_metrics.push(CardMetric::Videos);
                }
                if self.config.show_comments {
                    visible_metrics.push(CardMetric::Comments);
                }
                if self.config.show_likes {
                    visible_metrics.push(CardMetric::Likes);
                }

                let right_w = ui.available_width().max(80.0);
                let col_gap = 4.0_f32;
                let box_w = ((right_w - col_gap) / 2.0_f32).floor();
                let box_h = 18.0_f32;

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = gap_y;

                    for (row_idx, chunk) in visible_metrics.chunks(2).enumerate() {
                        if row_idx > 0 {
                            ui.add_space(gap_y);
                        }
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = col_gap;
                            if chunk.len() == 2 {
                                let box_size = Vec2::new(box_w, box_h);
                                if let Some(m) =
                                    render_compact_metric_box(ui, box_size, chunk[0], channel)
                                {
                                    approve_metric = Some(m);
                                }
                                if let Some(m) =
                                    render_compact_metric_box(ui, box_size, chunk[1], channel)
                                {
                                    approve_metric = Some(m);
                                }
                            } else {
                                let box_size = Vec2::new(right_w, box_h);
                                if let Some(m) =
                                    render_compact_metric_box(ui, box_size, chunk[0], channel)
                                {
                                    approve_metric = Some(m);
                                }
                            }
                        });
                    }
                });
            });
        });

        approve_metric
    }
    /// Complete Profile Manager (CRUD) modal accessible via right-click or settings button
    fn render_profile_manager(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        alpha_factor: f32,
    ) {
        let total_w = ui.available_width();
        let header_h = 26.0_f32;

        // 1. Header with drag handle and close button
        let (header_rect, _) =
            ui.allocate_exact_size(Vec2::new(total_w, header_h), egui::Sense::hover());

        let close_w = 28.0_f32;
        let drag_w = (total_w - close_w).max(40.0);
        let drag_rect = Rect::from_min_size(header_rect.min, Vec2::new(drag_w, header_h));
        let drag_resp = ui.interact(
            drag_rect,
            ui.id().with("profile_drag"),
            egui::Sense::click_and_drag(),
        );

        if drag_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if drag_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if drag_resp.drag_started_by(egui::PointerButton::Primary)
            || drag_resp.dragged_by(egui::PointerButton::Primary)
        {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.painter().text(
            Pos2::new(header_rect.min.x + 4.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Manage YouTube Channels",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(255, 255, 255),
        );

        let mut header_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect)
                .layout(*ui.layout()),
        );
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let close_resp = ui
                .allocate_response(Vec2::splat(22.0), egui::Sense::click())
                .on_hover_text("Close");
            let close_hover = close_resp.hovered();
            if close_hover {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                ui.painter().circle_filled(
                    close_resp.rect.center(),
                    11.0,
                    Color32::from_rgba_unmultiplied(239, 68, 68, 45),
                );
            }
            let close_color = if close_hover {
                Color32::from_rgb(248, 113, 113)
            } else {
                Color32::from_rgb(175, 180, 195)
            };
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
                    let compact_h = (22.0 + self.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                        215.0, compact_h,
                    )));
                }
            }
        });

        // Divider line
        ui.add_space(8.0);
        let sep_alpha = (25.0 * alpha_factor).round() as u8;
        let (sep_rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
        if sep_alpha > 0 {
            ui.painter().hline(
                sep_rect.x_range(),
                sep_rect.center().y,
                Stroke::new(1.0_f32, Color32::from_white_alpha(sep_alpha)),
            );
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

            let enter_pressed =
                input_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let add_btn = ui.add_sized(
                [80.0, 24.0],
                egui::Button::new(
                    egui::RichText::new("+ Add")
                        .size(12.0)
                        .color(Color32::WHITE),
                )
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

                    if self
                        .config
                        .channels
                        .iter()
                        .any(|h| h.eq_ignore_ascii_case(&formatted))
                    {
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
                        let api_key = self.config.effective_api_key.clone();
                        std::thread::spawn(move || {
                            let fetcher =
                                std::sync::Arc::new(YouTubeFetcher::with_api_key(api_key));
                            let data = fetcher.fetch_channel(&h);
                            let _ = tx.send(WorkerMessage::ChannelUpdated(Box::new(data)));
                        });
                    }
                }
            }
        });

        if let Some(err) = &self.profile_error_msg {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!("Error: {}", err))
                    .size(11.0)
                    .color(Color32::from_rgb(239, 68, 68)),
            );
        }

        ui.add_space(6.0);

        // 3. CHANNELS LIST (READ, UPDATE, DELETE)
        let list_h = (ui.available_height() - 95.0).max(80.0);
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
                                            [(ui.available_width() - 85.0).max(100.0), 22.0],
                                            egui::TextEdit::singleline(
                                                &mut self.editing_channel_input,
                                            ),
                                        );

                                        if ui.small_button("Save").on_hover_text("Save").clicked() {
                                            to_save_edit = Some((
                                                idx,
                                                self.editing_channel_input.trim().to_string(),
                                            ));
                                        }
                                        if ui
                                            .small_button("Cancel")
                                            .on_hover_text("Cancel")
                                            .clicked()
                                        {
                                            self.editing_channel_idx = None;
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        // Mini avatar or icon
                                        let (avatar_rect, _) = ui.allocate_exact_size(
                                            Vec2::splat(22.0),
                                            egui::Sense::hover(),
                                        );
                                        let mut drawn = false;
                                        if let Some(c) = ch_data {
                                            if let Some(tex) = self.textures.get(&c.handle) {
                                                ui.painter().image(
                                                    tex.id(),
                                                    avatar_rect,
                                                    Rect::from_min_max(
                                                        Pos2::ZERO,
                                                        Pos2::new(1.0, 1.0),
                                                    ),
                                                    Color32::WHITE,
                                                );
                                                drawn = true;
                                            }
                                        }
                                        if !drawn {
                                            crate::icons::draw_youtube_icon(
                                                ui.painter(),
                                                Rect::from_center_size(
                                                    avatar_rect.center(),
                                                    Vec2::new(16.0, 11.5),
                                                ),
                                            );
                                        }

                                        ui.add_space(4.0);

                                        // Title and handle
                                        let title = ch_data
                                            .map(|c| c.title.as_str())
                                            .unwrap_or(handle.as_str());
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
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // Delete (D)
                                                let del_btn = ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("Delete")
                                                                .size(10.5)
                                                                .color(Color32::from_rgb(
                                                                    248, 113, 113,
                                                                )),
                                                        )
                                                        .fill(Color32::from_rgba_unmultiplied(
                                                            239, 68, 68, 25,
                                                        ))
                                                        .rounding(Rounding::same(4.0)),
                                                    )
                                                    .on_hover_text("Delete channel");
                                                if del_btn.clicked() {
                                                    to_delete_idx = Some(idx);
                                                }

                                                // Edit (U)
                                                let edit_btn = ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("Edit")
                                                                .size(10.5)
                                                                .color(Color32::from_rgb(
                                                                    147, 197, 253,
                                                                )),
                                                        )
                                                        .fill(Color32::from_white_alpha(15))
                                                        .rounding(Rounding::same(4.0)),
                                                    )
                                                    .on_hover_text("Edit handle");
                                                if edit_btn.clicked() {
                                                    self.editing_channel_idx = Some(idx);
                                                    self.editing_channel_input = handle.clone();
                                                }

                                                // Move down (U - Reorder)
                                                if idx + 1 < self.config.channels.len()
                                                    && ui
                                                        .small_button("▼")
                                                        .on_hover_text("Move down")
                                                        .clicked()
                                                {
                                                    to_move_down_idx = Some(idx);
                                                }

                                                // Move up (U - Reorder)
                                                if idx > 0
                                                    && ui
                                                        .small_button("▲")
                                                        .on_hover_text("Move up")
                                                        .clicked()
                                                {
                                                    to_move_up_idx = Some(idx);
                                                }
                                            },
                                        );
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
                let api_key = self.config.effective_api_key.clone();
                std::thread::spawn(move || {
                    let fetcher = std::sync::Arc::new(YouTubeFetcher::with_api_key(api_key));
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

        // 4. DISPLAYED METRICS CONFIGURATION
        egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 8))
            .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(15)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(8.0, 5.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Displayed Metrics in Cards:")
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(226, 232, 240)),
                );
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    let mut changed = false;
                    if ui
                        .checkbox(&mut self.config.show_subscribers, "Subscribers")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui.checkbox(&mut self.config.show_views, "Views").changed() {
                        changed = true;
                    }
                    if ui
                        .checkbox(&mut self.config.show_videos, "Videos")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .checkbox(&mut self.config.show_comments, "Comments")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui.checkbox(&mut self.config.show_likes, "Likes").changed() {
                        changed = true;
                    }
                    if changed {
                        let _ = self.config.save();
                    }
                });
            });

        ui.add_space(6.0);

        // 5. FOOTER
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Total channels: {}", self.config.channels.len()))
                    .size(11.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let done_btn = ui.add_sized(
                    [80.0, 22.0],
                    egui::Button::new(egui::RichText::new("Done").size(12.0).color(Color32::WHITE))
                        .fill(Color32::from_rgb(59, 130, 246))
                        .rounding(Rounding::same(5.0)),
                );
                if done_btn.clicked() {
                    self.show_profile_manager = false;
                    self.editing_channel_idx = None;
                    if self.is_compact {
                        let compact_h =
                            (22.0 + self.channels.len() as f32 * 50.0).clamp(60.0, 500.0);
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                            215.0, compact_h,
                        )));
                    }
                }
            });
        });
    }

    /// Detailed view for a single channel with full metrics and recent video history from SQLite
    #[allow(unused_variables)]
    fn render_channel_details_view(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        _alpha_factor: f32,
    ) {
        let handle = match &self.selected_channel_details {
            Some(h) => h.clone(),
            None => return,
        };

        let channel = match self.channels.iter().find(|c| c.handle == handle) {
            Some(c) => c.clone(),
            None => {
                self.selected_channel_details = None;
                return;
            }
        };

        let total_w = ui.available_width();
        let header_h = 26.0_f32;

        let (header_rect, _) =
            ui.allocate_exact_size(Vec2::new(total_w, header_h), egui::Sense::hover());

        let drag_w = (total_w - 100.0).max(40.0);
        let drag_rect = Rect::from_min_size(
            Pos2::new(header_rect.min.x + 65.0, header_rect.min.y),
            Vec2::new(drag_w, header_h),
        );
        let drag_resp = ui.interact(
            drag_rect,
            ui.id().with("details_drag"),
            egui::Sense::click_and_drag(),
        );

        if drag_resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
        if drag_resp.dragged() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        if drag_resp.drag_started_by(egui::PointerButton::Primary)
            || drag_resp.dragged_by(egui::PointerButton::Primary)
        {
            #[cfg(windows)]
            self.start_win32_drag();
            #[cfg(not(windows))]
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let mut header_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect)
                .layout(*ui.layout()),
        );

        let mut back_clicked = false;
        header_ui.horizontal(|ui| {
            let back_resp = ui.add_sized(
                [64.0, 22.0],
                egui::Button::new(
                    egui::RichText::new("< Back")
                        .size(11.5)
                        .strong()
                        .color(Color32::from_rgb(226, 232, 240)),
                )
                .fill(Color32::from_rgba_unmultiplied(42, 48, 64, 190))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(35)))
                .rounding(Rounding::same(6.0)),
            );
            if back_resp.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
            if back_resp.clicked() {
                back_clicked = true;
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Channel Details")
                    .size(14.0)
                    .strong()
                    .color(Color32::WHITE),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let close_resp = ui
                    .allocate_response(Vec2::splat(22.0), egui::Sense::click())
                    .on_hover_text("Close");
                if close_resp.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                if close_resp.clicked() {
                    back_clicked = true;
                }
                let close_color = if close_resp.hovered() {
                    Color32::from_rgb(239, 68, 68)
                } else {
                    Color32::from_rgb(148, 163, 184)
                };
                let r = close_resp.rect.center();
                let sz = 4.0_f32;
                ui.painter().line_segment(
                    [Pos2::new(r.x - sz, r.y - sz), Pos2::new(r.x + sz, r.y + sz)],
                    Stroke::new(1.4_f32, close_color),
                );
                ui.painter().line_segment(
                    [Pos2::new(r.x + sz, r.y - sz), Pos2::new(r.x - sz, r.y + sz)],
                    Stroke::new(1.4_f32, close_color),
                );
            });
        });

        if back_clicked {
            self.selected_channel_details = None;
            return;
        }

        ui.add_space(8.0);

        let scroll_h = (ui.available_height() - 10.0).max(100.0);
        egui::ScrollArea::vertical()
            .max_height(scroll_h)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let avatar_size = 40.0;
                    let (avatar_rect, _) =
                        ui.allocate_exact_size(Vec2::splat(avatar_size), egui::Sense::hover());
                    if let Some(texture) = self.textures.get(&channel.handle) {
                        let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                        ui.painter()
                            .image(texture.id(), avatar_rect, uv, Color32::WHITE);
                    } else {
                        ui.painter().circle_filled(
                            avatar_rect.center(),
                            avatar_size / 2.0,
                            Color32::from_rgb(45, 52, 70),
                        );
                    }

                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&channel.title)
                                .strong()
                                .size(15.0)
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(&channel.handle)
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let yt_btn = ui.add_sized(
                            [120.0, 24.0],
                            egui::Button::new(
                                egui::RichText::new("Open on YouTube")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(204, 0, 0))
                            .stroke(Stroke::new(
                                1.0_f32,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 45),
                            ))
                            .rounding(Rounding::same(6.0)),
                        );
                        if yt_btn.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        if yt_btn.clicked() {
                            let _ = open::that(&channel.channel_url);
                        }

                        let export_btn = ui.add_sized(
                            [85.0, 24.0],
                            egui::Button::new(
                                egui::RichText::new("Export CSV")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(16, 149, 106))
                            .stroke(Stroke::new(
                                1.0_f32,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 45),
                            ))
                            .rounding(Rounding::same(6.0)),
                        );
                        if export_btn.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        if export_btn.clicked() {
                            match export_channel_to_csv(&channel) {
                                Ok(path) => {
                                    self.export_notification =
                                        Some((format!("Exported to {}", path), Instant::now()));
                                }
                                Err(err) => {
                                    self.export_notification =
                                        Some((format!("Export failed: {}", err), Instant::now()));
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                let mut channel_id_to_query = channel.channel_id.clone();
                if channel_id_to_query.is_none() {
                    if let Ok(db) = crate::db::Database::open() {
                        if let Ok(Some(cached)) = db.get_channel_by_handle(&channel.handle) {
                            channel_id_to_query = Some(cached.channel_id);
                        }
                    }
                }

                let mut approve_metric = None;
                let card_w = ((ui.available_width() - 8.0) / 2.0).max(80.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let p = render_detail_stat_pill(
                            ui,
                            Vec2::new(card_w, 42.0),
                            "Subscribers",
                            &crate::youtube::clean_stat_number(&channel.subscriber_text),
                            channel.delta_subscribers(),
                            Color32::from_rgb(16, 44, 28),
                            Color32::from_rgb(52, 211, 153),
                            egui::Sense::click(),
                        );
                        if p.double_clicked() {
                            approve_metric = Some(CardMetric::Subscribers);
                        }
                    });
                    ui.vertical(|ui| {
                        let p = render_detail_stat_pill(
                            ui,
                            Vec2::new(card_w, 42.0),
                            "Views",
                            &crate::youtube::clean_stat_number(&channel.views_text),
                            channel.delta_views(),
                            Color32::from_rgb(18, 36, 58),
                            Color32::from_rgb(96, 165, 250),
                            egui::Sense::click(),
                        );
                        if p.double_clicked() {
                            approve_metric = Some(CardMetric::Views);
                        }
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let v_num = if channel.video_count > 0 {
                            crate::youtube::clean_stat_number(&channel.video_count.to_string())
                        } else {
                            crate::youtube::clean_stat_number(&channel.video_count_text)
                        };
                        let p = render_detail_stat_pill(
                            ui,
                            Vec2::new(card_w, 42.0),
                            "Total Videos",
                            &v_num,
                            channel.delta_video_count(),
                            Color32::from_rgb(38, 22, 54),
                            Color32::from_rgb(192, 132, 252),
                            egui::Sense::click(),
                        );
                        if p.double_clicked() {
                            approve_metric = Some(CardMetric::Videos);
                        }
                    });
                    ui.vertical(|ui| {
                        let total_likes = if channel.total_likes > 0 {
                            channel.total_likes
                        } else if let Some(cid) = &channel_id_to_query {
                            crate::db::Database::open()
                                .ok()
                                .and_then(|db| db.get_channel_total_likes(cid).ok())
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        let l_num = crate::youtube::clean_stat_number(&total_likes.to_string());
                        let p = render_detail_stat_pill(
                            ui,
                            Vec2::new(card_w, 42.0),
                            "Tracked Likes",
                            &l_num,
                            channel.delta_likes(),
                            Color32::from_rgb(48, 18, 28),
                            Color32::from_rgb(251, 113, 133),
                            egui::Sense::click(),
                        );
                        if p.double_clicked() {
                            approve_metric = Some(CardMetric::Likes);
                        }
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let total_comments = if channel.total_comments > 0 {
                            channel.total_comments
                        } else if let Some(cid) = &channel_id_to_query {
                            crate::db::Database::open()
                                .ok()
                                .and_then(|db| db.get_channel_total_comments(cid).ok())
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        let c_num =
                            crate::youtube::clean_stat_number(&total_comments.to_string());

                        let latest_comm = channel.latest_comment.clone().or_else(|| {
                            channel_id_to_query.as_ref().and_then(|cid| {
                                crate::db::Database::open().ok().and_then(|db| db.get_channel_latest_comment(cid).ok().flatten())
                            })
                        });

                        let pill_resp = render_detail_stat_pill(
                            ui,
                            Vec2::new(ui.available_width(), 42.0),
                            "Tracked Comments",
                            &c_num,
                            channel.delta_comments(),
                            Color32::from_rgb(46, 36, 16),
                            Color32::from_rgb(251, 191, 36),
                            egui::Sense::click(),
                        );

                        if pill_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        if pill_resp.clicked() {
                            let _ = open::that(channel.comments_inbox_url());
                        }

                        if pill_resp.double_clicked() {
                            approve_metric = Some(CardMetric::Comments);
                        }

                        show_comments_metric_tooltip(&pill_resp, &channel, &latest_comm, &c_num, channel.delta_comments());
                    });
                });

                if let Some(m) = approve_metric {
                    if let Some(ch) = self.channels.iter_mut().find(|c| c.handle == handle) {
                        ch.approve_metric(m);
                    }
                }

                if let Some((msg, time)) = &self.export_notification {
                    if time.elapsed() < std::time::Duration::from_secs(5) {
                        ui.add_space(6.0);
                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(16, 149, 106, 60))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(Margin::symmetric(10.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(msg)
                                        .size(11.5)
                                        .color(Color32::from_rgb(209, 250, 229)),
                                );
                            });
                    }
                }

                ui.add_space(12.0);

                let mut videos = Vec::new();
                if let Some(cid) = &channel_id_to_query {
                    if let Ok(db) = crate::db::Database::open() {
                        if let Ok(list) = db.get_recent_videos(cid, 50) {
                            videos = list;
                        }
                    }
                }

                if videos.is_empty() {
                    ui.label(
                        egui::RichText::new("Recent Videos & Tracked Comments")
                            .strong()
                            .size(13.0)
                            .color(Color32::from_rgb(226, 232, 240)),
                    );
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(30, 34, 46, 180))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("No cached videos yet in SQLite database.")
                                    .size(11.5)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(
                                    "When a YouTube API key is configured, SocialCount automatically indexes recent uploads and tracks per-video views, likes, and comments.",
                                )
                                .size(10.5)
                                .color(Color32::from_rgb(100, 116, 139)),
                            );
                        });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Videos ({})", videos.len()))
                                .strong()
                                .size(13.0)
                                .color(Color32::from_rgb(226, 232, 240)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Click column to sort")
                                    .size(10.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        });
                    });
                    ui.add_space(6.0);

                    let table_w = ui.available_width().max(350.0);
                    let type_w = 44.0_f32;
                    let age_w = 48.0_f32;
                    let views_w = 46.0_f32;
                    let likes_w = 44.0_f32;
                    let comments_w = 40.0_f32;
                    let col_gap = 4.0_f32;
                    let pad_x = 6.0_f32;
                    let metrics_w =
                        type_w + age_w + views_w + likes_w + comments_w + (col_gap * 5.0) + (pad_x * 2.0);
                    let title_w = (table_w - metrics_w).max(80.0);

                    egui::ScrollArea::horizontal()
                        .id_salt("videos_table_h_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // 1. Table Header
                            let header_h = 24.0_f32;
                            let (header_rect, _) = ui.allocate_exact_size(
                                Vec2::new(table_w, header_h),
                                egui::Sense::hover(),
                            );

                            ui.painter().rect_filled(
                                header_rect,
                                Rounding {
                                    nw: 6.0,
                                    ne: 6.0,
                                    sw: 0.0,
                                    se: 0.0,
                                },
                                Color32::from_rgba_unmultiplied(32, 38, 52, 220),
                            );
                            ui.painter().rect_stroke(
                                header_rect,
                                Rounding {
                                    nw: 6.0,
                                    ne: 6.0,
                                    sw: 0.0,
                                    se: 0.0,
                                },
                                Stroke::new(1.0_f32, Color32::from_white_alpha(20)),
                            );

                            let x0 = header_rect.left() + pad_x;
                            let y0 = header_rect.top();
                            let title_hdr_rect = Rect::from_min_size(
                                Pos2::new(x0, y0),
                                Vec2::new(title_w, header_h),
                            );
                            let type_hdr_rect = Rect::from_min_size(
                                Pos2::new(title_hdr_rect.right() + col_gap, y0),
                                Vec2::new(type_w, header_h),
                            );
                            let age_hdr_rect = Rect::from_min_size(
                                Pos2::new(type_hdr_rect.right() + col_gap, y0),
                                Vec2::new(age_w, header_h),
                            );
                            let views_hdr_rect = Rect::from_min_size(
                                Pos2::new(age_hdr_rect.right() + col_gap, y0),
                                Vec2::new(views_w, header_h),
                            );
                            let likes_hdr_rect = Rect::from_min_size(
                                Pos2::new(views_hdr_rect.right() + col_gap, y0),
                                Vec2::new(likes_w, header_h),
                            );
                            let comms_hdr_rect = Rect::from_min_size(
                                Pos2::new(likes_hdr_rect.right() + col_gap, y0),
                                Vec2::new(comments_w, header_h),
                            );

                            let mut clicked_sort = None;

                            // Title Column Header
                            let is_title_active =
                                self.video_sort_column == VideoSortColumn::Title;
                            let is_title_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                title_hdr_rect,
                                "Video",
                                false,
                                is_title_active,
                                is_title_asc,
                                Color32::WHITE,
                                if is_title_active {
                                    if is_title_asc {
                                        "Sorted A to Z. Click to sort Z to A"
                                    } else {
                                        "Sorted Z to A. Click to sort A to Z"
                                    }
                                } else {
                                    "Click to sort alphabetically by title"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Title);
                            }

                            // Type Column Header
                            let is_type_active =
                                self.video_sort_column == VideoSortColumn::Type;
                            let is_type_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                type_hdr_rect,
                                "Type",
                                true,
                                is_type_active,
                                is_type_asc,
                                Color32::from_rgb(251, 113, 133),
                                if is_type_active {
                                    if is_type_asc {
                                        "Sorted Videos first. Click to sort Shorts first"
                                    } else {
                                        "Sorted Shorts first. Click to sort Videos first"
                                    }
                                } else {
                                    "Click to sort by video type (Short / Video)"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Type);
                            }

                            // Age Column Header
                            let is_age_active = self.video_sort_column == VideoSortColumn::Age;
                            let is_age_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                age_hdr_rect,
                                "Age",
                                true,
                                is_age_active,
                                is_age_asc,
                                Color32::from_rgb(147, 197, 253),
                                if is_age_active {
                                    if is_age_asc {
                                        "Sorted oldest first. Click to sort newest first"
                                    } else {
                                        "Sorted newest first. Click to sort oldest first"
                                    }
                                } else {
                                    "Click to sort by age / publication date"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Age);
                            }

                            // Views Column Header
                            let is_views_active =
                                self.video_sort_column == VideoSortColumn::Views;
                            let is_views_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                views_hdr_rect,
                                "Views",
                                true,
                                is_views_active,
                                is_views_asc,
                                Color32::from_rgb(96, 165, 250),
                                if is_views_active {
                                    if is_views_asc {
                                        "Sorted lowest views first. Click to sort highest first"
                                    } else {
                                        "Sorted highest views first. Click to sort lowest first"
                                    }
                                } else {
                                    "Click to sort by views"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Views);
                            }

                            // Likes Column Header
                            let is_likes_active =
                                self.video_sort_column == VideoSortColumn::Likes;
                            let is_likes_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                likes_hdr_rect,
                                "Likes",
                                true,
                                is_likes_active,
                                is_likes_asc,
                                Color32::from_rgb(52, 211, 153),
                                if is_likes_active {
                                    if is_likes_asc {
                                        "Sorted lowest likes first. Click to sort highest first"
                                    } else {
                                        "Sorted highest likes first. Click to sort lowest first"
                                    }
                                } else {
                                    "Click to sort by likes"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Likes);
                            }

                            // Comments Column Header
                            let is_comms_active =
                                self.video_sort_column == VideoSortColumn::Comments;
                            let is_comms_asc =
                                self.video_sort_direction == SortDirection::Ascending;
                            if render_table_header_cell(
                                ui,
                                comms_hdr_rect,
                                "Com.",
                                true,
                                is_comms_active,
                                is_comms_asc,
                                Color32::from_rgb(251, 191, 36),
                                if is_comms_active {
                                    if is_comms_asc {
                                        "Sorted lowest comments first. Click to sort highest first"
                                    } else {
                                        "Sorted highest comments first. Click to sort lowest first"
                                    }
                                } else {
                                    "Click to sort by comments"
                                },
                            ) {
                                clicked_sort = Some(VideoSortColumn::Comments);
                            }

                            if let Some(col) = clicked_sort {
                                if self.video_sort_column == col {
                                    self.video_sort_direction = match self.video_sort_direction {
                                        SortDirection::Ascending => SortDirection::Descending,
                                        SortDirection::Descending => SortDirection::Ascending,
                                    };
                                } else {
                                    self.video_sort_column = col;
                                    self.video_sort_direction = match col {
                                        VideoSortColumn::Title => SortDirection::Ascending,
                                        VideoSortColumn::Type => SortDirection::Ascending,
                                        _ => SortDirection::Descending,
                                    };
                                }
                            }

                            // 2. Sort videos according to selected column & direction
                            self.sort_videos(&mut videos);

                            // 3. Render Video Data Rows
                            let row_h = 24.0_f32;
                            let num_videos = videos.len();
                            for (idx, v) in videos.iter().enumerate() {
                                let (row_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(table_w, row_h),
                                    egui::Sense::hover(),
                                );
                                let is_last = idx == num_videos - 1;
                                let is_hovered = ui.rect_contains_pointer(row_rect);

                                let row_bg = if is_hovered {
                                    Color32::from_rgba_unmultiplied(46, 54, 76, 210)
                                } else if idx % 2 == 0 {
                                    Color32::from_rgba_unmultiplied(22, 26, 36, 180)
                                } else {
                                    Color32::from_rgba_unmultiplied(28, 33, 46, 180)
                                };

                                let row_rounding = if is_last {
                                    Rounding {
                                        nw: 0.0,
                                        ne: 0.0,
                                        sw: 6.0,
                                        se: 6.0,
                                    }
                                } else {
                                    Rounding::ZERO
                                };

                                ui.painter().rect_filled(row_rect, row_rounding, row_bg);
                                ui.painter().rect_stroke(
                                    row_rect,
                                    row_rounding,
                                    Stroke::new(1.0_f32, Color32::from_white_alpha(15)),
                                );

                                let rx0 = row_rect.left() + pad_x;
                                let ry0 = row_rect.top();
                                let title_cell_rect = Rect::from_min_size(
                                    Pos2::new(rx0, ry0),
                                    Vec2::new(title_w, row_h),
                                );
                                let type_cell_rect = Rect::from_min_size(
                                    Pos2::new(title_cell_rect.right() + col_gap, ry0),
                                    Vec2::new(type_w, row_h),
                                );
                                let age_cell_rect = Rect::from_min_size(
                                    Pos2::new(type_cell_rect.right() + col_gap, ry0),
                                    Vec2::new(age_w, row_h),
                                );
                                let views_cell_rect = Rect::from_min_size(
                                    Pos2::new(age_cell_rect.right() + col_gap, ry0),
                                    Vec2::new(views_w, row_h),
                                );
                                let likes_cell_rect = Rect::from_min_size(
                                    Pos2::new(views_cell_rect.right() + col_gap, ry0),
                                    Vec2::new(likes_w, row_h),
                                );
                                let comms_cell_rect = Rect::from_min_size(
                                    Pos2::new(likes_cell_rect.right() + col_gap, ry0),
                                    Vec2::new(comments_w, row_h),
                                );

                                // Video Title Cell (Clickable Link)
                                let is_short = v.is_short_video();
                                let video_url = if is_short {
                                    format!("https://www.youtube.com/shorts/{}", v.video_id)
                                } else {
                                    format!("https://www.youtube.com/watch?v={}", v.video_id)
                                };
                                let title_resp = ui
                                    .interact(
                                        title_cell_rect,
                                        ui.id().with(("v_title_click", &v.video_id)),
                                        egui::Sense::click(),
                                    )
                                    .on_hover_text(format!(
                                        "{}\nClick to open in browser",
                                        v.title
                                    ));
                                if title_resp.hovered() {
                                    ui.output_mut(|o| {
                                        o.cursor_icon = egui::CursorIcon::PointingHand
                                    });
                                }
                                if title_resp.clicked() {
                                    let _ = open::that(&video_url);
                                }

                                let title_color = if title_resp.hovered() {
                                    Color32::from_rgb(125, 211, 252)
                                } else {
                                    Color32::from_rgb(241, 245, 249)
                                };

                                let font_id = egui::FontId::proportional(10.5);
                                let title_avail_w = (title_w - 4.0).max(10.0);
                                let truncated_title = truncate_text(
                                    ui,
                                    &v.title,
                                    title_avail_w,
                                    font_id.clone(),
                                );
                                let title_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        truncated_title,
                                        font_id.clone(),
                                        title_color,
                                    )
                                });
                                let title_pos = Pos2::new(
                                    title_cell_rect.left() + 2.0,
                                    title_cell_rect.center().y - title_galley.size().y * 0.5,
                                );
                                ui.painter().galley(title_pos, title_galley.clone(), title_color);
                                if title_resp.hovered() {
                                    ui.painter().hline(
                                        title_pos.x..=(title_pos.x + title_galley.size().x),
                                        title_pos.y + title_galley.size().y,
                                        Stroke::new(1.0_f32, title_color),
                                    );
                                }

                                // Type Badge Cell
                                let type_badge_w = 36.0_f32;
                                let type_badge_h = 15.0_f32;
                                let badge_rect = Rect::from_center_size(
                                    type_cell_rect.center(),
                                    Vec2::new(type_badge_w, type_badge_h),
                                );
                                let (
                                    badge_bg,
                                    badge_stroke,
                                    badge_text_color,
                                    badge_text,
                                    badge_tooltip,
                                ) = if is_short {
                                    (
                                        Color32::from_rgba_unmultiplied(225, 29, 72, 45),
                                        Stroke::new(
                                            1.0_f32,
                                            Color32::from_rgba_unmultiplied(244, 63, 94, 90),
                                        ),
                                        Color32::from_rgb(251, 113, 133),
                                        "Short",
                                        "YouTube Short (vertical format)",
                                    )
                                } else {
                                    (
                                        Color32::from_rgba_unmultiplied(59, 130, 246, 35),
                                        Stroke::new(
                                            1.0_f32,
                                            Color32::from_rgba_unmultiplied(96, 165, 250, 75),
                                        ),
                                        Color32::from_rgb(147, 197, 253),
                                        "Video",
                                        "Standard YouTube Video",
                                    )
                                };

                                ui.painter().rect_filled(
                                    badge_rect,
                                    Rounding::same(3.5_f32),
                                    badge_bg,
                                );
                                ui.painter().rect_stroke(
                                    badge_rect,
                                    Rounding::same(3.5_f32),
                                    badge_stroke,
                                );

                                let badge_font = egui::FontId::proportional(9.0);
                                let badge_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        badge_text.to_string(),
                                        badge_font,
                                        badge_text_color,
                                    )
                                });
                                let badge_pos = Pos2::new(
                                    badge_rect.center().x - badge_galley.size().x * 0.5,
                                    badge_rect.center().y - badge_galley.size().y * 0.5,
                                );
                                ui.painter().galley(badge_pos, badge_galley, badge_text_color);

                                ui.interact(
                                    type_cell_rect,
                                    ui.id().with(("v_type_h", &v.video_id)),
                                    egui::Sense::hover(),
                                )
                                .on_hover_text(badge_tooltip);

                                // Age Cell
                                let age_str = format_unix_timestamp(v.published_at);
                                ui.interact(
                                    age_cell_rect,
                                    ui.id().with(("v_age_h", &v.video_id)),
                                    egui::Sense::hover(),
                                )
                                .on_hover_text(format!("Published: {}", age_str));
                                let age_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        age_str,
                                        font_id.clone(),
                                        Color32::from_rgb(148, 163, 184),
                                    )
                                });
                                let age_pos = Pos2::new(
                                    age_cell_rect.right() - age_galley.size().x - 2.0,
                                    age_cell_rect.center().y - age_galley.size().y * 0.5,
                                );
                                ui.painter().galley(
                                    age_pos,
                                    age_galley,
                                    Color32::from_rgb(148, 163, 184),
                                );

                                // Views Cell
                                let views_str = format_compact_metric(v.views);
                                let views_fmt =
                                    crate::youtube::clean_stat_number(&v.views.to_string());
                                ui.interact(
                                    views_cell_rect,
                                    ui.id().with(("v_views_h", &v.video_id)),
                                    egui::Sense::hover(),
                                )
                                .on_hover_text(format!("{} views", views_fmt));
                                let views_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        views_str,
                                        font_id.clone(),
                                        Color32::from_rgb(147, 197, 253),
                                    )
                                });
                                let views_pos = Pos2::new(
                                    views_cell_rect.right() - views_galley.size().x - 2.0,
                                    views_cell_rect.center().y - views_galley.size().y * 0.5,
                                );
                                ui.painter().galley(
                                    views_pos,
                                    views_galley,
                                    Color32::from_rgb(147, 197, 253),
                                );

                                // Likes Cell
                                let likes_str = format_compact_metric(v.likes);
                                let likes_fmt =
                                    crate::youtube::clean_stat_number(&v.likes.to_string());
                                ui.interact(
                                    likes_cell_rect,
                                    ui.id().with(("v_likes_h", &v.video_id)),
                                    egui::Sense::hover(),
                                )
                                .on_hover_text(format!("{} likes", likes_fmt));
                                let likes_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        likes_str,
                                        font_id.clone(),
                                        Color32::from_rgb(52, 211, 153),
                                    )
                                });
                                let likes_pos = Pos2::new(
                                    likes_cell_rect.right() - likes_galley.size().x - 2.0,
                                    likes_cell_rect.center().y - likes_galley.size().y * 0.5,
                                );
                                ui.painter().galley(
                                    likes_pos,
                                    likes_galley,
                                    Color32::from_rgb(52, 211, 153),
                                );

                                // Comments Cell
                                let comms_str = format_compact_metric(v.comments);
                                let comms_fmt =
                                    crate::youtube::clean_stat_number(&v.comments.to_string());
                                let comms_tooltip = if v.comments == 0 {
                                    "0 comments".to_string()
                                } else if v.last_comment_at > 0 {
                                    format!(
                                        "{} comments\nLast comment: {}",
                                        comms_fmt,
                                        format_unix_timestamp(v.last_comment_at)
                                    )
                                } else {
                                    format!("{} comments", comms_fmt)
                                };
                                ui.interact(
                                    comms_cell_rect,
                                    ui.id().with(("v_comms_h", &v.video_id)),
                                    egui::Sense::hover(),
                                )
                                .on_hover_text(comms_tooltip);
                                let comms_galley = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        comms_str,
                                        font_id.clone(),
                                        Color32::from_rgb(253, 230, 138),
                                    )
                                });
                                let comms_pos = Pos2::new(
                                    comms_cell_rect.right() - comms_galley.size().x - 2.0,
                                    comms_cell_rect.center().y - comms_galley.size().y * 0.5,
                                );
                                ui.painter().galley(
                                    comms_pos,
                                    comms_galley,
                                    Color32::from_rgb(253, 230, 138),
                                );
                            }
                            ui.add_space(4.0);
                        });
                }
            });
    }

    fn sort_videos(&self, videos: &mut [crate::db::CachedVideo]) {
        sort_videos_list(videos, self.video_sort_column, self.video_sort_direction);
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

#[allow(clippy::too_many_arguments)]
fn render_detail_stat_pill(
    ui: &mut egui::Ui,
    size: Vec2,
    label: &str,
    val_str: &str,
    delta: i64,
    bg_color: Color32,
    accent_color: Color32,
    sense: egui::Sense,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    ui.painter()
        .rect_filled(rect, Rounding::same(8.0), bg_color);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(8.0),
        Stroke::new(1.0_f32, accent_color),
    );

    let delta_str = crate::youtube::format_compact_delta(delta);
    let top_y = rect.min.y + 4.0;
    let bot_y = rect.min.y + 20.0;

    ui.painter().text(
        Pos2::new(rect.min.x + 8.0, top_y),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(10.5),
        Color32::from_rgb(148, 163, 184),
    );

    ui.painter().text(
        Pos2::new(rect.min.x + 8.0, bot_y),
        egui::Align2::LEFT_TOP,
        val_str,
        egui::FontId::proportional(14.0),
        Color32::WHITE,
    );

    if !delta_str.is_empty() {
        let delta_color = if delta > 0 {
            Color32::from_rgb(252, 165, 165)
        } else {
            Color32::from_rgb(253, 186, 116)
        };
        ui.painter().text(
            Pos2::new(rect.max.x - 8.0, bot_y + 2.0),
            egui::Align2::RIGHT_TOP,
            delta_str,
            egui::FontId::proportional(11.0),
            delta_color,
        );
    }
    let resp = if delta != 0 {
        resp.on_hover_text(format!(
            "{}: {}\nSince launch: {:+}\nDouble-click to acknowledge changes",
            label, val_str, delta
        ))
    } else {
        resp.on_hover_text(format!("{}: {}", label, val_str))
    };
    if sense == egui::Sense::click() && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn format_unix_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "-".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if now >= ts {
        let diff = now - ts;
        if diff < 3600 {
            format!("{}m ago", (diff / 60).max(1))
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else if diff < 86400 * 30 {
            format!("{}d ago", diff / 86400)
        } else if diff < 86400 * 365 {
            format!("{}mo ago", diff / (86400 * 30))
        } else {
            format!("{}y ago", diff / (86400 * 365))
        }
    } else {
        "recent".to_string()
    }
}

fn show_comments_metric_tooltip(
    resp: &egui::Response,
    channel: &ChannelData,
    latest_comm: &Option<crate::db::LatestCommentInfo>,
    comments_num: &str,
    delta_comments: i64,
) {
    resp.clone().on_hover_ui(|ui| {
        ui.set_max_width(320.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Tracked Comments")
                        .strong()
                        .size(12.5)
                        .color(Color32::from_rgb(251, 191, 36)),
                );
                if delta_comments > 0 {
                    ui.label(
                        egui::RichText::new(format!("(+{} new)", delta_comments))
                            .size(11.0)
                            .color(Color32::from_rgb(239, 68, 68))
                            .strong(),
                    );
                }
            });
            ui.add_space(2.0);

            if let Some(init) = channel.initial_total_comments {
                if delta_comments != 0 {
                    ui.label(
                        egui::RichText::new(format!(
                            "Total: {} | Since launch: {:+} (initial: {})",
                            comments_num, delta_comments, init
                        ))
                        .size(11.0)
                        .color(Color32::from_rgb(226, 232, 240)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!(
                            "Total: {} (no change since launch)",
                            comments_num
                        ))
                        .size(11.0)
                        .color(Color32::from_rgb(226, 232, 240)),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new(format!("Total: {}", comments_num))
                        .size(11.0)
                        .color(Color32::from_rgb(226, 232, 240)),
                );
            }

            if let Some(comm) = latest_comm {
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("Latest Comment:")
                        .strong()
                        .size(11.5)
                        .color(Color32::from_rgb(253, 186, 116)),
                );

                if !comm.author.is_empty() {
                    let author_display = if comm.author.starts_with('@') {
                        comm.author.clone()
                    } else {
                        format!("@{}", comm.author)
                    };
                    ui.label(
                        egui::RichText::new(format!("By {}", author_display))
                            .strong()
                            .size(11.0)
                            .color(Color32::from_rgb(147, 197, 253)),
                    );
                }

                if !comm.video_title.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("On \"{}\"", comm.video_title))
                            .italics()
                            .size(10.5)
                            .color(Color32::from_rgb(203, 213, 225)),
                    );
                }

                if !comm.text.is_empty() {
                    ui.add_space(2.0);
                    let text_clean = comm.text.replace('\n', " ").trim().to_string();
                    let display_text = if text_clean.chars().count() > 180 {
                        let truncated: String = text_clean.chars().take(180).collect();
                        format!("\"{}...\"", truncated.trim())
                    } else {
                        format!("\"{}\"", text_clean)
                    };
                    ui.label(
                        egui::RichText::new(display_text)
                            .size(11.0)
                            .color(Color32::WHITE),
                    );
                }

                if comm.timestamp > 0 {
                    ui.add_space(1.0);
                    let time_str = format_unix_timestamp(comm.timestamp);
                    ui.label(
                        egui::RichText::new(time_str)
                            .size(10.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                }

                ui.add_space(4.0);
                let hint_text = if delta_comments != 0 || channel.comments_changed {
                    "Click to open YouTube Studio | Double-click to acknowledge changes"
                } else {
                    "Click to open YouTube Studio comments inbox"
                };
                ui.label(
                    egui::RichText::new(hint_text)
                        .size(10.5)
                        .underline()
                        .color(Color32::from_rgb(96, 165, 250)),
                );
            } else {
                ui.add_space(4.0);
                let hint_text = if delta_comments != 0 || channel.comments_changed {
                    "Click to open YouTube Studio | Double-click to acknowledge changes"
                } else {
                    "Click to open YouTube Studio comments inbox"
                };
                ui.label(
                    egui::RichText::new(hint_text)
                        .size(10.5)
                        .underline()
                        .color(Color32::from_rgb(96, 165, 250)),
                );
            }
        });
    });
}

pub fn sort_videos_list(
    videos: &mut [crate::db::CachedVideo],
    col: VideoSortColumn,
    dir: SortDirection,
) {
    videos.sort_by(|a, b| {
        let cmp = match col {
            VideoSortColumn::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            VideoSortColumn::Type => {
                let a_type = if a.is_short_video() { 1 } else { 0 };
                let b_type = if b.is_short_video() { 1 } else { 0 };
                a_type.cmp(&b_type)
            }
            VideoSortColumn::Age => a.published_at.cmp(&b.published_at),
            VideoSortColumn::Views => a.views.cmp(&b.views),
            VideoSortColumn::Likes => a.likes.cmp(&b.likes),
            VideoSortColumn::Comments => a.comments.cmp(&b.comments),
        };
        let dir_cmp = match dir {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        };
        dir_cmp.then_with(|| b.published_at.cmp(&a.published_at))
    });
}

fn truncate_text(ui: &egui::Ui, text: &str, max_w: f32, font_id: egui::FontId) -> String {
    let full_w = ui.fonts(|f| {
        f.layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE)
            .size()
            .x
    });
    if full_w <= max_w {
        return text.to_string();
    }
    let mut end = text.len();
    while end > 0 {
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        let candidate = format!("{}...", &text[..end]);
        let w = ui.fonts(|f| {
            f.layout_no_wrap(candidate.clone(), font_id.clone(), Color32::WHITE)
                .size()
                .x
        });
        if w <= max_w {
            return candidate;
        }
        end = end.saturating_sub(1);
    }
    "...".to_string()
}

#[allow(clippy::too_many_arguments)]
fn render_table_header_cell(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    align_right: bool,
    is_active: bool,
    ascending: bool,
    active_color: Color32,
    tooltip: &str,
) -> bool {
    let resp = ui
        .interact(rect, ui.id().with(("th_cell", label)), egui::Sense::click())
        .on_hover_text(tooltip);
    if resp.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.hovered() {
        Color32::from_white_alpha(20)
    } else if is_active {
        Color32::from_white_alpha(10)
    } else {
        Color32::TRANSPARENT
    };
    if bg_color != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, Rounding::same(4.0), bg_color);
    }

    let text_color = if is_active {
        active_color
    } else if resp.hovered() {
        Color32::from_rgb(226, 232, 240)
    } else {
        Color32::from_rgb(148, 163, 184)
    };

    let font_id = egui::FontId::proportional(10.5);
    let galley = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font_id, text_color));

    let text_pos = if align_right {
        if is_active {
            Pos2::new(
                rect.right() - galley.size().x - 12.0,
                rect.center().y - galley.size().y * 0.5,
            )
        } else {
            Pos2::new(
                rect.right() - galley.size().x - 4.0,
                rect.center().y - galley.size().y * 0.5,
            )
        }
    } else {
        Pos2::new(rect.left() + 4.0, rect.center().y - galley.size().y * 0.5)
    };
    ui.painter().galley(text_pos, galley.clone(), text_color);

    if is_active {
        let arrow_x = if align_right {
            rect.right() - 5.0
        } else {
            text_pos.x + galley.size().x + 7.0
        };
        let arrow_center = Pos2::new(arrow_x, rect.center().y);
        crate::icons::draw_sort_arrow(ui.painter(), arrow_center, ascending, text_color);
    }

    resp.clicked()
}

pub fn format_compact_metric(val: u64) -> String {
    if val >= 1_000_000_000 {
        format!("{:.1}B", val as f64 / 1_000_000_000.0)
    } else if val >= 1_000_000 {
        format!("{:.1}M", val as f64 / 1_000_000.0)
    } else if val >= 10_000 {
        format!("{:.1}K", val as f64 / 1_000.0)
    } else {
        crate::youtube::clean_stat_number(&val.to_string())
    }
}

pub use crate::youtube::CardMetric;

fn render_card_metric_box(
    ui: &mut egui::Ui,
    box_size: Vec2,
    metric: CardMetric,
    channel: &ChannelData,
) -> Option<CardMetric> {
    let font_id = egui::FontId::proportional(13.0);
    let item_gap = 5.0_f32;
    let mut approved = None;

    match metric {
        CardMetric::Subscribers => {
            let delta_sub = channel.delta_subscribers();
            let sub_increased = delta_sub > 0;
            let sub_changed = delta_sub != 0 || channel.sub_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Subscribers);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(8.0), Color32::from_rgb(16, 44, 28));
            let stroke_color = if sub_increased {
                Color32::from_rgb(239, 68, 68)
            } else if sub_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(52, 211, 153)
            };
            let stroke_width = if sub_changed { 1.8_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(8.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let sub_num = crate::youtube::clean_stat_number(&channel.subscriber_text);
            let sub_galley = ui.fonts(|f| {
                f.layout_no_wrap(sub_num.clone(), font_id, Color32::from_rgb(209, 250, 229))
            });
            let icon_w = 14.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_sub);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if sub_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if sub_increased { 10.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 5.0);
            let total_w = icon_w + item_gap + sub_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::splat(14.0),
            );
            crate::icons::draw_user_icon(ui.painter(), icon_rect, Color32::from_rgb(52, 211, 153));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - sub_galley.size().y / 2.0,
                ),
                sub_galley,
                Color32::from_rgb(209, 250, 229),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 5.0;
            if sub_increased {
                let circle_pos = Pos2::new(cur_x + 3.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 3.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    3.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 10.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_subscriber_count {
                if delta_sub != 0 {
                    format!(
                        "Subscribers: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        sub_num, delta_sub, init
                    )
                } else {
                    format!("Subscribers: {} (no change since launch)", sub_num)
                }
            } else {
                format!("Subscribers: {}", sub_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Views => {
            let delta_views = channel.delta_views();
            let views_increased = delta_views > 0;
            let views_changed = delta_views != 0 || channel.views_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Views);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(8.0), Color32::from_rgb(18, 36, 58));
            let stroke_color = if views_increased {
                Color32::from_rgb(239, 68, 68)
            } else if views_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(96, 165, 250)
            };
            let stroke_width = if views_changed { 1.8_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(8.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let views_num = crate::youtube::clean_stat_number(&channel.views_text);
            let views_galley = ui.fonts(|f| {
                f.layout_no_wrap(views_num.clone(), font_id, Color32::from_rgb(224, 242, 254))
            });
            let icon_w = 16.0_f32;

            let delta_views_str = crate::youtube::format_compact_delta(delta_views);
            let (delta_galley, delta_color) = if !delta_views_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if views_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_views_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if views_increased { 10.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 5.0);
            let total_w = icon_w + item_gap + views_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(16.0, 12.0),
            );
            crate::icons::draw_eye_icon(ui.painter(), icon_rect, Color32::from_rgb(96, 165, 250));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - views_galley.size().y / 2.0,
                ),
                views_galley,
                Color32::from_rgb(224, 242, 254),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 5.0;
            if views_increased {
                let circle_pos = Pos2::new(cur_x + 3.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 3.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    3.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 10.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_views_count {
                if delta_views != 0 {
                    format!(
                        "Views: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        views_num, delta_views, init
                    )
                } else {
                    format!("Views: {} (no change since launch)", views_num)
                }
            } else {
                format!("Views: {}", views_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Videos => {
            let delta_videos = channel.delta_video_count();
            let vid_increased = delta_videos > 0;
            let vid_changed = delta_videos != 0 || channel.video_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Videos);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(8.0), Color32::from_rgb(38, 22, 54));
            let stroke_color = if vid_increased {
                Color32::from_rgb(239, 68, 68)
            } else if vid_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(192, 132, 252)
            };
            let stroke_width = if vid_changed { 1.8_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(8.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let videos_num = if channel.video_count > 0 {
                crate::youtube::clean_stat_number(&channel.video_count.to_string())
            } else if !channel.video_count_text.is_empty() && channel.video_count_text != "-" {
                crate::youtube::clean_stat_number(&channel.video_count_text)
            } else {
                "-".to_string()
            };

            let vid_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    videos_num.clone(),
                    font_id,
                    Color32::from_rgb(243, 232, 255),
                )
            });
            let icon_w = 15.0_f32;

            let delta_videos_str = crate::youtube::format_compact_delta(delta_videos);
            let (delta_galley, delta_color) = if !delta_videos_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if vid_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_videos_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if vid_increased { 10.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 5.0);
            let total_w = icon_w + item_gap + vid_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(15.0, 13.0),
            );
            crate::icons::draw_video_icon(
                ui.painter(),
                icon_rect,
                Color32::from_rgb(192, 132, 252),
            );

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - vid_galley.size().y / 2.0,
                ),
                vid_galley,
                Color32::from_rgb(243, 232, 255),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 5.0;
            if vid_increased {
                let circle_pos = Pos2::new(cur_x + 3.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 3.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    3.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 10.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_video_count {
                if delta_videos != 0 {
                    format!(
                        "Videos: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        videos_num, delta_videos, init
                    )
                } else {
                    format!("Videos: {}", videos_num)
                }
            } else {
                format!("Videos: {}", videos_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Comments => {
            let delta_comments = channel.delta_comments();
            let comm_increased = delta_comments > 0;
            let comm_changed = delta_comments != 0 || channel.comments_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(8.0), Color32::from_rgb(46, 36, 16));
            let stroke_color = if comm_increased {
                Color32::from_rgb(239, 68, 68)
            } else if comm_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(251, 191, 36)
            };
            let stroke_width = if comm_changed { 1.8_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(8.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let comments_num = if channel.total_comments > 0 {
                crate::youtube::clean_stat_number(&channel.total_comments.to_string())
            } else {
                "-".to_string()
            };

            let comm_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    comments_num.clone(),
                    font_id,
                    Color32::from_rgb(254, 243, 199),
                )
            });
            let icon_w = 15.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_comments);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if comm_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if comm_increased { 10.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 5.0);
            let total_w = icon_w + item_gap + comm_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(15.0, 13.0),
            );
            crate::icons::draw_comment_icon(
                ui.painter(),
                icon_rect,
                Color32::from_rgb(251, 191, 36),
            );

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - comm_galley.size().y / 2.0,
                ),
                comm_galley,
                Color32::from_rgb(254, 243, 199),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 5.0;
            if comm_increased {
                let circle_pos = Pos2::new(cur_x + 3.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 3.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    3.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 10.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let latest_comm = channel.latest_comment.clone().or_else(|| {
                channel.channel_id.as_ref().and_then(|cid| {
                    crate::db::Database::open()
                        .ok()
                        .and_then(|db| db.get_channel_latest_comment(cid).ok().flatten())
                })
            });

            if box_resp.clicked() {
                let _ = open::that(channel.comments_inbox_url());
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Comments);
            }

            show_comments_metric_tooltip(
                &box_resp,
                channel,
                &latest_comm,
                &comments_num,
                delta_comments,
            );
        }
        CardMetric::Likes => {
            let delta_likes = channel.delta_likes();
            let likes_increased = delta_likes > 0;
            let likes_changed = delta_likes != 0 || channel.likes_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Likes);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(8.0), Color32::from_rgb(48, 18, 28));
            let stroke_color = if likes_increased {
                Color32::from_rgb(239, 68, 68)
            } else if likes_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(251, 113, 133)
            };
            let stroke_width = if likes_changed { 1.8_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(8.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let likes_num = if channel.total_likes > 0 {
                crate::youtube::clean_stat_number(&channel.total_likes.to_string())
            } else {
                "-".to_string()
            };

            let likes_galley = ui.fonts(|f| {
                f.layout_no_wrap(likes_num.clone(), font_id, Color32::from_rgb(255, 228, 230))
            });
            let icon_w = 15.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_likes);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if likes_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if likes_increased { 10.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 5.0);
            let total_w = icon_w + item_gap + likes_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(15.0, 13.0),
            );
            crate::icons::draw_like_icon(ui.painter(), icon_rect, Color32::from_rgb(251, 113, 133));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - likes_galley.size().y / 2.0,
                ),
                likes_galley,
                Color32::from_rgb(255, 228, 230),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 5.0;
            if likes_increased {
                let circle_pos = Pos2::new(cur_x + 3.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 3.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    3.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 10.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_total_likes {
                if delta_likes != 0 {
                    format!(
                        "Likes: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        likes_num, delta_likes, init
                    )
                } else {
                    format!("Likes: {}", likes_num)
                }
            } else {
                format!("Likes: {}", likes_num)
            };
            box_resp.on_hover_text(tooltip);
        }
    }
    approved
}

fn render_compact_metric_box(
    ui: &mut egui::Ui,
    box_size: Vec2,
    metric: CardMetric,
    channel: &ChannelData,
) -> Option<CardMetric> {
    let font_id = egui::FontId::proportional(11.0);
    let delta_font = egui::FontId::proportional(9.0);
    let item_gap = 3.0_f32;
    let mut approved = None;

    match metric {
        CardMetric::Subscribers => {
            let delta_sub = channel.delta_subscribers();
            let sub_increased = delta_sub > 0;
            let sub_changed = delta_sub != 0 || channel.sub_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Subscribers);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(4.0), Color32::from_rgb(16, 44, 28));
            let stroke_color = if sub_increased {
                Color32::from_rgb(239, 68, 68)
            } else if sub_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(52, 211, 153)
            };
            let stroke_width = if sub_changed { 1.5_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(4.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let sub_num = crate::youtube::clean_stat_number(&channel.subscriber_text);
            let sub_galley = ui.fonts(|f| {
                f.layout_no_wrap(sub_num.clone(), font_id, Color32::from_rgb(209, 250, 229))
            });
            let icon_w = 11.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_sub);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let color = if sub_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if sub_increased { 8.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 3.0);
            let total_w = icon_w + item_gap + sub_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::splat(11.0),
            );
            crate::icons::draw_user_icon(ui.painter(), icon_rect, Color32::from_rgb(52, 211, 153));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - sub_galley.size().y / 2.0,
                ),
                sub_galley,
                Color32::from_rgb(209, 250, 229),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 3.0;
            if sub_increased {
                let circle_pos = Pos2::new(cur_x + 2.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 2.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    2.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 8.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_subscriber_count {
                if delta_sub != 0 {
                    format!(
                        "Subscribers: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        sub_num, delta_sub, init
                    )
                } else {
                    format!("Subscribers: {} (no change since launch)", sub_num)
                }
            } else {
                format!("Subscribers: {}", sub_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Views => {
            let delta_views = channel.delta_views();
            let views_increased = delta_views > 0;
            let views_changed = delta_views != 0 || channel.views_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Views);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(4.0), Color32::from_rgb(18, 36, 58));
            let stroke_color = if views_increased {
                Color32::from_rgb(239, 68, 68)
            } else if views_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(96, 165, 250)
            };
            let stroke_width = if views_changed { 1.5_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(4.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let views_num = crate::youtube::clean_stat_number(&channel.views_text);
            let views_galley = ui.fonts(|f| {
                f.layout_no_wrap(views_num.clone(), font_id, Color32::from_rgb(224, 242, 254))
            });
            let icon_w = 12.0_f32;

            let delta_views_str = crate::youtube::format_compact_delta(delta_views);
            let (delta_galley, delta_color) = if !delta_views_str.is_empty() {
                let color = if views_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_views_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if views_increased { 8.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 3.0);
            let total_w = icon_w + item_gap + views_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(12.0, 9.0),
            );
            crate::icons::draw_eye_icon(ui.painter(), icon_rect, Color32::from_rgb(96, 165, 250));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - views_galley.size().y / 2.0,
                ),
                views_galley,
                Color32::from_rgb(224, 242, 254),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 3.0;
            if views_increased {
                let circle_pos = Pos2::new(cur_x + 2.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 2.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    2.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 8.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_views_count {
                if delta_views != 0 {
                    format!(
                        "Views: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        views_num, delta_views, init
                    )
                } else {
                    format!("Views: {} (no change since launch)", views_num)
                }
            } else {
                format!("Views: {}", views_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Videos => {
            let delta_videos = channel.delta_video_count();
            let vid_increased = delta_videos > 0;
            let vid_changed = delta_videos != 0 || channel.video_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Videos);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(4.0), Color32::from_rgb(38, 22, 54));
            let stroke_color = if vid_increased {
                Color32::from_rgb(239, 68, 68)
            } else if vid_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(192, 132, 252)
            };
            let stroke_width = if vid_changed { 1.5_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(4.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let videos_num = if channel.video_count > 0 {
                crate::youtube::clean_stat_number(&channel.video_count.to_string())
            } else if !channel.video_count_text.is_empty() && channel.video_count_text != "-" {
                crate::youtube::clean_stat_number(&channel.video_count_text)
            } else {
                "-".to_string()
            };

            let vid_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    videos_num.clone(),
                    font_id,
                    Color32::from_rgb(243, 232, 255),
                )
            });
            let icon_w = 11.0_f32;

            let delta_videos_str = crate::youtube::format_compact_delta(delta_videos);
            let (delta_galley, delta_color) = if !delta_videos_str.is_empty() {
                let delta_font = egui::FontId::proportional(11.0);
                let color = if vid_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_videos_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if vid_increased { 8.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 3.0);
            let total_w = icon_w + item_gap + vid_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(11.0, 9.5),
            );
            crate::icons::draw_video_icon(
                ui.painter(),
                icon_rect,
                Color32::from_rgb(192, 132, 252),
            );

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - vid_galley.size().y / 2.0,
                ),
                vid_galley,
                Color32::from_rgb(243, 232, 255),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 3.0;
            if vid_increased {
                let circle_pos = Pos2::new(cur_x + 2.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 2.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    2.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 8.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_video_count {
                if delta_videos != 0 {
                    format!(
                        "Videos: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        videos_num, delta_videos, init
                    )
                } else {
                    format!("Videos: {}", videos_num)
                }
            } else {
                format!("Videos: {}", videos_num)
            };
            box_resp.on_hover_text(tooltip);
        }
        CardMetric::Comments => {
            let delta_comments = channel.delta_comments();
            let comm_increased = delta_comments > 0;
            let comm_changed = delta_comments != 0 || channel.comments_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(4.0), Color32::from_rgb(46, 36, 16));
            let stroke_color = if comm_increased {
                Color32::from_rgb(239, 68, 68)
            } else if comm_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(251, 191, 36)
            };
            let stroke_width = if comm_changed { 1.5_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(4.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let comments_num = if channel.total_comments > 0 {
                crate::youtube::clean_stat_number(&channel.total_comments.to_string())
            } else {
                "-".to_string()
            };

            let comm_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    comments_num.clone(),
                    font_id,
                    Color32::from_rgb(254, 243, 199),
                )
            });
            let icon_w = 11.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_comments);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let color = if comm_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if comm_increased { 8.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 3.0);
            let total_w = icon_w + item_gap + comm_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(11.0, 9.5),
            );
            crate::icons::draw_comment_icon(
                ui.painter(),
                icon_rect,
                Color32::from_rgb(251, 191, 36),
            );

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - comm_galley.size().y / 2.0,
                ),
                comm_galley,
                Color32::from_rgb(254, 243, 199),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 3.0;
            if comm_increased {
                let circle_pos = Pos2::new(cur_x + 2.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 2.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    2.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 8.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let latest_comm = channel.latest_comment.clone().or_else(|| {
                channel.channel_id.as_ref().and_then(|cid| {
                    crate::db::Database::open()
                        .ok()
                        .and_then(|db| db.get_channel_latest_comment(cid).ok().flatten())
                })
            });

            if box_resp.clicked() {
                let _ = open::that(channel.comments_inbox_url());
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Comments);
            }

            show_comments_metric_tooltip(
                &box_resp,
                channel,
                &latest_comm,
                &comments_num,
                delta_comments,
            );
        }
        CardMetric::Likes => {
            let delta_likes = channel.delta_likes();
            let likes_increased = delta_likes > 0;
            let likes_changed = delta_likes != 0 || channel.likes_changed;

            let (box_rect, box_resp) = ui.allocate_exact_size(box_size, egui::Sense::click());
            if box_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if box_resp.double_clicked() {
                approved = Some(CardMetric::Likes);
            }
            ui.painter()
                .rect_filled(box_rect, Rounding::same(4.0), Color32::from_rgb(48, 18, 28));
            let stroke_color = if likes_increased {
                Color32::from_rgb(239, 68, 68)
            } else if likes_changed {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(251, 113, 133)
            };
            let stroke_width = if likes_changed { 1.5_f32 } else { 1.0_f32 };
            ui.painter().rect_stroke(
                box_rect,
                Rounding::same(4.0),
                Stroke::new(stroke_width, stroke_color),
            );

            let likes_num = if channel.total_likes > 0 {
                crate::youtube::clean_stat_number(&channel.total_likes.to_string())
            } else {
                "-".to_string()
            };

            let likes_galley = ui.fonts(|f| {
                f.layout_no_wrap(likes_num.clone(), font_id, Color32::from_rgb(255, 228, 230))
            });
            let icon_w = 11.0_f32;

            let delta_str = crate::youtube::format_compact_delta(delta_likes);
            let (delta_galley, delta_color) = if !delta_str.is_empty() {
                let color = if likes_increased {
                    Color32::from_rgb(252, 165, 165)
                } else {
                    Color32::from_rgb(253, 186, 116)
                };
                (
                    Some(ui.fonts(|f| f.layout_no_wrap(delta_str, delta_font, color))),
                    color,
                )
            } else {
                (None, Color32::TRANSPARENT)
            };

            let circle_w = if likes_increased { 8.0_f32 } else { 0.0_f32 };
            let delta_w = delta_galley.as_ref().map_or(0.0, |g| g.size().x + 3.0);
            let total_w = icon_w + item_gap + likes_galley.size().x + circle_w + delta_w;
            let start_x = box_rect.center().x - total_w / 2.0;

            let icon_rect = Rect::from_center_size(
                Pos2::new(start_x + icon_w / 2.0, box_rect.center().y),
                Vec2::new(11.0, 9.5),
            );
            crate::icons::draw_like_icon(ui.painter(), icon_rect, Color32::from_rgb(251, 113, 133));

            ui.painter().galley(
                Pos2::new(
                    start_x + icon_w + item_gap,
                    box_rect.center().y - likes_galley.size().y / 2.0,
                ),
                likes_galley,
                Color32::from_rgb(255, 228, 230),
            );

            let mut cur_x = start_x + icon_w + item_gap + total_w - circle_w - delta_w + 3.0;
            if likes_increased {
                let circle_pos = Pos2::new(cur_x + 2.5, box_rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 2.5, Color32::from_rgb(239, 68, 68));
                ui.painter().circle_stroke(
                    circle_pos,
                    2.5,
                    Stroke::new(0.5_f32, Color32::from_rgb(254, 202, 202)),
                );
                cur_x += 8.0;
            }

            if let Some(galley) = delta_galley {
                ui.painter().galley(
                    Pos2::new(cur_x, box_rect.center().y - galley.size().y / 2.0),
                    galley,
                    delta_color,
                );
            }

            let tooltip = if let Some(init) = channel.initial_total_likes {
                if delta_likes != 0 {
                    format!(
                        "Likes: {} | Since launch: {:+} (initial: {})\nDouble-click to acknowledge changes",
                        likes_num, delta_likes, init
                    )
                } else {
                    format!("Likes: {}", likes_num)
                }
            } else {
                format!("Likes: {}", likes_num)
            };
            box_resp.on_hover_text(tooltip);
        }
    }
    approved
}

fn export_channel_to_csv(channel: &ChannelData) -> Result<String, std::io::Error> {
    let clean_handle = channel
        .handle
        .trim_start_matches('@')
        .replace(['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>'], "_");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    std::fs::create_dir_all("exports")?;
    let filename = format!("exports/{}_export_{}.csv", clean_handle, now);

    let db_opt = crate::db::Database::open().ok();
    let cid: Option<String> = channel.channel_id.clone().or_else(|| {
        db_opt.as_ref().and_then(|db| {
            db.get_channel_by_handle(&channel.handle)
                .ok()
                .flatten()
                .map(|c| c.channel_id)
        })
    });

    let total_likes = if channel.total_likes > 0 {
        channel.total_likes
    } else if let (Some(db), Some(cid_ref)) = (&db_opt, &cid) {
        db.get_channel_total_likes(cid_ref).unwrap_or(0)
    } else {
        0
    };

    let total_comments = if channel.total_comments > 0 {
        channel.total_comments
    } else if let (Some(db), Some(cid_ref)) = (&db_opt, &cid) {
        db.get_channel_total_comments(cid_ref).unwrap_or(0)
    } else {
        0
    };

    let videos = if let (Some(db), Some(cid_ref)) = (&db_opt, &cid) {
        db.get_all_videos(cid_ref).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut csv = String::new();
    csv.push_str("# SocialCount Channel Export\n");
    csv.push_str(&format!(
        "# Channel Title: {}\n",
        channel.title.replace('\n', " ")
    ));
    csv.push_str(&format!("# Handle: {}\n", channel.handle));
    csv.push_str(&format!(
        "# Channel ID: {}\n",
        cid.as_deref().unwrap_or("N/A")
    ));
    csv.push_str(&format!("# Subscribers: {}\n", channel.subscriber_count));
    csv.push_str(&format!("# Total Views: {}\n", channel.views_count));
    csv.push_str(&format!("# Total Videos: {}\n", channel.video_count));
    csv.push_str(&format!("# Tracked Likes: {}\n", total_likes));
    csv.push_str(&format!("# Tracked Comments: {}\n", total_comments));
    csv.push_str(&format!("# Exported At (Unix): {}\n", now));
    csv.push('\n');
    csv.push_str("Video ID,Type,Title,Published Date,Views,Likes,Comments,Last Comment Date,URL\n");

    for v in videos {
        let v_type = if v.is_short_video() { "Short" } else { "Video" };
        let escaped_title = v.title.replace('"', "\"\"").replace('\n', " ");
        let pub_date = format_unix_timestamp(v.published_at);
        let last_comm_date = if v.last_comment_at > 0 {
            format_unix_timestamp(v.last_comment_at)
        } else {
            String::new()
        };
        let url = if v.is_short_video() {
            format!("https://www.youtube.com/shorts/{}", v.video_id)
        } else {
            format!("https://www.youtube.com/watch?v={}", v.video_id)
        };
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},\"{}\",\"{}\"\n",
            v.video_id,
            v_type,
            escaped_title,
            pub_date,
            v.views,
            v.likes,
            v.comments,
            last_comm_date,
            url
        ));
    }

    std::fs::write(&filename, csv)?;
    let _ = open::that(&filename);
    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CachedVideo;

    fn sample_video(
        id: &str,
        title: &str,
        published_at: u64,
        views: u64,
        likes: u64,
        comments: u64,
        is_short: bool,
    ) -> CachedVideo {
        CachedVideo {
            video_id: id.to_string(),
            channel_id: "test_ch".to_string(),
            title: title.to_string(),
            published_at,
            views,
            likes,
            comments,
            last_checked_at: 0,
            is_short,
            last_comment_at: 0,
            last_comment_count_checked: 0,
            last_comment_author: String::new(),
            last_comment_text: String::new(),
        }
    }

    #[test]
    fn test_format_compact_metric() {
        assert_eq!(format_compact_metric(0), "0");
        assert_eq!(format_compact_metric(54), "54");
        assert_eq!(format_compact_metric(999), "999");
        assert_eq!(format_compact_metric(1500), "1.500");
        assert_eq!(format_compact_metric(10500), "10.5K");
        assert_eq!(format_compact_metric(150000), "150.0K");
        assert_eq!(format_compact_metric(1200000), "1.2M");
        assert_eq!(format_compact_metric(2500000000), "2.5B");
    }

    #[test]
    fn test_sort_videos_by_views() {
        let mut videos = vec![
            sample_video("v1", "Alpha", 100, 50, 10, 5, false),
            sample_video("v2", "Beta", 200, 500, 5, 20, false),
            sample_video("v3", "Gamma", 300, 10, 50, 2, false),
        ];

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Views,
            SortDirection::Descending,
        );
        assert_eq!(videos[0].video_id, "v2");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v3");

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Views,
            SortDirection::Ascending,
        );
        assert_eq!(videos[0].video_id, "v3");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v2");
    }

    #[test]
    fn test_sort_videos_by_type() {
        let mut videos = vec![
            sample_video("v1", "Alpha", 100, 50, 10, 5, false),
            sample_video("v2", "Beta", 200, 500, 5, 20, true),
            sample_video("v3", "Gamma", 300, 10, 50, 2, false),
        ];

        // Ascending type: Videos first (0), then Shorts (1)
        sort_videos_list(&mut videos, VideoSortColumn::Type, SortDirection::Ascending);
        assert_eq!(videos[0].video_id, "v3"); // Video (newer)
        assert_eq!(videos[1].video_id, "v1"); // Video (older)
        assert_eq!(videos[2].video_id, "v2"); // Short

        // Descending type: Shorts first (1), then Videos (0)
        sort_videos_list(
            &mut videos,
            VideoSortColumn::Type,
            SortDirection::Descending,
        );
        assert_eq!(videos[0].video_id, "v2"); // Short
        assert_eq!(videos[1].video_id, "v3"); // Video (newer)
        assert_eq!(videos[2].video_id, "v1"); // Video (older)
    }

    #[test]
    fn test_sort_videos_by_likes() {
        let mut videos = vec![
            sample_video("v1", "Alpha", 100, 50, 10, 5, false),
            sample_video("v2", "Beta", 200, 500, 5, 20, false),
            sample_video("v3", "Gamma", 300, 10, 50, 2, false),
        ];

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Likes,
            SortDirection::Descending,
        );
        assert_eq!(videos[0].video_id, "v3");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v2");

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Likes,
            SortDirection::Ascending,
        );
        assert_eq!(videos[0].video_id, "v2");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v3");
    }

    #[test]
    fn test_sort_videos_by_comments() {
        let mut videos = vec![
            sample_video("v1", "Alpha", 100, 50, 10, 5, false),
            sample_video("v2", "Beta", 200, 500, 5, 20, false),
            sample_video("v3", "Gamma", 300, 10, 50, 2, false),
        ];

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Comments,
            SortDirection::Descending,
        );
        assert_eq!(videos[0].video_id, "v2");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v3");

        sort_videos_list(
            &mut videos,
            VideoSortColumn::Comments,
            SortDirection::Ascending,
        );
        assert_eq!(videos[0].video_id, "v3");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v2");
    }

    #[test]
    fn test_sort_videos_by_age() {
        let mut videos = vec![
            sample_video("v1", "Alpha", 100, 50, 10, 5, false),
            sample_video("v2", "Beta", 200, 500, 5, 20, false),
            sample_video("v3", "Gamma", 300, 10, 50, 2, false),
        ];

        // Descending age: newest first (highest published_at)
        sort_videos_list(&mut videos, VideoSortColumn::Age, SortDirection::Descending);
        assert_eq!(videos[0].video_id, "v3");
        assert_eq!(videos[1].video_id, "v2");
        assert_eq!(videos[2].video_id, "v1");

        // Ascending age: oldest first (lowest published_at)
        sort_videos_list(&mut videos, VideoSortColumn::Age, SortDirection::Ascending);
        assert_eq!(videos[0].video_id, "v1");
        assert_eq!(videos[1].video_id, "v2");
        assert_eq!(videos[2].video_id, "v3");
    }

    #[test]
    fn test_sort_videos_by_title() {
        let mut videos = vec![
            sample_video("v1", "Bravo", 100, 50, 10, 5, false),
            sample_video("v2", "charlie", 200, 500, 5, 20, false),
            sample_video("v3", "Alpha", 300, 10, 50, 2, false),
        ];

        // Ascending title: A to Z (case-insensitive)
        sort_videos_list(
            &mut videos,
            VideoSortColumn::Title,
            SortDirection::Ascending,
        );
        assert_eq!(videos[0].video_id, "v3");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v2");

        // Descending title: Z to A
        sort_videos_list(
            &mut videos,
            VideoSortColumn::Title,
            SortDirection::Descending,
        );
        assert_eq!(videos[0].video_id, "v2");
        assert_eq!(videos[1].video_id, "v1");
        assert_eq!(videos[2].video_id, "v3");
    }
}
