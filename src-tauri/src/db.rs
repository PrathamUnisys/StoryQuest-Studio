use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::{Connection, OptionalExtension};

pub struct Migration {
    pub version: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "001_initial_schema",
        sql: include_str!("../migrations/001_initial_schema.sql"),
    },
    Migration {
        version: "002_seed_data",
        sql: include_str!("../migrations/002_seed_data.sql"),
    },
    Migration {
        version: "003_foundation",
        sql: include_str!("../migrations/003_foundation.sql"),
    },
    Migration {
        version: "004_repair_achievement_seed",
        sql: include_str!("../migrations/004_repair_achievement_seed.sql"),
    },
    Migration {
        version: "005_gamification_notifications",
        sql: include_str!("../migrations/005_achievement_notifications.sql"),
    },
    Migration {
        version: "006_youtube_sync",
        sql: include_str!("../migrations/006_youtube_sync.sql"),
    },
];

fn is_memory(path: &Path) -> bool {
    path.to_string_lossy() == ":memory:"
}

fn ensure_migrations_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     TEXT PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create schema_migrations table: {e}"))
}

fn applied_versions(conn: &Connection) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare("SELECT version FROM schema_migrations")
        .map_err(|e| format!("Failed to read schema_migrations: {e}"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query schema versions: {e}"))?;

    let mut versions = HashSet::new();
    for row in rows {
        versions.insert(row.map_err(|e| format!("Failed to read schema version: {e}"))?);
    }
    Ok(versions)
}

pub fn pending_migrations(conn: &Connection) -> Result<Vec<&'static str>, String> {
    ensure_migrations_table(conn)?;
    let applied = applied_versions(conn)?;
    Ok(MIGRATIONS
        .iter()
        .filter(|m| !applied.contains(m.version))
        .map(|m| m.version)
        .collect())
}

pub fn schema_version(conn: &Connection) -> Result<String, String> {
    ensure_migrations_table(conn)?;
    conn.query_row(
        "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| format!("Failed to read schema version: {e}"))
    .map(|v| v.unwrap_or_else(|| "0".to_string()))
}

fn backup_before_migration(db_path: &Path) -> Result<Option<PathBuf>, String> {
    if is_memory(db_path) || !db_path.exists() {
        return Ok(None);
    }

    let backup_dir = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("Backups");
    fs::create_dir_all(&backup_dir).map_err(|e| {
        format!(
            "Failed to create backup folder {}: {e}",
            backup_dir.display()
        )
    })?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("storyquest_pre_migration_{timestamp}.db"));

    let source = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database for pre-migration backup: {e}"))?;
    source
        .execute(
            "VACUUM INTO ?1",
            rusqlite::params![backup_path.to_string_lossy().to_string()],
        )
        .map_err(|e| format!("Failed to create pre-migration backup: {e}"))?;

    Ok(Some(backup_path))
}

fn apply_migration(conn: &mut Connection, migration: &Migration) -> Result<(), String> {
    conn.execute_batch("BEGIN IMMEDIATE;")
        .map_err(|e| format!("Failed to begin migration {}: {e}", migration.version))?;

    let result = (|| -> Result<(), String> {
        conn.execute_batch(migration.sql).map_err(|e| {
            format!(
                "Migration {} failed: {e}",
                migration.version
            )
        })?;
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            rusqlite::params![migration.version],
        )
        .map_err(|e| format!("Failed to record migration {}: {e}", migration.version))?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .map_err(|e| format!("Failed to commit migration {}: {e}", migration.version))?;
            Ok(())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

pub fn init_database(db_path: &Path) -> Result<Connection, String> {
    if !is_memory(db_path) {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create data folder {}: {e}", parent.display())
                })?;
            }
        }
    }

    let existed = db_path.exists();
    let mut conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database {}: {e}", db_path.display()))?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {e}"))?;

    ensure_migrations_table(&conn)?;
    let pending = pending_migrations(&conn)?;

    if existed && !pending.is_empty() {
        if let Some(path) = backup_before_migration(db_path)? {
            println!("Pre-migration backup created: {}", path.display());
        }
    }

    for migration in MIGRATIONS.iter().filter(|m| pending.contains(&m.version)) {
        println!("Applying migration {}", migration.version);
        apply_migration(&mut conn, migration)?;
    }

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")
        .map_err(|e| format!("Failed to configure SQLite journal mode: {e}"))?;

    Ok(conn)
}

pub fn create_database_backup(db_path: &Path, backup_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(backup_dir)
        .map_err(|e| format!("Failed to create backup folder {}: {e}", backup_dir.display()))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("storyquest_backup_{timestamp}.db"));

    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database for backup: {e}"))?;
    conn.execute(
        "VACUUM INTO ?1",
        rusqlite::params![backup_path.to_string_lossy().to_string()],
    )
    .map_err(|e| format!("Failed to write database backup: {e}"))?;

    Ok(backup_path)
}
