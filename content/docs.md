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

Read the [Ash specification]({{route.spec}}) on this site, with railroad diagrams for
the grammar and proof trees for inference rules. Its chapters cover:

- [Source text and tokens]({{route.spec}}lexical-structure/)
- [Syntax]({{route.spec}}syntax/)
- [Names, imports, and scope]({{route.spec}}name-resolution/)
- [Types and checking]({{route.spec}}static-semantics/)
- [Elaboration into typed Core]({{route.spec}}elaboration/)
- [Evaluation]({{route.spec}}dynamic-semantics/)

The specification is imported from ash-lang main at build time. Each page links
to its exact source revision. It may describe a newer language than the
[compiler checkpoint used by the tour]({{source.ash-lang}}/blob/{{checkpoint.revision}}/spec/README.md).

## Command-line tool

Read the [ash tool guide]({{route.tools}}ash/) for invocation, builds, module roots,
output, diagnostics, locales, and failure statuses. The [tools overview]({{route.tools}})
links the available executables.

The guide and specification come from the same ash-lang revision. The
[installation instructions](/docs/install/) and tour retain their pinned compiler
checkpoint; use the guide's source revision when exploring newer commands.

## Design notes

Explore the [design notes]({{route.design}}) for research, alternatives, experiments,
and supporting evidence. These documents are informative; proposals may be
incomplete or superseded. Use the language specification for current rules.

## Architectural decisions

Read the [architectural decisions]({{route.decisions}}) for accepted choices about
the parser, compiler representations, evaluation, tooling, and repository
workflow. Each record retains its status, context, consequences, and verification
requirements. These choices concern non-language architecture; the specification
defines current language rules.

Decisions, design notes, the specification, and the tool guide share the same
imported ash-lang revision.

## Lessons and worked programs

[Study Ash](/study/) leads to ash-doc's sequential lessons, examples, exercises, and cases. Its current study target is separate from the website's language checkpoint. Use the instructions accompanying the material you choose.

## Before planning a project

Read [language status](/status/) for implemented capabilities, experimental boundaries, and missing facilities. [Contribute](/contribute/) explains how to investigate or help.
