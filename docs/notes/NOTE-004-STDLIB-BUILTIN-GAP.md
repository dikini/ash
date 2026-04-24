# NOTE-004: Stdlib Builtin Gap Analysis

## Purpose

Systematic audit of every `builtin fn` declaration in the Ash stdlib against the Rust implementation surface. This note will be produced by TASK-660.

## Audit Template

For each stdlib module, enumerate:

| Module | File | Declared builtins | Rust handler exists? | Handler location | Effect level | Notes |
|---|---|---|---|---|---|---|
| json | std/src/json.ash | parse, stringify, stringify_pretty | Yes | eval.rs L1323-1345 | Epistemic | Working |
| list | std/src/list.ash | len, head, tail, append, concat, filter, map | Partial | eval.rs L193-196, L1015-1100 | Epistemic | filter, map need closure support |
| string | std/src/string.ash | concat, starts_with, ends_with, is_empty | Yes | eval.rs L926-985 | Epistemic | Working |
| record | std/src/record.ash | keys, values, record | Partial | eval.rs L1222-1252 | Epistemic | record() unclear |
| regex | std/src/regex.ash | find, matches, replace | Yes | eval.rs L989-1013 | Epistemic | Working |
| predicate | std/src/predicate.ash | is_int, is_string, is_bool, is_list, is_record, is_null | Yes | eval.rs | Epistemic | Working |
| markdown | std/src/markdown.ash | parse | Yes | eval.rs L1346+ | Epistemic | Working |
| process | std/src/process.ash | run | Yes | eval.rs L1195+ | Operational | Needs constraint model |
| io::stdio | std/src/io/stdio.ash | ? | No (engine has StdioProvider) | — | Operational | Not wired as builtin |
| io::fs | std/src/io/fs.ash | ? | No (engine has FsProvider) | — | Operational | Not wired as builtin |
| io::path | std/src/io/path.ash | ? | No | — | Epistemic | Needs implementation |
| io::dir | std/src/io/dir.ash | ? | No | — | Operational | Needs implementation |
| io::meta | std/src/io/meta.ash | ? | No | — | Epistemic | Needs implementation |
| io::buf | std/src/io/buf.ash | ? | No | — | Epistemic | Needs implementation |
| runtime::error | std/src/runtime/error.ash | ? | Partial | — | Epistemic | Types only |
| runtime::args | std/src/runtime/args.ash | ? | No | — | Epistemic | Needs implementation |
| runtime::supervisor | std/src/runtime/supervisor.ash | ? | No | — | Operational | Stub |
| llm::* | std/src/llm/*.ash | ? | No (engine has LlmProvider) | — | Operational | Not wired as builtins |

**Note:** This template will be filled in completely by TASK-660. The `?` entries indicate modules whose declared functions need to be enumerated by reading the actual .ash files.

## Priority Classification

- **Critical:** io::stdio, io::fs (without these, no real program can do I/O)
- **High:** io::path, runtime::error, runtime::args (needed for real programs)
- **Medium:** list::filter, list::map (need closure support — may require eval.rs changes)
- **Low:** io::buf, io::meta, runtime::supervisor (nice-to-have, stubs acceptable)
