# Hardware-in-the-Loop (HIL) CI Setup

Setup for flashing and measuring firmware benchmarks on real hardware from GitLab CI. The
Raspberry Pi Pico 1 (RP2040) is wired up and benchmarking on hardware in CI now; the host
benchmark runs natively alongside it. More to follow. Each board drops into the same per-target CI structure.

## Design

The benchmarks live in a Cargo workspace (`benchmarks/`), one crate per target
(`mrs-benchmark-host`, `mrs-benchmark-rp2040`, `mrs-benchmark-stm32`, ...). CI runs three
stages (`build`, `run`, `check`), wired as a `needs:` DAG so each firmware flashes as soon
as its build finishes, in parallel with the other build jobs. The checks are running last.

The Pico connects to the lab Threadripper over SWD through an RPi debug probe. One `gitlab-runner` daemon there runs two registered runners:

- **build** (`shared` tag, Docker executor, `rust:latest`). Runs the per-target lints,
  builds and runs the host benchmark natively (`build-host` / `run-host`), and
  cross-compiles the firmware (`build-rp2040` / `build-stm32`), then uploads the binary as
  a flat artifact. CI keeps `CARGO_HOME` and the target dir under `/persist`, so builds
  stay warm between jobs.
- **hil** (`hil` tag, shell executor). Pulls the firmware binary and flashes + runs it with
  `probe-rs`, then uploads the captured RTT log as an artifact. `probe-rs` picks the chip by
  `--chip`, so one hil runner flashes every board (`run-rp2040`, `run-stm32`, ...). The shell executor avoids fragile Docker USB stuff.

Output comes back over RTT with `probe-rs`.

## Probes

Use existing J-Links if we have them. One covers all three chips with the best RTT.
Otherwise ST-Link / RPi Debug Probe is the cheap default.

### Rpi debug probe

`probe-rs` already ships the `RP2040` and `RP235x` (RP2350) targets.

**Firmware** probe-rs needs firmware 2.2.0 or newer, and my probe had a much older one by default. It then fails with an "outdated firmware" error. Fix it once: unplug the probe, hold the button (BOOTSEL) while replugging it. Then it comes up as volume labelled `RPI-RP2`. Enter this:

```bash
curl -fL https://github.com/raspberrypi/debugprobe/releases/latest/download/debugprobe.uf2 -o /tmp/dp.uf2
# Find device by label
DEV=$(lsblk -pnro NAME,LABEL | awk '$2=="RPI-RP2"{print $1; exit}')
# Verify device exists
[ -n "$DEV" ] || { echo "BOOTSEL button not pressed while plugging in?"; exit 1; }
# Mount device and copy
sudo mkdir -p /mnt/rp2 && sudo mount "$DEV" /mnt/rp2
sudo cp /tmp/dp.uf2 /mnt/rp2/
```

## Provisioning

All steps run on the Threadripper and need **root**: the
runner is a system-mode install (`/etc/gitlab-runner` is root-only) and shell jobs run as
the `gitlab-runner` user.

### build runner

Give the shared runner its persistent Cargo volume in `/etc/gitlab-runner/config.toml`,
then `sudo gitlab-runner restart`:

```toml
# under the shared runner's [runners.docker]
volumes = ["/cache", "mrs-cargo-cache:/persist"]
```

### hil runner

First create the runner in the GitLab UI and tag it `hil`. Copy the token to below.

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

# 4) probe-rs udev rules (root-owned). Supports all: RPI debug probe and stmlink/j-link
sudo curl --proto "=https" --tlsv1.2 -sSf https://probe.rs/files/69-probe-rs.rules \
  -o /etc/udev/rules.d/69-probe-rs.rules
sudo udevadm control --reload && sudo udevadm trigger

# 5) Probe access for the runner user.
sudo usermod -aG plugdev gitlab-runner
sudo gitlab-runner restart

# 6) Verify (probe must be physically connected over USB/SWD).
sudo -u gitlab-runner bash -c 'source $HOME/.cargo/env && probe-rs list'
```

If the target wedges, `probe-rs` resets it over SWD. A locked-up probe is different: it needs a USB power cycle, which means using a PPPS-capable hub, e.g. Rosonway RSH‑A10 or Rosonway RSH‑A16.
