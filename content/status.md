---
title: "A clear view of where Ash stands"
description: "Implemented capabilities, bounded experiments, and the work still ahead."
section: "Status"
route: "status"
updated: "2026-10-18"
---

Ash is an evolving language with an executable implementation and a normative specification. This page is a reviewed snapshot, not a live view of every branch.

**Website checkpoint:** [`bade38c`](https://github.com/dikini/ash-lang/tree/bade38c635755a608695dd58c8cbef1254aace4c). **Reviewed:** 18 October 2026. The [specification at that revision](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/spec/README.md) governs the language claims below.

## Implemented and specified

- Functions, rank-1 polymorphism, nominal data, tuples, pattern matching, and checked coverage.
- Passive source modules with explicit imports, visibility, and re-exports.
- Interfaces, explicit implementation selection, higher-kinded parameters, and nominal wrappers.
- Explicit effect rows and lexical handlers with checked continuation use.
- Scoped integer cells, native buffers, cleanup, and hooks with checked resource dependencies.
- A command-line evaluator with human and JSON diagnostics, including English and Bulgarian presentation.

The [tour](/learn/tour/) provides a small starting point. Consult the [specification](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/spec/README.md) and [tool guide](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/docs/tools/ash.md) for exact restrictions.

## Experimental work

The repository also retains bounded experiments involving foreign providers, SQLite, and scalar Arena storage through Rust-authored adapters. They do not establish a general FFI, arbitrary foreign bindings, or production readiness.

The [feature inventory](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/docs/project/implemented-features.md) records capabilities, evidence, and deferred work at its own declared inspected revision. Check that revision before treating it as a statement about a newer checkout.

## Current boundaries

The specified language does not provide first-class polymorphic schemes, polymorphic recursion, or a general foreign-binding command-line workflow. Concurrency, asynchronous execution, and dynamic loading are not facilities of this checkpoint's effect kernel.

This website offers source-based setup and examples. It makes no stability, compatibility, performance, or production-readiness promise. Evaluate the specific facilities your program needs.

## Direction and research

Connecting programs with properties and evidence is a long-term goal. It should not be read as a claim that arbitrary Ash programs already carry machine-checked correctness proofs.

Follow [development updates](/updates/) and the upstream [trajectory](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/docs/trajectory.md). Design notes explore possibilities; their existence does not mean the feature is accepted or implemented.

## A separate study checkpoint

Ash-doc's current material targets `diag-ash` at `9ec5920`. Its examples and checks belong to that edition. [Study Ash](/study/) explains the difference and links to the study source.
