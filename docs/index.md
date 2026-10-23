# Documentation

> Build the tool, run a program, and find the right source for a precise answer.

Page: https://dikini.github.io/ash/docs/  
Authority: website-guidance  
Source revision: c3f17060693478cb52ff73acc0568a51fb6337fa  
Updated: 2026-10-19

Use these entry points to move from an introduction to a working program or a precise language rule.

## Get running

[Install the pinned Ash tool](/ash/docs/install/) and run a complete example from the [short tour](/ash/learn/tour/). The current tool is built from source. No packaged installer is offered here.

## Language reference

Read the [Ash specification](/ash/spec/) on this site, with railroad diagrams for
the grammar and proof trees for inference rules. Its chapters cover:

- [Source text and tokens](/ash/spec/lexical-structure/)
- [Syntax](/ash/spec/syntax/)
- [Names, imports, and scope](/ash/spec/name-resolution/)
- [Types and checking](/ash/spec/static-semantics/)
- [Elaboration into typed Core](/ash/spec/elaboration/)
- [Evaluation](/ash/spec/dynamic-semantics/)

The specification is imported from ash-lang main at build time. Each page links
to its exact source revision. It may describe a newer language than the
[compiler checkpoint used by the tour](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/spec/README.md).

## Command-line tool

Read the [ash tool guide](/ash/docs/tools/ash/) for invocation, builds, module roots,
output, diagnostics, locales, and failure statuses. The [tools overview](/ash/docs/tools/)
links the available executables.

The guide and specification come from the same ash-lang revision. The
[installation instructions](/ash/docs/install/) and tour retain their pinned compiler
checkpoint; use the guide's source revision when exploring newer commands.

## Design notes

Explore the [design notes](/ash/docs/design/) for research, alternatives, experiments,
and supporting evidence. These documents are informative; proposals may be
incomplete or superseded. Use the language specification for current rules.

## Architectural decisions

Read the [architectural decisions](/ash/docs/decisions/) for accepted choices about
the parser, compiler representations, evaluation, tooling, and repository
workflow. Each record retains its status, context, consequences, and verification
requirements. These choices concern non-language architecture; the specification
defines current language rules.

Decisions, design notes, the specification, and the tool guide share the same
imported ash-lang revision.

## Lessons and worked programs

[Study Ash](/ash/study/) leads to ash-doc's sequential lessons, examples, exercises, and cases. Its current study target is separate from the website's language checkpoint. Use the instructions accompanying the material you choose.

## Before planning a project

Read [language status](/ash/status/) for implemented capabilities, experimental boundaries, and missing facilities. [Contribute](/ash/contribute/) explains how to investigate or help.
