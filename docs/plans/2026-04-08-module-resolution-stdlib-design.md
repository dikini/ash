# General Module Resolution And Stdlib-Backed Workflows Design

**Goal:** Make general stdlib-backed file workflows executable, add module resolution for user-defined multi-file modules, and support optional version-qualified library imports with one concrete version per library name across a loaded graph.

**Architecture:** Move import resolution into `ash-engine` as the authoritative loader for file-backed workflows. Resolve modules from the workflow file's local tree first, then `ASH_LIBRARY_PATH` in order, then the built-in stdlib root. Keep entry verification/bootstrap semantics, but have them run on top of the same resolved graph rather than a separate import-loading path.

**Scope:** ordinary file execution with imports, user multi-file module trees, stdlib imports such as `option`, `prelude`, and `std/lib`, `ASH_LIBRARY_PATH` roots, and version-qualified external libraries like `math@1::vector`.

Version-qualified imports are in scope only as a bootstrap-time resolver/version-selection mechanism. Packaging, installation, dependency manifests, and package-level version solving remain future work.

## Resolution Model

### Local Tree

The local tree means the directory tree rooted at the workflow file being executed. A module import may resolve to files under that root before any external library root or stdlib root is consulted.

### Filesystem Candidates

Module spellings map to filesystem candidates deterministically:

- `foo::bar` may resolve to `foo/bar.ash`
- `foo::bar` may also resolve to `foo/bar/mod.ash`
- deeper paths follow the same rule, for example `foo::bar::baz` may resolve to `foo/bar/baz.ash` or `foo/bar/baz/mod.ash`

The resolver should treat a directory-style module spelling and a direct file spelling as equivalent candidates within the same root, with the same precedence order.

### Precedence And Shadowing

Resolution precedence is:

1. local tree
2. `ASH_LIBRARY_PATH`, in listed order
3. built-in stdlib root

Local modules may shadow both library and stdlib modules because they are searched first. `ASH_LIBRARY_PATH` modules may shadow the stdlib root because they are searched before it. This shadowing is intentional and is part of the bootstrap-time resolver contract.

### External Library Imports

Bare imports and version-qualified external-library imports coexist, but they are not treated the same way:

- bare imports are valid for local-tree modules and stdlib modules
- version-qualified imports such as `use math@1::vector` are valid for external libraries and select one concrete installed version for that library name
- unqualified external-library imports are rejected as ambiguous unless future packaging work defines a package manifest that can disambiguate them

The single-version rule is bootstrap-time only: a loaded graph may select one concrete version for a library name, but packaging, installation, dependency manifests, and dependency solving remain future work.
