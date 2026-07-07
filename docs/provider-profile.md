# Provider 系统画像

> 量潮 DevOps Provider — 后台收敛循环系统
> 画像日期：2026-07-07

## 系统概述

Provider 是一个后台守护进程，以固定频率扫描所有 scope 的 Tag/CHANGELOG/Release 一致性，发现断裂后按规则自动收敛。核心思路来自 `intention/index.md`：**从约束行为转向约束结果**。

## 架构

```
main.go       → 入口：启动 HTTP 服务 + 后台收敛循环
handler.go    → HTTP 端点（chi 路由）
scan.go       → Artifact 三角扫描器
judge.go      → 状态判定引擎（13 行判决表 + 聚合统计）
repair.go     → 反脆弱修复执行器
github.go     → GitHub API 客户端（go-github v88）
shelved.go    → 搁置队列（JSON 文件持久化）
scope.go      → Scope 解析器（owner/repo 格式）
types.go      → 核心数据类型
```

## 核心模型：Artifact 三角

```
CHANGELOG ← 规范事实源，PR 提交
Tag       ← 事实源，不可移动
Release   ← 派生制品，API 重建
```

扫描结果判定：

| 状态 | 含义 | 可修复 |
|------|------|--------|
| `normal` | 完整一致 | — |
| `missing_changelog` | 缺 CHANGELOG | ✅ 从 git log 补 |
| `missing_release` | 缺 Release | ✅ gh release create |
| `only_tag` | 只有 tag | ✅ 标记搁置 |
| `causal_break` | 版本不一致 / 矛盾态 | ❌ 人工介入 |
| `unreleased` | 未发布 | — |

## 运行时数据

### 扫描 scope 列表

| Scope | GitHub 仓库 | 状态 |
|-------|------------|------|
| `quanttide/qtcloud-devops/cli` | qtcloud-devops | causal_break |
| `quanttide/quanttide-devops-toolkit` | quanttide-devops-toolkit | causal_break |
| `quanttide/qtcloud-code` | qtcloud-code | missing_changelog |
| `quanttide/qtadmin` | qtadmin | causal_break |
| `quanttide/quanttide-website` | quanttide-website | unreleased (404) |

### 发现的真实问题

| 仓库 | 问题 | 详情 |
|------|------|------|
| qtcloud-devops/cli | 因果断裂 | tag(v0.10.0)=rel(v0.10.0)，CL(v0.1.0) 不匹配 |
| quanttide-devops-toolkit | 因果断裂 | 有 Release(v0.1.0) 但无 tag — 派生制品悬空 |
| qtcloud-code | 缺 CHANGELOG | tag(v0.1.0)=rel(v0.1.0)，CHANGELOG 无 |
| qtadmin | 因果断裂 | tag(v0.1.1)=rel(v0.1.1)，CL(v1.0.0) 不匹配 |

### 已知缺陷

| 缺陷 | 影响 | 状态 |
|------|------|------|
| 补 CHANGELOG 时 PR base=head="main" 导致空 diff | 修复执行器不可用 | 待修 |
| `tags[0]` 取 API 默认第一个而非 semver 最新 | 已修复 |
| scope 无前缀过滤，跨 scope 比较 tag 和 CHANGELOG | 已修复 |
| scope 列表硬编码 | 新增仓库需改代码 | ROADMAP [ ] |
| `shelved.json` 单进程文件锁 | 多实例覆盖 | ponytail: 原型够用 |

## API 参考

| 方法 | 路径 | 用途 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/scan/:owner/:repo` | 扫描单个 scope |
| GET | `/scan` | 扫描全部 scope + 报告 |
| GET | `/report` | 最近一次全扫描报告缓存 |
| POST | `/repair/:owner/:repo` | 修复单个 scope |

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `GITHUB_TOKEN` | — | GitHub PAT（必填） |
| `CONVERGE_INTERVAL` | `5m` | 后台收敛循环间隔 |
