// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ChannelData {
    pub handle: String,
    pub title: String,
    pub subscriber_text: String,
    pub subscriber_count: u64,
    pub views_text: String,
    #[allow(dead_code)]
    pub video_count_text: String,
    pub avatar_url: String,
    pub avatar_bytes: Option<Vec<u8>>,
    pub channel_url: String,
    pub error: Option<String>,
    pub sub_changed: bool,
    pub views_changed: bool,
    pub sub_changed_at: Option<std::time::Instant>,
    pub views_changed_at: Option<std::time::Instant>,
}

impl ChannelData {
    pub fn new_loading(handle: &str) -> Self {
        Self {
            handle: handle.to_string(),
            title: handle.trim_start_matches('@').to_string(),
            subscriber_text: "Loading...".to_string(),
            subscriber_count: 0,
            views_text: "-".to_string(),
            video_count_text: "-".to_string(),
            avatar_url: String::new(),
            avatar_bytes: None,
            channel_url: format!("https://www.youtube.com/{}", handle),
            error: None,
            sub_changed: false,
            views_changed: false,
            sub_changed_at: None,
            views_changed_at: None,
        }
    }
}

pub struct YouTubeFetcher {
    client: reqwest::blocking::Client,
}

impl YouTubeFetcher {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self { client }
    }

    pub fn fetch_channel(&self, handle: &str) -> ChannelData {
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
                // If avatar URL is found, download image
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
        let json: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON decode error: {}", e))?;

        // Extract title
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

        // Extract avatar URL
        let mut avatar_url = String::new();
        if let Some(thumbs) = json.pointer("/metadata/channelMetadataRenderer/avatar/thumbnails").and_then(|v| v.as_array()) {
            if let Some(last) = thumbs.last().and_then(|t| t.get("url")).and_then(|u| u.as_str()) {
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

        // Search recursively for aboutChannelViewModel
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

        // Fallbacks if not in aboutChannelViewModel
        if subscriber_text.is_empty() {
            // Check metadata rows in page header
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

        Ok(ChannelData {
            handle: handle.to_string(),
            title,
            subscriber_text,
            subscriber_count,
            views_text,
            video_count_text,
            avatar_url,
            avatar_bytes: None,
            channel_url: format!("https://www.youtube.com/{}", handle),
            error: None,
            sub_changed: false,
            views_changed: false,
            sub_changed_at: None,
            views_changed_at: None,
        })
    }
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
        "subscribers", "subscriber", "subs",
        "views", "view",
        "video", "videos",
        "visualizzazioni", "visualizzazion*", "visualizzazion",
        "iscritti", "iscritt*", "iscritt",
    ];

    let mut cleaned = text.to_string();
    for word in words_to_remove {
        if let Some(idx) = cleaned.to_lowercase().find(word) {
            cleaned.replace_range(idx..idx + word.len(), "");
        }
    }

    let trimmed = cleaned.trim().trim_matches('•').trim().to_string();

    // If trimmed is all digits and at least 4 digits, format with thousands separator dot: 2507 -> 2.507
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
    let lower = text.to_lowercase();

    // Determine multiplier (order matters to avoid 'm' matching 'mila')
    let multiplier: f64 = if lower.contains("mrd") || lower.contains("miliard") || lower.contains("billion") {
        1_000_000_000.0
    } else if lower.contains("mila") || lower.contains("thousand") || lower.contains('k') {
        1_000.0
    } else if lower.contains("mln") || lower.contains("milion") || lower.contains("million") || lower.contains('m') {
        1_000_000.0
    } else {
        1.0
    };

    // Extract digits and decimal point/comma
    let mut num_str = String::new();
    let mut has_digit = false;

    for ch in lower.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            has_digit = true;
        } else if (ch == ',' || ch == '.') && has_digit {
            // Replace comma with dot for standard float parsing
            num_str.push('.');
        } else if has_digit && !ch.is_whitespace() {
            // Reached unit suffix
            break;
        }
    }

    // Clean up multiple dots if formatted like 1.250.000
    let dots = num_str.chars().filter(|&c| c == '.').count();
    let normalized = if dots > 1 {
        // Thousand separators without decimal
        num_str.replace('.', "")
    } else {
        num_str
    };

    if let Ok(val) = normalized.parse::<f64>() {
        (val * multiplier).round() as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_stat_number() {
        assert_eq!(clean_stat_number("26 iscritti"), "26");
        assert_eq!(clean_stat_number("26 subscribers"), "26");
        assert_eq!(clean_stat_number("2507 visualizzazioni"), "2.507");
        assert_eq!(clean_stat_number("2507 views"), "2.507");
        assert_eq!(clean_stat_number("559 visualizzazioni"), "559");
        assert_eq!(clean_stat_number("559 views"), "559");
        assert_eq!(clean_stat_number("1,3 mila iscritti"), "1,3 mila");
        assert_eq!(clean_stat_number("1.3K subscribers"), "1.3K");
        assert_eq!(clean_stat_number("10.2M subscribers"), "10.2M");
    }

    #[test]
    fn test_parse_subscriber_counts() {
        assert_eq!(parse_subscriber_count("26 iscritti"), 26);
        assert_eq!(parse_subscriber_count("26 subscribers"), 26);
        assert_eq!(parse_subscriber_count("18 iscritti"), 18);
        assert_eq!(parse_subscriber_count("1,3 mila iscritti"), 1300);
        assert_eq!(parse_subscriber_count("1.3K subscribers"), 1300);
        assert_eq!(parse_subscriber_count("2,5 mln iscritti"), 2500000);
        assert_eq!(parse_subscriber_count("2.5M subscribers"), 2500000);
        assert_eq!(parse_subscriber_count("10M subscribers"), 10000000);
        assert_eq!(parse_subscriber_count("1.250.000 iscritti"), 1250000);
        assert_eq!(parse_subscriber_count("1,250,000 subscribers"), 1250000);
    }

    #[test]
    fn test_fetch_live_channel() {
        let fetcher = YouTubeFetcher::new();
        let data = fetcher.fetch_channel("@matteobaccan");
        println!("Fetched channel: handle={}, title={}, subscribers='{}' ({}), views='{}', avatar_len={:?}",
            data.handle, data.title, data.subscriber_text, data.subscriber_count, data.views_text, data.avatar_bytes.as_ref().map(|b| b.len())
        );
        assert_eq!(data.handle, "@matteobaccan");
        assert!(!data.title.is_empty());
        assert!(data.subscriber_count > 0);
        assert!(data.avatar_bytes.is_some());
    }
}
