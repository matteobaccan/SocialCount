// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardMetric {
    Subscribers,
    Views,
    Videos,
    Comments,
    Likes,
}

#[derive(Debug, Clone)]
pub struct ChannelData {
    pub handle: String,
    pub title: String,
    pub subscriber_text: String,
    pub subscriber_count: u64,
    pub initial_subscriber_count: Option<u64>,
    pub views_text: String,
    pub views_count: u64,
    pub initial_views_count: Option<u64>,
    pub video_count: u64,
    pub initial_video_count: Option<u64>,
    pub total_comments: u64,
    pub initial_total_comments: Option<u64>,
    pub total_likes: u64,
    pub initial_total_likes: Option<u64>,
    pub latest_comment: Option<crate::db::LatestCommentInfo>,
    pub channel_id: Option<String>,
    pub uploads_playlist_id: Option<String>,
    pub video_count_text: String,
    pub avatar_url: String,
    pub avatar_bytes: Option<Vec<u8>>,
    pub channel_url: String,
    pub error: Option<String>,
    pub sub_changed: bool,
    pub views_changed: bool,
    pub video_changed: bool,
    pub comments_changed: bool,
    pub likes_changed: bool,
    pub sub_changed_at: Option<std::time::Instant>,
    pub views_changed_at: Option<std::time::Instant>,
    pub video_changed_at: Option<std::time::Instant>,
    pub comments_changed_at: Option<std::time::Instant>,
    pub likes_changed_at: Option<std::time::Instant>,
}

impl ChannelData {
    pub fn new_loading(handle: &str) -> Self {
        Self {
            handle: handle.to_string(),
            title: handle.trim_start_matches('@').to_string(),
            subscriber_text: "Loading...".to_string(),
            subscriber_count: 0,
            initial_subscriber_count: None,
            views_text: "-".to_string(),
            views_count: 0,
            initial_views_count: None,
            video_count: 0,
            initial_video_count: None,
            total_comments: 0,
            initial_total_comments: None,
            total_likes: 0,
            initial_total_likes: None,
            latest_comment: None,
            channel_id: None,
            uploads_playlist_id: None,
            video_count_text: "-".to_string(),
            avatar_url: String::new(),
            avatar_bytes: None,
            channel_url: format!("https://www.youtube.com/{}", handle),
            error: None,
            sub_changed: false,
            views_changed: false,
            video_changed: false,
            comments_changed: false,
            likes_changed: false,
            sub_changed_at: None,
            views_changed_at: None,
            video_changed_at: None,
            comments_changed_at: None,
            likes_changed_at: None,
        }
    }

    pub fn delta_subscribers(&self) -> i64 {
        match self.initial_subscriber_count {
            Some(init) => self.subscriber_count as i64 - init as i64,
            None => 0,
        }
    }

    pub fn delta_views(&self) -> i64 {
        match self.initial_views_count {
            Some(init) => self.views_count as i64 - init as i64,
            None => 0,
        }
    }

    #[allow(dead_code)]
    pub fn delta_video_count(&self) -> i64 {
        match self.initial_video_count {
            Some(init) => self.video_count as i64 - init as i64,
            None => 0,
        }
    }

    #[allow(dead_code)]
    pub fn delta_comments(&self) -> i64 {
        match self.initial_total_comments {
            Some(init) => self.total_comments as i64 - init as i64,
            None => 0,
        }
    }

    #[allow(dead_code)]
    pub fn delta_likes(&self) -> i64 {
        match self.initial_total_likes {
            Some(init) => self.total_likes as i64 - init as i64,
            None => 0,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
            || self.subscriber_text == "N/A"
            || self.subscriber_text == "N/D"
            || self.views_text == "N/A"
            || self.views_text == "N/D"
    }

    pub fn comments_inbox_url(&self) -> String {
        let channel_id = self.channel_id.clone().or_else(|| {
            if self.handle.starts_with("UC") {
                Some(self.handle.clone())
            } else {
                crate::db::Database::open()
                    .ok()
                    .and_then(|db| db.get_channel_by_handle(&self.handle).ok().flatten())
                    .map(|c| c.channel_id)
            }
        });

        if let Some(cid) = channel_id {
            if !cid.is_empty() {
                return format!(
                    "https://studio.youtube.com/channel/{}/comments/inbox?filter=%5B%7B%22isDisabled%22%3Afalse%2C%22isPinned%22%3Atrue%2C%22name%22%3A%22SORT_BY%22%2C%22value%22%3A%22SORT_BY_MOST_RELEVANT%22%7D%2C%7B%22name%22%3A%22ENGAGED_STATUS%22%2C%22value%22%3A%5B%22COMMENT_CATEGORY_NOT_ENGAGED%22%2C%22COMMENT_CATEGORY_NEEDS_FOLLOW_UP%22%5D%7D%2C%7B%22name%22%3A%22PARENT_ENTITY_CONTENT_TYPE%22%2C%22value%22%3A%5B%22PARENT_ENTITY_CONTENT_TYPE_WATCH%22%2C%22PARENT_ENTITY_CONTENT_TYPE_SHORT%22%2C%22PARENT_ENTITY_CONTENT_TYPE_CREATOR_POST%22%5D%7D%5D",
                    cid
                );
            }
        }

        format!(
            "https://www.youtube.com/@{}/community",
            self.handle.trim_start_matches('@')
        )
    }

    pub fn approve_metric(&mut self, metric: CardMetric) {
        match metric {
            CardMetric::Subscribers => {
                self.initial_subscriber_count = Some(self.subscriber_count);
                self.sub_changed = false;
                self.sub_changed_at = None;
            }
            CardMetric::Views => {
                self.initial_views_count = Some(self.views_count);
                self.views_changed = false;
                self.views_changed_at = None;
            }
            CardMetric::Videos => {
                self.initial_video_count = Some(self.video_count);
                self.video_changed = false;
                self.video_changed_at = None;
            }
            CardMetric::Comments => {
                self.initial_total_comments = Some(self.total_comments);
                self.comments_changed = false;
                self.comments_changed_at = None;
            }
            CardMetric::Likes => {
                self.initial_total_likes = Some(self.total_likes);
                self.likes_changed = false;
                self.likes_changed_at = None;
            }
        }
    }
}

pub struct YouTubeFetcher {
    client: reqwest::blocking::Client,
    api_key: Option<String>,
}

impl Default for YouTubeFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeFetcher {
    pub fn new() -> Self {
        Self::with_api_key(None)
    }

    pub fn with_api_key(api_key: Option<String>) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self { client, api_key }
    }

    pub fn fetch_channel(&self, handle: &str) -> ChannelData {
        if let Some(api_key) = &self.api_key {
            match self.fetch_channel_api(handle, api_key) {
                Ok(data) => return data,
                Err(err) => {
                    eprintln!(
                        "YouTube API lookup failed for {}: {}. Falling back to scraping.",
                        handle, err
                    );
                }
            }
        }
        self.fetch_channel_scrape(handle)
    }

    fn fetch_channel_api(&self, handle: &str, api_key: &str) -> Result<ChannelData, String> {
        let clean_handle = if !handle.starts_with('@') && !handle.starts_with("UC") {
            format!("@{}", handle)
        } else {
            handle.to_string()
        };

        let url = if clean_handle.starts_with("UC") {
            format!(
                "https://www.googleapis.com/youtube/v3/channels?part=snippet,statistics,contentDetails&id={}&key={}",
                clean_handle, api_key
            )
        } else {
            format!(
                "https://www.googleapis.com/youtube/v3/channels?part=snippet,statistics,contentDetails&forHandle={}&key={}",
                clean_handle, api_key
            )
        };

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().unwrap_or_default();
            return Err(format!("Status {}: {}", status, err_body));
        }

        let text = response
            .text()
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let json: Value =
            serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        let item = json
            .get("items")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| "Channel not found in API response".to_string())?;

        let channel_id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let title = item
            .pointer("/snippet/title")
            .and_then(|v| v.as_str())
            .unwrap_or(clean_handle.trim_start_matches('@'))
            .to_string();

        let subscriber_count = item
            .pointer("/statistics/subscriberCount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let views_count = item
            .pointer("/statistics/viewCount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let video_count = item
            .pointer("/statistics/videoCount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let uploads_playlist_id = item
            .pointer("/contentDetails/relatedPlaylists/uploads")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let avatar_url = item
            .pointer("/snippet/thumbnails/high/url")
            .or_else(|| item.pointer("/snippet/thumbnails/medium/url"))
            .or_else(|| item.pointer("/snippet/thumbnails/default/url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut avatar_bytes = None;
        if !avatar_url.is_empty() {
            if let Ok(img_resp) = self.client.get(&avatar_url).send() {
                if img_resp.status().is_success() {
                    if let Ok(bytes) = img_resp.bytes() {
                        avatar_bytes = Some(bytes.to_vec());
                    }
                }
            }
        }

        let subscriber_text = format!("{} subscribers", subscriber_count);
        let views_text = format!("{} views", views_count);
        let video_count_text = format!("{} videos", video_count);

        let mut total_comments = 0;
        let mut total_likes = 0;
        let mut latest_comment = None;
        if let Ok(db) = crate::db::Database::open() {
            let _ = db.upsert_channel(
                &channel_id,
                &clean_handle,
                &title,
                &uploads_playlist_id,
                subscriber_count,
                views_count,
                video_count,
            );

            if !uploads_playlist_id.is_empty() {
                self.sync_channel_videos(&db, &channel_id, &uploads_playlist_id, api_key);
            }

            if let Ok(comments) = db.get_channel_total_comments(&channel_id) {
                total_comments = comments;
            }
            if let Ok(likes) = db.get_channel_total_likes(&channel_id) {
                total_likes = likes;
            }
            if let Ok(lc) = db.get_channel_latest_comment(&channel_id) {
                latest_comment = lc;
            }
        }

        Ok(ChannelData {
            handle: clean_handle.clone(),
            title,
            subscriber_text,
            subscriber_count,
            initial_subscriber_count: None,
            views_text,
            views_count,
            initial_views_count: None,
            video_count,
            initial_video_count: None,
            total_comments,
            initial_total_comments: None,
            total_likes,
            initial_total_likes: None,
            latest_comment,
            channel_id: Some(channel_id),
            uploads_playlist_id: Some(uploads_playlist_id),
            video_count_text,
            avatar_url,
            avatar_bytes,
            channel_url: format!("https://www.youtube.com/{}", clean_handle),
            error: None,
            sub_changed: false,
            views_changed: false,
            video_changed: false,
            comments_changed: false,
            likes_changed: false,
            sub_changed_at: None,
            views_changed_at: None,
            video_changed_at: None,
            comments_changed_at: None,
            likes_changed_at: None,
        })
    }

    fn sync_channel_videos(
        &self,
        db: &crate::db::Database,
        channel_id: &str,
        uploads_playlist_id: &str,
        api_key: &str,
    ) {
        let mut ids_to_fetch = Vec::new();

        let playlist_url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet,contentDetails&maxResults=50&playlistId={}&key={}",
            uploads_playlist_id, api_key
        );

        if let Ok(resp) = self.client.get(&playlist_url).send() {
            if resp.status().is_success() {
                if let Ok(text) = resp.text() {
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                            for item in items {
                                let video_id = item
                                    .pointer("/contentDetails/videoId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let title = item
                                    .pointer("/snippet/title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let published_str = item
                                    .pointer("/snippet/publishedAt")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let published_at = parse_iso8601_to_unix(published_str);

                                if !video_id.is_empty() {
                                    let is_short_guess = title.to_lowercase().contains("#short");
                                    let _ = db.register_discovered_video(
                                        video_id,
                                        channel_id,
                                        title,
                                        published_at,
                                        is_short_guess,
                                    );
                                    if !ids_to_fetch.contains(&video_id.to_string()) {
                                        ids_to_fetch.push(video_id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also query tiered videos needing refresh from DB
        if let Ok(needing_refresh) = db.get_videos_needing_refresh(channel_id, 50) {
            for vid in needing_refresh {
                if !ids_to_fetch.contains(&vid) {
                    ids_to_fetch.push(vid);
                }
            }
        }

        // Fetch statistics for all collected videos in batches of up to 50
        for chunk in ids_to_fetch.chunks(50) {
            let ids_csv = chunk.join(",");
            let videos_url = format!(
                "https://www.googleapis.com/youtube/v3/videos?part=snippet,contentDetails,statistics&id={}&key={}",
                ids_csv, api_key
            );

            if let Ok(resp) = self.client.get(&videos_url).send() {
                if resp.status().is_success() {
                    if let Ok(text) = resp.text() {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                                for item in items {
                                    let vid = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                    let title = item
                                        .pointer("/snippet/title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let published_str = item
                                        .pointer("/snippet/publishedAt")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let published_at = parse_iso8601_to_unix(published_str);

                                    let views = item
                                        .pointer("/statistics/viewCount")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(0);
                                    let likes = item
                                        .pointer("/statistics/likeCount")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(0);
                                    let comments = item
                                        .pointer("/statistics/commentCount")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(0);

                                    let duration_str = item
                                        .pointer("/contentDetails/duration")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let duration_secs = parse_iso8601_duration(duration_str);
                                    let title_lower = title.to_lowercase();
                                    let is_short = (duration_secs > 0 && duration_secs <= 60)
                                        || title_lower.contains("#short");

                                    let cached = db.get_video(vid).ok().flatten();
                                    let mut last_comment_at =
                                        cached.as_ref().map(|v| v.last_comment_at).unwrap_or(0);
                                    let mut last_comment_count_checked = cached
                                        .as_ref()
                                        .map(|v| v.last_comment_count_checked)
                                        .unwrap_or(0);
                                    let mut last_comment_author = cached
                                        .as_ref()
                                        .map(|v| v.last_comment_author.clone())
                                        .unwrap_or_default();
                                    let mut last_comment_text = cached
                                        .as_ref()
                                        .map(|v| v.last_comment_text.clone())
                                        .unwrap_or_default();

                                    // Only query commentThreads if comments > 0 and count changed or timestamp is missing
                                    if comments > 0
                                        && (last_comment_count_checked != comments
                                            || last_comment_at == 0)
                                    {
                                        let comment_url = format!(
                                            "https://www.googleapis.com/youtube/v3/commentThreads?part=snippet&videoId={}&order=time&maxResults=1&key={}",
                                            vid, api_key
                                        );
                                        if let Ok(c_resp) = self.client.get(&comment_url).send() {
                                            if c_resp.status().is_success() {
                                                if let Ok(c_text) = c_resp.text() {
                                                    if let Ok(c_json) =
                                                        serde_json::from_str::<Value>(&c_text)
                                                    {
                                                        if let Some(snippet) = c_json.pointer(
                                                            "/items/0/snippet/topLevelComment/snippet",
                                                        ) {
                                                            if let Some(c_date) = snippet
                                                                .pointer("/publishedAt")
                                                                .and_then(|v| v.as_str())
                                                            {
                                                                last_comment_at =
                                                                    parse_iso8601_to_unix(c_date);
                                                            }
                                                            if let Some(author) = snippet
                                                                .pointer("/authorDisplayName")
                                                                .and_then(|v| v.as_str())
                                                            {
                                                                last_comment_author =
                                                                    author.to_string();
                                                            }
                                                            if let Some(text) = snippet
                                                                .pointer("/textOriginal")
                                                                .or_else(|| {
                                                                    snippet.pointer("/textDisplay")
                                                                })
                                                                .and_then(|v| v.as_str())
                                                            {
                                                                last_comment_text =
                                                                    text.to_string();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        last_comment_count_checked = comments;
                                    } else if comments == 0 {
                                        last_comment_at = 0;
                                        last_comment_count_checked = 0;
                                        last_comment_author.clear();
                                        last_comment_text.clear();
                                    }

                                    if !vid.is_empty() {
                                        let _ = db.upsert_video(
                                            vid,
                                            channel_id,
                                            title,
                                            published_at,
                                            views,
                                            likes,
                                            comments,
                                            is_short,
                                            last_comment_at,
                                            last_comment_count_checked,
                                            &last_comment_author,
                                            &last_comment_text,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn fetch_channel_scrape(&self, handle: &str) -> ChannelData {
        let clean_handle = if !handle.starts_with('@') && !handle.starts_with("UC") {
            format!("@{}", handle)
        } else {
            handle.to_string()
        };

        let url = format!("https://www.youtube.com/{}/about", clean_handle);

        let mut data = ChannelData::new_loading(&clean_handle);
        data.channel_url = format!("https://www.youtube.com/{}", clean_handle);

        let response = match self
            .client
            .get(&url)
            .header("Accept-Language", "en-US,en;q=0.9,it-IT;q=0.8,it;q=0.7")
            .header("Cache-Control", "no-cache")
            .send()
        {
            Ok(resp) => resp,
            Err(e) => {
                data.error = Some(format!("Connection error: {}", e));
                data.subscriber_text = "N/A".to_string();
                data.views_text = "N/A".to_string();
                return data;
            }
        };

        if !response.status().is_success() {
            data.error = Some(format!("HTTP status: {}", response.status()));
            data.subscriber_text = "N/A".to_string();
            data.views_text = "N/A".to_string();
            return data;
        }

        let html = match response.text() {
            Ok(txt) => txt,
            Err(e) => {
                data.error = Some(format!("Error reading response: {}", e));
                return data;
            }
        };

        match self.parse_youtube_html(&html, &clean_handle) {
            Ok(mut parsed) => {
                if !parsed.avatar_url.is_empty() {
                    if let Ok(img_resp) = self.client.get(&parsed.avatar_url).send() {
                        if img_resp.status().is_success() {
                            if let Ok(bytes) = img_resp.bytes() {
                                parsed.avatar_bytes = Some(bytes.to_vec());
                            }
                        }
                    }
                }
                parsed
            }
            Err(e) => {
                data.error = Some(format!("Error extracting data: {}", e));
                data.subscriber_text = "N/A".to_string();
                data.views_text = "N/A".to_string();
                data
            }
        }
    }

    fn parse_youtube_html(&self, html: &str, handle: &str) -> Result<ChannelData, String> {
        let marker = "var ytInitialData = ";
        let start_idx = html
            .find(marker)
            .ok_or_else(|| "ytInitialData not found in HTML".to_string())?
            + marker.len();

        let end_idx = html[start_idx..]
            .find(";</script>")
            .map(|i| start_idx + i)
            .ok_or_else(|| "ytInitialData closing script tag not found".to_string())?;

        let json_str = &html[start_idx..end_idx];
        let json: Value =
            serde_json::from_str(json_str).map_err(|e| format!("JSON decode error: {}", e))?;

        let title = json
            .pointer("/header/pageHeaderRenderer/pageTitle")
            .and_then(|v| v.as_str())
            .or_else(|| {
                json.pointer("/header/pageHeaderRenderer/content/pageHeaderViewModel/title/dynamicTextViewModel/text/content")
                    .and_then(|v| v.as_str())
            })
            .or_else(|| {
                json.pointer("/metadata/channelMetadataRenderer/title")
                    .and_then(|v| v.as_str())
            })
            .unwrap_or(handle.trim_start_matches('@'))
            .to_string();

        let mut avatar_url = String::new();
        if let Some(thumbs) = json
            .pointer("/metadata/channelMetadataRenderer/avatar/thumbnails")
            .and_then(|v| v.as_array())
        {
            if let Some(last) = thumbs
                .last()
                .and_then(|t| t.get("url"))
                .and_then(|u| u.as_str())
            {
                avatar_url = last.to_string();
            }
        }
        if avatar_url.is_empty() {
            if let Some(sources) = json.pointer("/header/pageHeaderRenderer/content/pageHeaderViewModel/image/decoratedAvatarViewModel/avatar/avatarViewModel/image/sources").and_then(|v| v.as_array()) {
                if let Some(last) = sources.last().and_then(|t| t.get("url")).and_then(|u| u.as_str()) {
                    avatar_url = last.to_string();
                }
            }
        }

        let about_vm = find_key_recursive(&json, "aboutChannelViewModel");

        let mut subscriber_text = String::new();
        let mut views_text = String::new();
        let mut video_count_text = String::new();

        if let Some(about) = about_vm {
            if let Some(sub) = about.get("subscriberCountText").and_then(|v| v.as_str()) {
                subscriber_text = sub.to_string();
            }
            if let Some(view) = about.get("viewCountText").and_then(|v| v.as_str()) {
                views_text = view.to_string();
            }
            if let Some(videos) = about.get("videoCountText").and_then(|v| v.as_str()) {
                video_count_text = videos.to_string();
            }
        }

        if subscriber_text.is_empty() {
            if let Some(rows) = json.pointer("/header/pageHeaderRenderer/content/pageHeaderViewModel/metadata/contentMetadataViewModel/metadataRows").and_then(|v| v.as_array()) {
                for row in rows {
                    if let Some(parts) = row.get("metadataParts").and_then(|v| v.as_array()) {
                        for part in parts {
                            if let Some(txt) = part.pointer("/text/content").and_then(|v| v.as_str()) {
                                let lower = txt.to_lowercase();
                                if lower.contains("sub") || lower.contains("iscritt") {
                                    subscriber_text = txt.to_string();
                                } else if lower.contains("video") && video_count_text.is_empty() {
                                    video_count_text = txt.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        if subscriber_text.is_empty() {
            subscriber_text = "0 subscribers".to_string();
        }
        if views_text.is_empty() {
            views_text = "0 views".to_string();
        }

        let subscriber_count = parse_subscriber_count(&subscriber_text);
        let views_count = parse_subscriber_count(&views_text);
        let video_count = parse_subscriber_count(&video_count_text);

        let channel_id = json
            .pointer("/metadata/channelMetadataRenderer/externalId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut total_comments = 0;
        let mut total_likes = 0;
        let mut latest_comment = None;
        if let Some(cid) = &channel_id {
            if let Ok(db) = crate::db::Database::open() {
                let _ = db.upsert_channel(
                    cid,
                    handle,
                    &title,
                    "",
                    subscriber_count,
                    views_count,
                    video_count,
                );
                if let Ok(comments) = db.get_channel_total_comments(cid) {
                    total_comments = comments;
                }
                if let Ok(likes) = db.get_channel_total_likes(cid) {
                    total_likes = likes;
                }
                if let Ok(lc) = db.get_channel_latest_comment(cid) {
                    latest_comment = lc;
                }
            }
        }

        Ok(ChannelData {
            handle: handle.to_string(),
            title,
            subscriber_text,
            subscriber_count,
            initial_subscriber_count: None,
            views_text,
            views_count,
            initial_views_count: None,
            video_count,
            initial_video_count: None,
            total_comments,
            initial_total_comments: None,
            total_likes,
            initial_total_likes: None,
            latest_comment,
            channel_id,
            uploads_playlist_id: None,
            video_count_text,
            avatar_url,
            avatar_bytes: None,
            channel_url: format!("https://www.youtube.com/{}", handle),
            error: None,
            sub_changed: false,
            views_changed: false,
            video_changed: false,
            comments_changed: false,
            likes_changed: false,
            sub_changed_at: None,
            views_changed_at: None,
            video_changed_at: None,
            comments_changed_at: None,
            likes_changed_at: None,
        })
    }
}

pub fn parse_iso8601_to_unix(s: &str) -> u64 {
    let parts: Vec<&str> = s
        .split(['-', 'T', ':', 'Z', '.'])
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() >= 6 {
        let year: i64 = parts[0].parse().unwrap_or(1970);
        let month: u64 = parts[1].parse().unwrap_or(1);
        let day: u64 = parts[2].parse().unwrap_or(1);
        let hour: u64 = parts[3].parse().unwrap_or(0);
        let min: u64 = parts[4].parse().unwrap_or(0);
        let sec: u64 = parts[5].parse().unwrap_or(0);

        let mut days = (year - 1970) * 365 + ((year - 1969) / 4) - ((year - 1901) / 100)
            + ((year - 1601) / 400);
        let days_in_months = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        for m in 1..month {
            days += days_in_months[m as usize];
            if m == 2 && is_leap {
                days += 1;
            }
        }
        days += (day as i64) - 1;
        let total_secs = days * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64);
        total_secs.max(0) as u64
    } else {
        0
    }
}

pub fn parse_iso8601_duration(duration: &str) -> u64 {
    if !duration.starts_with('P') {
        return 0;
    }
    let mut total_secs = 0u64;
    let mut current_num = 0u64;
    let mut in_time = false;

    for ch in duration.chars() {
        match ch {
            'P' => {}
            'T' => in_time = true,
            '0'..='9' => {
                current_num = current_num
                    .saturating_mul(10)
                    .saturating_add(ch as u64 - '0' as u64);
            }
            'D' => {
                total_secs = total_secs.saturating_add(current_num.saturating_mul(86_400));
                current_num = 0;
            }
            'H' => {
                total_secs = total_secs.saturating_add(current_num.saturating_mul(3_600));
                current_num = 0;
            }
            'M' => {
                if in_time {
                    total_secs = total_secs.saturating_add(current_num.saturating_mul(60));
                } else {
                    total_secs = total_secs.saturating_add(current_num.saturating_mul(86_400 * 30));
                }
                current_num = 0;
            }
            'S' => {
                total_secs = total_secs.saturating_add(current_num);
                current_num = 0;
            }
            _ => {}
        }
    }
    total_secs
}

fn find_key_recursive<'a>(value: &'a Value, target_key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if let Some(found) = map.get(target_key) {
                return Some(found);
            }
            for (_k, v) in map {
                if let Some(found) = find_key_recursive(v, target_key) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(arr) => {
            for v in arr {
                if let Some(found) = find_key_recursive(v, target_key) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

pub fn clean_stat_number(text: &str) -> String {
    let words_to_remove = [
        "subscribers",
        "subscriber",
        "subs",
        "views",
        "view",
        "video",
        "videos",
        "visualizzazioni",
        "visualizzazion*",
        "visualizzazion",
        "iscritti",
        "iscritt*",
        "iscritt",
    ];

    let mut cleaned = text.to_string();
    for word in words_to_remove {
        if let Some(idx) = cleaned.to_lowercase().find(word) {
            cleaned.replace_range(idx..idx + word.len(), "");
        }
    }

    let trimmed = cleaned.trim().trim_matches('•').trim().to_string();

    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(num) = trimmed.parse::<u64>() {
            let s = num.to_string();
            let mut formatted = String::new();
            let len = s.len();
            for (i, ch) in s.chars().enumerate() {
                if i > 0 && (len - i) % 3 == 0 {
                    formatted.push('.');
                }
                formatted.push(ch);
            }
            return formatted;
        }
    }

    trimmed
}

pub fn parse_subscriber_count(text: &str) -> u64 {
    let cleaned = clean_stat_number(text);
    let lower = cleaned.to_lowercase();

    let multiplier: f64 = if lower.contains("mrd")
        || lower.contains("miliard")
        || lower.contains("billion")
        || lower.contains('b')
    {
        1_000_000_000.0
    } else if lower.contains("mila") || lower.contains("thousand") || lower.contains('k') {
        1_000.0
    } else if lower.contains("mln")
        || lower.contains("milion")
        || lower.contains("million")
        || lower.contains('m')
    {
        1_000_000.0
    } else {
        1.0
    };

    let mut num_str = String::new();
    let mut has_digit = false;

    for ch in lower.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            has_digit = true;
        } else if (ch == ',' || ch == '.') && has_digit {
            num_str.push('.');
        } else if has_digit && !ch.is_whitespace() {
            break;
        }
    }

    let normalized = if (multiplier - 1.0).abs() < f64::EPSILON {
        num_str.replace('.', "")
    } else {
        let dots = num_str.chars().filter(|&c| c == '.').count();
        if dots > 1 {
            num_str.replace('.', "")
        } else {
            num_str
        }
    };

    if let Ok(val) = normalized.parse::<f64>() {
        (val * multiplier).round() as u64
    } else {
        0
    }
}

pub fn format_compact_delta(delta: i64) -> String {
    if delta == 0 {
        return String::new();
    }
    let abs_val = delta.unsigned_abs();
    let sign = if delta > 0 { "+" } else { "-" };
    if abs_val >= 1_000_000_000 {
        format!("{}{:.1}B", sign, abs_val as f64 / 1_000_000_000.0)
    } else if abs_val >= 1_000_000 {
        format!("{}{:.1}M", sign, abs_val as f64 / 1_000_000.0)
    } else if abs_val >= 10_000 {
        format!("{}{:.1}K", sign, abs_val as f64 / 1_000.0)
    } else {
        format!("{}{}", sign, abs_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601() {
        let ts = parse_iso8601_to_unix("2026-11-04T12:00:00Z");
        assert!(ts > 1_700_000_000);
    }

    #[test]
    fn test_parse_iso8601_duration() {
        assert_eq!(parse_iso8601_duration("PT45S"), 45);
        assert_eq!(parse_iso8601_duration("PT1M"), 60);
        assert_eq!(parse_iso8601_duration("PT1M15S"), 75);
        assert_eq!(parse_iso8601_duration("PT2H30M10S"), 9010);
        assert_eq!(parse_iso8601_duration("P1DT2H"), 93600);
        assert_eq!(parse_iso8601_duration("invalid"), 0);
    }

    #[test]
    fn test_parse_subscriber_count() {
        assert_eq!(parse_subscriber_count("1.25M subscribers"), 1_250_000);
        assert_eq!(parse_subscriber_count("105K iscritti"), 105_000);
        assert_eq!(parse_subscriber_count("2.507 iscritti"), 2_507);
        assert_eq!(parse_subscriber_count("2,507 subscribers"), 2_507);
        assert_eq!(parse_subscriber_count("500 iscritti"), 500);
        assert_eq!(parse_subscriber_count("3.2B views"), 3_200_000_000);
        assert_eq!(parse_subscriber_count("0 subscribers"), 0);
    }

    #[test]
    fn test_clean_stat_number() {
        assert_eq!(clean_stat_number("1.25M subscribers"), "1.25M");
        assert_eq!(clean_stat_number("105K iscritti"), "105K");
        assert_eq!(clean_stat_number("2507"), "2.507");
        assert_eq!(clean_stat_number("2507 iscritti"), "2.507");
        assert_eq!(clean_stat_number("• 12.345 views •"), "12.345");
    }

    #[test]
    fn test_channel_data_deltas() {
        let mut ch = ChannelData::new_loading("@test");
        assert_eq!(ch.delta_subscribers(), 0);
        assert_eq!(ch.delta_views(), 0);

        ch.initial_subscriber_count = Some(1000);
        ch.subscriber_count = 1050;
        assert_eq!(ch.delta_subscribers(), 50);
        assert_eq!(format_compact_delta(ch.delta_subscribers()), "+50");

        ch.subscriber_count = 980;
        assert_eq!(ch.delta_subscribers(), -20);
        assert_eq!(format_compact_delta(ch.delta_subscribers()), "-20");

        ch.initial_views_count = Some(2000);
        ch.views_count = 2150;
        assert_eq!(ch.delta_views(), 150);
        assert_eq!(format_compact_delta(ch.delta_views()), "+150");

        ch.initial_total_likes = Some(500);
        ch.total_likes = 525;
        assert_eq!(ch.delta_likes(), 25);
        assert_eq!(format_compact_delta(ch.delta_likes()), "+25");

        assert_eq!(format_compact_delta(0), "");
        assert_eq!(format_compact_delta(15000), "+15.0K");
    }

    #[test]
    fn test_channel_data_approve_metric() {
        let mut ch = ChannelData::new_loading("@test");
        ch.initial_subscriber_count = Some(100);
        ch.subscriber_count = 120;
        ch.sub_changed = true;
        assert_eq!(ch.delta_subscribers(), 20);

        ch.approve_metric(CardMetric::Subscribers);
        assert_eq!(ch.delta_subscribers(), 0);
        assert!(!ch.sub_changed);
        assert_eq!(ch.initial_subscriber_count, Some(120));

        ch.initial_total_comments = Some(50);
        ch.total_comments = 55;
        ch.comments_changed = true;
        ch.approve_metric(CardMetric::Comments);
        assert_eq!(ch.delta_comments(), 0);
        assert!(!ch.comments_changed);
    }

    #[test]
    fn test_comments_inbox_url() {
        let mut ch = ChannelData::new_loading("@LeonardoAnchisi");
        ch.channel_id = Some("UC_x5XG1OV2P6uZZ5FSM9Ttw".to_string());
        let url = ch.comments_inbox_url();
        assert!(url.starts_with(
            "https://studio.youtube.com/channel/UC_x5XG1OV2P6uZZ5FSM9Ttw/comments/inbox"
        ));
        assert!(url.contains("SORT_BY_MOST_RELEVANT"));
        assert!(url.contains("COMMENT_CATEGORY_NOT_ENGAGED"));

        let ch_fallback = ChannelData::new_loading("@LeonardoAnchisi");
        let fallback_url = ch_fallback.comments_inbox_url();
        assert!(fallback_url.contains("LeonardoAnchisi"));
    }

    #[test]
    fn test_fetch_live_channel() {
        let fetcher = YouTubeFetcher::new();
        let data = fetcher.fetch_channel("@matteobaccan");
        println!(
            "Fetched channel: handle={}, title={}, subscribers='{}' ({}), views='{}', avatar_len={:?}",
            data.handle,
            data.title,
            data.subscriber_text,
            data.subscriber_count,
            data.views_text,
            data.avatar_bytes.as_ref().map(|b| b.len())
        );
        assert_eq!(data.handle, "@matteobaccan");
        assert!(!data.title.is_empty());
        assert!(data.subscriber_count > 0);
        assert!(data.avatar_bytes.is_some());
    }

    #[test]
    fn test_fetch_with_api_key() {
        let config = crate::config::Config::load_or_create();
        if let Some(key) = &config.effective_api_key {
            let fetcher = YouTubeFetcher::with_api_key(Some(key.clone()));
            let data = fetcher.fetch_channel("@matteobaccan");
            assert!(data.channel_id.is_some());
            assert!(data.video_count > 0);
            if let Some(cid) = &data.channel_id {
                let db = crate::db::Database::open().unwrap();
                let videos = db.get_recent_videos(cid, 10).unwrap();
                assert!(!videos.is_empty());
                let has_views = videos.iter().any(|v| v.views > 0);
                assert!(has_views);
            }
        }
    }
}
