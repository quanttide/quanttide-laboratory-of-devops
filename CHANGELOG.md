# CHANGELOG

## [0.1.1] - 2026-07-07

### Changed
- 重写 lab 文档：基于 intention/insight/report 最新理解，串联出完整叙事弧
  - 新增 index.md、problem.md、modeling.md、release.md、plan.md、adr.md
  - 更新 architecture.md（新增约束模型）、testing.md（新增绕过场景）
  - 删除过时的 ROADMAP_test.md、devops-plan-skill.md、provider.md

### Added
- problem.md：AI 绕过流程的 5 类事件分析（来自 insight 巡视记录）
- modeling.md：Artifact 三角状态空间 + 反脆弱修复策略 + 状态机收敛模型
- release.md：增量 vs 汇总发布流程设计 + 本地/云端分界线
- plan.md：计划阶段设计 + 已知缺陷清单
- adr.md：架构决策记录（devops-vs-code 边界、git 库分工）

## [0.1.0] - 2026-07-07

### Added
- AGENTS.md：生产映射表（cli→qtcloud-devops, src→provider, packages→toolkit）
- AGENTS.md：双线开发结构（Python 测试编排 + Go provider + Rust CLI）
- provider 原型（Go / src/main.go）
- 集成测试目录（Python / tests/）
- pyproject.toml：补充版本号、描述、构建系统配置

### Changed
- 从单包 Rust 项目重构为三线结构
- Rust 代码迁移至 cli/ 子目录
- 文档迁移至 cli/docs/
