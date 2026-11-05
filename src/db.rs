// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DB_FILENAME: &str = "socialcount.db";

pub struct Database {
    conn: Connection,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CachedChannel {
    pub channel_id: String,
    pub handle: String,
    pub title: String,
    pub uploads_playlist_id: String,
    pub subscriber_count: u64,
    pub view_count: u64,
    pub video_count: u64,
    pub last_checked_at: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CachedVideo {
    pub video_id: String,
    pub channel_id: String,
    pub title: String,
    pub published_at: u64,
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub last_checked_at: u64,
    pub is_short: bool,
    pub last_comment_at: u64,
    pub last_comment_count_checked: u64,
}

impl CachedVideo {
    pub fn is_short_video(&self) -> bool {
        self.is_short || self.title.to_lowercase().contains("#short")
    }

    #[allow(dead_code)]
    pub fn video_type_label(&self) -> &'static str {
        if self.is_short_video() {
            "Short"
        } else {
            "Video"
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Database {
    pub fn open() -> Result<Self> {
        Self::open_path(Path::new(DB_FILENAME))
    }

    pub fn open_path(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                channel_id           TEXT PRIMARY KEY,
                handle               TEXT NOT NULL,
                title                TEXT NOT NULL DEFAULT '',
                uploads_playlist_id  TEXT NOT NULL DEFAULT '',
                subscriber_count     INTEGER NOT NULL DEFAULT 0,
                view_count           INTEGER NOT NULL DEFAULT 0,
                video_count          INTEGER NOT NULL DEFAULT 0,
                last_checked_at      INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_channels_handle ON channels(handle);

            CREATE TABLE IF NOT EXISTS videos (
                video_id                    TEXT PRIMARY KEY,
                channel_id                  TEXT NOT NULL,
                title                       TEXT NOT NULL DEFAULT '',
                published_at                INTEGER NOT NULL DEFAULT 0,
                views                       INTEGER NOT NULL DEFAULT 0,
                likes                       INTEGER NOT NULL DEFAULT 0,
                comments                    INTEGER NOT NULL DEFAULT 0,
                last_checked_at             INTEGER NOT NULL DEFAULT 0,
                is_short                    INTEGER NOT NULL DEFAULT 0,
                last_comment_at             INTEGER NOT NULL DEFAULT 0,
                last_comment_count_checked  INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(channel_id) REFERENCES channels(channel_id)
            );

            CREATE INDEX IF NOT EXISTS idx_videos_channel_tier 
            ON videos(channel_id, published_at, last_checked_at);
            "#,
        )?;
        let _ = self.conn.execute(
            "ALTER TABLE videos ADD COLUMN is_short INTEGER NOT NULL DEFAULT 0;",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE videos ADD COLUMN last_comment_at INTEGER NOT NULL DEFAULT 0;",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE videos ADD COLUMN last_comment_count_checked INTEGER NOT NULL DEFAULT 0;",
            [],
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_channel(
        &self,
        channel_id: &str,
        handle: &str,
        title: &str,
        uploads_playlist_id: &str,
        subscriber_count: u64,
        view_count: u64,
        video_count: u64,
    ) -> Result<()> {
        let now = current_timestamp();
        self.conn.execute(
            r#"
            INSERT INTO channels (
                channel_id, handle, title, uploads_playlist_id,
                subscriber_count, view_count, video_count, last_checked_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(channel_id) DO UPDATE SET
                handle = excluded.handle,
                title = excluded.title,
                uploads_playlist_id = CASE WHEN excluded.uploads_playlist_id != '' THEN excluded.uploads_playlist_id ELSE uploads_playlist_id END,
                subscriber_count = excluded.subscriber_count,
                view_count = excluded.view_count,
                video_count = excluded.video_count,
                last_checked_at = excluded.last_checked_at;
            "#,
            params![
                channel_id,
                handle,
                title,
                uploads_playlist_id,
                subscriber_count as i64,
                view_count as i64,
                video_count as i64,
                now as i64
            ],
        )?;
        Ok(())
    }

    pub fn get_channel_by_handle(&self, handle: &str) -> Result<Option<CachedChannel>> {
        let clean = if !handle.starts_with('@') && !handle.starts_with("UC") {
            format!("@{}", handle)
        } else {
            handle.to_string()
        };

        let mut stmt = self.conn.prepare(
            r#"
            SELECT channel_id, handle, title, uploads_playlist_id,
                   subscriber_count, view_count, video_count, last_checked_at
            FROM channels
            WHERE handle = ?1 OR channel_id = ?1
            LIMIT 1;
            "#,
        )?;

        let mut rows = stmt.query(params![clean])?;
        if let Some(row) = rows.next()? {
            Ok(Some(CachedChannel {
                channel_id: row.get(0)?,
                handle: row.get(1)?,
                title: row.get(2)?,
                uploads_playlist_id: row.get(3)?,
                subscriber_count: row.get::<_, i64>(4)? as u64,
                view_count: row.get::<_, i64>(5)? as u64,
                video_count: row.get::<_, i64>(6)? as u64,
                last_checked_at: row.get::<_, i64>(7)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn register_discovered_video(
        &self,
        video_id: &str,
        channel_id: &str,
        title: &str,
        published_at: u64,
        is_short: bool,
    ) -> Result<()> {
        let is_short_val = if is_short { 1i64 } else { 0i64 };
        self.conn.execute(
            r#"
            INSERT INTO videos (
                video_id, channel_id, title, published_at,
                views, likes, comments, last_checked_at, is_short
            )
            VALUES (?1, ?2, ?3, ?4, 0, 0, 0, 0, ?5)
            ON CONFLICT(video_id) DO UPDATE SET
                channel_id = excluded.channel_id,
                title = CASE WHEN excluded.title != '' THEN excluded.title ELSE title END,
                published_at = CASE WHEN excluded.published_at > 0 THEN excluded.published_at ELSE published_at END,
                is_short = CASE WHEN excluded.is_short != 0 THEN excluded.is_short ELSE is_short END;
            "#,
            params![video_id, channel_id, title, published_at as i64, is_short_val],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_video(
        &self,
        video_id: &str,
        channel_id: &str,
        title: &str,
        published_at: u64,
        views: u64,
        likes: u64,
        comments: u64,
        is_short: bool,
        last_comment_at: u64,
        last_comment_count_checked: u64,
    ) -> Result<()> {
        let now = current_timestamp();
        let is_short_val = if is_short { 1i64 } else { 0i64 };
        self.conn.execute(
            r#"
            INSERT INTO videos (
                video_id, channel_id, title, published_at,
                views, likes, comments, last_checked_at, is_short,
                last_comment_at, last_comment_count_checked
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(video_id) DO UPDATE SET
                channel_id = excluded.channel_id,
                title = CASE WHEN excluded.title != '' THEN excluded.title ELSE title END,
                published_at = CASE WHEN excluded.published_at > 0 THEN excluded.published_at ELSE published_at END,
                views = excluded.views,
                likes = excluded.likes,
                comments = excluded.comments,
                last_checked_at = excluded.last_checked_at,
                is_short = CASE WHEN excluded.is_short != 0 THEN excluded.is_short ELSE is_short END,
                last_comment_at = CASE WHEN excluded.last_comment_at > 0 THEN excluded.last_comment_at ELSE last_comment_at END,
                last_comment_count_checked = CASE WHEN excluded.last_comment_count_checked > 0 THEN excluded.last_comment_count_checked ELSE last_comment_count_checked END;
            "#,
            params![
                video_id,
                channel_id,
                title,
                published_at as i64,
                views as i64,
                likes as i64,
                comments as i64,
                now as i64,
                is_short_val,
                last_comment_at as i64,
                last_comment_count_checked as i64
            ],
        )?;
        Ok(())
    }

    pub fn get_video(&self, video_id: &str) -> Result<Option<CachedVideo>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT video_id, channel_id, title, published_at, views, likes, comments, last_checked_at, is_short, last_comment_at, last_comment_count_checked
            FROM videos
            WHERE video_id = ?1
            LIMIT 1;
            "#,
        )?;

        let mut rows = stmt.query(params![video_id])?;
        if let Some(row) = rows.next()? {
            let is_short_num: i64 = row.get::<_, Option<i64>>(8)?.unwrap_or(0);
            let last_comment_at: i64 = row.get::<_, Option<i64>>(9)?.unwrap_or(0);
            let last_comment_count_checked: i64 = row.get::<_, Option<i64>>(10)?.unwrap_or(0);
            Ok(Some(CachedVideo {
                video_id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                published_at: row.get::<_, i64>(3)? as u64,
                views: row.get::<_, i64>(4)? as u64,
                likes: row.get::<_, i64>(5)? as u64,
                comments: row.get::<_, i64>(6)? as u64,
                last_checked_at: row.get::<_, i64>(7)? as u64,
                is_short: is_short_num != 0,
                last_comment_at: last_comment_at.max(0) as u64,
                last_comment_count_checked: last_comment_count_checked.max(0) as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_channel_total_comments(&self, channel_id: &str) -> Result<u64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(SUM(comments), 0) FROM videos WHERE channel_id = ?1;")?;
        let total: i64 = stmt.query_row(params![channel_id], |row| row.get(0))?;
        Ok(total.max(0) as u64)
    }

    pub fn get_recent_videos(&self, channel_id: &str, limit: usize) -> Result<Vec<CachedVideo>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT video_id, channel_id, title, published_at, views, likes, comments, last_checked_at, is_short, last_comment_at, last_comment_count_checked
            FROM videos
            WHERE channel_id = ?1
            ORDER BY published_at DESC
            LIMIT ?2;
            "#,
        )?;

        let rows = stmt.query_map(params![channel_id, limit as i64], |row| {
            let is_short_num: i64 = row.get::<_, Option<i64>>(8)?.unwrap_or(0);
            let last_comment_at: i64 = row.get::<_, Option<i64>>(9)?.unwrap_or(0);
            let last_comment_count_checked: i64 = row.get::<_, Option<i64>>(10)?.unwrap_or(0);
            Ok(CachedVideo {
                video_id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                published_at: row.get::<_, i64>(3)? as u64,
                views: row.get::<_, i64>(4)? as u64,
                likes: row.get::<_, i64>(5)? as u64,
                comments: row.get::<_, i64>(6)? as u64,
                last_checked_at: row.get::<_, i64>(7)? as u64,
                is_short: is_short_num != 0,
                last_comment_at: last_comment_at.max(0) as u64,
                last_comment_count_checked: last_comment_count_checked.max(0) as u64,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_videos_needing_refresh(
        &self,
        channel_id: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let now = current_timestamp() as i64;
        let day = 86_400i64;

        let tier1_pub = now - (7 * day);
        let tier1_check = now - day;

        let tier2_pub = now - (30 * day);
        let tier2_check = now - (3 * day);

        let tier3_check = now - (7 * day);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT video_id FROM videos
            WHERE channel_id = ?1 AND (
                last_checked_at = 0
                OR (views = 0 AND comments = 0)
                OR (published_at >= ?2 AND last_checked_at < ?3)
                OR (published_at BETWEEN ?4 AND ?2 AND last_checked_at < ?5)
                OR (published_at < ?4 AND last_checked_at < ?6)
            )
            ORDER BY published_at DESC
            LIMIT ?7;
            "#,
        )?;

        let rows = stmt.query_map(
            params![
                channel_id,
                tier1_pub,
                tier1_check,
                tier2_pub,
                tier2_check,
                tier3_check,
                limit as i64
            ],
            |row| row.get::<_, String>(0),
        )?;

        let mut video_ids = Vec::new();
        for id in rows {
            video_ids.push(id?);
        }
        Ok(video_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_init_and_channel_upsert() {
        let db = Database::open_in_memory().expect("failed to open in-memory db");
        db.upsert_channel(
            "UC12345",
            "@matteobaccan",
            "Matteo Baccan",
            "UU12345",
            10000,
            500000,
            120,
        )
        .expect("upsert channel failed");

        let ch = db
            .get_channel_by_handle("@matteobaccan")
            .expect("fetch channel failed")
            .expect("channel should exist");

        assert_eq!(ch.channel_id, "UC12345");
        assert_eq!(ch.handle, "@matteobaccan");
        assert_eq!(ch.subscriber_count, 10000);
        assert_eq!(ch.video_count, 120);
    }

    #[test]
    fn test_db_video_comments_sum() {
        let db = Database::open_in_memory().expect("failed to open in-memory db");
        db.upsert_channel("UC12345", "@test", "Test", "UU12345", 100, 1000, 2)
            .unwrap();

        db.upsert_video("v1", "UC12345", "Video 1", 1000, 100, 10, 5, false, 0, 0)
            .unwrap();
        db.upsert_video(
            "v2",
            "UC12345",
            "Video 2 #shorts",
            2000,
            200,
            20,
            15,
            true,
            2050,
            15,
        )
        .unwrap();

        let total_comments = db.get_channel_total_comments("UC12345").unwrap();
        assert_eq!(total_comments, 20);

        let list = db.get_recent_videos("UC12345", 10).unwrap();
        assert_eq!(list.len(), 2);
        assert!(list[0].is_short_video());
        assert_eq!(list[0].video_type_label(), "Short");
        assert_eq!(list[0].last_comment_at, 2050);
        assert_eq!(list[0].last_comment_count_checked, 15);
        assert!(!list[1].is_short_video());
        assert_eq!(list[1].video_type_label(), "Video");
        assert_eq!(list[1].last_comment_at, 0);
    }
}
