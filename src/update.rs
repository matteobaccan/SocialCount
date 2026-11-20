// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use std::time::Duration;

/// Page opened when the user clicks the update notice
pub const RELEASES_URL: &str = "https://github.com/matteobaccan/SocialCount/releases/latest";

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/matteobaccan/SocialCount/releases/latest";

/// Returns the tag of the latest published release when it is newer than the running version
pub fn check_for_update() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(concat!("SocialCount/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let response = client
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = serde_json::from_str(&response.text().ok()?).ok()?;
    let tag = body.get("tag_name")?.as_str()?.trim().to_string();

    if is_newer_version(&tag, env!("CARGO_PKG_VERSION")) {
        Some(tag)
    } else {
        None
    }
}

/// Parses a version such as `v1.2.3` or `1.2` into its numeric components
fn parse_version(version: &str) -> Option<Vec<u64>> {
    let trimmed = version.trim().trim_start_matches(['v', 'V']);
    // Ignore pre-release / build suffixes (e.g. 1.2.3-beta1)
    let core = trimmed.split(['-', '+']).next().unwrap_or("");
    if core.is_empty() {
        return None;
    }
    core.split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

/// True when `candidate` is a strictly higher version than `current`
pub fn is_newer_version(candidate: &str, current: &str) -> bool {
    let (Some(mut a), Some(mut b)) = (parse_version(candidate), parse_version(current)) else {
        return false;
    };
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a > b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("v1.6.0", "1.5.0"));
        assert!(is_newer_version("1.5.1", "1.5.0"));
        assert!(is_newer_version("v2.0", "1.9.9"));
        assert!(is_newer_version("v1.10.0", "1.9.0"));
        assert!(!is_newer_version("v1.5.0", "1.5.0"));
        assert!(!is_newer_version("v1.5", "1.5.0"));
        assert!(!is_newer_version("v1.4.9", "1.5.0"));
    }

    #[test]
    fn test_is_newer_version_invalid_input() {
        assert!(!is_newer_version("", "1.5.0"));
        assert!(!is_newer_version("latest", "1.5.0"));
        assert!(!is_newer_version("v1.x.0", "1.5.0"));
        assert!(is_newer_version("v1.6.0-beta1", "1.5.0"));
    }
}
