---
title: "Build Ash. Run a program."
description: "Install the command-line tool from an exact source checkpoint, then evaluate your first program."
section: "Documentation"
route: "docs/install"
updated: "2026-10-18"
---

You need Git and a Rust toolchain managed by rustup. The repository's `rust-toolchain.toml` selects Rust {{checkpoint.toolchain}}. Cargo may download the selected toolchain and dependencies during the build.

These instructions target the website's examples. For ash-doc lessons, use [that study material's setup](/study/) instead.

## Get the source

Clone with the public HTTPS URL, then select the exact checkpoint:

```sh
git clone {{repository.ash-lang}}
cd ash-lang
git switch --detach {{checkpoint.revision}}
cargo build --locked --bin ash
```

The executable is `target/debug/ash` (`target/debug/ash.exe` on Windows). Detaching at the checkpoint keeps the tool aligned with the programs shown on this site.

## Run a first program

In the cloned repository, save the following complete source in `main.ash`:

```text
main : Unit -> Int
main () = 42
```

In a Unix-like shell, run:

```sh
./target/debug/ash main.ash
```

In PowerShell, use `.\target\debug\ash.exe main.ash` instead. The tool reports:

```text
42 : Int
```

The `42` is the returned value; `Int` is its type. This output comes from the command-line tool.

## Continue exploring

Run the complete programs from the [tour](/learn/tour/). In each case, give the tool the entry file's path. Relative module imports are resolved from source files; additional named module roots use `--root` options before the entry path.

The [pinned tool guide]({{source.ash-lang}}/blob/{{checkpoint.revision}}/docs/tools/ash.md) describes all options and diagnostic formats. If an example fails, compare the exact source and checkpoint before changing it.
