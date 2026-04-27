# Phase 101 Capability/Resource Parser Substrate

Phase 101 implements parser, surface AST, and module metadata carriers for the capability/resource syntax introduced by SPEC-052 and SPEC-053. It is intentionally a substrate phase only.

## Covered syntax carriers

The Phase 101 parser accepts and preserves AST/module metadata for:

```ash
pub capability interface KVStore:
    observe get(key: String) returns Option<String>
  | execute put(key: String, value: String) returns Unit;

pub resource type WorkflowKV {
    map: Map<String, String>
}

pub capability impl MemoryKV for KVStore
    requires resource kv: WorkflowKV
{
    observe get(key: String) returns Option<String> { key }
    execute put(key: String, value: String) returns Unit { value }
}

workflow example
    owns kv: WorkflowKV
    uses store: KVStore = MemoryKV(kv)
{
    done
}
```

The module layer also transports public/private metadata for capability interfaces, capability implementations, and resource types so imported names can retain their definition kind.

## Non-executable status

This syntax is not executable by virtue of Phase 101 support. In particular, Phase 101 does not implement:

1. typechecking of interface operation environments;
2. conformance checking that a `capability impl` satisfies its target interface;
3. typechecking of `owns` or `uses` workflow header clauses;
4. resource allocation, resource identity, split/join/share/move policy, or authority provenance runtime behavior;
5. execution of Ash-defined capability implementation bodies;
6. admission-time capability binding or dependency validation.

Those behaviors are owned by later Phase 102 through Phase 104 tasks. Until then, examples using `capability interface`, `capability impl`, `resource type`, `owns`, or `uses` should be treated as parser/module metadata examples, not runnable workflow programs.
