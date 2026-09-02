# GitHub migration

Status: **planned**. Moving the repo from `git.tu-berlin.de` to a public GitHub repo (most
likely `github.com/IMRCLab/mrs-microbenchmarks`) and the CI from GitLab pipelines to GitHub
Actions. Hard cut-over, no history of issues/MRs carried over, only the git repo.

`.github/workflows/ci.yml` is the ported pipeline. This doc is the plan around it: runner
setup, repo settings, and the cut-over checklist.

## Already provisioned on the lab box (`mrslab`, Ubuntu 22.04)

The lab already runs GitHub Actions self-hosted runners, so most of the groundwork exists.

| Thing | State |
| --- | --- |
| `github` user (uid 1008) | groups `github`, `plugdev`, `docker`; has an `/etc/subuid` range |
| Runner service | `actions.runner.IMRCLab.threadripper.service`, org-level runner for `IMRCLab`, runs as `github` |
| Docker | 29.6.2 + buildx v0.35.0; `github` in the `docker` group, so `container:` jobs and `docker build` both work with no extra privilege |
| probe access | `plugdev` + udev rules already installed (`69-probe-rs.rules`, `99-bitcraze.rules`, `99-uhubctl.rules`) |
| Persistent cargo cache | docker volume `mrs-cargo-cache` (carried over from GitLab, mounted at `/persist`) |
| Probes attached | 2x J-Link, 2x RPi Debug Probe (RP2040 + RP2350, shared VID:PID), ESP32-S3 USB JTAG |

`gitlab-runner` (uid 999) is **not** in the `docker` group and does not need to be: its systemd
unit has no `User=`, so the daemon runs as root and only drops to `gitlab-runner` for shell
steps. The GitHub runner runs as `User=github`, which is why `github` is in `docker`.

## Runner topology

GitHub Actions has no "executor with concurrency N". One runner process runs one job at a time;
parallelism means more runner processes. So the GitLab model maps as:

| GitLab | GitHub | Labels |
| --- | --- | --- |
| Docker executor, custom image | N runner instances, `container:` jobs | `self-hosted, mrs, build` |
| one `hil` shell runner, several boards | one runner instance per board | `self-hosted, mrs, flash` |

`ci.yml` expects:

- **1x `build`** runner (start here, add more later). Runs `image`, `build`, `check`,
  `run-host`, `collect`, `report`, `publish`.
- **5x `flash`** runners (`stm32`, `nrf52840`, `rp2040`, `rp2350`, `esp32s3`). GitHub spreads
  the `run-firmware` matrix legs across whichever `flash` runners are free, so boards flash in
  parallel. Add one per new board (rp2350-riscv).

All run as the `github` user, each in its own directory with its own service. The lab already
runs several runners side by side, so this is a known pattern.

Keep them in a **runner group** scoped to `mrs-microbenchmarks` (org Settings, Actions, Runner
groups) so other `IMRCLab` repos cannot grab the `flash` runners and vice versa. This is the
equivalent of GitLab's tag scoping.

### Provisioning (run as `github` on `mrslab`)

Per runner, in its own directory:

```bash
mkdir ~/runner-build && cd ~/runner-build
curl -o actions-runner.tar.gz -L https://github.com/actions/runner/releases/latest/download/actions-runner-linux-x64.tar.gz
tar xzf actions-runner.tar.gz
./config.sh --url https://github.com/IMRCLab/mrs-microbenchmarks \
  --token <REG_TOKEN> --name threadripper-build --labels mrs,build \
  --runnergroup mrs-microbenchmarks --unattended --replace
sudo ./svc.sh install github && sudo ./svc.sh start
```

`<REG_TOKEN>` comes from the repo or org Settings, Actions, Runners, "New self-hosted runner".
Repeat with `--name threadripper-flash-stm32 --labels mrs,flash` and so on for the flash
runners (their directories can skip the container toolchain, they only run `probe-rs`).

One-time, also as `github`:

```bash
cargo install probe-rs-tools --locked   # flash runners
docker volume create mrs-cargo-cache    # no-op, it already exists from GitLab
```

The `run-firmware` job prepends `~/.cargo/bin` to `PATH` itself (via `$GITHUB_PATH`). To drop
that step, put `~/.cargo/bin` on the flash runner's `PATH` (its `.env` file or the `github`
shell profile) instead.

Check the `mrs` label is not already claimed by another `IMRCLab` runner. If it is, pick
another (e.g. `mrsbench`) and update `runs-on:` in `ci.yml`.

No `rm` cleanup between jobs. Checkout jobs get a `git clean`; the artifact-only jobs
(`run-host`, `run-firmware`, `publish`) never check out, so their workspace persists across
runs on a long-lived runner. Both `run-*` jobs therefore write their logs under `$RUNNER_TEMP`,
which the runner empties at the start and end of every job. Without that, a leg that dies
part-way re-uploads the previous run's logs for the profiles it never reached, and `collect`
merges them as fresh results.

## GitHub repo setup

1. **Create** `github.com/IMRCLab/mrs-microbenchmarks`, public. Push `main` + tags. Submodules
   already resolve to `github.com` (`crazyflie-firmware`, `CMSIS-DSP`).
2. **Variables** (Settings, Secrets and variables, Actions, Variables), not secrets, they are
   USB probe selectors from `probe-rs list`:
   - `HIL_PROBE_STM32`, `HIL_PROBE_NRF52840`, `HIL_PROBE_RP2040`, `HIL_PROBE_RP2350`,
     `HIL_PROBE_ESP32S3`
3. **Fork PRs**: Settings, Actions, General, uncheck "Run workflows from fork pull requests".
   `ci.yml` triggers on `push` and `workflow_dispatch` only, never `pull_request`, so a fork
   triggers nothing. Branches pushed to the repo run the full pipeline.
4. **ghcr package**: nothing to do. The container jobs pull with `credentials:` and the
   run's `GITHUB_TOKEN`, so a private package works. Set the `ci` package to **Public** (repo,
   Packages) only if you want to `docker pull` it by hand off CI.
5. **Branch protection** (optional): require the `build` and `run-*` checks on `main`. `check`
   (fmt/clippy) is advisory today; making it required is a later call.

## Pipeline mapping

| GitLab | GitHub Actions | Notes |
| --- | --- | --- |
| `image` stage, kaniko, `rules: changes:` | `image` job, `docker build`, content-hash tag | no kaniko: the runner can run docker |
| `.cargo-build` + `parallel: matrix` | `build` job, one matrix leg per (target, profile) | `cargo make` still owns the flags |
| `check` stage, `allow_failure: true` | `check` job, `continue-on-error: true` | |
| `.firmware-flash`, `hil` tag, `GIT_STRATEGY: none` | `run-firmware` job, `flash` runners, no checkout | one matrix leg per board |
| `run-host` | `run-host` job, in the container | glibc parity with the build |
| `collect-results`, optional `needs:` | `collect` job, `if: ${{ !cancelled() }}` | tolerates failed flash legs |
| `report-pdf`, `allow_failure: true` | `report` job, `continue-on-error: true` | |
| `expire_in: 1 year` artifact | `retention-days: 90` (public repo max) + `publish` job | see below |

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
`ghcr.io/IMRCLab/mrs-microbenchmarks/ci`,
the GitHub equivalent of GitLab's Docker executor. The image is `.github/docker/Dockerfile.ci`
(moved from `.gitlab/ci/`, still built by GitLab too until cut-over). The `image` job tags it
with a 16-char hash of the Dockerfile, so an unchanged Dockerfile is a fast no-op and every
pipeline builds against the image matching that commit.

A container job runs with `HOME=/github/home`, not `/root` as GitLab's Docker executor left
it. `espup` installs `export-esp.sh` into `/root`, so the esp32s3 tasks read
`${ESP_ENV_SCRIPT:-$HOME/export-esp.sh}` and CI sets `ESP_ENV_SCRIPT`. Local runs are unaffected.

`mrs-cargo-cache:/persist` is mounted into each container, with `CARGO_HOME`,
`CARGO_TARGET_DIR` and the `uv` caches pointed at it, exactly as the GitLab pipeline did. One
`build` runner means builds serialize and the shared `/persist/target` is safe; when you add
`build` runners on the same host they share that volume and cargo serializes them on a file
lock (correct, mildly slower).

### Artifacts and the report link

- Every run uploads its artifacts at `retention-days: 90` (the public-repo maximum). Grab them
  from the run page or `gh run download`.
- `main` runs also push `report.pdf`, `results.csv`, `accuracy_results.csv/json` to a rolling
  `latest` prerelease. Release assets never expire, and the tag-addressed URL is stable:
  - `https://github.com/IMRCLab/mrs-microbenchmarks/releases/download/latest/report.pdf`
  - Not `/releases/latest/download/...`: that resolves the newest **non**-prerelease release
    and 404s here. Drop `prerelease: true` in `ci.yml` if you want that form instead.

  This replaces the GitLab "latest main" raw-artifact link in
  [benchmark-viz.md](../benchmark-viz.md).

## Cut-over checklist

1. Runners registered and idle in the repo/org (see them under Settings, Actions, Runners).
2. `HIL_PROBE_*` variables set.
3. Add a `LICENSE` before the repo goes public.
4. One green pipeline on a branch, then on `main`, then check the release URL above resolves.
5. `git rm -r .gitlab .gitlab-ci.yml` (the moved `Dockerfile.ci` stays under `.github/`).
6. Stop / deregister the GitLab HIL and build runners, or leave `gitlab-runner` installed and
   idle. Do not drive both CI systems at the probes at once.
7. Rewrite the docs that describe GitLab CI, in the same commit as step 5:
   - [hil-setup.md](../hil-setup.md): replace the GitLab runner provisioning with the runner
     setup above; keep the probe, udev, `uhubctl` and RP2350-rescue sections.
   - [adding_a_platform.md](../adding_a_platform.md) step 4: `.github/workflows/ci.yml` matrix
     rows (`build`, `check`, `run-firmware`) + a `flash` runner + a `HIL_PROBE_<PLAT>` variable,
     instead of the `.gitlab/ci/` checklist.
   - [benchmark-viz.md](../benchmark-viz.md): the report link and the `glab job artifact` line
     (use `gh run download` or the release URL).
   - [running-benchmarks.md](../running-benchmarks.md): drop the `glab` reference if present.
   - [benchmark.md](../benchmark.md): the HIL-setup index line.
   - [CLAUDE.md](../../CLAUDE.md): "GitLab HIL CI" wording, the State section.
   - [draft/rp2350-riscv-design.md](rp2350-riscv-design.md): the CI section talks in GitLab
     job/`needs:` terms.
8. Update the git remote, README, and any TU-Berlin GitLab links.

## Hardening, once it runs

- Pin `docker/login-action` and `softprops/action-gh-release` to full commit SHAs. They are the
  only third-party actions, and the `github` user is in the `docker` group, so a hijacked
  mutable tag is root on the lab box. Pair with an org Actions allowlist and a
  `.github/dependabot.yml` for the `github-actions` ecosystem.
- `actions/checkout`, `upload-artifact` and `download-artifact` are on v4; v5 exists. Bump
  deliberately after checking the runner's node version.

## Deferred, confirm once the repo exists

- Repo really lands in the `IMRCLab` org (org runner only serves that org).
- The single `threadripper` org runner's labels and whether other repos share it (drove the
  runner-group decision).
- Org Actions policy: `GITHUB_TOKEN` allowed to elevate to `contents: write` (`publish`) and
  `packages: write` (`image`); fork-PR policy at the org level.
- Whether to add `build` runners now or live with serial builds.
