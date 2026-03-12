#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

VERSION=""
ARCH_INPUT=""
BINARY_PATH=""
MAN_DIR=""
OUTPUT_PREFIX=""
FORMAT="all"

PACKAGE_NAME="flo"
PACKAGE_SUMMARY="Terminal-native working-memory sidecar for developers"
PACKAGE_DESCRIPTION="Liminal Flow is a local-first terminal application for tracking active work, branching into sub-tasks, and preserving context across CLI and TUI workflows."
PACKAGE_HOMEPAGE="https://github.com/liminal-hq/flow"
PACKAGE_VENDOR="Liminal HQ"
PACKAGE_CONTACT="Liminal HQ <contact@liminalhq.ca>"

usage() {
	cat <<'USAGE'
Usage: scripts/build-linux-packages.sh [options]

Options:
  --version <version>         Package version or tag (for example, v0.0.1)
  --arch <x64|arm64>          Target architecture
  --binary <path>             Built binary path
  --man-dir <path>            Directory containing generated man pages
  --output-prefix <prefix>    Output file prefix (without extension)
  --format <all|deb|rpm>      Package format to build (default: all)
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
		--format)
			FORMAT="$2"
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

case "${FORMAT}" in
	all | deb | rpm)
		;;
	*)
		echo "Unsupported format: ${FORMAT}" >&2
		exit 1
		;;
esac

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

VERSION_NO_V="${VERSION#v}"
mkdir -p "$(dirname "${OUTPUT_PREFIX}")"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

MAN_SOURCE_DIR="${TMP_DIR}/man"
mkdir -p "${MAN_SOURCE_DIR}"

while IFS= read -r man_page; do
	gzip -n -c "${man_page}" > "${MAN_SOURCE_DIR}/$(basename "${man_page}").gz"
done < <(find "${MAN_DIR}" -maxdepth 1 -type f -name '*.1' | sort)

if ! find "${MAN_SOURCE_DIR}" -maxdepth 1 -type f -name '*.1.gz' | grep -q .; then
	echo "No generated man pages were found in ${MAN_DIR}" >&2
	exit 1
fi

DEB_ARCH=""
RPM_ARCH=""
RPM_TARGET=""
case "${ARCH}" in
	x64)
		DEB_ARCH="amd64"
		RPM_ARCH="x86_64"
		RPM_TARGET="x86_64-linux"
		;;
	arm64)
		DEB_ARCH="arm64"
		RPM_ARCH="aarch64"
		RPM_TARGET="aarch64-linux"
		;;
esac

build_deb() {
	local deb_root="${TMP_DIR}/deb-root"
	mkdir -p "${deb_root}/DEBIAN" "${deb_root}/usr/bin" "${deb_root}/usr/share/man/man1"

	install -m 0755 "${BINARY_PATH}" "${deb_root}/usr/bin/flo"
	install -m 0644 "${MAN_SOURCE_DIR}"/*.1.gz "${deb_root}/usr/share/man/man1/"

	cat > "${deb_root}/DEBIAN/control" <<CONTROL
Package: ${PACKAGE_NAME}
Version: ${VERSION_NO_V}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: ${PACKAGE_CONTACT}
Homepage: ${PACKAGE_HOMEPAGE}
Depends: libc6
Description: ${PACKAGE_SUMMARY}
 ${PACKAGE_DESCRIPTION}
CONTROL

	cat > "${deb_root}/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e
if command -v mandb >/dev/null 2>&1; then
	mandb -q >/dev/null 2>&1 || true
fi
POSTINST

	cat > "${deb_root}/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e
if command -v mandb >/dev/null 2>&1; then
	mandb -q >/dev/null 2>&1 || true
fi
POSTRM

	chmod 0755 "${deb_root}/DEBIAN/postinst" "${deb_root}/DEBIAN/postrm"

	local deb_output="${OUTPUT_PREFIX}.deb"
	dpkg-deb --build "${deb_root}" "${deb_output}"
	write_sha256_file "${deb_output}" "${deb_output}.sha256"
	echo "Built ${deb_output}"
}

build_rpm() {
	if ! command -v rpmbuild >/dev/null 2>&1; then
		echo "rpmbuild is required to create RPM packages" >&2
		exit 1
	fi

	local rpm_root="${TMP_DIR}/rpm"
	mkdir -p "${rpm_root}/BUILD" "${rpm_root}/BUILDROOT" "${rpm_root}/RPMS" "${rpm_root}/SOURCES" "${rpm_root}/SPECS" "${rpm_root}/SRPMS"

	install -m 0755 "${BINARY_PATH}" "${rpm_root}/SOURCES/flo"
	install -m 0644 "${MAN_SOURCE_DIR}"/*.1.gz "${rpm_root}/SOURCES/"

	cat > "${rpm_root}/SPECS/flo.spec" <<SPEC
Name:           ${PACKAGE_NAME}
Version:        ${VERSION_NO_V}
Release:        1%{?dist}
Summary:        ${PACKAGE_SUMMARY}
License:        MIT
URL:            ${PACKAGE_HOMEPAGE}
Vendor:         ${PACKAGE_VENDOR}
Packager:       ${PACKAGE_CONTACT}
BuildArch:      ${RPM_ARCH}

%description
${PACKAGE_DESCRIPTION}

%install
mkdir -p %{buildroot}/usr/bin %{buildroot}/usr/share/man/man1
install -m 0755 %{_sourcedir}/flo %{buildroot}/usr/bin/flo
install -m 0644 %{_sourcedir}/*.1.gz %{buildroot}/usr/share/man/man1/

%post
if command -v mandb >/dev/null 2>&1; then
	mandb -q >/dev/null 2>&1 || true
fi

%postun
if command -v mandb >/dev/null 2>&1; then
	mandb -q >/dev/null 2>&1 || true
fi

%files
/usr/bin/flo
/usr/share/man/man1/*.1.gz

%changelog
* $(date '+%a %b %d %Y') Liminal HQ <contact@liminalhq.ca> - ${VERSION_NO_V}-1
- Add Linux RPM package for flo binary and generated man pages.
SPEC

	rpmbuild \
		--define "_topdir ${rpm_root}" \
		--define "__os_install_post %{nil}" \
		--target "${RPM_TARGET}" \
		-bb "${rpm_root}/SPECS/flo.spec"

	local rpm_built
	rpm_built="$(find "${rpm_root}/RPMS" -type f -name '*.rpm' | head -n 1)"
	if [[ -z "${rpm_built}" ]]; then
		echo "RPM build succeeded but no RPM file was produced" >&2
		exit 1
	fi

	local rpm_output="${OUTPUT_PREFIX}.rpm"
	cp "${rpm_built}" "${rpm_output}"
	write_sha256_file "${rpm_output}" "${rpm_output}.sha256"
	echo "Built ${rpm_output}"
}

if [[ "${FORMAT}" == "all" || "${FORMAT}" == "deb" ]]; then
	build_deb
fi

if [[ "${FORMAT}" == "all" || "${FORMAT}" == "rpm" ]]; then
	build_rpm
fi
