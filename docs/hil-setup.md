# Hardware-in-the-loop (HIL) CI setup

Flashing and measuring firmware benchmarks on real hardware from GitHub Actions. Firmware runs
over a debug probe on the lab machine; the host benchmark runs natively alongside it. GitLab-era
setup archived at [archive/gitlab-hil-setup.md](archive/gitlab-hil-setup.md).

## Design

[`ci.yml`](../.github/workflows/ci.yml) builds fan out per target; each `run-firmware-<target>`
job starts as soon as its own target's build finishes, `collect` fans in, then `report` and
`publish`. The concurrency rules, the `!cancelled()` pattern, and why builds/flashes are one job
per target rather than a matrix are explained inline in `ci.yml`'s comments.

Seven repo-scoped self-hosted runners on `mrslab`, all under the `github` service account:

- **1x `build`** (labels `mrsbench,build`): per-target lints, the host benchmark natively, and
  the firmware cross-compiles, inside the `ci` container image. `CARGO_HOME` and the target dir
  live on the `mrs-cargo-cache` docker volume at `/persist`.
- **1x `hostjob`** (labels `mrsbench,hostjob`): runs `image` and `publish` on the bare host, not
  in a container.
- **5x `flash`** (labels `mrsbench,flash`): a plain pool. Each job picks its board by probe
  serial, so leg-to-runner mapping doesn't matter.

### CI image

[`Dockerfile.ci`](../.github/docker/Dockerfile.ci) layers the firmware Rust targets, the C
cross-compiler, and Espressif's `espup` Xtensa toolchain onto `rust:latest`. The `image` job in
`ci.yml` tags it with a content hash and pushes to `ghcr.io`, so an unchanged Dockerfile is a
fast no-op. Targets are baked into the image, so a new one must be added there too.

## Provisioning

Registering (or re-registering, e.g. after a lab box rebuild) the seven runners above needs
hands-on work on `mrslab`, from a root shell (`sudo -i`), with repo **Admin** or a registration
token from someone who has it.

1. Repo Settings, Actions, Runners, **New self-hosted runner**: copy the registration token (one
   token covers all seven runners, ~1h TTL).
2. Paste the token when prompted and run:

```bash
set -euo pipefail

# bash, not sh/zsh: an array so the loops cannot silently collapse into one runner
RUNNERS=(build hostjob flash-1 flash-2 flash-3 flash-4 flash-5)

# keeps the token out of ps and .bash_history
read -rsp 'registration token: ' TOKEN; echo

# 1) probe-rs for the github user: prebuilt, version-pinned, no rustup on a shared account
sudo -u github bash -c "curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/download/v0.32.0/probe-rs-tools-installer.sh \
  | sh -s -- --no-modify-path"
sudo -u github /home/github/.cargo/bin/probe-rs list   # expect 5 probes

# 2) fetch actions/runner, unpack per runner dir (skips an already-configured one)
sudo -u github mkdir -p /home/github/runners
sudo -u github curl -fLo /home/github/runners/runner.tgz \
  https://github.com/actions/runner/releases/download/v2.337.0/actions-runner-linux-x64-2.337.0.tar.gz
for r in "${RUNNERS[@]}"; do
  [ -e "/home/github/runners/$r/.runner" ] && { echo "$r configured, skipping"; continue; }
  sudo -u github mkdir -p "/home/github/runners/$r"
  sudo -u github tar xzf /home/github/runners/runner.tgz -C "/home/github/runners/$r"
done

# 3) register. PATH is passed explicitly: config.sh bakes it into .path, which runsvc.sh then
#    uses verbatim, and sudo's secure_path would otherwise drop ~/.cargo/bin.
for r in "${RUNNERS[@]}"; do
  case "$r" in build|hostjob) L="mrsbench,$r" ;; *) L=mrsbench,flash ;; esac
  sudo -u github env "PATH=/home/github/.cargo/bin:/usr/local/bin:/usr/bin:/bin" bash -c \
    "cd /home/github/runners/$r && ./config.sh --token '$TOKEN' \
       --url 'https://github.com/IMRCLab/embedded-math-benchmark-rs' \
       --name 'mrsbench-$r' --labels '$L' --unattended --replace"
done
unset TOKEN

# 4) install + start as services. Root, from each runner dir, no inner sudo.
for r in "${RUNNERS[@]}"; do
  ( cd "/home/github/runners/$r" && ./svc.sh install github && ./svc.sh start )
done

systemctl list-units 'actions.runner.*' --no-pager
```

3. Verify: `systemctl is-active 'actions.runner.*'` returns seven `active`, and all seven show
   **Idle** under repo Settings, Actions, Runners.

Creates `actions.runner.IMRCLab-embedded-math-benchmark-rs.mrsbench-*.service` x7,
`User=github`, enabled at boot.

Notes:

- The pinned `2.337.0` is only the bootstrap version; the runner self-updates in place after
  that. Bump it in this doc when it drifts far enough that a fresh install matters.
- `svc.sh` sets no `Restart=`; a crashed runner stays down until restarted by hand, same as the
  org runner. Left as is.
- The `env PATH=...` on the `config.sh` call is load-bearing: `config.sh` bakes `$PATH` into
  `.path` and `runsvc.sh` uses it verbatim, and `secure_path` would otherwise drop
  `~/.cargo/bin`.
- `probe-rs` installs to `/home/github/.cargo/bin`; the `run-firmware` job prepends that via
  `$GITHUB_PATH`, so `ci.yml` needs no change. Flash runner dirs never run `container:` jobs, so
  they need nothing else.
- No `rm` between jobs: checkout jobs get a `git clean`; the artifact-only jobs (`run-host`,
  `run-firmware`, `publish`) never check out, so their workspace persists on a long-lived runner.
  Both `run-*` jobs keep logs under `$RUNNER_TEMP`, which the runner wipes at job start and end;
  without that a half-dead leg re-uploads stale logs for the profiles it never reached and
  `collect` merges them as fresh results.

Background, already on `mrslab` (Ubuntu 22.04) from the lab's other GitHub Actions self-hosted
runners:

| Thing                    | State                                                                                                                                                                                            |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `github` user (uid 1008) | service account for runners; groups `github`, `plugdev`, `docker`                                                                                                                                |
| Docker                   | 29.6.2 + buildx v0.35.0; `github` in `docker` group                                                                                                                                              |
| Probe access             | `github` in `plugdev`; udev rules installed (`69-probe-rs.rules`, `99-bitcraze.rules`, `99-uhubctl.rules`)                                                                                       |
| Cargo cache              | docker volume `mrs-cargo-cache`, mounted at `/persist`                                                                                                                                            |
| Probes attached          | J-Link `000801038809` (stm32), J-Link `000802013924` (nrf52840), RPi Debug Probe `E6654854571E7F26` (rp2040), RPi Debug Probe `E66548545758AD26` (rp2350), ESP32-S3 USB JTAG `74:4D:BD:95:A9:CC` |
| Existing org runner      | `actions.runner.IMRCLab.threadripper.service`, runs as `github`. Unrelated to the repo-scoped runners above, leave it running                                                                    |

### Probes

Use existing J-Links where available (best RTT throughput). Otherwise an ST-Link or RPi Debug
Probe is the cheap default. ESP32-S3 needs no external probe: its USB JTAG enumerates directly,
see [platforms.md](platforms.md#esp32-s3).

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

### Probe selection

Several probes are attached (two RPi Debug Probes share the VID:PID `2e8a:000c`), so `probe-rs
run` fails with "multiple probes found" unless the job names one. Each `run-firmware-<target>`
job in `ci.yml` passes a `probe_var` input naming a `HIL_PROBE_<TARGET>` repo Actions variable
(Settings, Secrets and variables, Actions, Variables tab), which `flash-target.yml` reads via
`vars[inputs.probe_var]` and passes to `probe-rs run --probe`.

After a probe swap, run `probe-rs list` and paste the new selectors into those variables. The two
Debug Probes differ only by serial, so tell them apart with `probe-rs info --verbose --probe
<selector>`: the RP2350 reports `PARTNO: Cortex-M33` and an `RP235x CoreSight ROM`, the RP2040
reports `Part: 0x1002` with no CPUID.

## Troubleshooting

### Probe speed

`probe-rs` picks a conservative SWD clock, and it gates flashing and RTT reads both, so raising it
roughly halves a run job. Each target's `run-firmware-<target>` call in `ci.yml` sets a `speed`
input (kHz), passed through to `flash-target.yml` as `--speed`: 4000 is the fastest step the
J-Link accepts, and the Debug Probes take 10000. Left empty for esp32s3, whose USB JTAG is a
bridge with no settable clock.

### Flaky first flash (RPi Debug Probe / CMSIS-DAP)

`probe-rs run` intermittently fails the first flash after a fresh USB attach with `Failed to erase
flash sector`, `Failed to read register DRW`, or `Target device did not respond`. This is an AP
communication error mid-erase, not a real hardware fault, and it matches
[probe-rs#1424](https://github.com/probe-rs/probe-rs/discussions/1424). `flash-target.yml` retries
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
