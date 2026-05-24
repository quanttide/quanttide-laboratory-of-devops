use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseStatus {
    Staged,
    Published,
    Cancelled,
    Retired,
}

// Journal 中的一行：不可变事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEntry {
    pub id: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub created_at: String,
}

// 内存中的业务实体：由事件回放投影得出
#[derive(Debug, Clone)]
pub struct ReleaseRecord {
    pub id: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl ReleaseRecord {
    pub fn new_staged(version: &str) -> Self {
        let now = timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version: version.to_string(),
            status: ReleaseStatus::Staged,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

fn timestamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

#[derive(Debug)]
pub enum TransitionError {
    AlreadyPublished(String),
    NotStaged(String),
    NotPublished(String),
    InvalidTransition {
        from: ReleaseStatus,
        to: ReleaseStatus,
    },
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyPublished(v) => write!(f, "版本 {} 已发布，不可重复 stage", v),
            Self::NotStaged(v) => write!(f, "版本 {} 不处于 Staged 状态", v),
            Self::NotPublished(v) => write!(f, "版本 {} 不处于 Published 状态", v),
            Self::InvalidTransition { from, to } => {
                write!(f, "不允许从 {:?} 转换到 {:?}", from, to)
            }
        }
    }
}

impl std::error::Error for TransitionError {}

pub fn validate_transition(from: &ReleaseStatus, to: &ReleaseStatus) -> Result<(), TransitionError> {
    match (from, to) {
        (ReleaseStatus::Staged, ReleaseStatus::Published) => Ok(()),
        (ReleaseStatus::Staged, ReleaseStatus::Cancelled) => Ok(()),
        (ReleaseStatus::Cancelled, ReleaseStatus::Staged) => Ok(()),
        (ReleaseStatus::Published, ReleaseStatus::Retired) => Ok(()),
        _ => Err(TransitionError::InvalidTransition {
            from: from.clone(),
            to: to.clone(),
        }),
    }
}

pub trait Storage {
    fn save(&mut self, record: &ReleaseRecord) -> Result<(), Box<dyn std::error::Error>>;
    fn load(&self, version: &str) -> Option<ReleaseRecord>;
    fn list(&self) -> Vec<ReleaseRecord>;
}

fn replay_events(path: &Path) -> Vec<ReleaseRecord> {
    if !path.exists() {
        return Vec::new();
    }
    let mut records: HashMap<String, ReleaseRecord> = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<ReleaseEntry>(line) {
                let first_created = records
                    .get(&entry.version)
                    .map(|r| r.created_at.clone())
                    .unwrap_or_else(|| entry.created_at.clone());
                records.insert(
                    entry.version.clone(),
                    ReleaseRecord {
                        id: entry.id,
                        version: entry.version,
                        status: entry.status,
                        created_at: first_created,
                        updated_at: entry.created_at,
                    },
                );
            }
        }
    }
    records.into_values().collect()
}

pub struct FileStorage {
    events_path: std::path::PathBuf,
    records: Vec<ReleaseRecord>,
}

impl FileStorage {
    pub fn new(base_path: &Path) -> Self {
        let events_path = base_path.join(".quanttide/devops/release-journal.jsonl");
        let records = replay_events(&events_path);
        Self {
            events_path,
            records,
        }
    }
}

impl Storage for FileStorage {
    fn save(&mut self, record: &ReleaseRecord) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|r| r.version == record.version)
        {
            *existing = record.clone();
        } else {
            self.records.push(record.clone());
        }

        let entry = ReleaseEntry {
            id: record.id.clone(),
            version: record.version.clone(),
            status: record.status.clone(),
            created_at: record.updated_at.clone(),
        };

        if let Some(parent) = self.events_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&entry)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        writeln!(f, "{}", json)?;

        Ok(())
    }

    fn load(&self, version: &str) -> Option<ReleaseRecord> {
        self.records
            .iter()
            .find(|r| r.version == version)
            .cloned()
    }

    fn list(&self) -> Vec<ReleaseRecord> {
        self.records.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(version: &str, status: ReleaseStatus) -> ReleaseRecord {
        let now = timestamp();
        ReleaseRecord {
            id: uuid::Uuid::new_v4().to_string(),
            version: version.to_string(),
            status,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    // ---- ReleaseStatus ----

    #[test]
    fn test_release_status_debug() {
        assert_eq!(format!("{:?}", ReleaseStatus::Staged), "Staged");
        assert_eq!(format!("{:?}", ReleaseStatus::Published), "Published");
        assert_eq!(format!("{:?}", ReleaseStatus::Cancelled), "Cancelled");
        assert_eq!(format!("{:?}", ReleaseStatus::Retired), "Retired");
    }

    #[test]
    fn test_release_status_clone_eq() {
        let a = ReleaseStatus::Staged;
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ---- validate_transition ----

    #[test]
    fn test_validate_staged_to_published() {
        assert!(validate_transition(&ReleaseStatus::Staged, &ReleaseStatus::Published).is_ok());
    }

    #[test]
    fn test_validate_staged_to_cancelled() {
        assert!(validate_transition(&ReleaseStatus::Staged, &ReleaseStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_validate_cancelled_to_staged() {
        assert!(validate_transition(&ReleaseStatus::Cancelled, &ReleaseStatus::Staged).is_ok());
    }

    #[test]
    fn test_validate_published_to_retired() {
        assert!(validate_transition(&ReleaseStatus::Published, &ReleaseStatus::Retired).is_ok());
    }

    #[test]
    fn test_validate_invalid_transitions() {
        assert!(validate_transition(&ReleaseStatus::Staged, &ReleaseStatus::Retired).is_err());
        assert!(validate_transition(&ReleaseStatus::Published, &ReleaseStatus::Staged).is_err());
        assert!(validate_transition(&ReleaseStatus::Published, &ReleaseStatus::Cancelled).is_err());
        assert!(validate_transition(&ReleaseStatus::Cancelled, &ReleaseStatus::Published).is_err());
        assert!(validate_transition(&ReleaseStatus::Cancelled, &ReleaseStatus::Retired).is_err());
        assert!(validate_transition(&ReleaseStatus::Retired, &ReleaseStatus::Staged).is_err());
        assert!(validate_transition(&ReleaseStatus::Retired, &ReleaseStatus::Published).is_err());
        assert!(validate_transition(&ReleaseStatus::Retired, &ReleaseStatus::Cancelled).is_err());
    }

    // ---- ReleaseRecord ----

    #[test]
    fn test_record_fields() {
        let r = make_record("v1.0.0", ReleaseStatus::Staged);
        assert_eq!(r.version, "v1.0.0");
        assert_eq!(r.status, ReleaseStatus::Staged);
        assert!(!r.id.is_empty());
        assert!(!r.created_at.is_empty());
        assert_eq!(r.created_at, r.updated_at);
    }

    // ---- FileStorage ----

    #[test]
    fn test_storage_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let r = make_record("v1.0.0", ReleaseStatus::Staged);
        storage.save(&r).unwrap();
        let loaded = storage.load("v1.0.0");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().version, "v1.0.0");
    }

    #[test]
    fn test_storage_load_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::new(dir.path());
        assert!(storage.load("v9.9.9").is_none());
    }

    #[test]
    fn test_storage_update_existing() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let mut r = make_record("v1.0.0", ReleaseStatus::Staged);
        storage.save(&r).unwrap();

        r.status = ReleaseStatus::Published;
        r.updated_at = "999".into();
        storage.save(&r).unwrap();

        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Published);
        assert_eq!(loaded.updated_at, "999");
    }

    #[test]
    fn test_storage_journal_appended() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let r = make_record("v1.0.0", ReleaseStatus::Staged);
        storage.save(&r).unwrap();

        let journal = dir.path().join(".quanttide/devops/release-journal.jsonl");
        let content = std::fs::read_to_string(&journal).unwrap();
        assert!(content.contains("v1.0.0"));

        let mut r2 = r.clone();
        r2.status = ReleaseStatus::Published;
        storage.save(&r2).unwrap();

        let content = std::fs::read_to_string(&journal).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_storage_list() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        storage.save(&make_record("v1.0.0", ReleaseStatus::Staged)).unwrap();
        storage.save(&make_record("v2.0.0", ReleaseStatus::Published)).unwrap();
        assert_eq!(storage.list().len(), 2);
    }

    #[test]
    fn test_storage_persists_across_instances() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut storage = FileStorage::new(dir.path());
            storage.save(&make_record("v1.0.0", ReleaseStatus::Staged)).unwrap();
        }
        {
            let storage = FileStorage::new(dir.path());
            assert!(storage.load("v1.0.0").is_some());
        }
    }

    // ---- created_at preserved on replay ----

    #[test]
    fn test_created_at_preserved_across_updates() {
        let dir = tempfile::tempdir().unwrap();
        let first_ts: String;
        {
            let mut storage = FileStorage::new(dir.path());
            let r = make_record("v1.0.0", ReleaseStatus::Staged);
            first_ts = r.created_at.clone();
            storage.save(&r).unwrap();
        }
        {
            let mut storage = FileStorage::new(dir.path());
            let loaded = storage.load("v1.0.0").unwrap();
            assert_eq!(loaded.created_at, first_ts);

            let mut updated = loaded;
            updated.status = ReleaseStatus::Published;
            updated.updated_at = timestamp();
            storage.save(&updated).unwrap();
        }
        {
            let storage = FileStorage::new(dir.path());
            let loaded = storage.load("v1.0.0").unwrap();
            assert_eq!(loaded.created_at, first_ts);
            assert_eq!(loaded.status, ReleaseStatus::Published);
            assert!(loaded.updated_at >= first_ts);
        }
    }

    // ---- TransitionError Display ----

    #[test]
    fn test_transition_error_display() {
        let err = TransitionError::AlreadyPublished("v1.0.0".into());
        assert!(err.to_string().contains("已发布"));

        let err = TransitionError::NotStaged("v1.0.0".into());
        assert!(err.to_string().contains("Staged"));

        let err = TransitionError::NotPublished("v1.0.0".into());
        assert!(err.to_string().contains("Published"));
    }
}
