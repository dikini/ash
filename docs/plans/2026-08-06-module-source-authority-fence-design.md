# Module Source Authority Fence Design

**Date:** 2026-08-06  
**Status:** Approved for implementation  
**Scope:** PLAN-207 / TASK-2069 compatibility-boundary slice

## Decision

The canonical module parser owns root-level export facts. The Engine module loader
may retain source scanners only as a denylisted compatibility fallback when the
canonical parser cannot represent the input (currently versioned import syntax or
an explicitly legacy parser-failure path). A successful parser result prevents
source snippets from contributing root exports, callables, capabilities, child
modules, or public-use structure.

The compatibility fallback remains non-authorizing: it can support the legacy
loader's existing diagnostic/import behavior, but it must not override or augment
parser-owned facts when both views exist. Roles and policies are not expanded by
this work; any existing metadata is transported only as generic compatibility
data.

## Boundary

The fence applies to `collect_module_exports` and the public-callable visibility
helpers. The parser-owned `ModuleBody` is the source of truth for:

- root-level public functions and builtin functions;
- root-level public capability declarations;
- `pub mod` file-child declarations;
- public `use`/`pub use` declarations.

The old scanners remain reachable only when no authoritative body was obtained.
This preserves the existing versioned-import compatibility tests while preventing
a nested inline definition or source lookalike from being flattened into its
parent module.

## Invariants

1. A successful canonical parse cannot be supplemented by raw source snippets.
2. An inline child’s public declarations remain in that child’s namespace.
3. Builtin/capability exports from a successful parse are created from typed
   definitions, never reparsed from source text.
4. Parser failure does not make a compatibility result authoritative for the
   canonical checked Core/CPS transport.
5. No role or policy declaration gains runtime or admission authority.

## Evidence

The implementation will add a failing regression for nested inline public
callables and a typed-definition regression for builtin/capability publication.
Existing versioned-import compatibility tests remain as the explicit fallback
evidence. Focused Engine tests, workspace tests, strict clippy, formatting, and
the documentation gates will be run before completion.
