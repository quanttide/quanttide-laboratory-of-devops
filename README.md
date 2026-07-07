# 量潮DevOps实验室

`qtcloud-devops` CLI 和 Provider 的原型验证场。

## 结构

```
├── pyproject.toml    # Python — 测试编排
├── tests/            # 集成测试
├── src/              # Go — provider 原型
│   ├── go.mod
│   └── main.go
└── cli/              # Rust — CLI 原型
    ├── Cargo.toml
    └── src/
```

## 快速开始

```bash
# CLI 原型
cd src/cli && cargo run

# Provider 原型
cd src/provider && go run main.go

# 集成测试（使用 uv）
uv run pytest
```
