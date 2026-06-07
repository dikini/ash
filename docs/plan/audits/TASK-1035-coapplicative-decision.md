# TASK-1035 Coapplicative Decision

Decision: defer

No source module is added for `std::algebra::coapplicative` in Phase 134.

## Rationale

`Coapplicative` is not a single settled Ash-facing contract in the current
stdlib. The term can point at different duals of applicative structure, and the
current source surface has no lawful first-slice carrier that can demonstrate a
precise method set and laws through final stdlib paths.

Implementing a `Coapplicative` interface now would therefore be a placeholder
API. SPEC-079 requires either precise laws plus a lawful carrier or explicit
deferral. This phase chooses explicit deferral.

## Blockers

- No selected Ash-facing law formulation has been accepted for the name.
- No current stdlib carrier provides a lawful final-surface instance.
- No final-surface example can exercise the interface without adding a carrier
  only for the decision gate.
- A source module would imply an implemented contract that the project cannot
  currently test or instantiate honestly.

## Follow-up Shape

A later packet may introduce `Coapplicative` only after it names the intended
contract, laws, and at least one lawful source-denotable carrier. Until then,
`std/src/algebra/coapplicative.ash` must remain absent.
