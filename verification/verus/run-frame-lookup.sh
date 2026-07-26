#!/usr/bin/env bash
# Run TASK-1993's standalone frame-order model and required rejected witness, never Cargo.
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
manifest="$repo_root/verification/verus/frame-lookup-manifest.json"
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
p = json.load(open(sys.argv[1], encoding="utf-8")); r = p["release"]; s = p["sources"]
for value in (r["url"], r["archive"], r["sha256"], r["archive_root"], r["version_json"]["toolchain"], p["toolchain"]["rust"].split()[0], s["repaired"]["path"], s["repaired"]["sha256"], s["broken"]["path"], s["broken"]["sha256"]):
    print(value)
PY
)
release_url=${config[0]}; archive_name=${config[1]}; expected_archive_sha=${config[2]}; archive_root=${config[3]}
release_toolchain=${config[4]}; required_toolchain=${config[5]}; repaired_relative=${config[6]}; repaired_sha=${config[7]}
broken_relative=${config[8]}; broken_sha=${config[9]}
[[ "$release_toolchain" = "$required_toolchain" ]] || { printf 'pinned Verus Rust toolchain mismatch: release=%s Ash=%s\n' "$release_toolchain" "$required_toolchain" >&2; exit 67; }

repaired_file="$repo_root/$repaired_relative"; broken_file="$repo_root/$broken_relative"
for source_file in "$repaired_file" "$broken_file"; do
    [[ -f "$source_file" ]] || { printf 'missing frame-lookup source: %s\n' "$source_file" >&2; exit 66; }
done
[[ "sha256:$(sha256sum "$repaired_file" | awk '{print $1}')" = "$repaired_sha" ]] || { printf 'repaired source fingerprint mismatch\n' >&2; exit 65; }
[[ "sha256:$(sha256sum "$broken_file" | awk '{print $1}')" = "$broken_sha" ]] || { printf 'broken source fingerprint mismatch\n' >&2; exit 65; }
for source_file in "$repaired_file" "$broken_file"; do
    if rg -n '(^|[^[:alnum:]_])(assume|axiom|external_body|external_specification|external_item|external_trait_impl)[[:space:]!({]' "$source_file" >/dev/null; then
        printf 'frame-lookup source uses a forbidden logical escape: %s\n' "$source_file" >&2
        exit 68
    fi
done

cache_dir=${VERUS_FRAME_LOOKUP_CACHE_DIR:-"${XDG_CACHE_HOME:-$HOME/.cache}/ash-verus-frame-lookup"}
archive="$cache_dir/$archive_name"; install_dir="$cache_dir/$archive_root"; verus="$install_dir/verus"
mkdir -p "$cache_dir"
[[ -f "$archive" ]] || curl --fail --location --retry 3 --output "$archive" "$release_url"
[[ "$(sha256sum "$archive" | awk '{print $1}')" = "$expected_archive_sha" ]] || { printf 'pinned Verus archive checksum mismatch\n' >&2; exit 65; }
if [[ ! -x "$verus" ]]; then
    unpack_dir=$(mktemp -d "$cache_dir/.unpack.XXXXXX")
    trap 'rmdir "$unpack_dir" 2>/dev/null || true' EXIT
    unzip -q "$archive" -d "$unpack_dir"
    test -x "$unpack_dir/$archive_root/verus"
    mv "$unpack_dir/$archive_root" "$install_dir"
    rmdir "$unpack_dir"; trap - EXIT
fi
rustup run "$required_toolchain" rustc --version >/dev/null || { printf 'required Rust toolchain is unavailable: %s\n' "$required_toolchain" >&2; exit 69; }

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/ash-verus-frame-lookup.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT
repaired_json="$work_dir/repaired.json"; broken_json="$work_dir/broken.json"
set +e
cd "$work_dir"
RUSTUP_TOOLCHAIN="$required_toolchain" "$verus" "$repaired_file" --output-json --no-cheating --rlimit 120 >"$repaired_json" 2>"$work_dir/repaired.stderr"; repaired_exit=$?
RUSTUP_TOOLCHAIN="$required_toolchain" "$verus" "$broken_file" --output-json --no-cheating --rlimit 120 >"$broken_json" 2>"$work_dir/broken.stderr"; broken_exit=$?
set -e
python3 - "$manifest" "$repaired_json" "$broken_json" "$repaired_exit" "$broken_exit" "$output" <<'PY'
import json, sys
from pathlib import Path
manifest_path, repaired_path, broken_path, repaired_exit, broken_exit, output = sys.argv[1:]
manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
repaired = json.loads(Path(repaired_path).read_text(encoding="utf-8")); broken = json.loads(Path(broken_path).read_text(encoding="utf-8"))
rr = repaired.get("verification-results", {}); br = broken.get("verification-results", {})
expected_repaired = manifest["sources"]["repaired"]["expected_result"]
expected_broken = manifest["sources"]["broken"]["expected_result"]
checks = {"repaired_exit_zero": int(repaired_exit) == 0, "repaired_expected_result": all(rr.get(k) == v for k, v in expected_repaired.items()), "broken_exit_nonzero": int(broken_exit) != 0, "broken_expected_rejection": all(br.get(k) == v for k, v in expected_broken.items())}
payload = {"schema": "verus-frame-lookup-run/v1", "manifest": str(Path(manifest_path).resolve()), "outcome": "verified" if all(checks.values()) else "rejected", "checks": checks, "repaired": {"exit": int(repaired_exit), "verification_results": rr, "verus": repaired.get("verus")}, "broken": {"exit": int(broken_exit), "verification_results": br, "verus": broken.get("verus")}}
encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output == "-": print(encoded, end="")
else:
    target = Path(output); target.parent.mkdir(parents=True, exist_ok=True); target.write_text(encoded, encoding="utf-8")
if not all(checks.values()): raise SystemExit(1)
PY
