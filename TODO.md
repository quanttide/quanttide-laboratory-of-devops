# TODO

## Iter 2：`release status`

设计文档：`docs/dev/release-status.md`

- [ ] 从 `release-journal.jsonl` 读取发布记录
- [ ] 输出：当前版本号、最新发布记录、预发布版本列表
- [ ] 每次操作开始和结束时执行，形成操作前后的状态对比
- [ ] 支持 `--json` 输出格式

## Iter 3：`plan`

设计文档：`docs/dev/plan.md`

- [ ] 扫描 BUGS.md / ROADMAP.md / TODO.md 等项目管理文件
- [ ] 输出：BUGS 数量与分布、迭代进度、TODO 完成统计
- [ ] 只读，不修改任何文件

## Iter 4：`build`

设计文档：`docs/dev/build.md`

- [ ] 注册 `build` 子命令到 CLI
- [ ] 检测项目类型，选择对应构建方式
- [ ] 执行构建，捕获输出
- [ ] 输出构建结果摘要（成功/失败、产物路径、耗时）
- [ ] 支持 `--release` 参数

## Iter 5：`test`

设计文档：`docs/dev/test.md`

- [ ] 注册 `test` 子命令到 CLI
- [ ] 检测项目类型，选择对应测试框架
- [ ] 执行测试，解析输出
- [ ] 输出：总数 / 通过 / 失败 / 跳过
- [ ] 列出失败用例（文件 + 行号 + 错误信息）
- [ ] 支持 `--name <pattern>` 过滤
