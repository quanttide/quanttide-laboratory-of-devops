#!/bin/bash
# 覆盖率工具性能对比实验
#
# 对比: cargo-llvm-cov / cargo-tarpaulin / grcov / Rust 内置
#
# 用法:
#   bash bench-coverage.sh              # 全部测试
#   bash bench-coverage.sh llvm-cov     # 只测指定工具

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS="$SCRIPT_DIR/bench-results"
mkdir -p "$RESULTS"

# ── 被测工具 ────────────────────────────────────────────────────
TOOLS=()
if [ $# -gt 0 ]; then
    TOOLS=("$@")
else
    TOOLS=("llvm-cov" "tarpaulin" "grcov" "builtin")
fi

echo "=========================================="
echo "覆盖率工具性能对比实验"
echo "项目: $PROJECT_DIR"
echo "结果: $RESULTS"
echo "=========================================="

# ── 工具安装检查 ────────────────────────────────────────────────
install_if_missing() {
    local name="$1"
    local pkg="$2"
    if ! command -v "$name" &>/dev/null || [ "$pkg" = "cargo-llvm-cov" ] && ! cargo llvm-cov --help &>/dev/null; then
        echo "📦 安装 $pkg ..."
        cargo install "$pkg" 2>&1 | tail -1
    else
        echo "✅ $pkg 已安装"
    fi
}

# ── 单次测量 ────────────────────────────────────────────────────
bench() {
    local label="$1"
    local cmd="$2"
    local log="$RESULTS/$label.log"
    local time_log="$RESULTS/$label.time"

    echo ""
    echo "─── $label ───"

    # 清理覆盖率缓存（保留编译缓存）
    rm -f "$PROJECT_DIR/default.profraw" 2>/dev/null
    rm -rf "$PROJECT_DIR/target/coverage" 2>/dev/null
    mkdir -p "$PROJECT_DIR/target/coverage"

    # 耗时测量
    /usr/bin/time -o "$time_log" --format="%E real\n%U user\n%S sys\n%M maxmem(KB)" \
        bash -c "$cmd" &> "$log" || true

    # 提取结果
    local wall=$(head -1 "$time_log" 2>/dev/null || echo "N/A")
    local mem=$(grep "maxmem" "$time_log" 2>/dev/null | grep -oP '\d+' || echo "0")
    local mem_mb=$((mem / 1024))

    # 检查是否成功（排除 test result: 0 failed 行）
    local status="✅"
    if grep -qiE "^(error|killed|segment fault)" "$log" 2>/dev/null; then
        status="❌"
    elif grep -q "test result: FAILED" "$log" 2>/dev/null; then
        status="❌"
    elif grep -q "failed to generate report" "$log" 2>/dev/null; then
        status="❌"
    elif [ "$(tail -1 "$log" 2>/dev/null)" = "" ] && [ ! -f "$PROJECT_DIR/target/coverage/lcov.info" ]; then
        status="⚠"
    fi

    # 检查覆盖率报告
    local cov="N/A"
    if [ -f "$PROJECT_DIR/target/coverage/lcov.info" ]; then
        local lines=$(grep -c "^DA:" "$PROJECT_DIR/target/coverage/lcov.info" 2>/dev/null || echo 0)
        local hits=$(grep "^DA:" "$PROJECT_DIR/target/coverage/lcov.info" | grep -c ",1" 2>/dev/null || echo 0)
        if [ "$lines" -gt 0 ]; then
            cov=$(echo "scale=1; $hits * 100 / $lines" | bc 2>/dev/null || echo "N/A")
        fi
    fi

    printf "  %-15s %8s  %6s  %s\n" "" "$wall" "${cov}%" "$status"
    echo "$label|$wall|$mem_mb|$cov|$status" >> "$RESULTS/summary.csv"
}

# ── 清编译缓存（第一轮用） ──────────────────────────────────────
clean_build() {
    echo ""
    echo "清理编译缓存..."
    # 不删 target/ 全部，只清理覆盖率相关
    rm -rf "$PROJECT_DIR/target/debug" 2>/dev/null
    rm -f "$PROJECT_DIR/default.profraw" 2>/dev/null
    rm -rf "$PROJECT_DIR/target/coverage" 2>/dev/null
    echo "✅ 编译缓存已清理（下次从头编译）"
}

# ── 预热编译 ─────────────────────────────────────────────────────
echo ""
echo "🔄 预热: 先完整编译一次..."
cargo test --no-run 2>&1 | tail -1
echo "✅ 编译完成"

# ── 执行测试 ────────────────────────────────────────────────────
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│  工具              耗时        覆盖率    状态           │"
echo "├─────────────────────────────────────────────────────────┤"

echo "工具|耗时|内存(MB)|覆盖率|状态" > "$RESULTS/summary.csv"

for tool in "${TOOLS[@]}"; do
    case "$tool" in
        "llvm-cov")
            install_if_missing "cargo-llvm-cov" "cargo-llvm-cov"
            bench "llvm-cov_clean" "cargo llvm-cov --lcov --output-path target/coverage/lcov.info -j 2 2>&1"
            bench "llvm-cov_incr"  "cargo llvm-cov --lcov --output-path target/coverage/lcov.info -j 2 2>&1"
            ;;

        "tarpaulin")
            install_if_missing "cargo-tarpaulin" "cargo-tarpaulin"
            bench "tarpaulin_clean" "cargo tarpaulin --out lcov --output-dir target/coverage 2>&1"
            bench "tarpaulin_incr"  "cargo tarpaulin --out lcov --output-dir target/coverage 2>&1"
            ;;

        "grcov")
            echo "📦 安装 grcov ..."
            cargo install grcov 2>&1 | tail -1 || echo "⚠ grcov 安装失败"
            if command -v grcov &>/dev/null; then
                bench "grcov_clean" " \\
                    RUSTFLAGS='-Cinstrument-coverage' cargo test 2>&1 && \\
                    grcov . --binary-path target/debug/deps -s . -t lcov --branch \\
                      --output-path target/coverage/lcov.info 2>&1"
                bench "grcov_incr" " \\
                    RUSTFLAGS='-Cinstrument-coverage' cargo test 2>&1 && \\
                    grcov . --binary-path target/debug/deps -s . -t lcov --branch \\
                      --output-path target/coverage/lcov.info 2>&1"
            else
                echo "⚠ grcov 未安装，跳过"
            fi
            ;;

        "builtin")
            # Rust 内置: 用 llvm-profdata + llvm-cov
            if ! rustup component list 2>/dev/null | grep -q "llvm-tools.*installed"; then
                echo "📦 安装 llvm-tools-preview ..."
                rustup component add llvm-tools-preview 2>&1
            fi
            local toolchain=$(rustup default 2>/dev/null | grep -oP '^[\w-]+' || echo "stable")
            local profdata="$(rustup which llvm-profdata 2>/dev/null || echo "")"
            local llvm_cov="$(rustup which llvm-cov 2>/dev/null || echo "")"
            if [ -z "$profdata" ] || [ -z "$llvm_cov" ]; then
                # 回退: 从 rustup 路径查找
                local sysroot=$(rustup run "$toolchain" rustc --print sysroot 2>/dev/null || echo "")
                if [ -n "$sysroot" ]; then
                    profdata="$sysroot/lib/rustlib/$(rustc -vV | grep host | cut -d' ' -f2)/bin/llvm-profdata"
                    llvm_cov="$sysroot/lib/rustlib/$(rustc -vV | grep host | cut -d' ' -f2)/bin/llvm-cov"
                fi
            fi
            bench "builtin_clean" " \
                RUSTFLAGS='-Cinstrument-coverage' cargo test 2>&1 && \
                \"$profdata\" merge -sparse default.profraw -o default.profdata 2>&1 && \
                \"$llvm_cov\" show target/debug/quanttide_lab \\
                  --instr-profile=default.profdata \\
                  --format=lcov \\
                  --output-dir target/coverage 2>&1 || true"
            bench "builtin_incr" " \
                RUSTFLAGS='-Cinstrument-coverage' cargo test 2>&1 && \
                \"$profdata\" merge -sparse default.profraw -o default.profdata 2>&1 && \
                \"$llvm_cov\" show target/debug/quanttide_lab \\
                  --instr-profile=default.profdata \\
                  --format=lcov \\
                  --output-dir target/coverage 2>&1 || true"
            ;;
    esac
done

echo "└─────────────────────────────────────────────────────────┘"

# ── 报告 ────────────────────────────────────────────────────────
echo ""
echo "=========================================="
echo "📊 结果汇总"
echo "=========================================="
column -t -s'|' "$RESULTS/summary.csv" 2>/dev/null || cat "$RESULTS/summary.csv"

echo ""
echo "详细日志: $RESULTS/*.log"
echo "=========================================="
