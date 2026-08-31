# Hardware-in-the-Loop (HIL) CI Setup

Flashing and measuring firmware benchmarks on real hardware from GitLab CI. Firmware runs over a
debug probe on the lab machine; the host benchmark runs natively alongside it.

## Design

CI runs stages `image -> build -> run -> collect -> check` wired as a `needs:` DAG, so each
firmware flashes as soon as its own build finishes, in parallel with the other builds. Checks run
last.

One `gitlab-runner` daemon on the lab Threadripper runs two registered runners:

- **build** (`shared` tag, Docker, custom image): per-target lints, the host benchmark natively,
  and the firmware cross-compiles, each binary uploaded as a flat artifact. `CARGO_HOME` and the
  target dir live under `/persist`, so builds stay warm between jobs.
- **hil** (`hil` tag, shell executor): pulls a firmware binary, flashes and runs it with
  `probe-rs`, uploads the captured RTT log. One hil runner drives every board, each job naming its
  probe through a `HIL_PROBE_*` variable. The shell executor avoids fragile Docker USB passthrough.

`GIT_STRATEGY: none` never cleans the hil workspace and all `run-*` jobs share it, so
`.firmware-flash` scopes both its `rm -f` cleanup and its `artifacts.paths` glob to its own
target's filenames.

### CI image

`rust:latest` ships none of the firmware Rust targets, the C cross-compiler, or Espressif's
`espup` Xtensa toolchain. [`Dockerfile.ci`](../.gitlab/ci/Dockerfile.ci) layers those on top and
[`image.yml`](../.gitlab/ci/image.yml) rebuilds it when the Dockerfile or the job changes, using
kaniko so it can build unprivileged. Targets are baked into the image, so a new one must be added
there too.

## Provisioning

All steps run on the Threadripper and need **root**: the runner is a system-mode install
(`/etc/gitlab-runner` is root-only) and shell jobs run as the `gitlab-runner` user.

### Probes

Use existing J-Links where available (one covers all three chips, with the best RTT). Otherwise an
ST-Link or RPi Debug Probe is the cheap default. ESP32-S3 needs no external probe: its USB JTAG
enumerates directly, see [platforms.md](platforms.md#esp32-s3).

`probe-rs` needs RPi Debug Probe firmware 2.2.0 or newer; older firmware fails with an "outdated
firmware" error. Fix once by unplugging, holding BOOTSEL while replugging (it mounts as
`RPI-RP2`), then copying the latest UF2 onto it:

```bash
curl -fL https://github.com/raspberrypi/debugprobe/releases/latest/download/debugprobe.uf2 -o /tmp/dp.uf2
DEV=$(lsblk -pnro NAME,LABEL | awk '$2=="RPI-RP2"{print $1; exit}')
[ -n "$DEV" ] || { echo "BOOTSEL button not pressed while plugging in?"; exit 1; }
sudo mkdir -p /mnt/rp2 && sudo mount "$DEV" /mnt/rp2
sudo cp /tmp/dp.uf2 /mnt/rp2/
```

### build runner

Give the shared runner its persistent Cargo volume in `/etc/gitlab-runner/config.toml`, then
`sudo gitlab-runner restart`:

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

The hil runner only flashes pre-built ELFs, so it never needs the `espup` Xtensa toolchain; that
is a build-image concern. Install it here only to build `mrs-benchmark-esp32s3` manually from this
account.

### Probe selection

Four probes are attached and the two Pico ones share a VID:PID (`2e8a:000c`), so `probe-rs run`
fails with "multiple probes found" unless the job names one. Each run job passes `--probe` from a
`HIL_PROBE_*` GitLab project variable (Settings, CI/CD, Variables), wired to `PROBE` per job in
[`run.yml`](../.gitlab/ci/run.yml).

After a probe swap, run `probe-rs list` and paste the new selectors into those variables. The two
Debug Probes differ only by serial, so tell them apart with `probe-rs info --verbose --probe
<selector>`: the RP2350 reports `PARTNO: Cortex-M33` and an `RP235x CoreSight ROM`, the RP2040
reports `Part: 0x1002` with no CPUID.

## Troubleshooting

### Probe speed

`probe-rs` picks a conservative SWD clock, and it gates flashing and RTT reads both, so raising it
roughly halves a run job. Set `PROBE_SPEED` (kHz) per target in
[`run.yml`](../.gitlab/ci/run.yml): 4000 is the fastest step the J-Link accepts, and the Debug
Probes take 10000. Leave it unset for esp32s3, whose USB JTAG is a bridge with no settable clock.

### Flaky first flash (RPi Debug Probe / CMSIS-DAP)

`probe-rs run` intermittently fails the first flash after a fresh USB attach with `Failed to erase
flash sector`, `Failed to read register DRW`, or `Target device did not respond`. This is an AP
communication error mid-erase, not a real hardware fault, and it matches
[probe-rs#1424](https://github.com/probe-rs/probe-rs/discussions/1424). `.firmware-flash` retries
each profile up to 3x. Seen on two unrelated physical probe and board pairs, so retry before
concluding a board is dead.

### Power-cycling a board

`probe-rs` resets a wedged target over SWD. A locked-up *probe* is different and needs a USB power
cycle. The boards have no switched power of their own but sit behind daisy-chained VIA Labs VL817
hubs, which `uhubctl` can drive. One-time admin setup:

```bash
sudo apt install -y uhubctl
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="2109", ATTR{idProduct}=="2817", MODE="0664", GROUP="plugdev"' \
  | sudo tee /etc/udev/rules.d/99-uhubctl.rules
sudo udevadm control --reload && sudo udevadm trigger
```

After that, `plugdev` members run `uhubctl -l <location> -p <port> -a cycle` without sudo.

### RP2350 stuck on the RISC-V core

See [`tools/rp2350-rescue/`](../tools/rp2350-rescue/README.md).
