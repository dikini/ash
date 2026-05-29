#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/package-ash-toolchain.sh [--toolchain-id ID] [--version VERSION] [--output-dir DIR] [--profile debug|release]

Packages a local Ash release toolchain tarball. By default the script builds
ash and ashgrove with cargo; tests may provide ASH_PACKAGE_ASH_BIN and
ASH_PACKAGE_ASHGROVE_BIN to package prebuilt fixture binaries.
USAGE
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
version=""
toolchain_id=""
output_dir="$repo_root/target/ash-toolchain-dist"
profile="debug"
archive_schema_version="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --toolchain-id)
      toolchain_id="${2:?--toolchain-id requires a value}"
      shift 2
      ;;
    --version)
      version="${2:?--version requires a value}"
      shift 2
      ;;
    --output-dir)
      output_dir="${2:?--output-dir requires a value}"
      shift 2
      ;;
    --profile)
      profile="${2:?--profile requires a value}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "$profile" != "debug" && "$profile" != "release" ]]; then
  echo "--profile must be debug or release" >&2
  exit 2
fi

if [[ -z "$version" ]]; then
  version="$(
    awk '
      /^\[workspace.package\]$/ { in_package = 1; next }
      /^\[/ { in_package = 0 }
      in_package && /^version[[:space:]]*=/ {
        gsub(/"/, "", $3);
        print $3;
        exit
      }
    ' "$repo_root/Cargo.toml"
  )"
fi

if [[ -z "$version" ]]; then
  echo "could not determine workspace package version" >&2
  exit 1
fi
if [[ ! "$version" =~ ^[A-Za-z0-9._+-]+$ || "$version" == *..* ]]; then
  echo "version contains characters that cannot be safely encoded in release metadata: $version" >&2
  exit 1
fi

target_triple="${ASH_PACKAGE_TARGET_TRIPLE:-$(rustc -vV | awk '/^host:/ { print $2 }')}"
if [[ -z "$target_triple" ]]; then
  echo "could not determine target triple" >&2
  exit 1
fi
if [[ ! "$target_triple" =~ ^[A-Za-z0-9._+-]+$ || "$target_triple" == *..* ]]; then
  echo "target triple contains characters that cannot be safely encoded in release metadata: $target_triple" >&2
  exit 1
fi

if [[ -z "$toolchain_id" ]]; then
  toolchain_id="ash-${version}+tarball.${target_triple}"
fi

if [[ ! "$toolchain_id" =~ ^ash-[A-Za-z0-9._+-]+$ || "$toolchain_id" == *..* ]]; then
  echo "invalid toolchain id '$toolchain_id'" >&2
  exit 1
fi
case "$toolchain_id" in
  ash-"$version"+*) ;;
  *)
    echo "toolchain id '$toolchain_id' must begin with ash-${version}+" >&2
    exit 1
    ;;
esac

ash_bin="${ASH_PACKAGE_ASH_BIN:-}"
ashgrove_bin="${ASH_PACKAGE_ASHGROVE_BIN:-}"
if [[ -z "$ash_bin" || -z "$ashgrove_bin" ]]; then
  cargo_args=(build -p ash-cli -p ashgrove)
  if [[ "$profile" == "release" ]]; then
    cargo_args+=(--release)
  fi
  cargo "${cargo_args[@]}"
  target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
  ash_bin="$target_dir/$profile/ash"
  ashgrove_bin="$target_dir/$profile/ashgrove"
fi

if [[ ! -f "$ash_bin" || ! -x "$ash_bin" ]]; then
  echo "ash binary is missing or not executable: $ash_bin" >&2
  exit 1
fi
if [[ ! -f "$ashgrove_bin" || ! -x "$ashgrove_bin" ]]; then
  echo "ashgrove binary is missing or not executable: $ashgrove_bin" >&2
  exit 1
fi

mkdir -p "$output_dir"
stage_parent="$(mktemp -d "${TMPDIR:-/tmp}/ash-toolchain-package.XXXXXX")"
trap 'rm -rf "$stage_parent"' EXIT
root="$stage_parent/$toolchain_id"
mkdir -p "$root/bin" "$root/lib/ash/std"

install -m 0755 "$ash_bin" "$root/bin/ash"
install -m 0755 "$ashgrove_bin" "$root/bin/ashgrove"
cp "$repo_root/std/Cargo.toml" "$root/lib/ash/std/ash.toml"
cp -R "$repo_root/std/src" "$root/lib/ash/std/src"

cat >"$root/manifest.toml" <<EOF
toolchain_id = "$toolchain_id"
version = "$version"
archive_schema_version = $archive_schema_version
target_triple = "$target_triple"
source_kind = "tarball"

[stdlib]
version = "$version"
path = "lib/ash/std"

[[standard_tools]]
name = "ash"
path = "bin/ash"
required = true

[[standard_tools]]
name = "ashgrove"
path = "bin/ashgrove"
required = true
EOF

cat >"$root/install-record.toml" <<EOF
toolchain_id = "$toolchain_id"
source_kind = "tarball"
archive_schema_version = $archive_schema_version
reproducible = true
target_triple = "$target_triple"
EOF

archive="$output_dir/${toolchain_id}.tar.gz"
tar -C "$stage_parent" -czf "$archive" "$toolchain_id"

if command -v sha256sum >/dev/null 2>&1; then
  digest="$(sha256sum "$archive" | awk '{ print $1 }')"
else
  digest="$(shasum -a 256 "$archive" | awk '{ print $1 }')"
fi

printf 'archive=%s\n' "$archive"
printf 'digest=sha256:%s\n' "$digest"
