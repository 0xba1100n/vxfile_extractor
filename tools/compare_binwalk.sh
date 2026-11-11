#!/bin/bash
# compare_binwalk.sh — 全量对照：binwalk -Me vs 本工具
# 用法: bash tools/compare_binwalk.sh [样本路径...]；缺省读 work/cmp_list.txt
# 产物只统计不落盘（vx extract --no-write），避免 /tmp 被大样本撑爆
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VX="$ROOT/rust/vxcore/target/release/vx"
OUT="$ROOT/binwalk_compare.csv"

if [ $# -eq 0 ] && [ -f "$ROOT/work/cmp_list.txt" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    line="${line%$'\r'}"
    [ -n "$line" ] && set -- "$@" "$line"
  done < "$ROOT/work/cmp_list.txt"
fi

echo "sample,input_bytes,binwalk_sig,binwalk_files,binwalk_sec,our_files,our_sec,our_layers,our_table,our_symbols" > "$OUT"
num() { case "${1:-}" in ''|*[!0-9]*) echo 0 ;; *) echo "$1" ;; esac; }

for rel in "$@"; do
  src="$ROOT/$rel"
  [ -f "$src" ] || continue
  tmp=$(mktemp -d /tmp/vx1.XXXXXX)
  cd "$tmp"
  case "$src" in
    *.zip) unzip -o -q "$src" >/dev/null 2>&1 ;;
    *.rar|*.7z) 7z x -y -o. "$src" >/dev/null 2>&1 ;;
    *) cp "$src" . ;;
  esac
  inner=$(find . -type f -printf '%s %p\n' | sort -nr | head -1 | cut -d' ' -f2-)
  [ -n "$inner" ] || { echo "skip $rel (no inner)"; cd /; rm -rf "$tmp"; continue; }
  sz=$(stat -c%s "$inner")

  mkdir bw && cd bw && cp "../$inner" .
  t0=$(date +%s.%N)
  timeout 600 binwalk -Me "$(basename "$inner")" > log 2>&1 || true
  t1=$(date +%s.%N)
  sigs=$(num "$(grep -cE '^[0-9]+ +0x[0-9A-Fa-f]+ ' log 2>/dev/null)")
  bwfiles=$(num "$(find extractions -type f 2>/dev/null | wc -l)")
  bwsec=$(awk "BEGIN{printf \"%.1f\", $t1-$t0}")
  cd ..
  rm -rf bw

  t2=$(date +%s.%N)
  "$VX" extract --no-write "$inner" -o /tmp/nowrite > "$tmp/ours.json" 2>/dev/null || true
  t3=$(date +%s.%N)
  osec=$(awk "BEGIN{printf \"%.1f\", $t3-$t2}")
  eval "$(python3 - "$tmp/ours.json" <<'PY'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    t = d.get('table') or {}
    s = d.get('symbol') or {}
    files = len(d.get('layers', [])) + (d.get('carved') or 0) + (d.get('recovered_named') or 0)
    print("ofiles=%d;layers=%d;table=%d;symbols=%d" % (
        files, len(d.get('layers', [])), t.get('entries', 0), s.get('names', 0)))
except Exception:
    print("ofiles=0;layers=0;table=0;symbols=0")
PY
)"
  printf '"%s",%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$rel" "$sz" "$sigs" "$bwfiles" "$bwsec" "$ofiles" "$osec" "$layers" "$table" "$symbols" >> "$OUT"
  cd /; rm -rf "$tmp"
done
echo "written $OUT"