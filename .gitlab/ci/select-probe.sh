#!/usr/bin/env bash
# Picks the debug probe attached to <CHIP>. Probes on the HIL runner share a
# VID:PID, so probe-rs can't disambiguate on its own; a wrong pick just fails
# at the following `probe-rs run --chip` step.
set -euo pipefail

PROBE_RS="${PROBE_RS:-probe-rs}"
chip="$1"

# `unique` = only one board here uses this probe type, so `probe-rs info` (~3s, halts the target) can be skipped.
case "$chip" in
  RP2040) signature='Part: 0x1002'; kind='CMSIS-DAP'; unique=no ;;
  RP235x) signature="$chip"; kind='CMSIS-DAP'; unique=no ;;
  STM32*) signature='STMicroelectronics'; kind='J-Link'; unique=yes ;;
  esp32s3) signature='Tensilica'; kind='EspJtag'; unique=yes ;;
  *) signature="$chip"; kind=''; unique=no ;;
esac

# Lines read "[0]: Debug Probe (CMSIS-DAP) -- 2e8a:000c-0:E6654854571E7F26 (CMSIS-DAP)": selector is $(NF-1), type is $NF.
listing=$("$PROBE_RS" list 2>/dev/null || true)
probes=$(awk -v k="$kind" '/^\[/ { t = $NF; gsub(/[()]/, "", t); if (k == "" || t == k) print $(NF-1) }' <<<"$listing")
# Unknown chip, or a probe type that stopped reporting the expected name: scan everything rather than fail.
[ -n "$probes" ] || probes=$(awk '/^\[/{print $(NF-1)}' <<<"$listing")

if [ "$unique" = yes ] && [ "$(grep -c . <<<"$probes")" -eq 1 ]; then
  printf '%s\n' "$probes"
  exit 0
fi

# probe-rs 0.32 moved the detailed dump we grep behind --verbose; 0.31 rejects the flag.
verbose=()
if "$PROBE_RS" info --help 2>&1 | grep -qF -- --verbose; then
  verbose=(--verbose)
fi

while IFS= read -r probe; do
  [ -n "$probe" ] || continue
  # `|| true`: a failing probe-rs call here (unlike inside the old inline `if`) would otherwise exit the script under -e.
  info=$("$PROBE_RS" info "${verbose[@]}" --probe "$probe" 2>&1 || true)
  if grep -qF "$signature" <<<"$info"; then
    printf '%s\n' "$probe"
    exit 0
  fi
done <<<"$probes"

echo "select-probe: no probe matched chip '$chip'." >&2
exit 1
