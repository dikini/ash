---
title: "Documentation"
description: "Build the tool, run a program, and find the right source for a precise answer."
section: "Documentation"
route: "docs"
updated: "2026-10-19"
---

Use these entry points to move from an introduction to a working program or a precise language rule.

## Get running

[Install the pinned Ash tool](/docs/install/) and run a complete example from the [short tour](/learn/tour/). The current tool is built from source. No packaged installer is offered here.

## Language reference

Read the [Ash specification](/spec/) on this site, with railroad diagrams for
the grammar and proof trees for inference rules. Its chapters cover:

- [Source text and tokens](/spec/lexical-structure/)
- [Syntax](/spec/syntax/)
- [Names, imports, and scope](/spec/name-resolution/)
- [Types and checking](/spec/static-semantics/)
- [Elaboration into typed Core](/spec/elaboration/)
- [Evaluation](/spec/dynamic-semantics/)

The specification is imported from ash-lang main at build time. Each page links
to its exact source revision. It may describe a newer language than the
[compiler checkpoint used by the tour](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/spec/README.md).

## Command-line tool

Read the [ash tool guide](/docs/tools/ash/) for invocation, builds, module roots,
output, diagnostics, locales, and failure statuses. The [tools overview](/docs/tools/)
links the available executables.

The guide and specification come from the same ash-lang revision. The
[installation instructions](/docs/install/) and tour retain their pinned compiler
checkpoint; use the guide's source revision when exploring newer commands.

## Design notes

Explore the [design notes](/docs/design/) for research, alternatives, experiments,
and supporting evidence. These documents are informative; proposals may be
incomplete or superseded. Use the language specification for current rules.

The notes, specification, and tool guide share the same imported ash-lang revision.

## Lessons and worked programs

[Study Ash](/study/) leads to ash-doc's sequential lessons, examples, exercises, and cases. Its current study target is separate from the website's language checkpoint. Use the instructions accompanying the material you choose.

## Before planning a project

Read [language status](/status/) for implemented capabilities, experimental boundaries, and missing facilities. [Contribute](/contribute/) explains how to investigate or help.
