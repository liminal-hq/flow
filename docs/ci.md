# CI Pipeline

This document describes the continuous integration pipeline for `liminal-hq/flow`.

## Overview

The CI workflow (`.github/workflows/ci.yml`) runs on every pull request targeting `main`, every push to `main`, and on manual dispatch. It validates formatting, linting, and tests in parallel, then posts a summary comment on PRs.

All jobs use `ubuntu-24.04` runners with `dtolnay/rust-toolchain` pinned to the workspace's `rust-version` (currently **1.94**). There is no container — flow is a pure Rust workspace with no system-library dependencies.

## Jobs

| Job | Purpose | Key command |
|---|---|---|
| **Format** | Enforce consistent code style | `cargo fmt --all -- --check` |
| **Clippy** | Catch lint warnings and common mistakes | `cargo clippy --workspace --all-targets -- -D warnings` |
| **Test** | Run the full test suite with JUnit output | `cargo nextest run --workspace --profile ci` |
| **PR summary** | Post a bot comment with per-job pass/fail | `actions/github-script` (PR only, non-fork) |

Format runs without caching (no compilation needed). Clippy and Test both use `swatinem/rust-cache@v2`.

## Test reporting

The Test job produces JUnit XML via nextest's CI profile (`.config/nextest.toml`), then:

1. Uploads `target/nextest/ci/junit.xml` as a build artefact
2. Publishes results via `EnricoMi/publish-unit-test-result-action@v2`

Each job also writes a step summary to `GITHUB_STEP_SUMMARY`.

## Concurrency

A `concurrency` block (`ci-${{ github.ref }}`) cancels in-progress runs when a new commit is pushed to the same branch, avoiding wasted runner time during rapid iteration.

## Supporting files

| File | Purpose |
|---|---|
| `.config/nextest.toml` | Configures the `ci` profile with JUnit XML output (`path = "junit.xml"`) |
| `.node-version` | Sets Node 24 for JavaScript-based GitHub Actions, avoiding the `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` env hack |

## Design decisions

### Bare runners, no container

The shared org image (`ghcr.io/liminal-hq/tauri-ci-desktop:latest`) bundles Tauri/GTK/Node dependencies that flow doesn't need and pins an older Rust version. Bare `ubuntu-24.04` with `dtolnay/rust-toolchain` is lighter and already proven in `release.yml`.

### Rust version pinned to workspace `rust-version`

`dtolnay/rust-toolchain` is configured with `toolchain: "1.94"` matching `Cargo.toml`'s `rust-version` field. Bumping the minimum supported version is a single-file change.

### nextest over `cargo test`

Faster parallel execution, built-in JUnit XML output, and better failure reporting. Installed via `taiki-e/install-action@nextest` (pre-built binary, no compilation).

## Deferred

These tools are not yet integrated but are candidates for future iterations:

- **`cargo audit` / `cargo deny`** — Dependency vulnerability and licence scanning. Worth adding once the advisory database is clean for our dependency tree.
- **`cargo llvm-cov`** — Test coverage reporting. Useful but heavyweight for the current project size.

## Release workflow

`release.yml` is separate and intentionally uses `ubuntu-22.04` / `ubuntu-22.04-arm` runners for older glibc compatibility. It is unrelated to the CI pipeline.

## Branch protection

The `format`, `clippy`, and `test` status checks should be required on `main` to prevent merging broken code. See issue #41 for setup steps.
