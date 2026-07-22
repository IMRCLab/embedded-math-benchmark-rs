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

The `shared` runner's base image needs `arm-none-eabi-gcc`/`libnewlib-arm-none-eabi` and
`libclang-dev` (for `mrs-benchmark-crazyflie-sys`'s C compile + `bindgen` steps) plus the
firmware Rust targets, none of which `rust:latest` ships. It also needs Espressif's `espup`
Xtensa toolchain for `mrs-benchmark-esp32s3` (a separate rustc/LLVM fork, since Xtensa has
no upstream-stable Rust target), baked in the same way as the ARM targets.
[`Dockerfile.ci`](../.gitlab/ci/Dockerfile.ci) layers those on top of `rust:latest`;
[`image.yml`](../.gitlab/ci/image.yml) builds and republishes it (`:latest` +
`:$CI_COMMIT_SHORT_SHA`) only when the Dockerfile or the job itself changes.

Built with kaniko, not `docker:dind` — the runner isn't (and can't easily be)
configured with `privileged = true`, and kaniko builds from an unprivileged container
instead. Targets and components are baked into the image. A new target must be added to [`Dockerfile.ci`](../.gitlab/ci/Dockerfile.ci) too.

## Probes

Use existing J-Links if we have them (one covers all three chips with the best RTT).
Otherwise ST-Link / RPi Debug Probe is the cheap default.

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

### ESP32-S3 (native USB JTAG)

No external probe wiring needed: the ESP32-S3's built-in USB Serial/JTAG Controller
enumerates directly as a probe-rs-visible "ESP JTAG" probe over its one USB cable
(distinct VID:PID from the Debug Probe pool, so it never collides in
[probe selection](#multiple-probes)). This is unlike a bare dongle-style board with no
onboard debugger, which needs a separate SWD probe wired to its debug pads before
`probe-rs` can reach it at all.

### Multiple probes

Probes share the Debug Probe VID:PID (`2e8a:000c`), so `probe-rs run --chip <chip>` fails
with "multiple probes found". Each run job calls
[`select-probe.sh`](../.gitlab/ci/select-probe.sh) `<chip>` first, which greps each probe's
`probe-rs info` for a per-chip signature and returns the `VID:PID:Serial` of the match. A
wrong pick just fails at the flashing step that follows.

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
needs a USB power cycle, so use a PPPS-capable hub (e.g. Rosonway RSH-A10 or RSH-A16).
