# ROADMAP

## [0.2.0] — 进行中

### Added
- [ ] `plan audit` 支持 scope：穿透子模块审计规划文件
- [ ] ROADMAP 状态自动同步：TODO [x] → ROADMAP [x]
- [ ] 统一问题收集入口：build/test/release 的发现集中写入规划文件
- [ ] 正式版 CHANGELOG 汇总模式：从预发布 CHANGELOG 条目合并而非 git log 增量
- [ ] Audit 从"存在检查"升级为"一致性检查"：Tag/CHANGELOG/Release 三角关系扫描
- [ ] Provider 原型：artifact 扫描 + 反脆弱修复

### Fixed
- [ ] `determine_submodule_status` 长参数重构不完整（需重做）

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
