#!/bin/bash
# Linux System Status Monitoring Script
# Used as reference for Check Base operation in SSH tab
# Reads from /proc filesystem for portability (no root required)

echo "=========================================="
echo "          LINUX SYSTEM STATUS             "
echo "=========================================="
echo "Date / Time : $(date)"
echo "Hostname    : $(hostname)"
echo "Kernel      : $(uname -r)"
echo "------------------------------------------"

# 1. Uptime from /proc/uptime
if [[ -f /proc/uptime ]]; then
    read -r up_sec idle_sec < /proc/uptime
    up_seconds=${up_sec%.*}
    days=$(( up_seconds / 86400 ))
    hours=$(( (up_seconds % 86400) / 3600 ))
    mins=$(( (up_seconds % 3600) / 60 ))
    echo "Uptime      : ${days}d ${hours}h ${mins}m"
fi

# 2. CPU Information & Overall Usage from /proc/cpuinfo & /proc/stat
if [[ -f /proc/cpuinfo ]]; then
    cpu_model=$(grep -m 1 'model name' /proc/cpuinfo | awk -F: '{print $2}' | xargs)
    cpu_cores=$(grep -c '^processor' /proc/cpuinfo)
    echo "CPU Model   : $cpu_model"
    echo "CPU Cores   : $cpu_cores"
fi

if [[ -f /proc/stat ]]; then
    read -r _ user nice system idle iowait irq softirq steal _ < /proc/stat
    prev_idle=$((idle + iowait))
    prev_total=$((user + nice + system + idle + iowait + irq + softirq + steal))

    sleep 0.2

    read -r _ user nice system idle iowait irq softirq steal _ < /proc/stat
    idle=$((idle + iowait))
    total=$((user + nice + system + idle + iowait + irq + softirq + steal))

    diff_idle=$((idle - prev_idle))
    diff_total=$((total - prev_total))
    cpu_usage=$(( 100 * (diff_total - diff_idle) / diff_total ))
    echo "CPU Usage   : ${cpu_usage}%"
fi

# 3. Load Average from /proc/loadavg
if [[ -f /proc/loadavg ]]; then
    read -r load1 load5 load15 _ < /proc/loadavg
    echo "Load Avg    : $load1 (1m), $load5 (5m), $load15 (15m)"
fi

# 4. Memory Information from /proc/meminfo
if [[ -f /proc/meminfo ]]; then
    mem_total_kb=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
    mem_avail_kb=$(awk '/MemAvailable/ {print $2}' /proc/meminfo)

    mem_total_mb=$((mem_total_kb / 1024))
    mem_avail_mb=$((mem_avail_kb / 1024))
    mem_used_mb=$((mem_total_mb - mem_avail_mb))

    echo "Memory      : Used ${mem_used_mb}MB / Total ${mem_total_mb}MB (Available: ${mem_avail_mb}MB)"
fi

# 5. Disk Usage
echo "------------------------------------------"
echo "Disk Usage:"
df -h --type=ext4 --type=ext3 --type=xfs --type=btrfs 2>/dev/null | awk 'NR>1 {printf "  %-15s Mount: %-5s Used: %s/%s (%s)\n", $1, $6, $3, $2, $5}'

# 6. Network Activity from /proc/net/dev
if [[ -f /proc/net/dev ]]; then
    echo "------------------------------------------"
    echo "Network Interfaces (Bytes Sent/Received):"
    awk 'NR>2 {
        interface=$1; gsub(/:/, "", interface);
        if (interface != "lo") {
            printf "  %-10s RX: %.2f MB | TX: %.2f MB\n", interface, $2/1024/1024, $10/1024/1024
        }
    }' /proc/net/dev
fi

# 7. Top 20 Processes by Memory Usage
echo "------------------------------------------"
echo "Top 20 User Processes (Sorted by Memory Usage):"
for pid in $(ls -d /proc/[0-9]* 2>/dev/null); do
    if [[ -f $pid/stat && -f $pid/cmdline ]]; then
        read -r _ comm _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ rss _ < $pid/stat 2>/dev/null
        cmdline=$(tr '\0' ' ' < $pid/cmdline 2>/dev/null | head -c 60)
        [[ -n "$cmdline" ]] && echo "$comm $((rss * 4 / 1024))MB $cmdline"
    fi
done | sort -k2 -rn | head -20 | awk '{printf "  %-15s %6s  %s\n", $1, $2, substr($0, index($0,$3))}'

echo "=========================================="
