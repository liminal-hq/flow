#!/usr/bin/env bash
# Update release-facing version references before tagging a new Flow release
#
# (c) Copyright 2026 Liminal HQ, Scott Morris
# SPDX-License-Identifier: MIT

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RED=""
GREEN=""
YELLOW=""
BLUE=""
BOLD=""
RESET=""

if [[ -t 1 ]]; then
	RED="$(printf '\033[31m')"
	GREEN="$(printf '\033[32m')"
	YELLOW="$(printf '\033[33m')"
	BLUE="$(printf '\033[34m')"
	BOLD="$(printf '\033[1m')"
	RESET="$(printf '\033[0m')"
fi

usage() {
	cat <<'USAGE'
Usage: scripts/prepare-release-version.sh --version <version> [--dry-run]

Options:
  --version <version>   New release version, with or without a leading `v`
  --dry-run             Show planned changes without writing files
  -h, --help            Show this help

This script updates release-facing version references and prepares the repo for
review before a release tag is created on `main`.
USAGE
}

info() {
	printf '%b\n' "${BLUE}${1}${RESET}"
}

success() {
	printf '%b\n' "${GREEN}${1}${RESET}"
}

warn() {
	printf '%b\n' "${YELLOW}${1}${RESET}"
}

fail() {
	printf '%b\n' "${RED}${1}${RESET}" >&2
	exit 1
}

require_clean_repo() {
	if ! git -C "${REPO_ROOT}" diff --quiet || ! git -C "${REPO_ROOT}" diff --cached --quiet; then
		fail "Working tree has tracked changes. Commit or stash them before running this script."
	fi
}

current_workspace_version() {
	sed -n 's/^version = "\([^"]*\)"/\1/p' "${REPO_ROOT}/Cargo.toml" | head -n 1
}

replace_in_file() {
	local file="$1"
	local from="$2"
	local to="$3"

	perl -0pi -e "s/\\Q${from}\\E/${to}/g" "${file}"
}

VERSION_INPUT=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
	case "$1" in
		--version)
			VERSION_INPUT="${2:-}"
			shift 2
			;;
		--dry-run)
			DRY_RUN=true
			shift
			;;
		-h|--help)
			usage
			exit 0
			;;
		*)
			fail "Unknown option: $1"
			;;
	esac
done

if [[ -z "${VERSION_INPUT}" ]]; then
	fail "Missing required option: --version"
fi

if [[ ! "${VERSION_INPUT}" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	fail "Version must look like 0.0.3 or v0.0.3"
fi

require_clean_repo

CURRENT_VERSION="$(current_workspace_version)"
if [[ -z "${CURRENT_VERSION}" ]]; then
	fail "Could not determine the current workspace version from Cargo.toml"
fi

NEW_VERSION="${VERSION_INPUT#v}"
CURRENT_TAG="v${CURRENT_VERSION}"
NEW_TAG="v${NEW_VERSION}"

if [[ "${CURRENT_VERSION}" == "${NEW_VERSION}" ]]; then
	fail "Version is already ${NEW_VERSION}"
fi

FILES=(
	"${REPO_ROOT}/Cargo.toml"
	"${REPO_ROOT}/Cargo.lock"
	"${REPO_ROOT}/README.md"
	"${REPO_ROOT}/docs/release/distribution-strategy.md"
)

info "${BOLD}Preparing release version bump${RESET}"
printf '  from %b%s%b to %b%s%b\n' "${YELLOW}" "${CURRENT_VERSION}" "${RESET}" "${GREEN}" "${NEW_VERSION}" "${RESET}"

for file in "${FILES[@]}"; do
	if [[ ! -f "${file}" ]]; then
		fail "Expected file not found: ${file}"
	fi
done

if [[ "${DRY_RUN}" == true ]]; then
	warn "Dry run only. No files will be changed."
	for file in "${FILES[@]}"; do
		printf '  would update %s\n' "${file#${REPO_ROOT}/}"
	done
	exit 0
fi

replace_in_file "${REPO_ROOT}/Cargo.toml" "version = \"${CURRENT_VERSION}\"" "version = \"${NEW_VERSION}\""
replace_in_file "${REPO_ROOT}/Cargo.lock" "version = \"${CURRENT_VERSION}\"" "version = \"${NEW_VERSION}\""
replace_in_file "${REPO_ROOT}/README.md" "${CURRENT_TAG}" "${NEW_TAG}"
replace_in_file "${REPO_ROOT}/docs/release/distribution-strategy.md" "${CURRENT_TAG}" "${NEW_TAG}"
replace_in_file "${REPO_ROOT}/docs/release/distribution-strategy.md" "${CURRENT_VERSION}" "${NEW_VERSION}"

success "Updated release version references in:"
for file in "${FILES[@]}"; do
	printf '  %b- %s%b\n' "${GREEN}" "${file#${REPO_ROOT}/}" "${RESET}"
done

warn "Next steps:"
printf '  1. review the diff\n'
printf '  2. run cargo checks\n'
printf '  3. merge the PR to main\n'
printf '  4. create tag %b%s%b on main\n' "${BOLD}" "${NEW_TAG}" "${RESET}"
