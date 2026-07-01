# Hardware-in-the-Loop (HIL) CI Setup

The planned setup for flashing and measuring firmware benchmarks on real hardware from
GitLab CI. Nothing runs on hardware yet: today only the host benchmark runs in CI, and the
STM32 firmware just cross-compiles in software. STM32F405 is the first board to bring up,
with Raspberry Pi Pico 1 (RP2040) and Pico 2 (RP2350) to follow; each drops into the same
per-target CI structure, and nRF52840 or others may come later.

## Design

The benchmarks live in a Cargo workspace (`benchmarks/`), one crate per target
(`mrs-benchmark-host`, `mrs-benchmark-stm32`, more to come). CI runs three stages
(`check`, `build`, `run`) with per-target jobs.

The STM32 board is wired directly to the lab Threadripper over SWD. One `gitlab-runner` daemon there runs two registered runners:

- **build** (`shared` tag, Docker executor, `rust:latest`). Runs the per-target lints,
  builds and runs the host benchmark natively (`build-host` / `run-host`), and
  cross-compiles the firmware (`build-stm32`), then uploads the binary as a flat artifact
  (just the binary, at the project root).
- **hil** (`hil` tag, shell executor). Pulls that binary and flashes + runs it on the
  board (`run-stm32`), then uploads the captured RTT log as an artifact. The shell executor avoids fragile Docker USB stuff.

Output comes back over RTT with `probe-rs` (it replaces UART). We can switch to `defmt`
later for pass/fail asserts without changing any infra.

## Probes

Use existing J-Links if we have them. One covers all three chips with the best RTT.
Otherwise ST-Link / RPi Debug Probe is the cheap default.

**Pico probes & udev** `probe-rs` already ships the `RP2040` and `RP235x`
(RP2350) targets, so the toolchain from _Provisioning_ below works unchanged for the Picos.
The Pico is flashed over SWD by the Raspberry Pi Debug Probe or a second
Pico running `debugprobe` firmware. Both enumerate as CMSIS-DAP.

## Provisioning

All steps run on the Threadripper and need **root**: the
runner is a system-mode install (`/etc/gitlab-runner` is root-only) and shell jobs run as
the `gitlab-runner` user.

First create the runner in the GitLab UI and tag it `hil`. Copy the token to below.

```bash
# 1) Register the hil runner (shell executor).
sudo gitlab-runner register --non-interactive \
  --url "https://git.tu-berlin.de/" \
  --token "glrt-XXXXXXXX" \
  --executor "shell" \
  --description "hil-stm32"

# 2) Rust toolchain for the gitlab-runner user (jobs run as that user).
sudo -u gitlab-runner bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
sudo -u gitlab-runner bash -c 'source $HOME/.cargo/env && rustup target add thumbv7em-none-eabihf'

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

If the target wedges, `probe-rs` resets it over SWD. A locked-up probe is different: it needs a USB power cycle, which means using a PPPS-capable hub. That is optional for now.
