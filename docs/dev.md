# Git Submodule 专用编辑器 — 开发蓝图

## 1. 核心模型

### 实体定义

```rust
// 一个 Submodule 依赖项
struct Submodule {
    name: String,              // 显示名称
    path: PathBuf,             // 在父仓库中的路径
    url: String,               // 远程仓库地址
    tracked_branch: String,    // 跟踪分支，如 "main"

    // 三个关键 commit
    parent_pointer: CommitHash,  // 父仓库记录的期望 commit
    local_head: CommitHash,      // 当前检出的实际 commit
    remote_head: CommitHash,     // 远程跟踪分支的最新 commit

    status: SubmoduleStatus,
}

enum SubmoduleStatus {
    Clean,                   // 三方一致
    AheadOfParent,           // 本地有新提交，但父仓库未记录
    BehindRemote,            // 远程有更新，本地落后
    Detached,                // 处于游离 HEAD
    Dirty,                   // 有未提交的修改
    Orphaned,                // 父仓库记录的 commit 已不存在于远程
    Uninitialized,           // 尚未 init/update
}
```

### 父仓库聚合状态

```rust
struct RepoState {
    root_path: PathBuf,
    submodules: Vec<Submodule>,
    // 统计
    total: usize,
    clean_count: usize,
    needs_attention: Vec<String>, // 需要处理的 submodule 名
}
```

## 2. 原子事务操作（命令集）

每个操作都是不可分割的事务，对应 UI 中的一个按钮。

```rust
trait SubmoduleEditor {
    // 添加
    fn add_submodule(url: &str, path: &str, branch: &str) -> Result<()>;

    // 初始化/更新
    fn init_all() -> Result<()>;
    fn update_single(name: &str, strategy: UpdateStrategy) -> Result<()>;
    fn update_all() -> Result<()>;

    // 同步（子仓库有修改后，更新父仓库指针）
    fn sync_to_parent(name: &str) -> Result<()>;
    fn sync_all_to_parent() -> Result<()>;

    // 分支批量操作
    fn checkout_branch(name: &str, branch: &str) -> Result<()>;
    fn create_branch(name: &str, branch: &str) -> Result<()>;

    // 退役（软删除）
    fn retire_submodule(name: &str) -> Result<()>;

    // 健康检查
    fn health_check() -> Vec<HealthIssue>;
}
```

### 更新策略枚举

- FastForward（仅快进）
- Rebase（变基到跟踪分支）
- Merge（合并跟踪分支）

## 3. 界面布局蓝图

```
┌──────────────────────────────────────────────────┐
│  Repo: /home/user/my-project    [健康检查] [刷新] │
├────────────┬─────────────────────────────────────┤
│ 侧边栏     │                                     │
│            │  子模块列表 (表格)                   │
│ 依赖视图   │  ┌────────┬──────┬──────┬──────┐   │
│            │  │ 名称   │状态  │ 分支  │ 操作  │   │
│            │  ├────────┼──────┼──────┼──────┤   │
│            │  │ lib-1  │ 落后 │ main │[更新]│   │
│            │  │ lib-2  │ 游离 │ none │[修复]│   │
│            │  │ lib-3  │ 脏   │ dev  │[提交]│   │
│            │  └────────┴──────┴──────┴──────┘   │
│            │                                     │
│            │  批量操作: [全部更新] [全部同步]     │
│            │                                     │
│            ├─────────────────────────────────────┤
│            │  详情面板:                          │
│            │  选中: lib-1                        │
│            │  父仓库指针: abc1234                │
│            │  本地 HEAD: abc1234                 │
│            │  远程 HEAD: def5678 (+2 commits)    │
│            │  建议操作: 快进合并                 │
│            │                                     │
│            │  操作历史:                          │
│            │  · 2024-01-01 更新到 abc1234       │
│            │  · 2023-12-01 退役 lib-old          │
└────────────┴─────────────────────────────────────┘
```

### 关键视觉规则

- 状态列用颜色图标：绿色=干净，黄色=需要关注，红色=异常
- 点击任何操作按钮立即执行底层 Git 命令，无额外确认弹窗，但支持 Ctrl+Z 撤销（通过 reflog 恢复）
- 退役的子模块默认折叠到"已退役"分组

## 4. 状态机（每个 submodule 的 UI 引导）

```
                  [添加]
                    ↓
              Uninitialized ──→ Clean
                    ↑              ↓
                    │         (子仓库有新提交)
                    │              ↓
                    │       AheadOfParent
                    │              ↓
                    │        [同步到父仓库]
                    │              ↓
                    │           Clean
                    │
                 (父仓库记录被重置等)
                    ↓
              Detached 或 Dirty
                    ↓
              [修复/提交]
                    ↓
                  Clean

远程更新时:
Clean → BehindRemote → [更新] → Clean
```

所有状态最终都可收敛到 Clean，无法收敛的问题（如孤儿 commit）会标记为 Orphaned，需手动干预。

## 5. 技术选型

| 层级 | 技术 | 原因 |
|------|------|------|
| 核心逻辑 | Rust | 安全、高性能，直接调用 libgit2 或 git2 crate，完全避免 shell 命令字符串拼接 |
| UI | Tauri (Rust + Web 前端) | 轻量，跨平台，前端可用 React/Vue 快速构建 |
| Git 操作 | git2 (libgit2 绑定) | 比调用 CLI 稳定，可精确控制事务，避免用户机器上 Git 版本差异 |
| 数据持久化 | SQLite | 存储操作历史、退役记录、用户偏好 |

## 6. 开发路径（可立即执行）

1. **核心库**：在 Rust 中实现 `SubmoduleEditor` trait，完成所有原子操作，并编写单元测试。
2. **CLI 原型**：先做成命令行工具，验证逻辑正确性。
3. **Tauri 壳**：用 Tauri 封装，前端调用后端 Rust 函数。
4. **UI 实现**：根据布局蓝图构建前端，状态驱动渲染。
5. **打包分发**：用 Tauri 构建跨平台安装包。

---

此蓝图已去掉所有修饰，只保留可直接转化为代码的结构、类型、接口和布局。任何有经验的开发者拿到手即可开工。
