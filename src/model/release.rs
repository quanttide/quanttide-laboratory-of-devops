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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAttempt {
    pub id: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub created_at: String,
    pub updated_at: String,
    pub reason: String,
}

impl ReleaseAttempt {
    pub fn new(version: &str, reason: &str) -> Self {
        let now = timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version: version.to_string(),
            status: ReleaseStatus::Staged,
            created_at: now.clone(),
            updated_at: now,
            reason: reason.to_string(),
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
    fn save(&mut self, attempt: &ReleaseAttempt) -> Result<(), Box<dyn std::error::Error>>;
    fn load(&self, version: &str) -> Option<ReleaseAttempt>;
    fn list(&self) -> Vec<ReleaseAttempt>;
}

pub struct FileStorage {
    path: std::path::PathBuf,
    events_path: std::path::PathBuf,
    attempts: Vec<ReleaseAttempt>,
}

impl FileStorage {
    pub fn new(base_path: &Path) -> Self {
        let path = base_path.join(".qtcloud/releases.json");
        let events_path = base_path.join(".qtcloud/release-events.jsonl");
        let attempts = load_attempts(&path);
        Self {
            path,
            events_path,
            attempts,
        }
    }
}

impl Storage for FileStorage {
    fn save(&mut self, attempt: &ReleaseAttempt) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(existing) = self
            .attempts
            .iter_mut()
            .find(|a| a.version == attempt.version)
        {
            *existing = attempt.clone();
        } else {
            self.attempts.push(attempt.clone());
        }

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.attempts)?;
        std::fs::write(&self.path, json)?;

        if let Some(parent) = self.events_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let event = serde_json::to_string(attempt)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        writeln!(f, "{}", event)?;

        Ok(())
    }

    fn load(&self, version: &str) -> Option<ReleaseAttempt> {
        self.attempts
            .iter()
            .find(|a| a.version == version)
            .cloned()
    }

    fn list(&self) -> Vec<ReleaseAttempt> {
        self.attempts.clone()
    }
}

fn load_attempts(path: &Path) -> Vec<ReleaseAttempt> {
    if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // ---- ReleaseAttempt ----

    #[test]
    fn test_release_attempt_new() {
        let a = ReleaseAttempt::new("v1.0.0", "initial release");
        assert_eq!(a.version, "v1.0.0");
        assert_eq!(a.status, ReleaseStatus::Staged);
        assert_eq!(a.reason, "initial release");
        assert!(!a.id.is_empty());
        assert!(!a.created_at.is_empty());
        assert_eq!(a.created_at, a.updated_at);
    }

    #[test]
    fn test_release_attempt_unique_ids() {
        let a = ReleaseAttempt::new("v1.0.0", "");
        let b = ReleaseAttempt::new("v2.0.0", "");
        assert_ne!(a.id, b.id);
    }

    // ---- FileStorage ----

    #[test]
    fn test_storage_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let attempt = ReleaseAttempt::new("v1.0.0", "test");
        storage.save(&attempt).unwrap();
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
        let mut a = ReleaseAttempt::new("v1.0.0", "initial");
        storage.save(&a).unwrap();

        a.status = ReleaseStatus::Published;
        a.reason = "published".into();
        a.updated_at = "999".into();
        storage.save(&a).unwrap();

        let loaded = storage.load("v1.0.0").unwrap();
        assert_eq!(loaded.status, ReleaseStatus::Published);
        assert_eq!(loaded.reason, "published");
    }

    #[test]
    fn test_storage_event_log_appended() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        let a = ReleaseAttempt::new("v1.0.0", "first");
        storage.save(&a).unwrap();

        let events_path = dir.path().join(".qtcloud/release-events.jsonl");
        let content = std::fs::read_to_string(&events_path).unwrap();
        assert!(content.contains("v1.0.0"));

        let mut b = a.clone();
        b.status = ReleaseStatus::Published;
        storage.save(&b).unwrap();

        let content = std::fs::read_to_string(&events_path).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_storage_list() {
        let dir = tempfile::tempdir().unwrap();
        let mut storage = FileStorage::new(dir.path());
        storage.save(&ReleaseAttempt::new("v1.0.0", "")).unwrap();
        storage.save(&ReleaseAttempt::new("v2.0.0", "")).unwrap();
        assert_eq!(storage.list().len(), 2);
    }

    #[test]
    fn test_storage_persists_across_instances() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut storage = FileStorage::new(dir.path());
            storage.save(&ReleaseAttempt::new("v1.0.0", "")).unwrap();
        }
        {
            let storage = FileStorage::new(dir.path());
            assert!(storage.load("v1.0.0").is_some());
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
