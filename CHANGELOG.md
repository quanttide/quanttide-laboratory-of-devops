# Changelog

## [0.1.0] — 2026-05-24

### 新增

- `code status` — 扫描子模块状态（7 种状态判定 + 三路 commit 比对）
- `code sync` — 子模块指针同步到父仓库
- `code retire` — 子模块自动反注册

### 建模贡献

- 三路 commit 比对模型（parent_pointer / local_head / remote_head）
- 7 种子模块状态分类与优先级排序
- 聚合统计（AggregateStatus）
- Orphaned 孤儿 commit 检测
- 远程不可达时的离线降级策略

### 基础设施

- Rust + git2 + clap 命令行工具
- maturin + pyo3 Python 绑定
- Python Typer CLI 入口
- 集成测试（17 tests）+ 单元测试（51 tests）
