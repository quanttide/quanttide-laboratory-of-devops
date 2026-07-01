use std::path::Path;

/// 作用域（Scope）定义：tag 前缀到目录的映射。
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub dir: String,
}

/// 项目语言类型。
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
    Go,
    Dart,
    Unknown,
}

impl Language {
    pub fn name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Dart => "Dart",
            Language::Unknown => "Unknown",
        }
    }
}

/// 版本一致性状态。
#[derive(Debug)]
pub struct VersionStatus {
    pub tag_version: Option<String>,
    pub config_version: Option<String>,
    pub consistent: bool,
}

/// 从 `.quanttide/devops/contract.yaml` 加载作用域列表。
pub fn load_scopes(repo_path: &Path) -> Vec<Scope> {
    let path = repo_path.join(".quanttide/devops/contract.yaml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let parsed: Result<ContractFile, _> = serde_yaml::from_str(&content);
    match parsed {
        Ok(cf) => cf
            .scopes
            .into_iter()
            .map(|(name, dir)| Scope { name, dir })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// 检测目录下的项目语言类型。
pub fn detect_language(dir: &Path) -> Language {
    if dir.join("Cargo.toml").exists() {
        Language::Rust
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        Language::Python
    } else if dir.join("go.mod").exists() {
        Language::Go
    } else if dir.join("pubspec.yaml").exists() {
        Language::Dart
    } else {
        Language::Unknown
    }
}

/// 获取作用域的最新 git tag 与配置文件版本的比较结果。
pub fn version_status(repo_path: &Path, scope: &Scope) -> VersionStatus {
    let tag_version = get_latest_tag_for_scope(repo_path, &scope.name);
    let scope_dir = repo_path.join(&scope.dir);
    let config_version = read_config_version(&scope_dir, &detect_language(&scope_dir));
    let consistent = match (&tag_version, &config_version) {
        (Some(t), Some(c)) => t == c,
        (None, None) => true,
        _ => false,
    };
    VersionStatus {
        tag_version,
        config_version,
        consistent,
    }
}

/// 获取某个 scope 前缀下的最新 tag。
fn get_latest_tag_for_scope(repo_path: &Path, scope_name: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["tag", "--sort=-version:refname"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let prefix = format!("{}/", scope_name);
    let tags: Vec<&str> = std::str::from_utf8(&output.stdout)
        .ok()?
        .lines()
        .filter(|t| t.starts_with(&prefix) || !t.contains('/'))
        .collect();
    // 优先匹配 scope 前缀的 tag，否则取第一个非 scope 的 tag
    let scoped = tags.iter().find(|t| t.starts_with(&prefix));
    match scoped {
        Some(t) => Some(normalize_version(t)),
        None => tags.first().map(|t| normalize_version(t)),
    }
}

fn normalize_version(version: &str) -> String {
    // 先去掉 scope 前缀（如 cli/），再去掉 v 前缀
    let after_scope = version.split('/').last().unwrap_or(version);
    after_scope
        .strip_prefix('v')
        .unwrap_or(after_scope)
        .to_string()
}

/// 从配置文件中读取版本号。
fn read_config_version(dir: &Path, lang: &Language) -> Option<String> {
    match lang {
        Language::Rust => {
            let path = dir.join("Cargo.toml");
            let content = std::fs::read_to_string(path).ok()?;
            for line in content.lines() {
                let t = line.trim();
                if let Some(v) = t.strip_prefix("version = \"") {
                    if let Some(end) = v.find('"') {
                        return Some(v[..end].to_string());
                    }
                }
            }
            None
        }
        Language::Python => {
            let path = dir.join("pyproject.toml");
            let content = std::fs::read_to_string(path).ok()?;
            for line in content.lines() {
                let t = line.trim();
                if let Some(v) = t.strip_prefix("version = \"") {
                    if let Some(end) = v.find('"') {
                        return Some(v[..end].to_string());
                    }
                }
            }
            None
        }
        Language::Go => None,   // go.mod 无标准版本字段
        Language::Dart => None, // pubspec.yaml 待实现
        Language::Unknown => None,
    }
}

#[derive(Debug, serde::Deserialize)]
struct ContractFile {
    scopes: std::collections::BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_rust() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Rust);
    }

    #[test]
    fn test_detect_language_python() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Python);
    }

    #[test]
    fn test_detect_language_go() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("go.mod"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Go);
    }

    #[test]
    fn test_detect_language_dart() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
        assert_eq!(detect_language(d.path()), Language::Dart);
    }

    #[test]
    fn test_detect_language_unknown() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(detect_language(d.path()), Language::Unknown);
    }

    #[test]
    fn test_load_scopes_no_file() {
        let d = tempfile::tempdir().unwrap();
        let scopes = load_scopes(d.path());
        assert!(scopes.is_empty());
    }

    #[test]
    fn test_load_scopes_with_file() {
        let d = tempfile::tempdir().unwrap();
        let contract_dir = d.path().join(".quanttide/devops");
        std::fs::create_dir_all(&contract_dir).unwrap();
        std::fs::write(
            contract_dir.join("contract.yaml"),
            "scopes:\n  cli: src/cli\n  studio: src/studio\n",
        )
        .unwrap();
        let scopes = load_scopes(d.path());
        assert_eq!(scopes.len(), 2);
        assert_eq!(scopes[0].name, "cli");
        assert_eq!(scopes[0].dir, "src/cli");
    }

    #[test]
    fn test_read_config_version_from_cargo() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname = \"foo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let v = read_config_version(d.path(), &Language::Rust);
        assert_eq!(v.as_deref(), Some("0.1.0"));
    }

    #[test]
    fn test_normalize_version_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_normalize_version_scoped() {
        assert_eq!(normalize_version("cli/v0.1.0"), "0.1.0");
    }
}
