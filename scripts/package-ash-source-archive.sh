#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage: scripts/package-ash-source-archive.sh [--source-root DIR] [--origin-commit COMMIT] [--output-dir DIR] [--version VERSION]

Packages an Ash source archive with release-source metadata. The archive is
intended for ashgrove install --from source after extraction on the target
machine.
USAGE
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
source_root="$repo_root"
origin_commit=""
output_dir="$repo_root/target/ash-source-dist"
version=""
schema_version="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source-root)
      source_root="${2:?--source-root requires a value}"
      shift 2
      ;;
    --origin-commit)
      origin_commit="${2:?--origin-commit requires a value}"
      shift 2
      ;;
    --output-dir)
      output_dir="${2:?--output-dir requires a value}"
      shift 2
      ;;
    --version)
      version="${2:?--version requires a value}"
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

if [[ ! -d "$source_root" ]]; then
  echo "source root is not a directory: $source_root" >&2
  exit 1
fi

if [[ -z "$origin_commit" ]]; then
  origin_commit="$(git -C "$source_root" rev-parse HEAD)"
fi
if [[ ! "$origin_commit" =~ ^[0-9A-Fa-f]{7,64}$ ]]; then
  echo "origin commit must be a git commit hash: $origin_commit" >&2
  exit 1
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
    ' "$source_root/Cargo.toml"
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

mkdir -p "$output_dir"
stage_parent="$(mktemp -d "${TMPDIR:-/tmp}/ash-source-package.XXXXXX")"
trap 'rm -rf "$stage_parent"' EXIT
archive_root="ash-${version}+source.${origin_commit:0:12}"
stage="$stage_parent/$archive_root"
mkdir -p "$stage"

tar -C "$source_root" \
  --exclude .git \
  --exclude target \
  --exclude release-source.toml \
  -cf - . | tar -C "$stage" -xf -

cat >"$stage/release-source.toml" <<EOF
schema_version = $schema_version
origin_commit = "$origin_commit"
EOF

cat >"$stage/.source-rev" <<EOF
$origin_commit
EOF

archive="$output_dir/${archive_root}.tar.gz"
tar -C "$stage_parent" -czf "$archive" "$archive_root"

if command -v sha256sum >/dev/null 2>&1; then
  digest="$(sha256sum "$archive" | awk '{ print $1 }')"
else
  digest="$(shasum -a 256 "$archive" | awk '{ print $1 }')"
fi

printf 'archive=%s\n' "$archive"
printf 'digest=sha256:%s\n' "$digest"
printf 'origin_commit=%s\n' "$origin_commit"
