# Design Note: The `Process` Capability and Effect Model

## Status: Draft (Decision D1)

## 1. Question

Should subprocess spawn (`std::process::run`) be:

- **(A)** A raw operational primitive, available to any workflow without explicit permission?
- **(B)** A `Capability`-typed resource, requiring provider registration and role/policy mediation?
- **(C)** A built-in capability with an auto-registered default provider, giving convenience + governance?

## 2. Analysis

### 2.1 Why not (A) raw primitive

Making `process::run` a raw operational effect would contradict Ash's governance-first design. Subprocess execution is:
- **High-risk** (arbitrary code execution, side effects, resource exhaustion)
- **Non-undoable** (a spawned process cannot be rewound)
- **Audit-critical** (every subprocess must be traceable to a role/policy decision)

A raw primitive would also create an inconsistency: `stdio` and `fs` are capability-governed, but `process` is not.

### 2.2 Why pure (B) is too heavy

Treating `process::run` as a user-declared capability (like a custom HTTP provider) would create excessive friction:

```ash
-- Too verbose for a standard library primitive
capability process_spawn : act(cmd: String, args: List<String>) returns ProcessOutput

workflow main {
    act process_spawn("ash", ["check", "file.ash"]);
}
```

This would force every script that runs `ash check` to declare a capability, undermining the spec processor's usability and violating the "stdlib just works" expectation.

### 2.3 Resolution: (C) Built-in capability with auto-registered default provider

`Process` is a **built-in capability** with the same status as `stdio` and `fs`:

- It has the **Operational** effect.
- It is **auto-registered** by `Engine::default()` and the CLI builder.
- It is **governed**: policies can restrict it, roles can be required, and runtime constraints (timeout, allowed command whitelist, working-directory sandbox) are enforced by the provider.
- It is **implicit** in user code: `std::process::run(cmd, args)` delegates to the capability internally, so the caller does not write `act process_spawn(...)`.

## 3. Capability contract

```ash
-- Canonical stdlib interface (user-facing)
pub fn run(cmd: String, args: List<String>) -> Result<ProcessOutput, ProcessError>;

-- Capability provider contract (engine-facing)
capability process {
    effect: operational,
    params: [cmd: String, args: List<String>, options: ProcessOptions],
    returns: ProcessOutput,
    -- Role and policy enforcement happen at the provider level
}
```

The `process` capability provider is implemented in Rust as a built-in (`crates/ash-engine` or `ash-interp`), not as an embedder-supplied plugin.

## 4. Runtime constraints

The default `Process` provider enforces:

- **Timeout** (configurable at engine build time)
- **Working-directory sandbox** (optional)
- **Exit-code capture** (mandatory)
- **Stdout/stderr capture** (mandatory)
- **Command whitelist** (optional; default = unrestricted for CLI, restrictable for sandboxed embeddings)

## 5. Implications for the spec processor

- The spec processor can call `process::run("ash", ["check", path])` without declaring a capability in its source text.
- In a sandboxed embedding (e.g. a web IDE), the `Process` capability can be disabled or restricted without changing the processor's source code.
- The processor's `capability_boundary.ash` declares `process_spawn: true` when the default provider is present.

## 6. Implications for future stdlib modules

This pattern generalizes to other "dangerous but standard" operations:

| Module | Capability | Effect | Auto-registered? |
|--------|-----------|--------|------------------|
| `std::io::stdio` | `stdio` | Operational | Yes |
| `std::io::fs` | `filesystem` | Operational | Yes |
| `std::process` | `process` | Operational | Yes |
| `std::http` | `http` | Operational | **No** (embedder must opt-in) |
| `std::crypto` | `crypto` | Operational | **No** (embedder must opt-in) |

**Rule of thumb:** If a primitive is needed for the spec processor to bootstrap itself, it gets auto-registered. If it is domain-specific or carries elevated security risk, it requires explicit embedder registration.

## 7. Open question: `ProcessOptions` shape

The exact fields of `ProcessOptions` (timeout, cwd, env vars, stdin redirect) are not frozen here. They will be defined in the `std::process` interface spec (`std/src/process.ash`) during implementation.

## 8. Decision

**Adopt option (C).** Subprocess spawn is a built-in, auto-registered `Capability` with the `Operational` effect. It is governed by the capability system but implicit in user-facing stdlib code.

This unblocks:
- `std::process` interface design (Task B4)
- Example-syntax conformance via subprocess spawn (Task A6 / Gate D2)
- The spec processor's self-hosting validation loop
