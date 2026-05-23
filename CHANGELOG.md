# Changelog

## [1.0.0] — 2026-05-23

### 新增
- `kse health-check` — 扫描子模块状态（7 种状态判定）
- `kse add` — 添加子模块
- `kse init` — 初始化未初始化的子模块
- `kse update` / `kse update-all` — 更新子模块（支持 FastForward / Rebase / Merge）
- `kse sync` / `kse sync-all` — 同步子模块指针到父仓库
- `kse checkout` / `kse branch` — 切换/创建子模块分支
- `kse retire` — 退役子模块
- `kse history` — 查看操作历史（SQLite 持久化）
- `kse export-ci` — 导出 CI 脚本（shell / GitHub Actions / GitLab CI）
- `--dry-run` 全局预览模式
- Tauri 桌面应用：Web UI 仪表盘（子模块列表 + 详情面板 + 操作历史）
- 健康问题检测与建议引导

### 基础设施
- Rust + git2 + clap 命令行工具
- Tauri v1 跨平台桌面壳
- GitHub Actions CI（cargo check + test + clippy）
- SQLite 操作历史数据库（`.git/kse/history.db`）
- 模型层单元测试覆盖状态优先级与 CommitHash
