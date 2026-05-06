#!/usr/bin/env bash
# Run the full axhash benchmark suite.
#
# Usage:
#   ./scripts/bench.sh                    # run all, open HTML report
#   ./scripts/bench.sh --save-baseline v1 # save a named baseline
#   ./scripts/bench.sh --baseline v1      # compare against saved baseline
#   ./scripts/bench.sh throughput         # single suite only
#
# Output:
#   target/criterion/          — SVG charts + HTML report per benchmark
#   target/bench-summary.txt   — human-readable ns/op summary
#
set -euo pipefail

SUITES=(throughput latency streaming hashmap concurrent)
EXTRA_ARGS=()
FILTER=""

for arg in "$@"; do
    case "$arg" in
        throughput|latency|streaming|hashmap|concurrent)
            FILTER="$arg"
            ;;
        *)
            EXTRA_ARGS+=("$arg")
            ;;
    esac
done

run_suite() {
    local suite="$1"
    echo "==> Running bench: $suite"
    cargo bench --bench "$suite" -- "${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"}" \
        | tee -a target/bench-summary.txt
}

mkdir -p target
: > target/bench-summary.txt

echo "AxHash benchmark suite — $(date '+%Y-%m-%d %H:%M:%S')" | tee target/bench-summary.txt
echo "Host: $(uname -m) $(uname -s)" | tee -a target/bench-summary.txt
echo "Rustc: $(rustc --version)" | tee -a target/bench-summary.txt
echo "---" | tee -a target/bench-summary.txt

if [[ -n "$FILTER" ]]; then
    run_suite "$FILTER"
else
    for suite in "${SUITES[@]}"; do
        run_suite "$suite"
    done
fi

echo ""
echo "HTML reports: target/criterion/report/index.html"
echo "Summary:      target/bench-summary.txt"

# Open the HTML report if on macOS
if command -v open &>/dev/null && [[ -f target/criterion/report/index.html ]]; then
    open target/criterion/report/index.html
fi
