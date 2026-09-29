#!/bin/bash
# macOS / Bash 3.2: live samples only; no files, launches, or preference changes.
set -uo pipefail
export LC_ALL=C

usage() {
    cat <<'HELP'
用法：memory-check.sh [-p PID] [-i 间隔秒数] [-n 采样次数]
默认：查找已打开的 Shuffle Flow，每 5 秒采样一次，共 12 次。
  -p PID   指定进程；也可用 Finder 的 PID 做单独采样
  -i 秒数  采样之间的等待时间（正整数）
  -n 次数  采样次数（正整数），Ctrl+C 可提前结束并汇总
  -h       显示帮助
HELP
}
fail() { printf '%s\n' "$*" >&2; exit 1; }
positive() { [[ "$1" =~ ^[1-9][0-9]{0,8}$ ]]; }

target_pid='' interval=5 limit=12
while getopts ':p:i:n:h' option; do
    case "$option" in
        p) target_pid=$OPTARG ;;
        i) interval=$OPTARG ;;
        n) limit=$OPTARG ;;
        h) usage; exit 0 ;;
        :) fail "选项 -$OPTARG 缺少参数。" ;;
        \?) usage >&2; exit 1 ;;
    esac
done
shift $((OPTIND - 1))
[[ $# == 0 ]] || fail '多余参数；请用 -h 查看帮助。'
[[ $(uname -s) == Darwin ]] || fail '此脚本使用 macOS 的 ps 和 vmmap。'
positive "$interval" && positive "$limit" || fail '间隔与次数必须是正整数。'
if [[ -z "$target_pid" ]]; then
    candidates=$(ps -ax -o pid=,comm= | awk '/\/Shuffle Flow\.app\/Contents\/MacOS\/shuffle$/ { print $1 }')
    [[ -n "$candidates" ]] || fail '未找到 Shuffle Flow。请先打开应用，或用 -p PID 指定进程。'
    [[ "$candidates" != *$'\n'* ]] || fail "找到多个 Shuffle Flow 进程，请用 -p 指定其中一个 PID：$candidates"
    target_pid=$candidates
fi
positive "$target_pid" || fail 'PID 必须是正整数。'
target_command=$(ps -p "$target_pid" -o comm=) || fail "进程 $target_pid 不存在。"
[[ -n "$target_command" ]] || fail "进程 $target_pid 不存在。"

count=0 physical_count=0 rss_sum=0 rss_max=0 physical_sum=0 physical_max=0 cpu_sum=0 cpu_max=0 lifetime_max=0
summary() {
    [[ $count -gt 0 ]] || return 0
    awk -v n="$count" -v pn="$physical_count" -v rs="$rss_sum" -v rm="$rss_max" \
        -v ps="$physical_sum" -v pm="$physical_max" -v cs="$cpu_sum" -v cm="$cpu_max" -v lp="$lifetime_max" 'BEGIN {
        printf "\n汇总（%d 次采样）：\n", n
        printf "RSS：平均 %.1f MiB，采样最大 %.1f MiB\n", rs/n, rm
        if (pn) printf "物理占用：平均 %.1f MiB，采样最大 %.1f MiB（%d 次有效）\n", ps/pn, pm, pn
        else print "物理占用：无法读取 vmmap 数据"
        printf "CPU：平均 %.1f%%，采样最大 %.1f%%\n", cs/n, cm
        if (pn) printf "vmmap 报告的进程历史物理峰值：%.1f MiB\n", lp
    }'
}
trap summary EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf '进程：%s（PID %s）\n' "$target_command" "$target_pid"
printf '间隔 %s 秒，共 %s 次；Ctrl+C 提前结束。\n' "$interval" "$limit"
printf '只统计该进程；RSS 与物理占用口径不同，Quick Look/SSH 等外部进程不计入。\n\n'
printf '%-10s %12s %14s %8s\n' '时间' 'RSS(MiB)' '物理(MiB)' 'CPU(%)'
while [[ $count -lt $limit ]]; do
    current_command=$(ps -p "$target_pid" -o comm=) || { printf '\n进程已退出。\n'; break; }
    [[ "$current_command" == "$target_command" ]] || { printf '\nPID 对应进程已改变，停止采样。\n'; break; }
    sample=$(ps -p "$target_pid" -o rss=,%cpu=) || { printf '\n进程已退出。\n'; break; }
    read -r rss_kib cpu <<< "$sample"
    [[ -n "${rss_kib:-}" && -n "${cpu:-}" ]] || break
    read -r physical lifetime <<< "$(vmmap -summary "$target_pid" 2>/dev/null | awk '
        function mib(value, unit) {
            unit = substr(value, length(value), 1); value += 0
            if (unit == "G") return value * 1024
            if (unit == "M") return value
            if (unit == "K") return value / 1024
            return value / 1048576
        }
        /^Physical footprint:/ { physical = mib($NF); found = 1 }
        /^Physical footprint \(peak\):/ { peak = mib($NF) }
        END { if (found) printf "%.3f %.3f\n", physical, peak; else print "- -" }
    ')"
    read -r rss rss_sum rss_max physical_sum physical_max physical_count cpu_sum cpu_max lifetime_max <<< "$(awk \
        -v r="$rss_kib" -v p="$physical" -v c="$cpu" -v lp="$lifetime" \
        -v rs="$rss_sum" -v rm="$rss_max" -v ps="$physical_sum" -v pm="$physical_max" -v pn="$physical_count" \
        -v cs="$cpu_sum" -v cm="$cpu_max" -v lm="$lifetime_max" 'BEGIN {
            r /= 1024; rs += r; if (r > rm) rm = r
            if (p != "-") { ps += p; pn++; if (p > pm) pm = p; if (lp > lm) lm = lp }
            cs += c; if (c > cm) cm = c
            printf "%.3f %.3f %.3f %.3f %.3f %d %.3f %.3f %.3f\n", r,rs,rm,ps,pm,pn,cs,cm,lm
        }')"
    count=$((count + 1))
    printf '%-10s %12.1f %14s %8.1f\n' "$(date +%H:%M:%S)" "$rss" "$physical" "$cpu"
    [[ $count -ge $limit ]] || sleep "$interval"
done
