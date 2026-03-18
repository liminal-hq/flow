# Flow distribution strategy

This document sketches the first public release plan for Liminal Flow on `main`. It captures the current agreed release shape for `v0.0.4` and should evolve as the release automation and packaging work lands.

## Goals

- Ship `flo` as a Linux-first terminal application.
- Follow the broad SMDU release flow: tagged GitHub Releases, generated release notes, attached artefacts, and checksums.
- Start with `amd64` and `arm64` Linux support only.
- Keep the first release operationally simple enough to rehearse and repeat.

## Release scope

The first release should target:

- Linux `amd64`
- Linux `arm64`

The first release should publish:

- standalone `flo` binaries
- compressed release archives
- Linux package artefacts:
  - `.deb`
  - `.rpm`
- SHA256 checksum files for every published artefact
- generated GitHub release notes

The first release should not target yet:

- macOS
- Windows
- crates.io publication
- a separate install script

## Why this shape fits Flow

Flow already behaves like a product binary rather than a crate intended for library consumers:

- the primary deliverable is the `flo` CLI/TUI binary
- the CLI crate already generates man pages during build
- the README and SPEC already describe Flow as a local-first terminal tool

That makes a GitHub Releases-first approach the cleanest fit for `v0.0.4`.

## Proposed release flow

The release flow should mirror SMDU closely:

1. Merge the release-ready PR into `main`.
2. Run `scripts/prepare-release-version.sh --version <next-version>` in a clean working tree to create a release-bump branch and update release-facing version references before tagging.
3. Confirm release notes and docs reflect the merged behaviour.
4. Create a tag such as `v0.0.4`.
5. Let GitHub Actions build Linux artefacts for both supported architectures.
6. Create or update the GitHub Release for that tag.
7. Attach binaries, packages, tarballs, and checksum files.
8. Publish generated release notes, then do a quick install smoke test from the uploaded assets.

Manual dispatch should also be available so a tag can be rebuilt or a draft release can be prepared before publication. When the manual `release_tag` input is left blank, the workflow should derive `v<workspace version>` from `Cargo.toml` and then validate the resolved tag before continuing.

## Version prep

Before tagging a release, use:

```bash
scripts/prepare-release-version.sh --version 0.0.4
```

By default, this creates and switches to a branch named `chore/release-v0.0.4` before updating files. You can override that with:

```bash
scripts/prepare-release-version.sh --version 0.0.4 --branch chore/my-custom-release-branch
```

The script updates release-facing version references in:

- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `docs/release/distribution-strategy.md`

It expects a clean working tree and is designed to prepare a branch that will be reviewed and merged before the final tag is created on `main`.

## GitHub Actions shape

Flow should add a release workflow under `.github/workflows/release.yml` with two jobs:

- `prepare-release`
  - resolve the release tag
  - create or reuse a GitHub Release
  - enable generated release notes
  - write a concise summary to `$GITHUB_STEP_SUMMARY`
- `build-linux`
  - build release artefacts for a matrix of Linux targets
  - restore and save Cargo build caches
  - generate checksum files
  - upload the artefacts to the release
  - write a concise summary to `$GITHUB_STEP_SUMMARY`

Recommended matrix for the first pass:

| runner | target triple | target label | architecture |
|---|---|---|---|
| `ubuntu-22.04` | `x86_64-unknown-linux-gnu` | `linux-amd64` | `amd64` |
| `ubuntu-22.04-arm` | `aarch64-unknown-linux-gnu` | `linux-arm64` | `arm64` |

The workflow should pin fixed GitHub-hosted runner images rather than using `-latest` aliases so the release environment remains predictable over time.

For GNU-linked releases, the runner image should track the oldest Ubuntu/glibc baseline we intend to support. Building on Ubuntu 22.04 keeps the packaged `flo` binary compatible with Ubuntu 22.04 and common WSL2 installs that still provide glibc 2.35.

Recommended workflow details:

- use `dtolnay/rust-toolchain` to install the release toolchain explicitly
- use `Swatinem/rust-cache` to cache Cargo registry, git dependencies, and `target/`
- allow manual dispatches to omit `release_tag`, derive `v<workspace version>` from `Cargo.toml`, and validate the final tag in the workflow
- write per-job summaries showing:
  - resolved tag and release name
  - runner image and target triple
  - generated artefacts
  - checksum output
  - upload status

## Artefacts

Release asset naming should stay predictable and match the SMDU style:

- `flo-<tag>-linux-amd64`
- `flo-<tag>-linux-arm64`
- `flo-<tag>-linux-amd64.tar.gz`
- `flo-<tag>-linux-arm64.tar.gz`
- `flo-<tag>-linux-amd64.deb`
- `flo-<tag>-linux-arm64.deb`
- `flo-<tag>-linux-amd64.rpm`
- `flo-<tag>-linux-arm64.rpm`
- corresponding `.sha256` files for every artefact above

Each archive should include:

- the `flo` executable
- `flo.1`
- subcommand man pages generated from the CLI build
- a short `README` or install note if packaging needs one

Tarballs should unpack into a conventional prefix-friendly layout so manual installation is straightforward:

- `bin/flo`
- `share/man/man1/flo.1.gz`
- `share/man/man1/flo-*.1.gz`

## Packaging expectations

Linux packaging should install files into conventional package-managed locations:

- binary: `/usr/bin/flo`
- man pages: `/usr/share/man/man1/*.1.gz`

Manual archive installs can continue to use `/usr/local` or another user-managed prefix, but package installs should prefer distro-managed paths.

The first release should ship both package artefacts and tarballs together.

## Linux package metadata

The first release should keep Linux package metadata conservative and easy to review. Proposed package identity:

- package name: `flo`
- display name: `Liminal Flow`
- version: match the Git tag without the leading `v`, for example `0.0.4`
- licence: `MIT`
- vendor: `Liminal HQ`
- maintainer: `Liminal HQ <contact@liminalhq.ca>`
- homepage: `https://github.com/liminal-hq/flow`
- architecture mapping:
  - Debian: `amd64`, `arm64`
  - RPM: `x86_64`, `aarch64`

Proposed one-line summary:

- `Terminal-native working-memory sidecar for developers`

Proposed longer description:

- `Liminal Flow is a local-first terminal application for tracking active work, branching into sub-tasks, and preserving context across CLI and TUI workflows.`

Suggested Debian control fields for review:

```debcontrol
Package: flo
Version: 0.0.4
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Liminal HQ <contact@liminalhq.ca>
Homepage: https://github.com/liminal-hq/flow
Depends: libc6
Description: Terminal-native working-memory sidecar for developers
 Liminal Flow is a local-first terminal application for tracking active work,
 branching into sub-tasks, and preserving context across CLI and TUI workflows.
```

Suggested RPM spec metadata for review:

```spec
Name:           flo
Version:        0.0.4
Release:        1%{?dist}
Summary:        Terminal-native working-memory sidecar for developers
License:        MIT
URL:            https://github.com/liminal-hq/flow
Vendor:         Liminal HQ
Packager:       Liminal HQ <contact@liminalhq.ca>
BuildArch:      x86_64

%description
Liminal Flow is a local-first terminal application for tracking active work,
branching into sub-tasks, and preserving context across CLI and TUI workflows.
```

Initial dependency stance:

- prefer GNU-linked builds for `v0.0.4`
- keep runtime dependency declarations minimal and conventional
- Debian packages should declare `Depends: libc6` at minimum for GNU-linked binaries
- release binaries should be built on Ubuntu 22.04 runners so the effective glibc baseline remains compatible with Ubuntu 22.04
- RPM packages can lean on `rpmbuild` auto-detection for shared library requirements unless testing shows gaps
- defer musl or fully static builds until after the first release so the packaging and support surface stays smaller

Man page packaging expectations:

- compress `flo.1` and subcommand man pages with `gzip -n`
- install them under `/usr/share/man/man1/`
- ship all generated subcommand man pages in both packages and tarballs
- keep filenames aligned with the generated CLI output:
  - `flo.1.gz`
  - `flo-now.1.gz`
  - `flo-branch.1.gz`
  - `flo-back.1.gz`
  - `flo-park.1.gz`
  - `flo-note.1.gz`
  - `flo-where.1.gz`
  - `flo-resume.1.gz`
  - `flo-pause.1.gz`
  - `flo-done.1.gz`
  - `flo-archive.1.gz`
  - `flo-list.1.gz`

## Documentation updates required before release

Before the first tagged release, update:

- `README.md`
  - add Linux installation instructions from release artefacts
  - document package and archive install paths
  - add uninstall guidance
- `SPEC.md`
  - define canonical Linux install paths
  - define release packaging expectations
  - define the initial platform support statement

## Cargo metadata to tighten up

Before release, confirm or add the following package metadata where appropriate:

- `repository`
- `homepage`
- `documentation`
- `rust-version`

Flow should be treated as a binary product for now, so internal crates should remain unpublished by setting `publish = false` on non-distribution crates.

Sample workspace metadata update:

```toml
[workspace.package]
version = "0.0.4"
edition = "2021"
license = "MIT"
authors = ["Liminal HQ", "Scott Morris"]
repository = "https://github.com/liminal-hq/flow"
homepage = "https://github.com/liminal-hq/flow"
documentation = "https://github.com/liminal-hq/flow#readme"
rust-version = "1.94"
```

Sample crate-level publication guard for internal crates:

```toml
[package]
name = "liminal-flow-core"
publish = false
```

## Suggested `v0.0.4` checklist

- [ ] Add `.github/workflows/release.yml`
- [ ] Add `.github/release.yml` for changelog category mapping
- [ ] Add Linux packaging scripts for `.deb` and `.rpm`
- [ ] Produce compressed Linux archives that include binary and man pages
- [ ] Add workflow caching for Cargo dependencies and build outputs
- [ ] Add step summaries for release preparation and Linux builds
- [ ] Document install and uninstall flows in `README.md`
- [ ] Extend `SPEC.md` with Linux distribution expectations
- [ ] Confirm Cargo package metadata
- [ ] Mark internal crates `publish = false` if crates.io publication remains out of scope
- [ ] Run release verification locally:
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- [ ] Create a draft `v0.0.4` release and smoke-test the uploaded artefacts on both Linux architectures

## Why not an install script

A separate install script could:

- detect `amd64` versus `arm64`
- download the matching release artefact
- verify the SHA256 checksum
- unpack the binary and man pages
- copy files into a prefix such as `/usr/local`

That could be convenient, but it also adds maintenance overhead, more surface area to test, and another trust path for users. For `v0.0.4`, package artefacts and documented manual archive extraction are the simpler and more reliable release story.

## Immediate next step

Once the current PR is merged, update this document so the release notes, command list, and packaging expectations match the merged state exactly. After that, the next practical step is scaffolding the GitHub release workflow and Linux packaging scripts.
