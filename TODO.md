# TODO

## v0.3.0 — Provider 自治化（当前 sprint）

> 焦点：后台收敛循环 + 因果约束 + 全 scope 治理。
> 使 Provider 从"响应式 API"进化为"自治守护进程"。

### ✅ P0 — 因果约束模型

- [x] 版本号一致性校验：Tag/CHANGELOG/Release 三者版本号交叉验证
- [x] 因果链完整性检查：禁止 `HasCL && !HasTag` 等矛盾态
- [x] 事实源层级感知：Tag（不可移动）/ CHANGELOG（规范事实源）/ Release（派生制品）
- [x] 矛盾态修复策略：违反因果链 → 标记 `causal_break` 人工介入
- [x] 追加单元测试：10 种场景覆盖 + 聚合统计含 CausalBreaks

### ✅ P0 — 后台收敛循环

- [x] `CONVERGE_INTERVAL` 环境变量可配置固定频率
- [x] 全 scope 自动扫描 → 自动修复 → 状态收敛
- [x] 收敛日志：每次循环输出四象限（正常/已修/搁置/因果断裂）
- [x] 优雅退出：信号处理中停止收敛循环

### ✅ P0 — 全 scope 治理

- [x] `GET /scan`（无 scope 参数）：扫描全部 scope，返回聚合报告
- [x] `GET /report`：返回最近一次全扫描的持久化报告
- [x] `lastReport` 缓存，避免重复全量扫描

### P0 — 正式版 CHANGELOG 汇总

- [ ] 预发布条目收集：扫描 scope 内所有预发布 CHANGELOG 条目
- [ ] 正式版条目生成：从预发布条目合并为正式版 CHANGELOG
- [ ] CHANGELOG 自动 PR：汇总完成后提起 PR 到目标仓库

### P0 — 集成测试

- [ ] 场景 1（因果断裂）：版本不匹配 → 标记 causal_break
- [ ] 场景 2（后台收敛）：全 scope 定时扫描 → 自动修复 → 报告验证
- [ ] 场景 3（artifact 不一致）：缺 Release → 自动创建 Release
- [ ] 场景 4（artifact 不一致）：缺 CHANGELOG → 从 git log 补写
- [ ] 场景 5（网络分区）：缺 tag → 标记搁置，不自动修复

## v0.2.0 — Provider 服务端开发（已完成）

> 焦点 sprint。Provider 能扫描 Artifact 三角、发现不一致、按规则自动修复。
>
> 关联模拟场景：artifact 不一致（AGENTS.md）、跨仓库发布、网络分区。

### ✅ P0 — Provider 基础框架

- [x] 添加 Go module 依赖（`ghinstallation`、`go-github`、`chi`）
- [x] 定义核心数据结构：`ArtifactState`、`ScanResult`、`RepairAction`
- [x] 实现 GitHub API 客户端：使用 GitHub App 安装认证（`ghinstallation`）
- [x] 实现 scope 解析器：从 scope 列表映射到 GitHub 仓库路径
- [x] 实现 HTTP 端点：`GET /health`、`GET /scan/:scope`、`POST /repair/:scope`
- [x] 实现结构化日志（`slog`）与错误类型定义
- [x] 实现 graceful shutdown：信号处理 + 健康检查摘流
- [x] 编写单元测试：客户端 mock、scope 解析、端点路由

### ✅ P0 — Artifact 三角扫描

- [x] 实现 Tag 扫描：`GET /repos/{owner}/{repo}/git/refs/tags`，按 scope 前缀过滤
- [x] 实现 CHANGELOG 扫描：读取远程仓库 CHANGELOG.md，按版本提取条目
- [x] 实现 Release 扫描：`GET /repos/{owner}/{repo}/releases`，按 tag 名匹配
- [x] 实现合并器：给定 scope，输出 `{HasTag, HasChangelog, HasRelease}` 三元组
- [x] 编写测试：模拟 8 种组合状态，验证扫描结果正确

### ✅ P0 — 状态判定引擎

- [x] 实现判定表：8 种状态组合 → 判定（正常/缺 CHANGELOG/缺 Release/只有 tag/未发布）
- [x] 实现可修复性判断：缺 Release / 缺 CHANGELOG → 可自动修复；缺 tag → 标记搁置
- [x] 实现聚合统计：输入多 scope 扫描结果，输出统计摘要（正常/异常/搁置数量）
- [x] 编写测试：覆盖全部 8 种状态，验证判定和修复建议正确

### ✅ P0 — 反脆弱修复执行器

- [x] 缺 CHANGELOG 修复：从 tag~HEAD git log 生成 CHANGELOG 条目标记，PR 到仓库
- [x] 缺 Release 修复：`POST /repos/{owner}/{repo}/releases`，从已有 CHANGELOG 补
- [x] 缺 tag 标记搁置：写入搁置队列文件（`shelved.json`），记录 scope + version + 原因
- [x] 实现修复原子性：每个修复独立事务，失败不影响其他 scope
- [x] 编写测试：模拟各修复场景，验证修复动作和结果
