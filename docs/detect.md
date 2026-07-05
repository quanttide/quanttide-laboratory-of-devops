# detect — 版本号自动检测

读取最新 tag → 扫描提交 → 优先调用 LLM 按 devops-release skill 规则推断版本号，未配置 LLM 时回退到启发式规则。

```bash
cargo run --bin detect -- <repo-path>
```

## 依赖

需要 `quanttide-agent` crate。LLM 通过以下环境变量配置（可选）：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `LLM_API_KEY` | — | DeepSeek API Key（未设置时回退到启发式规则） |
| `LLM_MODEL` | `deepseek-chat` | 模型名 |
| `LLM_BASE_URL` | `https://api.deepseek.com` | API 地址 |

## LLM 决策

LLM 收到：提交记录列表、最新 tag、scope、当前预发布阶段。

LLM 输出结构化 JSON，决定：

| 字段 | 取值 | 含义 |
|------|------|------|
| `action` | `release` / `skip` / `human` | 发版 / 跳过 / 交给人类 |
| `increment` | `minor` / `patch` / `null` | 增量类型 |
| `prerelease` | `alpha` / `beta` / `rc` / `null` | 预发布阶段 |
| `reason` | 文本 | 判断理由 |

LLM 覆盖了硬编码规则无法处理的场景：
- **变更规模判断**：新增命令 vs 新增选项，都能区分为 minor 或 patch
- **预发布起始阶段**：根据功能完成度决定从 alpha/beta/rc 开始
- **阶段晋级**：判断 alpha → beta → rc 的晋级时机
- **紧急 bug 修复**：决定直发 patch 还是走 rc 验证

## 回退规则（LLM 未配置时）

规则基于以下假设（来自 devops-release skill）：

- 有 `feat:` → **minor**，走预发布（`-rc.1`）
  - 假设：新功能需要验证期，不能直接上正式版
- 仅 `fix: / refactor: / test:` → **patch**，直发正式
  - 假设：修复已知问题风险低，已有信心，无需走预发布
- 已在预发布系列 → **同阶段递增序号**
  - 假设：一旦进入预发布周期，继续当前验证阶段，不跳跃
- 仅 chore/docs → **不发版**
  - 假设：非用户可见的逻辑改动不值得发版本号
- breaking change → **拒绝，交给人类**
  - 假设：major 版本决策需要人类判断，AI 不越权

## 实现

核心逻辑在 `detect_version()`：

1. `detect_scope()` — 从 contract.yaml + changed files 推断 scope
2. `collect_tags_with_scope()` — 按 scope 分组，semver 排序
3. 扫描 tag→HEAD 提交，收集提交记录
4. `llm_decide()` — 调用 LLM 决策（回退 `fallback_heuristic()`）
5. `build_version()` — 根据决策构建版本号
