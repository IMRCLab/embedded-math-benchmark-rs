# GitHub migration

Status: **in progress**. Repo moved from `git.tu-berlin.de` to
`github.com/IMRCLab/embedded-math-benchmark-rs` (public), CI from GitLab pipelines to GitHub
Actions. Hard cut-over, only the git repo carried over (history re-authored, content identical).

`.github/workflows/ci.yml` is the ported pipeline. This doc covers runner provisioning, repo
settings, and the remaining cut-over steps.

## Provisioning the runners

The one step that still needs hands-on work on the lab box: seven repo-scoped self-hosted runners on `mrslab`, under the existing `github` service account. Everything the admin runs is in the Runbook; the rest is background.

### Runbook

Lab admin on `mrslab`, from a root shell (`sudo -i`). Needs repo **Admin**, or a registration token from someone who has it.

1. Repo Settings, Actions, Runners, **New self-hosted runner**: copy the registration token (one token covers all seven runners, ~1h TTL).
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

3. Verify: `systemctl is-active 'actions.runner.*'` returns seven `active`, and all seven show **Idle** under repo Settings, Actions, Runners.

Creates `actions.runner.IMRCLab-embedded-math-benchmark-rs.mrsbench-*.service` x7, `User=github`, enabled at boot.

Notes:

- The pinned `2.337.0` is only the bootstrap version; the runner self-updates in place after that. Bump it in this doc when it drifts far enough that a fresh install matters.
- `svc.sh` sets no `Restart=`; a crashed runner stays down until restarted by hand, same as the org runner. Left as is.
- The `env PATH=...` on the `config.sh` call is load-bearing: `config.sh` bakes `$PATH` into `.path` and `runsvc.sh` uses it verbatim, and `secure_path` would otherwise drop `~/.cargo/bin`.
- `probe-rs` installs to `/home/github/.cargo/bin`; the `run-firmware` job prepends that via `$GITHUB_PATH`, so `ci.yml` needs no change. Flash runner dirs never run `container:` jobs, so they need nothing else.
- No `rm` between jobs: checkout jobs get a `git clean`; the artifact-only jobs (`run-host`, `run-firmware`, `publish`) never check out, so their workspace persists on a long-lived runner. Both `run-*` jobs keep logs under `$RUNNER_TEMP`, which the runner wipes at job start and end; without that a half-dead leg re-uploads stale logs for the profiles it never reached and `collect` merges them as fresh results.

### Background

Already on `mrslab` (Ubuntu 22.04): the lab already runs GitHub Actions self-hosted runners for other repos, so most of the groundwork exists.

| Thing                    | State                                                                                                                                                                                            |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `github` user (uid 1008) | service account for runners; groups `github`, `plugdev`, `docker`                                                                                                                                |
| Docker                   | 29.6.2 + buildx v0.35.0; `github` in `docker` group                                                                                                                                              |
| Probe access             | `github` in `plugdev`; udev rules installed (`69-probe-rs.rules`, `99-bitcraze.rules`, `99-uhubctl.rules`)                                                                                       |
| Cargo cache              | docker volume `mrs-cargo-cache`, carried over from GitLab, mounted at `/persist`                                                                                                                 |
| Probes attached          | J-Link `000801038809` (stm32), J-Link `000802013924` (nrf52840), RPi Debug Probe `E6654854571E7F26` (rp2040), RPi Debug Probe `E66548545758AD26` (rp2350), ESP32-S3 USB JTAG `74:4D:BD:95:A9:CC` |
| Existing org runner      | `actions.runner.IMRCLab.threadripper.service`, runs as `github`. Leave it running, it's unrelated to the new repo-scoped runners                                                                 |

Needs 7 new repo-scoped runners (registered against this repo, not the org, so no runner group
needed), all as the `github` service account, each in its own directory with its own `svc.sh`
service:

- **1x `build`**, labels `mrsbench,build`: runs the `container:` jobs (`build`, `check`,
  `run-host`, `collect`, `report`).
- **1x `hostjob`**, labels `mrsbench,hostjob`: runs `image` and `publish`, the two jobs that
  run on the host rather than in a container. Kept off the `build` runner because a container
  job writes the shared workspace as root, and a host job's `git clean` can't remove what root
  left behind; separate runners keep each `_work` single-owner.
- **5x `flash`**, `mrsbench-flash-1..5`, labels `mrsbench,flash`: a plain pool. GitHub hands
  each `run-firmware` matrix leg to whichever runner is idle; the job picks its board by probe
  serial (`--probe "$HIL_PROBE_<TARGET>"`), so leg-to-runner mapping doesn't matter. Add a 6th
  only once 5 boards becomes a bottleneck.

## Remaining steps

1. ~~`HIL_PROBE_*` variables set.~~ Done. Note `HIL_PROBE_RP2350` was copied without the `-0:`
   port segment the other Debug Probe var has; probe-rs matches on serial so it works,
   normalise if the rp2350 leg misbehaves.
2. ~~`git remote`: GitHub is `origin`, old TU-Berlin GitLab kept as `gitlab`, pre-migration
   history parked on local branch `gitlab-archive-main`.~~ Done.
3. Register the seven runners (above).
4. Repo settings:
   - Settings, Actions, General: uncheck "Run workflows from fork pull requests" (belt and
     braces: `ci.yml` triggers on `push`/`workflow_dispatch` only, never `pull_request`, so a
     fork triggers nothing either way).
   - Confirm the org lets `GITHUB_TOKEN` use `packages: write` (the `image` job pushes to
     ghcr) and `contents: write` (the `publish` job moves the `latest` release). If restricted,
     those two jobs 403 and the rest still runs.
   - Optional: branch protection requiring the `build` and `run-*` checks on `main`. `check`
     (fmt/clippy) is advisory today; making it required is a later call.
5. One green pipeline on a branch, then on `main`; check the release URL resolves (below).
6. `git rm -r .gitlab .gitlab-ci.yml` (the moved `Dockerfile.ci` stays under `.github/`).
7. Stop/deregister the GitLab HIL and build runners, or leave `gitlab-runner` installed and
   idle. Never drive both CI systems at the probes at once.
8. Rewrite the docs that describe GitLab CI, in the same commit as step 6:
   - [hil-setup.md](hil-setup.md): swap the GitLab runner provisioning for the setup above;
     keep the probe, udev, `uhubctl` and RP2350-rescue sections.
   - [adding_a_platform.md](adding_a_platform.md) step 4: `.github/workflows/ci.yml` matrix
     rows (`build`, `check`, `run-firmware`) + a `HIL_PROBE_<PLAT>` variable, instead of the
     `.gitlab/ci/` checklist.
   - [benchmark-viz.md](benchmark-viz.md): the report link and the `glab job artifact` line.
   - [running-benchmarks.md](running-benchmarks.md): drop the `glab` reference if present.
   - [benchmark.md](benchmark.md): the HIL-setup index line.
   - [CLAUDE.md](../CLAUDE.md): "GitLab HIL CI" wording, the State section.
   - [draft/rp2350-riscv-design.md](draft/rp2350-riscv-design.md): the CI section talks in
     GitLab job/`needs:` terms.
9. Update the README and any remaining TU-Berlin GitLab links.

## Still open (blocking, not ours to resolve)

- Org Actions policy: does `GITHUB_TOKEN` get `contents: write` / `packages: write`? Org-level
  fork-PR policy?
- `LICENSE` file, before wide announcement (repo is already public).

## Design reference

### Pipeline mapping

| GitLab                                             | GitHub Actions                                         | Notes                                |
| -------------------------------------------------- | ------------------------------------------------------ | ------------------------------------ |
| `image` stage, kaniko, `rules: changes:`           | `image` job, `docker build`, content-hash tag          | no kaniko: the runner can run docker |
| `.cargo-build` + `parallel: matrix`                | one `build-<target>` job per target, calling the reusable `build-target.yml` (matrix leg per profile) | `cargo make` still owns the flags |
| `check` stage, `allow_failure: true`               | `check` job, `continue-on-error: true`                 |                                      |
| `.firmware-flash`, `hil` tag, `GIT_STRATEGY: none` | one `run-firmware-<target>` job per board, calling the reusable `flash-target.yml` | `flash` runners, no checkout |
| `run-host`                                         | `run-host` job, in the container                       | glibc parity with the build          |
| `collect-results`, optional `needs:`               | `collect` job, `if: ${{ !cancelled() }}`               | tolerates failed flash legs          |
| `report-pdf`, `allow_failure: true`                | `report` job, `continue-on-error: true`                |                                      |
| `expire_in: 1 year` artifact                       | `retention-days: 90` (public repo max) + `publish` job | see below                            |

The DAG is the same shape: builds fan out, each flash leg starts when its own target's build
finishes, `collect` fans in, `report` and `publish` follow.

- **`build` and `run-firmware` are split one job per target**, not one matrix job for every
  target, because GitHub's `needs:` is per job, not per matrix leg: a single matrixed `build`
  job would make `run-firmware-stm32` wait for the esp32s3 build too. Splitting means
  `run-firmware-<target>` needs only `build-<target>`, so a board starts flashing as soon as its
  own target is built, in parallel with every other target still building on the single `build`
  runner. Each job's actual steps live in one place, the reusable workflows `build-target.yml`
  and `flash-target.yml`; `ci.yml` itself only carries a short per-target call site.
- **A new push cancels the previous run on the same ref** (`concurrency: group: ci-${{
  github.ref }}, cancel-in-progress: true` at the top of `ci.yml`), the usual "latest push
  wins" behaviour. Cross-ref board safety is a separate, narrower guarantee: `flash-target.yml`
  gives its `flash` job its own `concurrency: group: flash-<target>` with `cancel-in-progress:
  false`, so two refs flashing the same board queue instead of racing, without blocking either
  ref's unrelated build or check work. `cancel-in-progress: false` matters here specifically
  because it only ever drops a still-pending job, never one already flashing.
- **Every `if:` that isn't already a status-check function needs `!cancelled()` added.** An `if:`
  without `success()`, `failure()`, `cancelled()` or `always()` gets an implicit `&& success()`
  from GitHub, and that `success()` checks the job's *entire* ancestor chain, not just its own
  `needs:`. `report` (no `if:` at all) and `publish` (`needs.collect.result == 'success'` with no
  status function) both learned this the hard way: an unrelated `build-esp32s3` failure silently
  skipped both, despite neither depending on it. `check`, `run-host`, `collect` and
  `run-firmware-<target>` already start their conditions with `!cancelled()` for this reason;
  any new job with a custom `if:` needs the same.

### The `container:` model

Every `build`, `check`, `run-host`, `collect` and `report` job runs inside
`ghcr.io/imrclab/embedded-math-benchmark-rs/ci` (the `image` job lowercases `github.repository`
for the tag), the GitHub equivalent of GitLab's Docker executor. The image is
`.github/docker/Dockerfile.ci` (moved from `.gitlab/ci/`, still built by GitLab too until
cut-over). The `image` job tags it with a 16-char hash of the Dockerfile, so an unchanged
Dockerfile is a fast no-op and every pipeline builds against the image matching that commit.

A container job runs with `HOME=/github/home`, not `/root` as GitLab's Docker executor left it,
and the runner forces that regardless of what `container.env` or job `env:` sets (confirmed on
a live run: setting `HOME` there is a no-op). `espup` installs `export-esp.sh` into `/root`, so
the esp32s3 tasks read `${ESP_ENV_SCRIPT:-$HOME/export-esp.sh}` and CI sets `ESP_ENV_SCRIPT`
explicitly. Local runs are unaffected.

`mrs-cargo-cache:/persist` is mounted into each container, with `CARGO_HOME`,
`CARGO_TARGET_DIR` and the `uv` caches pointed at it, exactly as the GitLab pipeline did.

A second `build` runner wouldn't help: it would share `/persist/target`, and cargo holds an
exclusive lock on the target dir, so the second leg would block on the lock instead of
building. Giving it a separate `CARGO_TARGET_DIR` would fix that, at the cost of a second cold
cargo cache on an already full disk.

### Artifacts and the report link

- Every run uploads its artifacts at `retention-days: 90` (the public-repo maximum). Grab them
  from the run page or `gh run download`.
- `main` runs also push `report.pdf`, `results.csv`, `accuracy_results.csv/json` to a rolling
  `latest` prerelease. Release assets never expire, and the tag-addressed URL is stable:
  - `https://github.com/IMRCLab/embedded-math-benchmark-rs/releases/download/latest/report.pdf`
  - Not `/releases/latest/download/...`: that resolves the newest **non**-prerelease release
    and 404s here. Drop `prerelease: true` in `ci.yml` if you want that form instead.

  This replaces the GitLab "latest main" raw-artifact link in
  [benchmark-viz.md](benchmark-viz.md).

## Hardening, once it runs

- Pin `docker/login-action` and `softprops/action-gh-release` to full commit SHAs. They are the
  only third-party actions, and the `github` user is in the `docker` group, so a hijacked
  mutable tag is root on the lab box. Pair with an org Actions allowlist and a
  `.github/dependabot.yml` for the `github-actions` ecosystem.
- `actions/checkout`, `upload-artifact` and `download-artifact` are on v4; v5 exists. Bump
  deliberately after checking the runner's node version.
