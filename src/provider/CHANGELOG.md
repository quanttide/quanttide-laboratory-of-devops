# CHANGELOG

## [0.1.0] - 2026-07-07

### Added
- Artifact 三角扫描：Tag/CHANGELOG/Release 一致性检查
- 状态判定引擎：13 行判决表，8 种 artifact 组合状态
- 反脆弱修复执行器：缺 Release 自动创建、缺 tag 标记搁置
- 后台收敛循环：可配置频率（`CONVERVE_INTERVAL`）的自动扫描+修复
- Scope 自动发现：从 tag 提取 scope 名，扫描 `src/`/`packages/`/`apps/` 匹配目录
- pending_release 状态：CHANGELOG 超前 tag 时可自动发版
- HTTP API：`GET /health`、`GET /scan`、`GET /scan/:scope`、`POST /repair/:scope`、`GET /report`
- 搁置队列：不可自动修复的 scope 持久化到 `shelved.json`
