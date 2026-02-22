use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub started_at: i64,
    pub duration_secs: u32,
    pub word_count: u32,
    pub model_used: String,
    pub transcription: String,
    pub latency_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsSummary {
    pub total_recordings: u32,
    pub total_seconds: u32,
    pub avg_words: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub id: i64,
    pub started_at: i64,
    pub duration_secs: u32,
    pub word_count: u32,
    pub transcription: String,
    pub latency_ms: u32,
    pub error: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create database directory at {:?}", parent))?;
            }
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open SQLite database at {:?}", db_path))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at INTEGER NOT NULL,
                duration_secs INTEGER NOT NULL,
                word_count INTEGER NOT NULL,
                model_used TEXT NOT NULL,
                transcription TEXT NOT NULL,
                latency_ms INTEGER NOT NULL,
                error TEXT
            )",
            [],
        ).context("Failed to initialize database schema")?;

        Ok(Self { conn })
    }

    pub fn log_session(&self, session: SessionRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (started_at, duration_secs, word_count, model_used, transcription, latency_ms, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.started_at,
                session.duration_secs,
                session.word_count,
                session.model_used,
                session.transcription,
                session.latency_ms,
                session.error,
            ],
        ).context("Failed to insert session record")?;

        Ok(())
    }

    pub fn get_stats_summary(&self) -> Result<StatsSummary> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*), COALESCE(SUM(duration_secs),0), COALESCE(AVG(word_count),0)
             FROM sessions WHERE error IS NULL"
        )?;
        let summary = stmt.query_row([], |row| {
            Ok(StatsSummary {
                total_recordings: row.get(0)?,
                total_seconds: row.get(1)?,
                avg_words: row.get(2)?,
            })
        })?;
        Ok(summary)
    }

    pub fn get_recent_sessions(&self, limit: u32) -> Result<Vec<SessionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, duration_secs, word_count, transcription, latency_ms, error
             FROM sessions ORDER BY started_at DESC LIMIT ?"
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                started_at: row.get(1)?,
                duration_secs: row.get(2)?,
                word_count: row.get(3)?,
                transcription: row.get(4)?,
                latency_ms: row.get(5)?,
                error: row.get(6)?,
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
