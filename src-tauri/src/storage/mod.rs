//! 数据存储模块
//! 使用 SQLite 存储专注会话数据和每日统计

use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 专注会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    /// 会话 ID
    pub id: i64,
    /// 开始时间 (Unix 时间戳，毫秒)
    pub start_time: i64,
    /// 结束时间 (Unix 时间戳，毫秒)
    pub end_time: i64,
    /// 专注时长 (毫秒)
    pub focus_duration_ms: i64,
    /// 分心时长 (毫秒)
    pub distracted_duration_ms: i64,
}

/// 每日统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// 日期 (YYYY-MM-DD 格式)
    pub date: String,
    /// 总专注时长 (毫秒)
    pub total_focus_ms: i64,
    /// 总分心时长 (毫秒)
    pub total_distracted_ms: i64,
    /// 会话数量
    pub session_count: i32,
    /// 最长单次专注时长 (毫秒)
    pub longest_focus_ms: i64,
}

/// 数据库管理器
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开或创建数据库
    pub fn open<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;

        // 启用 WAL 模式提升性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        let db = Self { conn };
        db.init_tables()?;

        Ok(db)
    }

    /// 创建内存数据库（用于测试）
    pub fn in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// 初始化数据库表
    fn init_tables(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            r#"
            -- 专注会话表
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                focus_duration_ms INTEGER NOT NULL,
                distracted_duration_ms INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- 每日统计表
            CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT PRIMARY KEY,
                total_focus_ms INTEGER NOT NULL DEFAULT 0,
                total_distracted_ms INTEGER NOT NULL DEFAULT 0,
                session_count INTEGER NOT NULL DEFAULT 0,
                longest_focus_ms INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time);
            CREATE INDEX IF NOT EXISTS idx_sessions_end_time ON sessions(end_time);
            "#,
        )?;

        Ok(())
    }

    /// 插入新的专注会话
    pub fn insert_session(&self, session: &FocusSession) -> SqliteResult<i64> {
        self.conn.execute(
            r#"
            INSERT INTO sessions (start_time, end_time, focus_duration_ms, distracted_duration_ms)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            (
                session.start_time,
                session.end_time,
                session.focus_duration_ms,
                session.distracted_duration_ms,
            ),
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 获取今日统计
    pub fn get_today_stats(&self) -> SqliteResult<Option<DailyStats>> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.get_stats_by_date(&today)
    }

    /// 获取指定日期的统计
    pub fn get_stats_by_date(&self, date: &str) -> SqliteResult<Option<DailyStats>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, total_focus_ms, total_distracted_ms, session_count, longest_focus_ms
            FROM daily_stats
            WHERE date = ?1
            "#,
        )?;

        let mut rows = stmt.query([date])?;

        if let Some(row) = rows.next()? {
            Ok(Some(DailyStats {
                date: row.get(0)?,
                total_focus_ms: row.get(1)?,
                total_distracted_ms: row.get(2)?,
                session_count: row.get(3)?,
                longest_focus_ms: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 更新今日统计
    pub fn update_today_stats(&self, focus_ms: i64, distracted_ms: i64) -> SqliteResult<()> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        self.conn.execute(
            r#"
            INSERT INTO daily_stats (date, total_focus_ms, total_distracted_ms, session_count, longest_focus_ms)
            VALUES (?1, ?2, ?3, 1, ?2)
            ON CONFLICT(date) DO UPDATE SET
                total_focus_ms = total_focus_ms + ?2,
                total_distracted_ms = total_distracted_ms + ?3,
                session_count = session_count + 1,
                longest_focus_ms = MAX(longest_focus_ms, ?2),
                updated_at = CURRENT_TIMESTAMP
            "#,
            (&today, focus_ms, distracted_ms),
        )?;

        Ok(())
    }

    /// 获取最近 N 天的统计数据
    pub fn get_recent_stats(&self, days: u32) -> SqliteResult<Vec<DailyStats>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, total_focus_ms, total_distracted_ms, session_count, longest_focus_ms
            FROM daily_stats
            ORDER BY date DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map([days], |row| {
            Ok(DailyStats {
                date: row.get(0)?,
                total_focus_ms: row.get(1)?,
                total_distracted_ms: row.get(2)?,
                session_count: row.get(3)?,
                longest_focus_ms: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        assert!(db.get_today_stats().unwrap().is_none());
    }

    #[test]
    fn test_update_stats() {
        let db = Database::in_memory().unwrap();

        db.update_today_stats(60000, 10000).unwrap();
        let stats = db.get_today_stats().unwrap().unwrap();

        assert_eq!(stats.total_focus_ms, 60000);
        assert_eq!(stats.total_distracted_ms, 10000);
        assert_eq!(stats.session_count, 1);
    }
}
