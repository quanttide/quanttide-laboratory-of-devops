# 量潮DevOps实验室 — 规划

## 已完成

- [x] `detect` 原型 — 版本号自动检测
- [x] 清理已集成模块（build/code/contract/plan/test）
- [x] docs/detect.md 记录规则与未覆盖项

## 下一步：LLM 重构版本判断模块

当前 `detect` 的版本判断逻辑是硬编码的二值规则（有 feat → minor，否则 patch），无法处理以下场景：

- **变更规模判断**：新增一个命令 vs 新增一个选项，都算 feat，但一个该 minor 一个该 patch
- **预发布起始阶段**：无法判断该从 alpha、beta 还是 rc 开始
- **阶段晋级**：无法判断 alpha 什么时候该晋级 beta
- **紧急 bug 修复**：无法判断该直发 patch 还是走 rc 验证

方案：引入 LLM（通过 `quanttide-agent`）替换硬编码的版本推断逻辑，让 LLM 根据提交记录和仓库上下文做出上述判断。

### 具体步骤

1. 将 `src/bin/detect.rs` 中的 `detect_version()` 逻辑（tag 读取、提交扫描、scope 检测）保留为基础设施函数
2. 将版本增量决策（minor/patch/预发布阶段）提取为 LLM prompt
3. 输入：提交记录、最新 tag、scope 信息、当前预发布阶段
4. 输出：minor / patch / alpha / beta / rc / 不发版 / 交给人类
5. 更新文档和测试
