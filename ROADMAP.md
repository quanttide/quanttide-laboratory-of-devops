# 量潮DevOps实验室 — 规划

## 已完成

- [x] `detect` 原型 — 版本号自动检测
- [x] 清理已集成模块（build/code/contract/plan/test）
- [x] docs/detect.md 记录规则与未覆盖项
- [x] **LLM 重构版本判断模块** — 将硬编码规则替换为 LLM 决策，保留基础设施函数，LLM 未配置时回退到启发式规则

## 进行中

- [x] 覆盖率工具性能对比实验 — bench-coverage.sh + bench-coverage.md

## 下一步

- 将 `detect` 集成到 `qtcloud-devops release publish` 的预检流程中
- 根据实验结论替换 CLI 的覆盖率工具
