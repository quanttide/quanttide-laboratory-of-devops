# ROADMAP

## [0.2.0] — 进行中

> 焦点：Provider 服务端开发 — Artifact 三角扫描 + 反脆弱修复

### Added
- [ ] Provider 基础框架：HTTP 服务、GitHub API 客户端、scope 路由
- [ ] Artifact 三角扫描：给定 scope + version，查询 Tag/CHANGELOG/Release 状态
- [ ] 状态判定引擎：8 种状态组合 → 是否可自动修复
- [ ] 反脆弱修复执行器：缺 Release 自动创建、缺 CHANGELOG 从 git log 补写
- [ ] 批量扫描：跨 scope 一致性检查，聚合输出不一致分布
- [ ] Tag 不可自动修复 → 标记搁置队列
- [ ] 集成测试：模拟 AI 绕过场景，验证扫描→发现→修复→收敛闭环

### Changed
- [ ] `plan audit` 支持 scope：穿透子模块审计规划文件
- [ ] ROADMAP 状态自动同步：TODO [x] → ROADMAP [x]
- [ ] 统一问题收集入口：build/test/release 的发现集中写入规划文件
- [ ] Audit 从"存在检查"升级为"一致性检查"

### Fixed
- [ ] `determine_submodule_status` 长参数重构不完整

## [0.1.1] — 已发布

### Changed
- [x] 重写 lab 文档：基于 intention/insight/report 最新理解串联叙事弧
- [x] 删除过时文档（ROADMAP_test.md、devops-plan-skill.md、provider.md）

### Added
- [x] problem.md：AI 绕过流程的 5 类事件分析
- [x] modeling.md：Artifact 三角状态空间 + 反脆弱修复
- [x] release.md：增量 vs 汇总发布 + 本地/云端分界
- [x] plan.md：计划阶段设计 + 已知缺陷清单
- [x] adr.md：架构决策记录（devops-vs-code 边界、git 库分工）

## [0.1.0] — 已发布

### Added
- [x] 实验室脚手架搭建（pyproject.toml、AGENTS.md）
- [x] 三线结构：Rust CLI + Go Provider + Python 测试（src/cli/、src/provider/、tests/）
- [x] 集成测试目录（tests/test_cli/、tests/test_provider/、tests/test_integration/）
- [x] 文档目录（docs/）：架构、detect、provider 原型、测试
