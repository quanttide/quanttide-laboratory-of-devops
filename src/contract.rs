use std::path::Path;

// ── 核心模型 ──────────────────────────────────────────────────────────

/// 作用域（Scope）定义：单仓多组件中一个可独立发布的组件。
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub dir: String,
    pub language: Language,
    pub framework: String,
    pub build_tool: BuildTool,
    pub registry: Registry,
    pub release: ReleaseConfig,
}

/// 编程语言。
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    Python,
    Go,
    Dart,
    TypeScript,
    Unknown(String),
}

impl Language {
    pub fn name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Dart => "Dart",
            Language::TypeScript => "TypeScript",
            Language::Unknown(s) => s,
        }
    }
}

/// 构建工具。
#[derive(Debug, Clone, PartialEq)]
pub enum BuildTool {
    Cargo,
    Uv,
    Go,
    Flutter,
    Npm,
    Unknown(String),
}

impl BuildTool {
    pub fn name(&self) -> &str {
        match self {
            BuildTool::Cargo => "cargo",
            BuildTool::Uv => "uv",
            BuildTool::Go => "go build",
            BuildTool::Flutter => "flutter build",
            BuildTool::Npm => "npm",
            BuildTool::Unknown(s) => s,
        }
    }
}

/// 制品库（发布目标）。
#[derive(Debug, Clone, PartialEq)]
pub enum Registry {
    Crates,
    PyPI,
    PubDev,
    Npm,
    GitHubReleases,
    Docker,
    None,
}

impl Registry {
    pub fn name(&self) -> &str {
        match self {
            Registry::Crates => "crates.io",
            Registry::PyPI => "PyPI",
            Registry::PubDev => "pub.dev",
            Registry::Npm => "npm",
            Registry::GitHubReleases => "GitHub Releases",
            Registry::Docker => "Docker",
            Registry::None => "无",
        }
    }
}

/// 发布配置。
#[derive(Debug, Clone)]
pub struct ReleaseConfig {
    pub changelog: String,
    pub pre_publish: Vec<String>,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            changelog: "CHANGELOG.md".into(),
            pre_publish: Vec::new(),
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

// ── YAML 解析 ─────────────────────────────────────────────────────────

/// 从 `.quanttide/devops/contract.yaml` 加载作用域列表。
pub fn load_scopes(repo_path: &Path) -> Vec<Scope> {
    let path = repo_path.join(".quanttide/devops/contract.yaml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let parsed: Result<ContractFile, _> = serde_yaml::from_str(&content);
    match parsed {
        Ok(cf) => cf.into_scopes(),
        Err(_) => {
            // 兼容旧格式：scopes 是字符串到字符串的映射
            fallback_parse(&content)
        }
    }
}

fn fallback_parse(content: &str) -> Vec<Scope> {
    let parsed: Result<OldContractFile, _> = serde_yaml::from_str(content);
    match parsed {
        Ok(old) => old
            .scopes
            .into_iter()
            .map(|(name, dir)| {
                let d = dir.clone();
                Scope {
                    name,
                    dir,
                    language: detect_language_by_files(&Path::new(".").join(&d)),
                    framework: String::new(),
                    build_tool: BuildTool::Unknown("auto".into()),
                    registry: Registry::None,
                    release: ReleaseConfig::default(),
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

// ── 语言检测（自动发现 vs 契约声明） ────────────────────────────────

/// 根据声明或自动检测获取语言类型。
/// 优先使用契约中的声明，fallback 到文件检测。
pub fn resolve_language(scope: &Scope, scope_dir: &Path) -> Language {
    match &scope.language {
        Language::Unknown(_) => detect_language_by_files(scope_dir),
        lang => lang.clone(),
    }
}

/// 通过目录下配置文件自动检测语言。
pub fn detect_language_by_files(dir: &Path) -> Language {
    if dir.join("Cargo.toml").exists() {
        Language::Rust
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        Language::Python
    } else if dir.join("go.mod").exists() {
        Language::Go
    } else if dir.join("pubspec.yaml").exists() {
        Language::Dart
    } else if dir.join("package.json").exists() {
        Language::TypeScript
    } else {
        Language::Unknown("无法识别".into())
    }
}

// ── 版本状态 ──────────────────────────────────────────────────────────

pub fn version_status(repo_path: &Path, scope: &Scope) -> VersionStatus {
    let tag_version = get_latest_tag_for_scope(repo_path, &scope.name);
    let scope_dir = repo_path.join(&scope.dir);
    let config_version = read_config_version(&scope_dir, &scope.language);
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
    let scoped = tags.iter().find(|t| t.starts_with(&prefix));
    match scoped {
        Some(t) => Some(normalize_version(t)),
        None => tags.first().map(|t| normalize_version(t)),
    }
}

fn normalize_version(version: &str) -> String {
    let after_scope = version.split('/').last().unwrap_or(version);
    after_scope
        .strip_prefix('v')
        .unwrap_or(after_scope)
        .to_string()
}

fn read_config_version(dir: &Path, lang: &Language) -> Option<String> {
    let filename = match lang {
        Language::Rust => "Cargo.toml",
        Language::Python => "pyproject.toml",
        Language::TypeScript => "package.json",
        _ => return None,
    };
    let path = dir.join(filename);
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("version = \"") {
            if let Some(v) = t.strip_prefix("version = \"") {
                if let Some(end) = v.find('"') {
                    return Some(v[..end].to_string());
                }
            }
        }
        if t.starts_with("\"version\":") {
            if let Some(rest) = t.strip_prefix("\"version\":") {
                let v = rest.trim().trim_matches(',').trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

// ── YAML 数据结构（私有） ─────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct ContractFile {
    scopes: std::collections::BTreeMap<String, ScopeConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct ScopeConfig {
    dir: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    build_tool: Option<String>,
    #[serde(default)]
    registry: Option<String>,
    #[serde(default)]
    release: Option<ReleaseConfigRaw>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseConfigRaw {
    #[serde(default)]
    changelog: Option<String>,
    #[serde(default)]
    pre_publish: Option<Vec<String>>,
}

impl ContractFile {
    fn into_scopes(self) -> Vec<Scope> {
        self.scopes
            .into_iter()
            .map(|(name, cfg)| {
                let lang = match cfg.language.as_deref() {
                    Some("rust") => Language::Rust,
                    Some("python") => Language::Python,
                    Some("go") => Language::Go,
                    Some("dart") => Language::Dart,
                    Some("typescript") | Some("ts") | Some("node") => Language::TypeScript,
                    Some(other) => Language::Unknown(other.into()),
                    None => Language::Unknown("auto".into()),
                };
                let build_tool = match cfg.build_tool.as_deref() {
                    Some("cargo") => BuildTool::Cargo,
                    Some("uv") => BuildTool::Uv,
                    Some("go") => BuildTool::Go,
                    Some("flutter") => BuildTool::Flutter,
                    Some("npm") => BuildTool::Npm,
                    Some(other) => BuildTool::Unknown(other.into()),
                    None => BuildTool::Unknown("auto".into()),
                };
                let registry = match cfg.registry.as_deref() {
                    Some("crates") => Registry::Crates,
                    Some("pypi") => Registry::PyPI,
                    Some("pubdev") => Registry::PubDev,
                    Some("npm") => Registry::Npm,
                    Some("github") | Some("github_releases") => Registry::GitHubReleases,
                    Some("docker") => Registry::Docker,
                    _ => Registry::None,
                };
                let release = cfg
                    .release
                    .map(|r| ReleaseConfig {
                        changelog: r.changelog.unwrap_or_else(|| "CHANGELOG.md".into()),
                        pre_publish: r.pre_publish.unwrap_or_default(),
                    })
                    .unwrap_or_default();
                Scope {
                    name,
                    dir: cfg.dir,
                    language: lang,
                    framework: cfg.framework.unwrap_or_default(),
                    build_tool,
                    registry,
                    release,
                }
            })
            .collect()
    }
}

/// 兼容旧格式：`scopes: { cli: src/cli, studio: src/studio }`
#[derive(Debug, serde::Deserialize)]
struct OldContractFile {
    scopes: std::collections::BTreeMap<String, String>,
}

// ── 简化 API（向后兼容） ──────────────────────────────────────────────

/// 快速检测语言（不依赖契约声明的简化版）。
pub fn detect_language(dir: &Path) -> Language {
    detect_language_by_files(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_rust() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::Rust);
    }

    #[test]
    fn test_detect_language_python() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::Python);
    }

    #[test]
    fn test_detect_language_go() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("go.mod"), "").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::Go);
    }

    #[test]
    fn test_detect_language_dart() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("pubspec.yaml"), "").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::Dart);
    }

    #[test]
    fn test_detect_language_typescript() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_language_by_files(d.path()), Language::TypeScript);
    }

    #[test]
    fn test_detect_language_unknown() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(
            detect_language_by_files(d.path()),
            Language::Unknown(_)
        ));
    }

    #[test]
    fn test_load_scopes_no_file() {
        let d = tempfile::tempdir().unwrap();
        let scopes = load_scopes(d.path());
        assert!(scopes.is_empty());
    }

    #[test]
    fn test_load_scopes_old_format() {
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
        // 旧格式自动检测语言
        assert!(matches!(scopes[0].language, Language::Unknown(_)));
    }

    #[test]
    fn test_load_scopes_new_format() {
        let d = tempfile::tempdir().unwrap();
        let contract_dir = d.path().join(".quanttide/devops");
        std::fs::create_dir_all(&contract_dir).unwrap();
        std::fs::write(
            contract_dir.join("contract.yaml"),
            r#"
scopes:
  cli:
    dir: src/cli
    language: rust
    framework: clap
    build_tool: cargo
    registry: crates
    release:
      changelog: CHANGELOG.md
      pre_publish:
        - scripts/preflight.sh
  studio:
    dir: src/studio
    language: dart
    framework: flutter
    build_tool: flutter
    registry: pubdev
"#,
        )
        .unwrap();
        let scopes = load_scopes(d.path());
        assert_eq!(scopes.len(), 2);

        let cli = &scopes[0];
        assert_eq!(cli.name, "cli");
        assert_eq!(cli.language, Language::Rust);
        assert_eq!(cli.framework, "clap");
        assert_eq!(cli.build_tool, BuildTool::Cargo);
        assert_eq!(cli.registry, Registry::Crates);
        assert_eq!(cli.release.changelog, "CHANGELOG.md");
        assert_eq!(cli.release.pre_publish.len(), 1);

        let studio = &scopes[1];
        assert_eq!(studio.name, "studio");
        assert_eq!(studio.language, Language::Dart);
        assert_eq!(studio.framework, "flutter");
    }

    #[test]
    fn test_resolve_language_declared() {
        let s = Scope {
            name: "cli".into(),
            dir: ".".into(),
            language: Language::Rust,
            framework: String::new(),
            build_tool: BuildTool::Cargo,
            registry: Registry::Crates,
            release: ReleaseConfig::default(),
        };
        assert_eq!(resolve_language(&s, Path::new("/tmp")), Language::Rust);
    }

    #[test]
    fn test_resolve_language_auto() {
        let s = Scope {
            name: "test".into(),
            dir: ".".into(),
            language: Language::Unknown("auto".into()),
            framework: String::new(),
            build_tool: BuildTool::Unknown("auto".into()),
            registry: Registry::None,
            release: ReleaseConfig::default(),
        };
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("go.mod"), "").unwrap();
        assert_eq!(resolve_language(&s, d.path()), Language::Go);
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
    fn test_read_config_version_from_package_json() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(
            d.path().join("package.json"),
            "{\n  \"version\": \"1.2.3\",\n}\n",
        )
        .unwrap();
        let v = read_config_version(d.path(), &Language::TypeScript);
        assert_eq!(v.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn test_normalize_version_v_prefix() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_normalize_version_scoped() {
        assert_eq!(normalize_version("cli/v0.1.0"), "0.1.0");
    }

    #[test]
    fn test_registry_names() {
        assert_eq!(Registry::Crates.name(), "crates.io");
        assert_eq!(Registry::GitHubReleases.name(), "GitHub Releases");
        assert_eq!(Registry::None.name(), "无");
    }
}
