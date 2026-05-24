# ROADMAP — 软件发布生命周期管理

examples/default 实现量潮发布规范的参考示例。每个迭代一个命令，原型验证后并入平台仓库。

## Iter 2：`release status`（当前）

从 journal 查询发布状态。只读，无副作用。

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/release-status.md`
需求复盘：`docs/requirements/release-status.md`

## Iter 3：`plan`

扫描 BUGS / ROADMAP / TODO / STATUS / CHANGELOG 等项目管理文件，生成变更摘要或进度报告。

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/plan.md`
需求复盘：`docs/requirements/plan.md`

## Iter 4：`build`

统一构建入口。运行本地构建，输出结果摘要。

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/build.md`
需求复盘：`docs/requirements/build.md`

## Iter 5：`test`

统一测试入口。运行本地测试套件，生成摘要报告。

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/test.md`
需求复盘：`docs/requirements/test.md`

## 设计风格

- Rust 实现
- 原子操作
- 与 `stage` / `publish` / `cancel` / `retire` 一致的 CLI 接口

## 待定

- `--dry-run` 支持所有命令
- `stage --ratio <0.0-1.0>` 灰度比例参数
