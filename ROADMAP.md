# ROADMAP

## [0.3.0] — 进行中

> 焦点：Provider 自治化 — 后台收敛循环 + 因果约束 + 全 scope 治理

### Added — 后台自动收敛

- [x] 后台收敛循环：定时器驱动，固定频率扫描全部 scope
- [ ] scope 发现：从配置文件或 packages/ 目录自动发现 scope 列表（当前硬编码）
- [x] 自动修复管道：扫描 → 判定 → 修复全程自动化，无需人工触发
- [x] 收敛状态报告：每次循环输出正常/已修/搁置/因果断裂四象限
- [x] 可配置频率（`CONVERGE_INTERVAL` 环境变量）

### Added — 因果约束模型

- [x] 版本号一致性校验：Tag/CHANGELOG/Release 三者版本号交叉验证
- [x] 因果链完整性检查：禁止出现 `{HasCL=true, HasTag=false, HasRelease=true}` 等矛盾态
- [x] 事实源层级感知：Tag（不可移动）/ CHANGELOG（规范事实源/PR）/ Release（派生制品/直接 API）
- [x] 矛盾态修复策略：违反因果链的 scope 标记 `causal_break` 人工介入

### Added — 正式版 CHANGELOG 汇总

- [ ] 预发布条目收集：扫描 scope 内所有预发布 CHANGELOG 条目
- [ ] 正式版条目生成：从预发布条目合并为正式版 CHANGELOG
- [ ] CHANGELOG 自动 PR：汇总完成后提起 PR 到目标仓库
- [ ] 集成测试：模拟预发布 → 正式发布的完整 CHANGELOG 流程

### Added — 全 scope 治理

- [x] `GET /scan`（无 scope 参数）：扫描全部 scope，返回聚合报告
- [x] `GET /report`：返回最近一次全扫描的持久化报告
- [x] 全 scope 扫描结果缓存（`lastReport` 内存缓存）

## [0.2.0] — 已发布

> 焦点：Provider 服务端开发 — Artifact 三角扫描 + 反脆弱修复

### Added
- [x] Provider 基础框架：HTTP 服务、GitHub API 客户端、scope 路由
- [x] Artifact 三角扫描：给定 scope + version，查询 Tag/CHANGELOG/Release 状态
- [x] 状态判定引擎：8 种状态组合 → 是否可自动修复
- [x] 反脆弱修复执行器：缺 Release 自动创建、缺 CHANGELOG 从 git log 补写
- [ ] 批量扫描：跨 scope 一致性检查，聚合输出不一致分布（P1，未完成）
- [x] Tag 不可自动修复 → 标记搁置队列
- [ ] 集成测试：模拟 AI 绕过场景，验证扫描→发现→修复→收敛闭环（未完成）

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
