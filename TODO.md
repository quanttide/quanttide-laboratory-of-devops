# TODO

## v0.2.0 — Provider 服务端开发

> 焦点 sprint。完成后 Provider 能扫描 Artifact 三角、发现不一致、按规则自动修复。

### P0 — Provider 基础框架

- [ ] 初始化 Go module 依赖（`ghinstallation`、`go-github`、`gin`/`chi`）
- [ ] 定义核心数据结构：`ArtifactState`、`ScanResult`、`RepairAction`
- [ ] 实现 GitHub API 客户端：使用 GitHub App 安装认证（`ghinstallation`）
- [ ] 实现 scope 解析器：从 scope 列表映射到 GitHub 仓库路径
- [ ] 实现 HTTP 端点：`GET /scan/:scope`、`POST /repair/:scope`
- [ ] 编写单元测试：客户端 mock、scope 解析、端点路由

### P0 — Artifact 三角扫描

- [ ] 实现 Tag 扫描：`GET /repos/{owner}/{repo}/git/refs/tags`，按 scope 前缀过滤
- [ ] 实现 CHANGELOG 扫描：读取远程仓库 CHANGELOG.md，按版本提取条目
- [ ] 实现 Release 扫描：`GET /repos/{owner}/{repo}/releases`，按 tag 名匹配
- [ ] 实现合并器：给定 scope，输出 `{HasTag, HasChangelog, HasRelease}` 三元组
- [ ] 编写测试：模拟 8 种组合状态，验证扫描结果正确

### P0 — 状态判定引擎

- [ ] 实现判定表：8 种状态组合 → 判定（正常/缺 CHANGELOG/缺 Release/只有 tag/未发布）
- [ ] 实现可修复性判断：缺 Release / 缺 CHANGELOG → 可自动修复；缺 tag → 标记搁置
- [ ] 实现聚合统计：输入多 scope 扫描结果，输出统计摘要（正常/异常/搁置数量）
- [ ] 编写测试：覆盖全部 8 种状态，验证判定和修复建议正确

### P0 — 反脆弱修复执行器

- [ ] 缺 CHANGELOG 修复：从 tag~HEAD git log 生成 CHANGELOG 条目标记，PR 到仓库
- [ ] 缺 Release 修复：`POST /repos/{owner}/{repo}/releases`，从已有 CHANGELOG 补
- [ ] 缺 tag 标记搁置：写入搁置队列文件（`shelved.json`），记录 scope + version + 原因
- [ ] 实现修复原子性：每个修复独立事务，失败不影响其他 scope
- [ ] 编写测试：模拟各修复场景，验证修复动作和结果

### P0 — 批量扫描

- [ ] 实现多 scope 并发扫描：goroutine 池，可配置并发数
- [ ] 实现扫描报告输出：JSON 格式，含每个 scope 的状态 + 整体统计
- [ ] 实现超时控制：单个 scope 超时 30s，整体超时可配置
- [ ] 编写测试：模拟 20+ scope 的不同不一致状态，验证扫描耗时在合理范围

### P0 — 集成测试

- [ ] 场景 1：缺 Release → 自动创建 Release
- [ ] 场景 2：缺 CHANGELOG → 从 git log 补写
- [ ] 场景 3：只有 tag（缺 CHANGELOG + Release）→ 修复 CHANGELOG + Release
- [ ] 场景 4：多个 scope 混合状态 → 批量扫描 + 批量修复
- [ ] 场景 5：缺 tag → 标记搁置，不自动修复

---

## 次要（当前 sprint 不优先）

### Changed — CLI 增强

- [ ] `plan audit` 支持 scope：穿透子模块审计规划文件
- [ ] ROADMAP 状态自动同步：TODO [x] → ROADMAP [x]
- [ ] 统一问题收集入口：build/test/release 的发现集中写入规划文件
- [ ] Audit 从"存在检查"升级为"一致性检查"

### Fixed

- [ ] `determine_submodule_status` 长参数重构不完整
