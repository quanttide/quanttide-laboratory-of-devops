use std::path::{Path, PathBuf};

pub struct HistoryDb {
    db: rusqlite::Connection,
    repo_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub id: i64,
    pub timestamp: String,
    pub action: String,
    pub submodule_name: String,
    pub detail: String,
    pub success: bool,
}

impl HistoryDb {
    pub fn open(repo_root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let db_dir = repo_root.join(".git").join("kse");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("history.db");
        let db = rusqlite::Connection::open(&db_path)?;
        let hist = Self {
            db,
            repo_path: repo_root.to_path_buf(),
        };
        hist.initialize()?;
        Ok(hist)
    }

    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                action TEXT NOT NULL,
                submodule_name TEXT NOT NULL,
                detail TEXT DEFAULT '',
                success INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS retired_submodules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT DEFAULT '',
                path TEXT DEFAULT '',
                retired_at TEXT NOT NULL DEFAULT (datetime('now')),
                reason TEXT DEFAULT ''
            );",
        )?;
        Ok(())
    }

    pub fn log_operation(
        &self,
        action: &str,
        submodule_name: &str,
        detail: &str,
        success: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.db.execute(
            "INSERT INTO operations (action, submodule_name, detail, success) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![action, submodule_name, detail, success as i32],
        )?;
        Ok(())
    }

    pub fn log_retire(
        &self,
        name: &str,
        url: &str,
        path: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.db.execute(
            "INSERT INTO retired_submodules (name, url, path, reason) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, url, path, reason],
        )?;
        self.log_operation("retire", name, &format!("退役子模块: {}", reason), true)
    }

    pub fn list_operations(
        &self,
        limit: usize,
        submodule_filter: Option<&str>,
    ) -> Result<Vec<OperationRecord>, Box<dyn std::error::Error>> {
        let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
            if let Some(name) = submodule_filter {
                (
                    "SELECT id, timestamp, action, submodule_name, detail, success FROM operations WHERE submodule_name = ?1 ORDER BY id DESC LIMIT ?2".into(),
                    vec![Box::new(name.to_string()), Box::new(limit as i64)],
                )
            } else {
                (
                    "SELECT id, timestamp, action, submodule_name, detail, success FROM operations ORDER BY id DESC LIMIT ?1".into(),
                    vec![Box::new(limit as i64)],
                )
            };

        let mut stmt = self.db.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(OperationRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                submodule_name: row.get(3)?,
                detail: row.get(4)?,
                success: row.get::<_, i32>(5)? != 0,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }
}
