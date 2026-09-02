# RP2350: 0.32.0 fails to flash on almost every attempt (reset leaves the debug port unresponsive)

### Describe the bug

Flashing an RP2350 / Pico 2 over a Raspberry Pi Debug Probe (CMSIS-DAP v2) works on 0.30.0
and 0.31.0 and almost always fails on 0.32.0. The flash loader erases a few sectors, then
the access port stops responding and the rest of the session fails.

Same probe, cable, board and ELF throughout, only the probe-rs binary swapped (official
release builds):

- 0.30.0: 3/3 flashes OK
- 0.31.0: 27/27 OK (23 of them alternated one-for-one with 0.32.0)
- 0.32.0, alternated with 0.31.0: 0/22
- 0.32.0, run alone back-to-back: 3/20 (first success on the 4th attempt)

0.31.0 recovers the board on the first try every time, including immediately after a 0.32.0
failure, so this is not a hardware fault. On 0.32.0 a flash does usually get through after
several retries.

Likely cause: c2438731 (#3898) is the only change to the RP2350 reset path between the two
releases (last commit to touch `rp235x.rs`; the `RP235x.yaml` flash algorithm blob is
byte-identical). It removed the retry loop in `Rp235x::reset_system`, whose comment
describes this exact failure:

> Reset seems to get stuck in a state where RAM is inaccessible. A second reset fixes this
> about 20% of the time.

0.32.0 now runs the sequence once and returns the first error.

Suggested fix: reinstate a bounded retry around `Rp235x::reset_system`.

### To Reproduce

Run `probe-rs run --chip RP235x --probe <selector> --binary-format elf fw.elf` against an
RP2350 a handful of times. On 0.32.0 most attempts fail; on 0.31.0 all succeed.

### Expected behavior

First-try flash success, as on 0.31.0.

### Stacktrace

Failing sector address varies between runs (0x10016000, 0x10017000, 0x1003e000, ...):

```
Error: An error with the flashing procedure has occurred.

Caused by:
    0: Failed to erase flash sector at address 0x10016000.
    1: Failed to read the core status.
    2: An ARM specific error occurred.
    3: Error using access port FullyQualifiedApAddress { dp: Default, ap: V2(ApV2Address(Some(8192))) }.
    4: Failed to read register DRW at address 0xd0c
    5: An error occurred in the communication with an access port or debug port.
    6: Target device did not respond to request.
```

### Operating System

Linux (Ubuntu)

### Additional context

RP2350A, Raspberry Pi Debug Probe (`2e8a:000c`), Ubuntu 22.04. Tested with the 0.30.0 /
0.31.0 / 0.32.0 release binaries; c2438731 was identified from the source history, not
bisected with a custom build. Possibly related: #3563, #3887, discussion #1424.
