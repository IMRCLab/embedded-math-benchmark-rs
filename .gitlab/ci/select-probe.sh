#!/usr/bin/env bash
# Picks the debug probe attached to <CHIP>. Probes on the HIL runner share a
# VID:PID, so probe-rs can't disambiguate on its own; a wrong pick just fails
# at the following `probe-rs run --chip` step.
set -euo pipefail

PROBE_RS="${PROBE_RS:-probe-rs}"
chip="$1"

# Fixed string identifying the chip in `probe-rs info` output. RP chips print
# their own name there; STM32 doesn't, so match the debug-port designer instead.
case "$chip" in
  STM32*) signature='STMicroelectronics' ;;
  *) signature="$chip" ;;
esac

# "probe-rs list" lines look like:
#   [0]: Debug Probe (CMSIS-DAP) -- 2e8a:000c-0:E6654854571E7F26 (CMSIS-DAP)
# The selector is always the field right before the trailing "(type)".
probes=$("$PROBE_RS" list 2>/dev/null | awk '/^\[/{print $(NF-1)}')

while IFS= read -r probe; do
  [ -n "$probe" ] || continue
  if "$PROBE_RS" info --probe "$probe" 2>&1 | grep -qF "$signature"; then
    printf '%s\n' "$probe"
    exit 0
  fi
done <<< "$probes"

echo "select-probe: no probe matched chip '$chip'." >&2
exit 1
