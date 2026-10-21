---
title: "A clear view of where Ash stands"
description: "Implemented capabilities, bounded experiments, and the work still ahead."
section: "Status"
route: "status"
updated: "2026-10-18"
---

Ash is an evolving language with an executable implementation and a normative specification. This page is {{checkpoint.reviewState}}, not a live view of every branch.

**Website checkpoint:** [`{{checkpoint.short}}`]({{source.ash-lang}}/tree/{{checkpoint.revision}}). **Checkpoint review:** {{checkpoint.reviewed}}. The [specification at that revision]({{source.ash-lang}}/blob/{{checkpoint.revision}}/spec/README.md) governs the language claims below.

## Implemented and specified

- Functions, rank-1 polymorphism, nominal data, tuples, pattern matching, and checked coverage.
- Passive source modules with explicit imports, visibility, and re-exports.
- Interfaces, explicit implementation selection, higher-kinded parameters, and nominal wrappers.
- Effect rows and lexical handlers with checked continuation use.
- Scoped integer cells, native buffers, cleanup, and hooks with checked resource dependencies.
- A command-line evaluator with human and JSON diagnostics, including English and Bulgarian presentation.

The [tour](/learn/tour/) provides a small starting point. Consult the [specification]({{source.ash-lang}}/blob/{{checkpoint.revision}}/spec/README.md) and [tool guide]({{source.ash-lang}}/blob/{{checkpoint.revision}}/docs/tools/ash.md) for exact restrictions.

## Experimental work

The repository also retains bounded experiments involving foreign providers, SQLite, and scalar Arena storage through Rust-authored adapters. They do not establish a general FFI, arbitrary foreign bindings, or production readiness.

The [feature inventory]({{source.ash-lang}}/blob/{{checkpoint.revision}}/docs/project/implemented-features.md) records capabilities, evidence, and deferred work at its own declared inspected revision. Check that revision before treating it as a statement about a newer checkout.

## Current boundaries

The specified language does not provide first-class polymorphic schemes, polymorphic recursion, or a general foreign-binding command-line workflow. Concurrency, asynchronous execution, and dynamic loading are not facilities of this checkpoint's effect kernel.

This website offers source-based setup and examples. It makes no stability, compatibility, performance, or production-readiness promise. Evaluate the specific facilities your program needs.

## Direction and research

Connecting programs with properties and evidence is a long-term goal. It should not be read as a claim that arbitrary Ash programs already carry machine-checked correctness proofs.

Follow [development updates](/updates/) and the upstream [trajectory]({{source.ash-lang}}/blob/{{checkpoint.revision}}/docs/trajectory.md). Design notes explore possibilities; their existence does not mean the feature is accepted or implemented.

## A separate study checkpoint

Every website build pulls the latest ash-doc material and renders it as a development snapshot. Its examples and checks belong to its own declared compiler target. [Study Ash](/study/) shows the exact documentation revision and target used for this build.
