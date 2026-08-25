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
  (`run-rp2040`, `run-stm32`, ...); each job names its probe
  through a `HIL_PROBE_*` variable, see [Probe selection](#probe-selection). The shell executor avoids fragile Docker USB
  passthrough.

Output comes back over RTT with `probe-rs`; since `GIT_STRATEGY: none` never cleans this
workspace and all four `run-*` jobs share it, `.firmware-flash` scopes both its `rm -f`
cleanup and its `artifacts.paths` glob to this target's own filenames, so a job never touches
or re-uploads another target's leftover log.

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

### Probe selection

Four probes are attached, and the two Pico ones share a VID:PID (`2e8a:000c`), so
`probe-rs run --chip <chip>` fails with "multiple probes found" unless the job names one.
Each run job passes `--probe` from a `HIL_PROBE_*` GitLab **project variable**
(Settings → CI/CD → Variables), wired to `PROBE` per job in
[`run.yml`](../.gitlab/ci/run.yml).

After a probe swap, run `probe-rs list` on the HIL host and paste the new selectors into the
project variables. The two Debug Probes differ only by serial, so tell them apart with
`probe-rs info --verbose --probe <selector>`: the RP2350 reports `PARTNO: Cortex-M33` and an
`RP235x CoreSight ROM`, the RP2040 reports `Part: 0x1002` with no CPUID.

Resolving the probe dynamically instead (grep each candidate's `probe-rs info` for a chip
signature) costs an attach per candidate: 10.4s on the RP2350 probe, paid by both Pico jobs,
about 21s per pipeline. Not worth it for a mapping that only changes when the hardware does.

**Known flaky first flash (RPi Debug Probe / CMSIS-DAP).** `probe-rs run` intermittently fails
the first flash after a fresh USB attach with `Failed to erase flash sector` / `Failed to read
register DRW` / `Target device did not respond` — an AP communication error mid-erase, not a
real hardware fault. Matches [probe-rs/probe-rs#1424](https://github.com/probe-rs/probe-rs/discussions/1424),
retry succeeds. `.firmware-flash` in [`run.yml`](../.gitlab/ci/run.yml) retries each profile up
to 3x before failing the job. Confirmed 2026-08-25 on two unrelated physical RP2350
probe+board pairs (the swapped-in replacement showed the identical error signature from its
first flash) -- don't assume this error means the board is dead; retry before replacing
hardware.

### Probe speed

`probe-rs` picks a conservative SWD clock. It gates flashing _and_ RTT reads, so raising it
where the probe allows shortens the whole run job. Set per target via `PROBE_SPEED` (kHz) in
[`run.yml`](../.gitlab/ci/run.yml).

| target    | probe       | speed   | flash | stream | job   |
| --------- | ----------- | ------- | ----- | ------ | ----- |
| STM32F405 | J-Link      | default | 30.8s | 20.6s  | 51.4s |
| STM32F405 | J-Link      | `4000`  | 17.3s | 7.6s   | 24.9s |
| RP2040    | Debug Probe | default | 9.6s  | 44.0s  | 53.6s |
| RP2040    | Debug Probe | `10000` | 6.4s  | 41.9s  | 48.3s |
| RP2350    | Debug Probe | default | 10.2s | 10.8s  | 33.1s |
| RP2350    | Debug Probe | `10000` | 5.1s  | 4.9s   | 22.3s |

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

Then paste each selector from step 6 into its `HIL_PROBE_*` project variable, unprotected
(see [Probe selection](#probe-selection)). This is the only step that needs no lab admin.

The hil runner only flashes pre-built ELFs with `probe-rs`, so it never needs the
`espup` Xtensa toolchain (that's a build-image concern, see [CI image](#ci-image)).
Installing it here is only useful for manually building/testing `mrs-benchmark-esp32s3`
from this account directly: `cargo install espup --locked && espup install --targets
esp32s3`, then `source $HOME/export-esp.sh` before `cargo build`.

If the target wedges, `probe-rs` resets it over SWD. A locked-up probe is different: it
needs a USB power cycle, so use a PPPS-capable hub.
