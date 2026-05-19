#!/usr/bin/env bash
set -euo pipefail

gate_repo_root() {
  git rev-parse --show-toplevel
}

gate_content_sha() {
  mapfile -t paths < <({
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
  } | sed '/^$/d' | sort -u)

  if [[ "${#paths[@]}" -eq 0 ]]; then
    printf '' | sha256sum | awk '{print $1}'
    return
  fi

  {
    for path in "${paths[@]}"; do
      if [[ -e "$path" ]]; then
        hash="$(git hash-object -- "$path" 2>/dev/null || printf '__nonregular__')"
      else
        hash="__deleted__"
      fi
      printf '%s\t%s\n' "$path" "$hash"
    done
  } | sha256sum | awk '{print $1}'
}

gate_head_ref() {
  if git rev-parse --verify HEAD >/dev/null 2>&1; then
    git rev-parse HEAD
  else
    printf 'UNBORN_HEAD'
  fi
}

gate_head_tree_ref() {
  if git rev-parse --verify HEAD^{tree} >/dev/null 2>&1; then
    git rev-parse HEAD^{tree}
  else
    printf 'UNBORN_TREE'
  fi
}

gate_index_tree_ref() {
  git write-tree
}

gate_empty_content_sha() {
  printf '' | sha256sum | awk '{print $1}'
}

gate_write_marker() {
  local marker_file="$1"
  local marker_tree="${GATE_MARKER_TREE:-$(gate_head_tree_ref)}"
  local marker_docs_only="${GATE_MARKER_DOCS_ONLY:-unknown}"

  mkdir -p "$(dirname "$marker_file")"
  {
    echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "head=$(gate_head_ref)"
    echo "tree=$marker_tree"
    echo "content_sha=$(gate_content_sha)"
    echo "docs_only=$marker_docs_only"
  } >"$marker_file"
}

gate_marker_value() {
  local marker_file="$1"
  local key="$2"

  if [[ ! -f "$marker_file" ]]; then
    return 1
  fi
  grep -E "^${key}=" "$marker_file" | tail -n 1 | cut -d= -f2-
}

gate_marker_matches() {
  local marker_file="$1"
  local marker_head
  local marker_tree
  local marker_content_sha
  local current_content_sha

  marker_head="$(gate_marker_value "$marker_file" head || true)"
  marker_tree="$(gate_marker_value "$marker_file" tree || true)"
  marker_content_sha="$(gate_marker_value "$marker_file" content_sha || true)"
  current_content_sha="$(gate_content_sha)"

  [[ -n "$marker_head" ]] || return 1
  [[ -n "$marker_content_sha" ]] || return 1
  [[ "$marker_head" == "$(gate_head_ref)" ]] || return 1
  if [[ -n "$marker_tree" ]]; then
    [[ "$marker_tree" == "$(gate_head_tree_ref)" ]] || return 1
  fi
  [[ "$marker_content_sha" == "$current_content_sha" ]] || return 1
}

gate_marker_matches_current_head_with_empty_content() {
  local marker_file="$1"
  local marker_head
  local marker_tree
  local marker_content_sha

  marker_head="$(gate_marker_value "$marker_file" head || true)"
  marker_tree="$(gate_marker_value "$marker_file" tree || true)"
  marker_content_sha="$(gate_marker_value "$marker_file" content_sha || true)"

  [[ -n "$marker_head" ]] || return 1
  [[ -n "$marker_content_sha" ]] || return 1
  [[ "$marker_head" == "$(gate_head_ref)" ]] || return 1
  if [[ -n "$marker_tree" ]]; then
    [[ "$marker_tree" == "$(gate_index_tree_ref)" ]] || return 1
  fi
  [[ "$marker_content_sha" == "$(gate_empty_content_sha)" ]] || return 1
}

gate_classifier_value() {
  local classifier_output="$1"
  local key="$2"
  printf '%s\n' "$classifier_output" | grep -E "^${key}=" | tail -n 1 | cut -d= -f2-
}
