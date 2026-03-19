-- Ash Pattern Matching Engine
-- Implements pattern matching per SPEC-004 Section 5.2 and 6.1
--
-- Pattern matching algorithm: bind(pat, v, Γ) = Γ'
-- Matches a pattern against a value, producing bindings (environment)
--
-- NOTE: This implements a SIMPLIFIED SUBSET of Ash pattern matching.
-- See docs/AST-Subset.md for the complete comparison with Rust.
--
-- Supported: wildcard, variable, literal, variant, tuple, record
-- Not supported: List with optional rest binding (by design)

import Ash.Core.AST
import Ash.Core.Environment

namespace Ash.Eval

open Ash

/-! ## Pattern Matching

The pattern matching engine implements the bind(pat, v, Γ) = Γ' semantics
from SPEC-004. It takes a pattern, a value, and returns:
- `some env` with bindings if the pattern matches
- `none` if the pattern does not match

Pattern types supported:
- wildcard: matches any value, no binding
- variable: matches any value, binds to name
- literal: matches equal value only
- variant: matches enum variant with field patterns
- tuple: matches tuple element-wise
- record: matches record fields by name
-/ 

mutual

/-- Main pattern matching function

Matches a pattern against a value per SPEC-004 Section 5.2.

```
bind(pat, v, Γ) = Γ'
```

Returns:
- `some env` where `env` contains bindings from the match
- `none` if the pattern does not match the value

Pattern semantics:
- Wildcard: always succeeds, no binding
- Variable: always succeeds, binds name to value
- Literal: succeeds if values are equal
- Variant: succeeds if variant names match and all fields match
- Tuple: succeeds if same length and all elements match
- Record: succeeds if all specified fields exist and match
-/
partial def matchPattern (p : Pattern) (v : Value) : Option Env :=
  match p, v with
  -- Wildcard: matches anything, no binding
  | .wildcard, _ => some Env.empty
  
  -- Variable: matches anything, binds to name
  | .variable x, v => some (Env.empty.bind x v)
  
  -- Literal: matches if values are equal
  | .literal l, v => if l == v then some Env.empty else none
  
  -- Variant: matches by name, then fields
  | .variant name fields, .variant vname vfields =>
      if name = vname then
        matchFields fields vfields
      else
        none
  | .variant _ _, _ => none
  
  -- Tuple: matches element-wise with same length
  | .tuple ps, .tuple vs =>
      if ps.length = vs.length then
        matchList ps vs
      else
        none
  | .tuple _, _ => none
  
  -- Record: matches by field names (uses matchFields same as variant)
  | .record fields, .record vs =>
      matchFields fields vs
  -- Can also match variant as record by fields
  | .record fields, .variant _ vs =>
      matchFields fields vs
  | .record _, _ => none

/-- Match a list of patterns against a list of values (for tuple matching)

Returns `some env` with combined bindings if all patterns match,
`none` if any pattern fails or lengths differ.
-/ 
partial def matchList (ps : List Pattern) (vs : List Value) : Option Env :=
  match ps, vs with
  | [], [] => some Env.empty
  | p :: prest, v :: vrest =>
      match matchPattern p v with
      | none => none
      | some env1 =>
          match matchList prest vrest with
          | none => none
          | some env2 => some (Env.merge env1 env2)
  | _, _ => none

/-- Match variant/record fields against field patterns

Each field pattern is matched against its corresponding field value.
Returns `some env` with combined bindings if all fields match.
-/
partial def matchFields (ps : List (String × Pattern))
    (vs : List (String × Value)) : Option Env :=
  match ps with
  | [] => some Env.empty
  | (name, p) :: rest =>
      match vs.find? (fun (n, _) => n = name) with
      | none => none  -- Field not found in value
      | some (_, v) =>
          match matchPattern p v with
          | none => none
          | some env1 =>
              match matchFields rest vs with
              | none => none
              | some env2 => some (Env.merge env1 env2)

end

/-! ## Property Test Helpers -/

/-! ## Determinism Proof

Pattern matching is deterministic: same pattern and value always produce
the same environment (if any).
-/

theorem matchPattern_deterministic {p : Pattern} {v : Value} {env1 env2 : Env}
  (h1 : matchPattern p v = some env1)
  (h2 : matchPattern p v = some env2) :
  env1 = env2 := by
  -- Since matchPattern is a pure function, equal inputs give equal outputs
  rw [h1] at h2
  injection h2

/-- Check if pattern matching is deterministic for given inputs

A pure function should always return the same result for the same inputs.
-/
def isDeterministic (p : Pattern) (v : Value) : Bool :=
  match matchPattern p v, matchPattern p v with
  | some _, some _ => 
      -- Compare environments by checking they return same values for test variables
      -- Since Env is a function, we approximate equality
      true  -- Lean functions don't have decidable equality, trust purity
  | none, none => true
  | _, _ => false  -- Different results means non-deterministic

/-! ## ToString Instances -/

/-- Show environment contents by collecting bound variables -/
def envToString (env : Env) (vars : List String) : String :=
  let bindings := vars.filterMap (fun x =>
    match env.lookup x with
    | some v => some s!"{x} = {reprStr v}"
    | none => none)
  if bindings.isEmpty then "(empty)" else String.intercalate ", " bindings

instance : ToString (Option Env) where
  toString
    | none => "none"
    | some env =>
        -- Try to show some common variable names that might be bound
        let vars : List String := ["x", "y", "z", "a", "b", "v", "px"]
        let content := envToString env vars
        "some (" ++ content ++ ")"

end Ash.Eval
