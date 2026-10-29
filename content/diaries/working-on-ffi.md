---
title: "Working on FFI in Ash"
description: "Why FFI became the next development target, what it should enable, and why the work reaches into resources, effects, and type checking."
author: "Vlado"
author_uri: "https://github.com/dikini"
draft: false
published: "2026-10-29"
updated: "2026-10-29"
topics:
  - ffi
  - language-design
section: "Developer Diaries"
route: "diaries/working-on-ffi"
---

<!-- Imported from https://github.com/dikini/ash-blog/blob/c87de4096b4a0873eefb5094cad87013148cfed6/entries/working-on-ffi/index.md; section and route supplied by ash-site. -->

Ash has reached a point where it is sufficiently expressive to write complex logic and check important aspects of its correctness. But, in practical terms, it still works as a self-contained calculator. I want to write parsers, serialisers, support different file formats, use networking, and build systems on top of those things.

That is why the foreign function interface, or FFI, became the next development target. It lets an Ash program call code in libraries written in other languages. There are already libraries for the services we need. We need a way to use them.

The work is in progress, now in its final stage. The planning revealed how broad it would be. Calling a function is relatively simple in theory. Keeping the static guarantees we want around that call brings in rather more of the language.

## What I want to use it for

The first things I want to build are fairly small. Support for system-aligned integers and integer arithmetic. Similar support for floating-point numbers and strings. Support for CSV files. Ports of useful libraries.

System-aligned integers, floating-point numbers, and strings were deliberately held back. Ash has basic primitive types, but I want these facilities to follow system conventions. The FFI work provides the boundary at which those choices have to become concrete. Mathematical functions and linear algebra are further examples of functionality we should be able to obtain from existing libraries.

These are useful individually. Together they make more kinds of programs possible. A parser becomes more useful when it can read a file. A serialiser needs somewhere to write its output. Supporting a file format should let us do something with actual files.

The aim is to link an Ash program against an arbitrary system library. The library author defines the Ash side of the interface, playing a role similar to a `.h` file in C. That interface expresses the required types, interfaces, and effects, so that the rest of the Ash program can use the library through Ash's own abstractions.

## Libraries and the runtime

There is a longer-term reason for doing this too. The goal is minimal to no general runtime. We are not there yet.

Memory allocation and management should be possible as Ash libraries, including optional garbage collection. Those libraries need access to operating-system services. File operations need the same sort of access. Different concurrency models should also be possible as libraries, with limited or no general runtime support.

These are examples of the avenues FFI opens. They are not all part of the immediate delivery. But access to system libraries is a foundation for exploring them without putting every capability into the language or its runtime.

## Why the work became broad

Consider a memory region: a block of memory acquired for some period of use, then disposed of.

Acquiring it is only the beginning. What happens if there is no memory available? How long does the region remain alive? Who owns the data stored in it? Who can read that data, and who can change it? What happens to references when the region is disposed of?

These questions belong to the interface we want Ash programs to use. Resources, lifetimes, ownership, references, and effects have to fit together, and type checking has to account for their relationships.

A test in the current development worktree connects this directly to native memory. It starts with an Ash declaration of the native interface:

```ash
import @builtin.foreign as F
pub foreign library RegionFixture {
    profile "x86_64-unknown-linux-gnu"
    symbol Make : Unit -> F.Address { name "region_make" }
    symbol Allocate : F.Address -> F.Address { name "region_allocate" }
    symbol Release : F.Address -> Unit { name "region_release" }
    symbol Inspect : Bytes -> Unit { name "region_inspect" }
    symbol InspectHost : Bytes -> Unit { name "region_inspect_host" }
}
```

This is the header-like part. It names the target platform and describes the functions supplied by the native library. `F.Address` represents a native address. The last two declarations support inspections in other cases from the same test file.

An Ash provider, omitted here, connects allocation requests to `Allocate` and pairs them with a `Release` action. It also describes the size and alignment of the supplied storage. The program then uses that provider:

```ash
main : Unit -> Int ! {Raw.Make, Raw.Allocate, Raw.Release}
main () = {
    let state = perform Raw.Make.call ()
    region<R> owner = (provider state) {
        match (20, 22) {
            (left, right) => match (left + right) == 42 {
                true => 42
                false => 0
            }
        }
    }
}
```

These are excerpts, rather than a standalone program: `Raw` imports the native declarations, and `provider` is defined between them and `main` in the test. The `!` part of the function signature lists the native effects the program uses.

The arithmetic is deliberately uninteresting. The point is where the tuple is stored and when that storage is released. The C fixture obtains aligned memory from the system. The test checks that Ash uses the supplied memory and that the paired release runs afterwards. The resulting integer can leave the region; access to the region's storage must not outlive it.

The [native region test](https://github.com/dikini/ash-lang/blob/b228277316025c37bf2b2da01ad412147729cad1/tests/region_native_storage.rs) contains the declarations, provider, C fixture, and lifecycle checks. Separate [region lifetime tests](https://github.com/dikini/ash-lang/blob/b228277316025c37bf2b2da01ad412147729cad1/tests/region_scope_runtime.rs) reject returning a function whose own storage belongs to a region that is closing. These examples come from the work-in-progress checkpoint, not a claim that the whole FFI effort is complete.

Effects bring control flow into the picture as well. Allocation can fail, and that failure needs to be handled. Ash implements effects using continuations: the remaining computation that a handler can resume. Resource use therefore has to agree with what happens to that computation, including when it does not continue.

This is where a seemingly small FFI task starts to involve much of the type and effect system. We are already holding back multi-shot continuations, which would allow a captured computation to be resumed more than once, to keep the work manageable.

## What the interface guarantees

We cannot give guarantees about the foreign code itself. Describing a native function does not establish that its implementation behaves correctly.

The guarantees belong at the Ash interface: the exported types and the stated contracts. The work is to make those promises meaningful at the boundary, while keeping the obligations on the foreign implementation explicit. A guarantee that Ash code uses a resource within its permitted lifetime does not establish that native code manages its own memory correctly.

In the future, I also want libraries to express contracts and constraints as laws, possibly supported by property tests. As discussed in [the verification entry](/diaries/exploring-verification/), stating a claim and establishing it are separate things. Those facilities remain future work.

## Where the work stands

The current FFI work is in its final stage. The remaining GitHub issues are part of that effort; this is still a progress report, not a completed release.

The immediate aim remains practical: link against system libraries and start building the small supporting libraries Ash needs. Integers, floating-point numbers, strings, files, and useful formats. Basic work, but work that makes it possible to write useful programs on top.
