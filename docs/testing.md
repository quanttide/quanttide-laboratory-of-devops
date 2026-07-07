# 集成测试

使用 Python + pytest 编排，覆盖 CLI 和 Provider 的交互场景。

## 快速开始

```bash
uv run pytest
```

## 测试目录结构

```
tests/
├── conftest.py       # 共享 fixture
├── test_cli/         # CLI 相关测试
├── test_provider/    # Provider 相关测试
└── test_integration/ # CLI + Provider 联合测试
```

## 测试场景

### artifact 不一致

模拟绕过 CLI 直接操作 GitHub API 造成的不一致，验证 provider 能否发现并修复：

```python
def test_missing_release_can_be_repaired():
    """给定 tag + CHANGELOG，缺 Release，验证 provider 能自动补全"""
```

### 网络分区

模拟发布中网络中断，部分 artifact 已创建、部分未创建：

```python
def test_partial_publish_on_network_failure():
    """模拟 tag 已推送但 Release 未创建，验证收敛结果"""
```

### 批量扫描

模拟 20+ scope 的 artifact 一致性扫描性能：

```python
def test_bulk_scan_performance():
    """创建 20 个 scope 的不同不一致状态，验证扫描耗时"""
```
