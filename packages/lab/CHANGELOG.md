# CHANGELOG

## [0.2.0] - 2026-07-04

### Added
- 新增 CLI monorepo 验证的 scope 结构，并扩展 ci_workflow 字段
- 实现 `plan clean`、`plan doctor`、`plan status` 等实验原型命令
- 新增四维契约架构（Stages/Platforms/Sources/Scopes）及 framework/registry/release 配置
- 实现 validate、preflight、code、test、build status 等核心模块
- 新增 release 命令、PyO3 绑定、操作历史持久化（SQLite）及 Tauri 集成

### Changed
- 重构 contract 模块为四维架构，重命名枚举变体以匹配文件/工具
- 将 BuildTool::Pip 回退为 Uv，并更新 check_syntax 支持多语言语法校验
- 将 check_ci 改用 `gh run list` 实时查询 CI 状态，并关联 scope workflow
- 为所有模块补充设计文档和实现全景文档
- 采纳评审建议，改进解析警告与覆盖语义

### Fixed
- 将 doctor 命令调整为只读验证模式
- 修复 git2 API 兼容性及 add_submodule 集成测试
- 修复包名依赖及编译错误（fmt/clippy）
- 更新 export 测试断言以匹配重命名

### Removed
- 删除旧格式解析代码和 validate 模块
- 移除 history、export-ci 等冗余命令
- 清理 Tauri、web-ui、GitHub Actions 等遗留组件
- 删除过时的 dev.md、user-guide 及需求文档目录
- 移除已完成迭代的 ROADMAP 和 TODO 条目

## [0.1.0] - 2026-07-04

### Added
- 实验室初始化
