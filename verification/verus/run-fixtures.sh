#!/usr/bin/env bash
# Run TASK-1991's tiny standalone Verus witnesses, never Cargo.
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
manifest="$repo_root/verification/verus/verus-spike-manifest.json"
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
for v in (r["url"], r["archive"], r["sha256"], r["archive_root"], r["version_json"]["toolchain"], p["required_rust_toolchain"]): print(v)
PY
)
release_url=${config[0]}; archive_name=${config[1]}; expected_sha=${config[2]}; archive_root=${config[3]}; release_toolchain=${config[4]}; required_toolchain=${config[5]}
if [[ "$release_toolchain" != "$required_toolchain" ]]; then
    python3 - "$manifest" "$release_toolchain" "$required_toolchain" "$output" <<'PY'
import json, sys
from pathlib import Path
manifest, released, required, output = sys.argv[1:]
payload = {"schema": "verus-spike-fixture-run/v1", "manifest": str(Path(manifest).resolve()), "outcome": "blocked", "checks": {"single_rust_toolchain": False, "archive_not_downloaded": True, "fixtures_not_run": True}, "no_go_reason": f"Verus release version.json requires {released}; Ash requires {required}."}
encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output == "-": print(encoded, end="")
else:
    target = Path(output); target.parent.mkdir(parents=True, exist_ok=True); target.write_text(encoded, encoding="utf-8")
PY
    exit 1
fi
rust_toolchain=$required_toolchain
cache_dir=${VERUS_SPIKE_CACHE_DIR:-"${XDG_CACHE_HOME:-$HOME/.cache}/ash-verus-spike"}
archive="$cache_dir/$archive_name"; install_dir="$cache_dir/$archive_root"; verus="$install_dir/verus"
mkdir -p "$cache_dir"
[[ -f "$archive" ]] || curl --fail --location --retry 3 --output "$archive" "$release_url"
actual_sha=$(sha256sum "$archive" | awk '{print $1}')
[[ "$actual_sha" = "$expected_sha" ]] || { printf 'pinned Verus archive checksum mismatch: expected %s, got %s\n' "$expected_sha" "$actual_sha" >&2; exit 65; }
if [[ ! -x "$verus" ]]; then
    unpack_dir=$(mktemp -d "$cache_dir/.unpack.XXXXXX")
    trap 'rmdir "$unpack_dir" 2>/dev/null || true' EXIT
    unzip -q "$archive" -d "$unpack_dir"
    test -x "$unpack_dir/$archive_root/verus"
    mv "$unpack_dir/$archive_root" "$install_dir"
    rmdir "$unpack_dir"
    trap - EXIT
fi
rustup run "$rust_toolchain" rustc --version >/dev/null || { printf 'required Rust toolchain is unavailable: %s\n' "$rust_toolchain" >&2; exit 69; }

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/ash-verus-spike.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT
pass_json="$work_dir/pass.json"; fail_json="$work_dir/fail.json"
cd "$work_dir"
set +e
RUSTUP_TOOLCHAIN="$rust_toolchain" "$verus" "$repo_root/verification/verus/fixtures/pass.rs" --output-json --no-cheating --rlimit 30 >"$pass_json" 2>"$work_dir/pass.stderr"; pass_exit=$?
RUSTUP_TOOLCHAIN="$rust_toolchain" "$verus" "$repo_root/verification/verus/fixtures/fail.rs" --output-json --no-cheating --rlimit 30 >"$fail_json" 2>"$work_dir/fail.stderr"; fail_exit=$?
set -e
python3 - "$manifest" "$pass_json" "$fail_json" "$pass_exit" "$fail_exit" "$output" <<'PY'
import json, sys
from pathlib import Path
manifest, pass_path, fail_path, pass_exit, fail_exit, output = sys.argv[1:]
passed = json.loads(Path(pass_path).read_text(encoding="utf-8")); failed = json.loads(Path(fail_path).read_text(encoding="utf-8"))
pr = passed.get("verification-results", {}); fr = failed.get("verification-results", {})
checks = {"pass_exit_zero": int(pass_exit) == 0, "pass_json_success": pr.get("success") is True and pr.get("verified") == 1 and pr.get("errors") == 0, "fail_exit_nonzero": int(fail_exit) != 0, "fail_json_rejected": fr.get("success") is False and fr.get("verified") == 0 and fr.get("errors") >= 1}
payload = {"schema": "verus-spike-fixture-run/v1", "manifest": str(Path(manifest).resolve()), "outcome": "verified" if all(checks.values()) else "rejected", "checks": checks, "pass": {"exit": int(pass_exit), "verification_results": pr, "verus": passed.get("verus")}, "fail": {"exit": int(fail_exit), "verification_results": fr, "verus": failed.get("verus")}}
encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
if output == "-": print(encoded, end="")
else:
    target = Path(output); target.parent.mkdir(parents=True, exist_ok=True); target.write_text(encoded, encoding="utf-8")
if not all(checks.values()): raise SystemExit(1)
PY
