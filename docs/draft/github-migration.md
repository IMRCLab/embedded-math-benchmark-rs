# GitHub migration

Status: **in progress**. Repo moved from `git.tu-berlin.de` to
`github.com/IMRCLab/embedded-math-benchmark-rs` (public), CI from GitLab pipelines to GitHub
Actions. Hard cut-over, only the git repo carried over (history re-authored, content identical).

Done: repo created, `main` pushed with `.github/workflows/ci.yml`, the five `HIL_PROBE_*`
variables set. Pending: the self-hosted runners (see below), then the cut-over checklist.

`.github/workflows/ci.yml` is the ported pipeline. This doc is the plan around it: runner
setup, repo settings, and the cut-over checklist.

## Already provisioned on the lab box (`mrslab`, Ubuntu 22.04)

The lab already runs GitHub Actions self-hosted runners, so most of the groundwork exists.

| Thing                    | State                                                                                                                                                                                            |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `github` user (uid 1008) | service account for runners; groups `github`, `plugdev`, `docker`                                                                                                                                |
| Runner service           | `actions.runner.IMRCLab.threadripper.service`, an existing org-level runner for `IMRCLab`, runs as `github`. Leave it; the new runners are repo-scoped and independent                           |
| Docker                   | 29.6.2 + buildx v0.35.0; `github` in the `docker` group, so `container:` jobs and `docker build` both work with no extra privilege                                                               |
| probe access             | `github` in `plugdev`; udev rules already installed (`69-probe-rs.rules`, `99-bitcraze.rules`, `99-uhubctl.rules`)                                                                               |
| Persistent cargo cache   | docker volume `mrs-cargo-cache` (carried over from GitLab, mounted at `/persist`)                                                                                                                |
| Probes attached          | J-Link `000801038809` (stm32), J-Link `000802013924` (nrf52840), RPi Debug Probe `E6654854571E7F26` (rp2040), RPi Debug Probe `E66548545758AD26` (rp2350), ESP32-S3 USB JTAG `74:4D:BD:95:A9:CC` |
| `probe-rs`               | not yet installed for `github`; provisioning step below builds it into `/home/github/.cargo`                                                                                                     |

Whoever drives the migration needs repo **Admin** (to mint a runner registration token, re-run
jobs, watch runner health) or a token handed over by an admin. Registering and running the
runners is otherwise all done as `github` via a lab admin's `sudo -u github`.

## Runner topology

GitHub Actions has no "executor with concurrency N". One runner process runs one job at a time;
parallelism means more runner processes. So the GitLab model maps as:

| GitLab                                 | GitHub                                  | `runs-on`                      |
| -------------------------------------- | --------------------------------------- | ------------------------------ |
| Docker executor, custom image          | 1 runner, `container:` jobs             | `self-hosted, mrsbench, build` |
| one `hil` shell runner, several boards | a pool of 5 runners on the one HIL host | `self-hosted, mrsbench, flash` |

Repo-scoped runners (registered against the repo, not the org), so no runner group is needed;
they only ever serve this repo. The `mrsbench` label identifies them in the org runner list
(`mrs` alone was a stale name and risked colliding with other `IMRCLab` runners).

`ci.yml` expects:

- **1x `build`** runner, `mrsbench-build`, labels `mrsbench,build`. Runs the `container:` jobs:
  `build`, `check`, `run-host`, `collect`, `report`.
- **1x `hostjob`** runner, `mrsbench-hostjob`, labels `mrsbench,hostjob`. Runs `image` and
  `publish`, the two jobs that run on the host rather than in the container. They need their own
  runner because a `container:` job writes the shared workspace as root while a host job writes it
  as `github`, and on the next run the host job's `git clean` cannot delete what root left behind.
  Keeping the two kinds on separate runners keeps each `_work` single-owner.
- **5x `flash`** runners, `mrsbench-flash-1` .. `mrsbench-flash-5`, all labels `mrsbench,flash`.
  A plain pool: GitHub hands each `run-firmware` matrix leg to whichever is idle, and the job
  picks its board by probe serial (`--probe "$HIL_PROBE_<TARGET>"`), so leg-to-runner mapping
  does not matter. All five probes are on the one host and visible to `github`. A new board is
  just a matrix leg in `ci.yml`; add a 6th runner only when five slots become a bottleneck.

All run as the `github` service account so nothing is tied to a personal login, each in its own
directory with its own `svc.sh` systemd service. The lab already runs runners this way.

### Provisioning

Run by a lab admin on `mrslab`, from a root shell (`sudo -i`). One registration token
(repo Settings, Actions, Runners, "New self-hosted runner"; ~1h TTL) covers all seven runners.
`SHA256` comes off that same page.

```bash
set -euo pipefail
VER=2.337.0
SHA256=70920811a4f8ad4328818682bca5c6469c1c942fab52448868071d0063816613
GH=https://github.com/IMRCLab/embedded-math-benchmark-rs
# bash, not sh/zsh: an array so the loops cannot silently collapse into one runner
RUNNERS=(build hostjob flash-1 flash-2 flash-3 flash-4 flash-5)

# keeps the token out of ps and .bash_history
read -rsp 'registration token: ' TOKEN; echo

# 1) probe-rs for the github user: prebuilt, version-pinned, no rustup on a shared account
sudo -u github bash -c "curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/download/v0.32.0/probe-rs-tools-installer.sh \
  | sh -s -- --no-modify-path"
sudo -u github /home/github/.cargo/bin/probe-rs list   # expect 5 probes

# 2) fetch, verify, unpack. Re-runnable: never unpacks over a configured runner.
sudo -u github mkdir -p /home/github/runners
sudo -u github curl -fLo /home/github/runners/runner.tgz \
  "https://github.com/actions/runner/releases/download/v${VER}/actions-runner-linux-x64-${VER}.tar.gz"
echo "${SHA256}  /home/github/runners/runner.tgz" | sha256sum -c -
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
    "cd /home/github/runners/$r && ./config.sh --url '$GH' --token '$TOKEN' \
       --name 'mrsbench-$r' --labels '$L' --unattended --replace"
done
unset TOKEN

# 4) install + start as services. Root, from each runner dir, no inner sudo.
for r in "${RUNNERS[@]}"; do
  ( cd "/home/github/runners/$r" && ./svc.sh install github && ./svc.sh start )
done

systemctl list-units 'actions.runner.*' --no-pager
```

Yields `actions.runner.IMRCLab-embedded-math-benchmark-rs.mrsbench-*.service` x7, `User=github`,
enabled at boot, same unit shape as the existing org runner. `svc.sh` generates no `Restart=`, so a
crashed runner stays down until someone restarts it; that matches the org runner and is left as is.
The runner self-updates in place, so `VER` only picks the version you bootstrap from.

`probe-rs` lands in `/home/github/.cargo/bin`, which the `run-firmware` job prepends to `PATH`
itself (`$GITHUB_PATH`), so `ci.yml` needs no change. The flash runner dirs never run
`container:` jobs, so they need nothing else. The `mrs-cargo-cache` volume already exists.

No `rm` cleanup between jobs. Checkout jobs get a `git clean`; the artifact-only jobs
(`run-host`, `run-firmware`, `publish`) never check out, so their workspace persists across
runs on a long-lived runner. Both `run-*` jobs therefore write their logs under `$RUNNER_TEMP`,
which the runner empties at the start and end of every job. Without that, a leg that dies
part-way re-uploads the previous run's logs for the profiles it never reached, and `collect`
merges them as fresh results.

## GitHub repo setup

1. ~~**Create** `github.com/IMRCLab/embedded-math-benchmark-rs`, public. Push `main`.~~ Done.
   Submodules already resolve to `github.com` (`crazyflie-firmware`, `CMSIS-DSP`).
2. ~~**Variables** (Settings, Secrets and variables, Actions, Variables), not secrets, the USB
   probe selectors from `probe-rs list`: `HIL_PROBE_STM32`, `HIL_PROBE_NRF52840`,
   `HIL_PROBE_RP2040`, `HIL_PROBE_RP2350`, `HIL_PROBE_ESP32S3`.~~ Done. Note `HIL_PROBE_RP2350`
   was copied without the `-0:` port segment the other Debug Probe var has; probe-rs matches on
   serial so it works, normalise if the rp2350 leg misbehaves.
3. **Fork PRs**: Settings, Actions, General, uncheck "Run workflows from fork pull requests".
   `ci.yml` triggers on `push` and `workflow_dispatch` only, never `pull_request`, so a fork
   triggers nothing. Branches pushed to the repo run the full pipeline.
4. **ghcr package**: nothing to do. The container jobs pull with `credentials:` and the
   run's `GITHUB_TOKEN`, so a private package works. Set the `ci` package to **Public** (repo,
   Packages) only if you want to `docker pull` it by hand off CI.
5. **Branch protection** (optional): require the `build` and `run-*` checks on `main`. `check`
   (fmt/clippy) is advisory today; making it required is a later call.
6. **Actions permissions**: confirm the org lets `GITHUB_TOKEN` use `packages: write` (the
   `image` job pushes to ghcr) and `contents: write` (the `publish` job moves the `latest`
   release). If restricted, those two jobs 403 and the rest still runs.

## Pipeline mapping

| GitLab                                             | GitHub Actions                                         | Notes                                |
| -------------------------------------------------- | ------------------------------------------------------ | ------------------------------------ |
| `image` stage, kaniko, `rules: changes:`           | `image` job, `docker build`, content-hash tag          | no kaniko: the runner can run docker |
| `.cargo-build` + `parallel: matrix`                | `build` job, one matrix leg per (target, profile)      | `cargo make` still owns the flags    |
| `check` stage, `allow_failure: true`               | `check` job, `continue-on-error: true`                 |                                      |
| `.firmware-flash`, `hil` tag, `GIT_STRATEGY: none` | `run-firmware` job, `flash` runners, no checkout       | one matrix leg per board             |
| `run-host`                                         | `run-host` job, in the container                       | glibc parity with the build          |
| `collect-results`, optional `needs:`               | `collect` job, `if: ${{ !cancelled() }}`               | tolerates failed flash legs          |
| `report-pdf`, `allow_failure: true`                | `report` job, `continue-on-error: true`                |                                      |
| `expire_in: 1 year` artifact                       | `retention-days: 90` (public repo max) + `publish` job | see below                            |

The DAG is the same shape: `build` fans out, each flash leg starts when `build` finishes,
`collect` fans in, `report` and `publish` follow.

Two differences worth knowing:

- **Flash waits for the whole `build` matrix.** GitLab let each flash job `needs:` only its own
  build job; GitHub `needs:` is per job, not per matrix leg, so stm32 cannot flash until the
  esp32s3 build is done. With one `build` runner the legs are serial anyway. If that changes,
  split `build` into one job per target and give each flash leg its own `needs:`.
- **A new run cancels the previous one** (`concurrency: cancel-in-progress: true`, keyed on the
  workflow and ref). Runs on two different refs still overlap and would contend for the probes;
  drop `-${{ github.ref }}` from the group to serialise the whole repo if that bites.

### The `container:` model

Every `build`, `check`, `run-host`, `collect` and `report` job runs inside
`ghcr.io/imrclab/embedded-math-benchmark-rs/ci` (the `image` job lowercases `github.repository`
for the tag), the GitHub equivalent of GitLab's Docker executor. The image is `.github/docker/Dockerfile.ci`
(moved from `.gitlab/ci/`, still built by GitLab too until cut-over). The `image` job tags it
with a 16-char hash of the Dockerfile, so an unchanged Dockerfile is a fast no-op and every
pipeline builds against the image matching that commit.

A container job runs with `HOME=/github/home`, not `/root` as GitLab's Docker executor left
it. `espup` installs `export-esp.sh` into `/root`, so the esp32s3 tasks read
`${ESP_ENV_SCRIPT:-$HOME/export-esp.sh}` and CI sets `ESP_ENV_SCRIPT`. Local runs are unaffected.

`mrs-cargo-cache:/persist` is mounted into each container, with `CARGO_HOME`,
`CARGO_TARGET_DIR` and the `uv` caches pointed at it, exactly as the GitLab pipeline did. One
`build` runner means builds serialize and the shared `/persist/target` is safe.

A second `build` runner would get its own `_work`, so the checkouts do not collide, but it would
share `/persist/target` and cargo holds an exclusive lock on the target dir. The second leg then
blocks on the lock instead of building, so a second runner buys nothing until it also gets its own
`CARGO_TARGET_DIR`, which costs a second cold cargo cache on an already full disk.

### Artifacts and the report link

- Every run uploads its artifacts at `retention-days: 90` (the public-repo maximum). Grab them
  from the run page or `gh run download`.
- `main` runs also push `report.pdf`, `results.csv`, `accuracy_results.csv/json` to a rolling
  `latest` prerelease. Release assets never expire, and the tag-addressed URL is stable:
  - `https://github.com/IMRCLab/embedded-math-benchmark-rs/releases/download/latest/report.pdf`
  - Not `/releases/latest/download/...`: that resolves the newest **non**-prerelease release
    and 404s here. Drop `prerelease: true` in `ci.yml` if you want that form instead.

  This replaces the GitLab "latest main" raw-artifact link in
  [benchmark-viz.md](../benchmark-viz.md).

## Cut-over checklist

1. ~~`HIL_PROBE_*` variables set.~~ Done.
2. Seven runners registered and idle (Settings, Actions, Runners): `mrsbench-build`,
   `mrsbench-hostjob`, `mrsbench-flash-1..5`.
3. `git remote`: on the working copy, GitHub is `origin`, old TU-Berlin GitLab kept as `gitlab`.
   Pre-migration history parked on local branch `gitlab-archive-main`.
4. One green pipeline on a branch, then on `main`, then check the release URL above resolves.
5. `git rm -r .gitlab .gitlab-ci.yml` (the moved `Dockerfile.ci` stays under `.github/`).
6. Stop / deregister the GitLab HIL and build runners, or leave `gitlab-runner` installed and
   idle. Do not drive both CI systems at the probes at once.
7. Rewrite the docs that describe GitLab CI, in the same commit as step 5:
   - [hil-setup.md](../hil-setup.md): replace the GitLab runner provisioning with the runner
     setup above; keep the probe, udev, `uhubctl` and RP2350-rescue sections.
   - [adding_a_platform.md](../adding_a_platform.md) step 4: `.github/workflows/ci.yml` matrix
     rows (`build`, `check`, `run-firmware`) + a `HIL_PROBE_<PLAT>` variable, instead of the
     `.gitlab/ci/` checklist. No new runner needed unless the five `flash` slots bottleneck.
   - [benchmark-viz.md](../benchmark-viz.md): the report link and the `glab job artifact` line
     (use `gh run download` or the release URL).
   - [running-benchmarks.md](../running-benchmarks.md): drop the `glab` reference if present.
   - [benchmark.md](../benchmark.md): the HIL-setup index line.
   - [CLAUDE.md](../../CLAUDE.md): "GitLab HIL CI" wording, the State section.
   - [draft/rp2350-riscv-design.md](rp2350-riscv-design.md): the CI section talks in GitLab
     job/`needs:` terms.
8. Update the README and any TU-Berlin GitLab links (git remote already switched).

## Hardening, once it runs

- Pin `docker/login-action` and `softprops/action-gh-release` to full commit SHAs. They are the
  only third-party actions, and the `github` user is in the `docker` group, so a hijacked
  mutable tag is root on the lab box. Pair with an org Actions allowlist and a
  `.github/dependabot.yml` for the `github-actions` ecosystem.
- `actions/checkout`, `upload-artifact` and `download-artifact` are on v4; v5 exists. Bump
  deliberately after checking the runner's node version.

## Still open

- Org Actions policy: `GITHUB_TOKEN` allowed to elevate to `contents: write` (`publish`) and
  `packages: write` (`image`); fork-PR policy at the org level.
- `LICENSE` before wide announcement (repo is already public).
