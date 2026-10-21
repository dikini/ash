---
title: "Meet Ash"
description: "A typed functional language connecting programs, behavior, and reasoning."
section: "Discover"
route: "discover"
updated: "2026-10-18"
---

Ash starts with functions and nominal data. It adds generics, interfaces, effect rows, lexical handlers, and scoped resources. The language is under active development.

## Begin with a concrete program

The [short language tour](/learn/tour/) develops three complete programs. You will see a value calculated by a function, a data definition handled by its cases, and a calculation whose request is answered by a surrounding handler.

## Make relationships visible

An effect row records what a computation can request. A handler gives an operation meaning in a lexical context. Scoped resource checking constrains where access can travel, including access captured in functions.

Interfaces describe required behavior. Explicit implementation selection determines the evidence supplied to constrained code. These are language mechanisms with precise rules, not promises that every program is correct.

Read the [specification overview]({{source.ash-lang}}/blob/{{checkpoint.revision}}/spec/README.md) for the exact model and the [status page](/status/) for its boundaries.

## Code and reasoning

Ash's longer-term goal is to connect executable programs with statements about their behavior, invariants, assumptions, and evidence. Ordinary computation, static guarantees, property checks, and proof-backed facts are part of that direction.

That ambition is broader than today's implemented language. Ash is not currently a general-purpose proof assistant. The project distinguishes accepted semantics, bounded experiments, and open research.

## Explore with the right expectations

Start with the [tour](/learn/tour/), [build the command-line tool](/docs/install/), or examine the [current capabilities and limitations](/status/). Researchers can follow the specification into the project's [design records]({{source.ash-lang}}/blob/{{checkpoint.revision}}/docs/design/README.md).
