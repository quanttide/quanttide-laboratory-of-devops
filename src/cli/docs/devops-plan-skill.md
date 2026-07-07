# devops-plan SKILL 设计

> 实验：基于 `qtcloud-devops plan` 设计 ROADMAP 规划管理 SKILL。
> 验证方式：在实验室目录中执行 plan 命令，确认输出符合预期。

## SKILL 设计

### 定位

`devops-plan` SKILL 覆盖 ROADMAP.md 的**查看进度 → 清理已完成 → 修复格式** 全流程。
对应 CLI 命令 `qtcloud-devops plan` 的三个子命令。

### ROADMAP.md 格式约定

```
# ROADMAP

## [0.2.0] — 进行中

### Added
- [x] 已完成功能 A
- [ ] 待办功能 B
```

规则：
- **首行**必须为 `# ROADMAP`
- **版本头**格式：`## [X.Y.Z]` 或 `## [X.Y.Z] — 状态`
- **分类**标准大小写：`### Added` / `### Changed` / `### Fixed` / `### Removed` / `### Deprecated` / `### Security`
- **条目**：`- [x]` 已完成 / `- [ ]` 待办

### 命令参考

| 命令 | 用途 | 典型输出 |
|------|------|---------|
| `plan status [scope]` | 查看各版本完成进度 | 版本号 + 完成数/总数 + 百分比 |
| `plan clean [scope]` | 删除已完成条目 | 提示 + 级联清理空分类/空版本 |
| `plan doctor [scope]` | 修复格式问题 | LLM 修复（可选）+ 规则修复（v 前缀、大小写、checkbox） |

### scope 解析规则

1. 显式指定：`plan status cli` → `cli/ROADMAP.md`
2. 自动检测：省略 scope → 按当前工作目录匹配 contract scope
3. 回退：无匹配 → 仓库根 `ROADMAP.md`

### 与 toolkit 的关系

| 功能 | CLI `plan.rs` | toolkit `source::roadmap` |
|------|---------------|--------------------------|
| 解析 ROADMAP | `parse_roadmap()` — 简单行解析 | `Roadmap::from_str()` — 结构化解析（版本/分类/条目） |
| 进度统计 | `VersionProgress { done, total }` | `RoadmapVersion.percent()` + `RoadmapProgress` |
| 格式验证 | `doctor_roadmap()` + `apply_rule_fixes()` | `Roadmap.validate()` |
| 格式修复 | `plan clean` + `plan doctor` | 无（SKILL 空间） |

**结论：** `plan status` 可委托给 toolkit 的 `Roadmap` 解析；`plan clean/doctor` 的修复逻辑在 CLI 层，toolkit 只做校验。

## 验证结果

### 验证条件

- CLI 版本：qtcloud-devops 0.10.0-alpha.2
- 测试目录：`examples/default/`
- 测试文件：标准格式 `ROADMAP.md` + 非标准格式注入

### 验证 1：plan status — 查看进度

```bash
$ qtcloud-devops plan status
[(auto)] 规划进度
  ----------------------------------------
  [0.2.0   ]  4/ 6 完成 (67%)
  [0.1.0   ]  3/ 3 完成 (100%)
  ----------------------------------------
  总计:  7/9 完成 (78%)
```

✅ 各版本进度正确，百分比计算准确。

### 验证 2：plan clean — 清理已完成

输入：标准 ROADMAP（2 个版本，7/9 完成 → 2 个 `- [ ]` 待办）

```bash
$ qtcloud-devops plan clean
✓ 已清理 265 字节
```

输出：
- ✅ `- [x]` 行全部删除
- ✅ 空 `### Added`、`### Fixed` 分类级联删除
- ✅ 空 `## [0.1.0]` 版本级联删除
- ✅ `### Changed` 中 2 个待办保留
- ✅ 文件已 git commit

### 验证 3：plan doctor — 修复格式

输入（非标准格式）：
```
## [v0.2.0] — 进行中
### added
### changed
```

输出：
```
## [0.2.0] — 进行中
### Added
### Changed
```

✅ `v` 前缀移除
✅ `### added` → `### Added` 大小写修正
✅ `### changed` → `### Changed` 大小写修正
✅ LLM 自动完成修复，规则层做二次验证

### 验证 4：非标准格式自动转换

输入（实验室原始 ROADMAP，无版本头）：
```
# 量潮DevOps实验室 — 规划
## 已完成
- [x] detect 原型
```

```bash
$ qtcloud-devops plan status
🔄 检测到非标准格式，调用 LLM 转换...
📋 LLM 格式修复已应用
[0.1.0]  5/7 完成 (71%)
```

✅ LLM 能识别非标准格式并转换为标准版本头

## 结论

| 功能 | 结果 | 说明 |
|------|------|------|
| `plan status` | ✅ | 进度统计准确，支持级联百分比 |
| `plan clean` | ✅ | 级联清理干净，保留待办 |
| `plan doctor` | ✅ | v 前缀、大小写、LLM 自动修复 |
| LLM 自动转换 | ✅ | 非标准格式也能处理 |
| scope 解析 | ✅ | 自动检测 + 回退到 repo 根 |

**devops-plan SKILL 设计验证通过。** 可以正式创建为 `.agents/skills/devops-plan/SKILL.md`。
