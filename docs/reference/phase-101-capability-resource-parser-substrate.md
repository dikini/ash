# Phase 101 Capability/Resource Parser Substrate

Phase 101 implements parser, surface AST, and module metadata carriers for the capability/resource syntax introduced by SPEC-052 and SPEC-053. It is intentionally a substrate phase only.

## Historical syntax carriers

Phase 101 once added parser and metadata carriers for the old capability declaration pair,
resource type declarations, and old entry ownership/use clauses. Phase 201 removed current
parser/typechecker/tooling support for the old capability declaration pair, and current Ash code
must use target provider/resource syntax instead of those historical declarations.

This page is retained only as a historical phase note. It is not current syntax guidance, and it
must not be used as an executable example source.

## Non-executable status

The historical substrate was not executable by virtue of Phase 101 support. In particular, Phase 101 did not implement:

1. typechecking of interface operation environments;
2. conformance checking between historical capability declarations;
3. typechecking of ownership/use header clauses;
4. resource allocation, resource identity, split/join/share/move policy, or authority provenance runtime behavior;
5. execution of historical Ash-defined provider bodies;
6. admission-time capability binding or dependency validation.

Those behaviors were owned by later Phase 102 through Phase 104 tasks. Current work should follow
Phase 201 target Ash syntax and should not reintroduce the removed declarations as parser fixtures
or examples.
