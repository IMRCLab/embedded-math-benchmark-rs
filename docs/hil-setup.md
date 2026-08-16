# Hardware-in-the-Loop (HIL) CI Setup

Setup for flashing and measuring firmware benchmarks on real hardware from GitLab CI.
Firmware runs over a debug probe on the lab machine; the host benchmark runs natively
alongside it. Each board drops into the same per-target CI structure.

## Design

The benchmarks are a Cargo workspace (`benchmarks/`), one crate per target
(`mrs-benchmark-host`, `mrs-benchmark-rp2040`, `mrs-benchmark-stm32`, ...). CI runs
stages `image → build → run → collect → check` wired as a `needs:` DAG, so each firmware
flashes as soon as its build finishes, in parallel with the other builds. Checks run
last. `image` only rebuilds the shared CI image when its Dockerfile changes — see
[CI image](#ci-image).

The boards connect to the lab Threadripper over SWD through debug probes. One
`gitlab-runner` daemon there runs two registered runners:

- **build** (`shared` tag, Docker, custom image — see [CI image](#ci-image)): runs the
  per-target lints, builds and runs the host benchmark natively (`build-host` /
  `run-host`), and cross-compiles the firmware (`build-rp2040` / `build-stm32`),
  uploading each binary as a flat artifact. `CARGO_HOME` and the target dir live under
  `/persist`, so builds stay warm between jobs.
- **hil** (`hil` tag, shell executor): pulls a firmware binary, flashes and runs it with
  `probe-rs`, uploads the captured RTT log. One hil runner flashes every board
  (`run-rp2040`, `run-stm32`, ...); it picks the right probe by chip with
  [`select-probe.sh`](#multiple-probes). The shell executor avoids fragile Docker USB
  passthrough.

Output comes back over RTT with `probe-rs`.

## CI image

The `shared` runner's base image needs multiple dependencies plus the
firmware Rust targets, none of which `rust:latest` ships. It also needs Espressif's `espup`
Xtensa toolchain for `mrs-benchmark-esp32s3`.
[`Dockerfile.ci`](../.gitlab/ci/Dockerfile.ci) layers those on top of `rust:latest`;
[`image.yml`](../.gitlab/ci/image.yml) builds and republishes it when the Dockerfile or the job itself changes.

Built with kaniko: it builds from an unprivileged container
instead. Targets and components are baked into the image. So a new target must be added to [`Dockerfile.ci`](../.gitlab/ci/Dockerfile.ci) too.

## Probes

Use existing J-Links if we have them (one covers all three chips with the best RTT).
Otherwise ST-Link / RPi Debug Probe is the cheap default. ESP32-S3 needs no external probe
at all: its USB JTAG enumerates directly, see [Targets & Platforms](platforms.md).

### RPi debug probe

`probe-rs` ships the `RP2040` and `RP235x` (RP2350) targets.

**Firmware:** `probe-rs` needs Debug Probe firmware 2.2.0+; older firmware fails with an
"outdated firmware" error. Fix once: unplug, hold BOOTSEL while replugging (it mounts as
`RPI-RP2`), then flash the latest UF2:

```bash
curl -fL https://github.com/raspberrypi/debugprobe/releases/latest/download/debugprobe.uf2 -o /tmp/dp.uf2
DEV=$(lsblk -pnro NAME,LABEL | awk '$2=="RPI-RP2"{print $1; exit}')
[ -n "$DEV" ] || { echo "BOOTSEL button not pressed while plugging in?"; exit 1; }
sudo mkdir -p /mnt/rp2 && sudo mount "$DEV" /mnt/rp2
sudo cp /tmp/dp.uf2 /mnt/rp2/
```

### Multiple probes

Probes share the Debug Probe VID:PID (`2e8a:000c`), so `probe-rs run --chip <chip>` fails
with "multiple probes found". Each run job calls
[`select-probe.sh`](../.gitlab/ci/select-probe.sh) `<chip>` first, which returns the
`VID:PID:Serial` of the matching probe. It resolves in two steps:

1. Filter `probe-rs list` by probe type. J-Link → STM32, ESP JTAG → ESP32-S3; those are
   one-to-one on this host and resolve in ~0.1s with no attach.
2. Only if the type is ambiguous (both Picos are behind CMSIS-DAP probes), grep each
   candidate's `probe-rs info` for a per-chip signature.

Step 2 halts the target, which is why step 1 exists: scanning all four probes was 11-13s of
every run job.

**probe-rs 0.31 vs 0.32.** 0.32 changed `probe-rs info` to autodetect and print a chip name,
moving the detailed debug-port dump that step 2 greps behind `--verbose`. Autodetect doesn't
cover RP chips, so plain `info` there just fails. 0.31 has no such flag, so the script probes
`info --help` once and passes `--verbose` only if it's offered. Verified on hardware against
0.32: all four chips resolve, and `probe-rs list` output is unchanged.

Step 2 costs 0.4s on the RP2040 probe but **10.4s on the RP2350 probe** (both versions), and
both Pico jobs pay it — `run-rp2040` scans the RP2350 probe first and only then matches. That
is ~21s per pipeline, the largest remaining fixed overhead in the run stage.

### Probe speed

`probe-rs` picks a conservative SWD clock. It gates flashing *and* RTT reads, so raising it
where the probe allows shortens the whole run job. Set per target via `PROBE_SPEED` (kHz) in
[`run.yml`](../.gitlab/ci/run.yml).

| target | probe | speed | flash | stream | job |
|---|---|---|---|---|---|
| STM32F405 | J-Link | default | 30.8s | 20.6s | 51.4s |
| STM32F405 | J-Link | `4000` | 17.3s | 7.6s | 24.9s |
| RP2040 | Debug Probe | default | 9.6s | 44.0s | 53.6s |
| RP2040 | Debug Probe | `10000` | 6.4s | 41.9s | 48.3s |
| RP2350 | Debug Probe | default | 10.2s | 10.8s | 33.1s |
| RP2350 | Debug Probe | `10000` | 5.1s | 4.9s | 22.3s |

4 MHz is the fastest step the J-Link accepts; 6/8/10/12 MHz are rejected outright. The
Debug Probe takes 10 MHz.

ESP32-S3 is the exception: `--speed` does nothing there (16.7s vs 17.9s, i.e. noise). Its
USB JTAG is a bridge, not an SWD link with a settable clock, so leave `PROBE_SPEED` unset.

## Power-cycling a board

When a real power cycle is what's needed: boards have no switched power of their own, but sit behind daisy-chained VIA Labs VL817 USB hubs, which are `uhubctl`-capable. One-time admin setup:

```bash
sudo apt install -y uhubctl
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="2109", ATTR{idProduct}=="2817", MODE="0664", GROUP="plugdev"' \
  | sudo tee /etc/udev/rules.d/99-uhubctl.rules
sudo udevadm control --reload && sudo udevadm trigger
```

After that, `plugdev` members can run `uhubctl` without sudo. Then `uhubctl -l <location> -p <port> -a cycle`.

## Provisioning

All steps run on the Threadripper and need **root**: the runner is a system-mode install
(`/etc/gitlab-runner` is root-only) and shell jobs run as the `gitlab-runner` user.

### build runner

Give the shared runner its persistent Cargo volume in `/etc/gitlab-runner/config.toml`,
then `sudo gitlab-runner restart`:

```toml
# under the shared runner's [runners.docker]
volumes = ["/cache", "mrs-cargo-cache:/persist"]
```

### hil runner

First create the runner in the GitLab UI and tag it `hil`. Copy the token in below.

```bash
# 1) Register the hil runner (shell executor).
sudo gitlab-runner register --non-interactive \
  --url "https://git.tu-berlin.de/" \
  --token "glrt-XXXXXXXX" \
  --executor "shell" \
  --description "hil"

# 2) Rust toolchain for the gitlab-runner user (only needed to build probe-rs below).
sudo -u gitlab-runner bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

# 3) probe-rs (compiles; a few minutes).
sudo -u gitlab-runner bash -c 'source $HOME/.cargo/env && cargo install probe-rs-tools --locked'

# 4) probe-rs udev rules (root-owned). Covers RPi debug probe, ST-Link, and J-Link.
sudo curl --proto "=https" --tlsv1.2 -sSf https://probe.rs/files/69-probe-rs.rules \
  -o /etc/udev/rules.d/69-probe-rs.rules
sudo udevadm control --reload && sudo udevadm trigger

# 5) Probe access for the runner user.
sudo usermod -aG plugdev gitlab-runner
sudo gitlab-runner restart

# 6) Verify (probe must be physically connected over USB/SWD).
sudo -u gitlab-runner bash -c 'source $HOME/.cargo/env && probe-rs list'
```

The hil runner only flashes pre-built ELFs with `probe-rs`, so it never needs the
`espup` Xtensa toolchain (that's a build-image concern, see [CI image](#ci-image)).
Installing it here is only useful for manually building/testing `mrs-benchmark-esp32s3`
from this account directly: `cargo install espup --locked && espup install --targets
esp32s3`, then `source $HOME/export-esp.sh` before `cargo build`.

If the target wedges, `probe-rs` resets it over SWD. A locked-up probe is different: it
needs a USB power cycle, so use a PPPS-capable hub.
