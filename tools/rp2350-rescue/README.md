# rp2350-rescue

Brings an RP2350 back to its Cortex-M33 core after it has booted the Hazard3 RISC-V core.

## Why it is needed

RP2350 has two cores sharing one flash. Which one runs is decided at every power-on by the
`IMAGE_DEF` block the boot ROM scans for in flash. Once Hazard3 is live the Cortex-M33 AP powers
down, and neither a reset nor a power cycle brings it back, because the ROM re-reads the same
`IMAGE_DEF` every time. This tool pokes RP2350's always-on Rescue DP (`RESCUE_RESTART`, CTRL bit
31 on AP `0x80000`) through `probe-rs`'s raw DAP API. That is the same mechanism `probe-rs`'s own
`Rp235x::reset_system` and Raspberry Pi's `rp2350-rescue.cfg` use, reached directly here because
their CLI only exposes it from an already-working ARM session.

## Getting back to ARM

```bash
cargo run -- --probe <selector>
probe-rs reset --chip RP235x
probe-rs download --chip RP235x --binary-format elf --speed 1000 <arm.elf>
```

The download is flaky right after the poke but makes monotonic progress, so retry it a few times.
Skip `--connect-under-reset`; the lab rig's probe reset pin is not reliably wired.

## Flashing RISC-V in the first place

`probe-rs` 0.32.0 cannot flash `RP235x_riscv` directly: that chip id is attach-only, with no
`download`, `run` or autodetect. Flash writes do not execute target-arch code, so the ARM chip id's
flash algorithm can push RISC-V bytes into flash. No OpenOCD or picotool is needed.

```bash
probe-rs download --chip RP235x --binary-format elf <riscv.elf>
probe-rs reset --chip RP235x        # re-triggers the IMAGE_DEF scan, switches the live core
probe-rs attach --chip RP235x_riscv <riscv.elf>   # RTT
```
