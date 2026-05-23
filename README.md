# quanttide-example-of-devops — KSE (Git Submodule Editor)

量潮DevOps实验室 — 多仓库项目的子模块可视化工具。

## 安装

### 前置依赖

```bash
# Ubuntu/Debian
sudo apt-get install libgit2-dev

# macOS
brew install libgit2
```

### 从源码构建

```bash
git clone <repo-url>
cd examples/default
cargo build --release
# 二进制位于 target/release/kse
```

### Tauri 桌面应用

```bash
cargo install tauri-cli
cargo tauri dev     # 开发模式
cargo tauri build   # 构建安装包
```

## 使用

### 健康检查

```bash
kse health-check [路径]
# 默认当前目录，输出每个子模块的名称、状态和跟踪分支
```

### 子模块管理

```bash
kse add <url> <path> -b main                     # 添加子模块
kse init                                          # 初始化所有
kse update <name> -s fast-forward                 # 更新单个
kse update-all -s merge                           # 批量更新
kse sync <name>                                   # 同步到父仓库
kse sync-all                                      # 全部同步
kse checkout <name> <branch>                      # 切换分支
kse branch <name> <new-branch>                    # 创建分支
kse retire <name>                                 # 退役子模块
```

### 预览模式（不执行）

```bash
kse --dry-run update <name>
kse --dry-run sync-all
```

### 操作历史

```bash
kse history                          # 最近 20 条
kse history -n 50                    # 最近 50 条
kse history -m <submodule-name>      # 按子模块筛选
```

### 导出 CI 脚本

```bash
kse export-ci                       # 输出 shell 脚本
kse export-ci -f github             # GitHub Actions
kse export-ci -f gitlab             # GitLab CI
kse export-ci -f shell -o script.sh # 写入文件
```

## 开发

```bash
cargo build                    # 编译
cargo test                     # 运行测试
cargo clippy -- -D warnings    # 代码检查
cargo fmt                      # 格式化
```

## 架构

```
src/
├── lib.rs              # 共享库入口
├── main.rs             # CLI 二进制
├── model/
│   └── mod.rs          # Submodule, SubmoduleStatus, RepoState
└── commands/
    ├── mod.rs          # SubmoduleEditor trait, UpdateStrategy
    ├── editor.rs       # GitSubmoduleEditor 实现
    ├── history.rs      # SQLite 操作历史
    └── export.rs       # CI 脚本导出
src-tauri/              # Tauri 桌面壳
web-ui/                 # 前端仪表盘
```

## 许可证

MIT
