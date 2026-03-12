#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

VERSION=""
ARCH_INPUT=""
BINARY_PATH=""
MAN_DIR=""
OUTPUT_PREFIX=""

usage() {
	cat <<'USAGE'
Usage: scripts/build-release-archive.sh [options]

Options:
  --version <version>         Release version or tag (for example, v0.0.1)
  --arch <x64|arm64>          Target architecture
  --binary <path>             Built binary path
  --man-dir <path>            Directory containing generated man pages
  --output-prefix <prefix>    Output file prefix (without extension)
  -h, --help                  Show this help
USAGE
}

write_sha256_file() {
	local input_file="$1"
	local output_file="$2"

	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "${input_file}" > "${output_file}"
		return
	fi

	if command -v shasum >/dev/null 2>&1; then
		shasum -a 256 "${input_file}" > "${output_file}"
		return
	fi

	echo "A SHA-256 utility is required (sha256sum or shasum)." >&2
	exit 1
}

normalise_arch() {
	case "$1" in
		x64 | amd64 | x86_64)
			echo "x64"
			;;
		arm64 | aarch64)
			echo "arm64"
			;;
		*)
			echo "Unsupported architecture: $1" >&2
			exit 1
			;;
	esac
}

discover_man_dir() {
	local binary_path="$1"
	local release_dir
	release_dir="$(cd "$(dirname "${binary_path}")" && pwd)"

	find "${release_dir}/build" -type d -path '*/out/man' -printf '%T@ %p\n' \
		| sort -nr \
		| head -n 1 \
		| cut -d' ' -f2-
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		--version)
			VERSION="$2"
			shift 2
			;;
		--arch)
			ARCH_INPUT="$2"
			shift 2
			;;
		--binary)
			BINARY_PATH="$2"
			shift 2
			;;
		--man-dir)
			MAN_DIR="$2"
			shift 2
			;;
		--output-prefix)
			OUTPUT_PREFIX="$2"
			shift 2
			;;
		-h | --help)
			usage
			exit 0
			;;
		*)
			echo "Unknown option: $1" >&2
			usage
			exit 1
			;;
	esac
done

if [[ -z "${VERSION}" || -z "${ARCH_INPUT}" || -z "${BINARY_PATH}" || -z "${OUTPUT_PREFIX}" ]]; then
	echo "Missing required options." >&2
	usage
	exit 1
fi

ARCH="$(normalise_arch "${ARCH_INPUT}")"

if [[ ! -f "${BINARY_PATH}" ]]; then
	echo "Built binary not found at ${BINARY_PATH}" >&2
	exit 1
fi

if [[ -z "${MAN_DIR}" ]]; then
	MAN_DIR="$(discover_man_dir "${BINARY_PATH}")"
fi

if [[ -z "${MAN_DIR}" || ! -d "${MAN_DIR}" ]]; then
	echo "Generated man directory was not found." >&2
	exit 1
fi

if ! find "${MAN_DIR}" -maxdepth 1 -type f -name '*.1' | grep -q .; then
	echo "No generated man pages were found in ${MAN_DIR}" >&2
	exit 1
fi

mkdir -p "$(dirname "${OUTPUT_PREFIX}")"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

ARCHIVE_ROOT="${TMP_DIR}/flo"
mkdir -p "${ARCHIVE_ROOT}/bin" "${ARCHIVE_ROOT}/share/man/man1"

install -m 0755 "${BINARY_PATH}" "${ARCHIVE_ROOT}/bin/flo"

while IFS= read -r man_page; do
	gzip -n -c "${man_page}" > "${ARCHIVE_ROOT}/share/man/man1/$(basename "${man_page}").gz"
done < <(find "${MAN_DIR}" -maxdepth 1 -type f -name '*.1' | sort)

ARCHIVE_OUTPUT="${OUTPUT_PREFIX}.tar.gz"
tar -C "${ARCHIVE_ROOT}" -czf "${ARCHIVE_OUTPUT}" .
write_sha256_file "${ARCHIVE_OUTPUT}" "${ARCHIVE_OUTPUT}.sha256"

echo "Built ${ARCHIVE_OUTPUT} for ${ARCH} from ${VERSION}"
