#!/usr/bin/env bash
# Run TASK-1992's standalone row-algebra model, never Cargo or production Ash.
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
manifest="$repo_root/verification/verus/row-algebra-manifest.json"
output=-
while (($#)); do
    case "$1" in
        --manifest) manifest=$2; shift 2 ;;
        --output) output=$2; shift 2 ;;
        *) printf 'unknown argument: %s\n' "$1" >&2; exit 64 ;;
    esac
done
[[ "$manifest" = /* ]] || manifest="$repo_root/$manifest"
[[ -f "$manifest" ]] || { printf 'missing manifest: %s\n' "$manifest" >&2; exit 66; }

mapfile -t config < <(python3 - "$manifest" <<'PY'
import json, sys
p = json.load(open(sys.argv[1], encoding="utf-8")); r = p["release"]
for value in (r["url"], r["archive"], r["sha256"], r["archive_root"], r["version_json"]["toolchain"], p["toolchain"]["rust"].split()[0], p["source"]["path"], p["source"]["sha256"]):
    print(value)
PY
)
release_url=${config[0]}; archive_name=${config[1]}; expected_archive_sha=${config[2]}; archive_root=${config[3]}; release_toolchain=${config[4]}; required_toolchain=${config[5]}; source_relative=${config[6]}; expected_source_sha=${config[7]}

if [[ "$release_toolchain" != "$required_toolchain" ]]; then
    printf 'pinned Verus Rust toolchain mismatch: release=%s Ash=%s\n' "$release_toolchain" "$required_toolchain" >&2
    exit 67
fi
source_file="$repo_root/$source_relative"
[[ -f "$source_file" ]] || { printf 'missing row-algebra source: %s\n' "$source_file" >&2; exit 66; }
actual_source_sha=$(sha256sum "$source_file" | awk '{print $1}')
[[ "sha256:$actual_source_sha" = "$expected_source_sha" ]] || { printf 'row-algebra source fingerprint mismatch\n' >&2; exit 65; }

# These are exactly the escape categories declared in the pilot manifest.
if rg -n '(^|[^[:alnum:]_])(assume|axiom|external_body|external_specification|external_item|external_trait_impl)[[:space:]!({]' "$source_file" >/dev/null; then
    printf 'row-algebra source uses a forbidden logical escape\n' >&2
    exit 68
fi

cache_dir=${VERUS_ROW_ALGEBRA_CACHE_DIR:-"${XDG_CACHE_HOME:-$HOME/.cache}/ash-verus-row-algebra"}
archive="$cache_dir/$archive_name"; install_dir="$cache_dir/$archive_root"; verus="$install_dir/verus"
mkdir -p "$cache_dir"
[[ -f "$archive" ]] || curl --fail --location --retry 3 --output "$archive" "$release_url"
actual_archive_sha=$(sha256sum "$archive" | awk '{print $1}')
[[ "$actual_archive_sha" = "$expected_archive_sha" ]] || { printf 'pinned Verus archive checksum mismatch\n' >&2; exit 65; }
if [[ ! -x "$verus" ]]; then
    unpack_dir=$(mktemp -d "$cache_dir/.unpack.XXXXXX")
    trap 'rmdir "$unpack_dir" 2>/dev/null || true' EXIT
    unzip -q "$archive" -d "$unpack_dir"
    test -x "$unpack_dir/$archive_root/verus"
    mv "$unpack_dir/$archive_root" "$install_dir"
    rmdir "$unpack_dir"; trap - EXIT
fi
rustup run "$required_toolchain" rustc --version >/dev/null || { printf 'required Rust toolchain is unavailable: %s\n' "$required_toolchain" >&2; exit 69; }

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/ash-verus-row-algebra.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT
result_json="$work_dir/result.json"
set +e
cd "$work_dir"
RUSTUP_TOOLCHAIN="$required_toolchain" "$verus" "$source_file" --output-json --no-cheating --rlimit 120 >"$result_json" 2>"$work_dir/verus.stderr"
verus_exit=$?
set -e
python3 - "$manifest" "$result_json" "$verus_exit" "$output" <<'PY'
import json, sys
from pathlib import Path
manifest_path, result_path, verus_exit, output = sys.argv[1:]
manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
result = json.loads(Path(result_path).read_text(encoding="utf-8"))
expected = manifest["expected_result"]
actual = result.get("verification-results", {})
checks = {
    "exit_zero": int(verus_exit) == 0,
    "success": actual.get("success") is expected["success"],
    "verified": actual.get("verified") == expected["verified"],
    "errors": actual.get("errors") == expected["errors"],
}
payload = {"schema": "verus-row-algebra-run/v1", "manifest": str(Path(manifest_path).resolve()), "outcome": "verified" if all(checks.values()) else "rejected", "checks": checks, "exit": int(verus_exit), "verification_results": actual, "verus": result.get("verus")}
encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output == "-": print(encoded, end="")
else:
    target = Path(output); target.parent.mkdir(parents=True, exist_ok=True); target.write_text(encoded, encoding="utf-8")
if not all(checks.values()): raise SystemExit(1)
PY
