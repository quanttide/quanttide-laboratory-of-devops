# TODO

所有迭代共用要求：
- 覆盖测试 ≥ 95%
- `docs/<command>.md` 用户文档

## Iter 2：`release status`

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/release-status.md`

- [x] 从 `release-journal.jsonl` 读取发布记录
- [x] 输出：当前版本号、最新发布记录、预发布版本列表
- [ ] 每次操作开始和结束时执行，形成操作前后的状态对比（待实现）
- [x] 测试覆盖率 ≥ 95%
- [x] 写用户文档 `docs/release-status.md`

## Iter 3：`plan`

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/plan.md`

- [x] 扫描 BUGS.md / ROADMAP.md / TODO.md / STATUS.md / CHANGELOG.md
- [x] 输出：BUGS 数量与分布、迭代进度、TODO 完成统计
- [x] 只读，不修改任何文件
- [x] 测试覆盖率 ≥ 95%
- [x] 写用户文档 `docs/plan.md`

## Iter 4：`build`

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/build.md`

- [x] 注册 `build` 子命令到 CLI
- [x] 检测项目类型，选择对应构建方式
- [x] 执行构建，捕获输出
- [x] 输出构建结果摘要（成功/失败、产物路径、耗时）
- [x] 支持 `--release` 参数
- [x] 测试覆盖率 ≥ 95%
- [x] 写用户文档 `docs/build.md`

## Iter 5：`test`

设计文档：`../apps/qtcloud-devops/src/cli/docs/dev/test.md`

- [x] 注册 `test` 子命令到 CLI
- [x] 检测项目类型，选择对应测试框架
- [x] 执行测试，解析输出
- [x] 输出：总数 / 通过 / 失败 / 跳过
- [x] 列出失败用例（文件 + 行号 + 错误信息）
- [x] 支持 `--name <pattern>` 过滤
- [x] 测试覆盖率 ≥ 95%
- [x] 写用户文档 `docs/test.md`
