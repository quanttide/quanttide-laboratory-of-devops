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

## 回退规则（LLM 未配置时）

- 有 `feat:` → **minor**，走预发布（`-rc.1`）
- 仅 `fix: / refactor: / test:` → **patch**，直发正式
- 已在预发布系列 → **同阶段递增序号**
- 仅 chore/docs → **不发版**
- breaking change → **拒绝，交给人类**

## 实现

1. `detect_scope()` — 从 contract.yaml + changed files 推断 scope
2. `collect_tags_with_scope()` — 按 scope 分组，semver 排序
3. 扫描 tag→HEAD 提交，收集提交记录
4. `llm_decide()` — 调用 LLM 决策（回退 `fallback_heuristic()`）
5. `build_version()` — 根据决策构建版本号
