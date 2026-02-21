use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
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

        // Initialize schema
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
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
