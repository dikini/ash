# NOTE-026: Newtype and Phantom Types

**Date:** 2026-06-28
**Status:** Living document — design direction captured; derivation deferred
**Purpose:** Define the `newtype` mechanism: a zero-cost nominal wrapper that gives an
existing type a distinct identity while sharing its runtime representation. This unifies
newtype wrappers and phantom types into one mechanism. Record the grammar, type-system
semantics, coercion model, effect-identity interaction, and worked examples.

Companion to NOTE-025 (effect identity via sorts and impls — bodyless types), NOTE-022
(declaration side: interfaces), NOTE-023 (dispatch side: handler marker), and NOTE-027
(transparent alias canonicalization, deferred or in SPEC-058).

## Pre-Spec Delta

This note is pre-spec. When the project moves to spec updates, reconcile:

- **Grammar:** SPEC-095b gains a `newtype_definition` production in the definition list
  (§3.2) and a dedicated subsection in §6 (Types). The production is distinct from
  `type_definition` (which covers alias, enum, struct, record, tuple, and bodyless).
- **Type system:** SPEC-097b gains definitional equality rules for newtypes: a newtype is
  never equal to its representation type (nominal), unlike a transparent alias which is
  definitionally equal to its origin.
- **Runtime representation:** the IR (SPEC-098b) erases the newtype wrapper at runtime —
  a `newtype Age = Age(Int)` value occupies the same memory as an `Int`. This is a
  representation erasure, not a semantic identity change.
- **Definitional equality:** a newtype is never definitionally equal to its representation.
  Two distinct newtypes wrapping the same representation are also never equal.

## 0. Motivation

NOTE-025 §7.1 resolved that bodyless types (`type PosixFs;`) suffice for effect identity
carriers that carry no data. But some effect identity carriers (and other types) need to
*wrap an existing inhabited type* while preserving distinct identity. The options were:

- `type CustomFs = PosixFs;` — transparent alias. Collapses identity: `CustomFs ≡ PosixFs`
  at definitional equality. Unsuitable for effect identity.
- `type CustomFs = { inner: PosixFs };` — record body. Gives distinct identity, but carries
  full runtime overhead: a struct with one field, allocated and accessed as a record.
- **Missing option:** a zero-cost wrapper that gives distinct identity while sharing the
  representation. This is Haskell's `newtype`, and it is what NOTE-026 introduces.

Additionally, phantom types — type parameters that appear in the type but not in the
representation — are a common need (type-level tagging, state-machine encoding). A
transparent alias `type Tagged<L> = String` erases `<L>` at equality (`Tagged<Admin> ≡
Tagged<User> ≡ String`). The newtype mechanism handles phantom parameters naturally without
a separate feature.

## 1. The Mechanism

A `newtype` wraps exactly one existing type (the *representation type*), giving the wrapper
a distinct nominal identity while sharing the representation type's runtime layout. The
wrapper is erased at runtime — zero-cost.

```ash
newtype Age = Age(Int);
newtype Milliseconds = Milliseconds(Float);
newtype Tagged<L> = Tagged(String);   -- L is phantom: not in representation
newtype CustomFs = CustomFs(PosixFs);
```

Each declares:
- A new nominal type (`Age`, `Milliseconds`, `Tagged<L>`, `CustomFs`) distinct from all
  other types, including its representation type.
- A constructor (`Age`, `Milliseconds`, `Tagged`, `CustomFs`) that wraps a value of the
  representation type.
- Zero runtime overhead: the representation type's memory layout is reused.

### 1.1 The four type forms compared

| Form | Identity | Constructors | Representation | Runtime cost | Inhabited |
|---|---|---|---|---|---|
| `type T = R;` (transparent alias) | ≡ R | R's | R | zero (same type) | R's values |
| `type T;` (bodyless) | distinct | none | none | N/A | no (uninhabited) |
| `newtype T = T(R);` | distinct | T (wrapper) | R (erased at runtime) | zero (same as R) | yes |
| `type T = { ... };` (record) | distinct | record literal | own | full | yes |

The critical rows are alias vs newtype: both are zero-cost at runtime, but alias
canonicalizes (T ≡ R, identity collapses) while newtype is nominal (T ≠ R, identity
preserved). This is exactly the distinction that makes newtype safe for effect identity
while alias is not.

## 2. Grammar

```ebnf
newtype_definition = "newtype" identifier [ type_params ] "=" constructor "(" type ")" ";" ;

constructor = identifier ;
```

A dedicated `newtype` keyword rather than overloading `type` further. The `type` keyword is
already split between transparent alias (`type T = R;`) and data forms (`type T = { ... };`,
`type T = Nil | Cons ...;`), plus the bodyless form (`type T;`). Newtype has fundamentally
different semantics (zero-cost representation erasure with a single wrapper constructor), so
it deserves its own keyword for clarity at the declaration site.

### 2.1 Constructor naming

The constructor name can match or differ from the type name (Haskell convention):

```ash
newtype Age = Age(Int);           -- constructor matches type name (common, readable)
newtype Email = MkEmail(String);  -- constructor differs (disambiguation when needed)
```

### 2.2 Type parameters

Newtype supports type parameters, same as other type definitions. A parameter that does not
appear in the representation type is phantom:

```ash
newtype Tagged<L> = Tagged(String);     -- L is phantom: appears in type, not in representation
newtype State<S> = State(Unit);         -- S is phantom: used only for type-level distinction
```

Multiple phantom parameters are supported:

```ash
newtype KV<K, V> = KV(V);               -- K is phantom, V is representation
```

Multiple parameters can mix phantom and representation roles:

```ash
newtype PhantomAndData<L, A> = Wrapper(A);  -- L is phantom, A is representation
```

### 2.3 Representation type constraint

The representation type must be inhabited (have at least one value). A newtype wrapping a
bodyless type is a type error — you cannot wrap a type that has no values:

```ash
type PosixFs;                      -- bodyless: uninhabited
newtype Wrap = Wrap(PosixFs);      -- ERROR: representation type `PosixFs` has no values
```

The bodyless form already covers the identity-only case (NOTE-025 §7.1). Newtype is for
wrapping an *inhabited* type — one that has runtime values.

## 3. Type-System Semantics

### 3.1 Definitional equality

A newtype is **never** definitionally equal to its representation type. This is the core
property that distinguishes newtype from transparent alias.

```text
newtype Age = Age(Int);

Age ≢ Int          -- never equal, even though runtime representation is the same
Age ≢ Age          -- well-formed: equal to itself
```

Two distinct newtypes wrapping the same representation are also never equal:

```text
newtype Age = Age(Int);
newtype Count = Count(Int);

Age ≢ Count        -- distinct nominal types, even though both wrap Int
```

This contrasts with transparent aliases:

```text
type AgeAlias = Int;
type CountAlias = Int;

AgeAlias ≡ CountAlias ≡ Int   -- transparent aliases canonicalize to Int
```

### 3.2 Constructor and pattern matching

The newtype constructor is the sole way to produce a value of the newtype:

```ash
let age: Age = Age(42);           -- wrap: constructor application, zero-cost
```

And the sole way to extract the representation:

```ash
let Age(n) = age;                 -- unwrap: pattern match, zero-cost
let Age(n): Age = Age(42);        -- wrap-then-unwrap: n is 42, zero-cost round-trip
```

Pattern matching on a newtype constructor is exhaustive (there is only one constructor) and
total.

### 3.3 Representation erasure at runtime

At runtime, the newtype wrapper is erased. A value of type `Age` occupies the same memory
and has the same runtime representation as its underlying `Int`. The constructor is a
no-op at the representation level — wrapping and unwrapping compile to identity operations.

This is a **representation erasure**, not a semantic identity change. The type system
maintains the nominal boundary; the runtime does not.

## 4. Coercion Model

### 4.1 Explicit wrapping and unwrapping

Wrapping and unwrapping are explicit and zero-cost:

```ash
let age = Age(42);              -- wrap: constructor application
let Age(n) = age;               -- unwrap: pattern match
```

### 4.2 No automatic coercion

`Age` and `Int` are completely distinct types. You cannot pass an `Age` where `Int` is
expected, or vice versa, without explicit wrapping:

```ash
fn add(a: Int, b: Int) -> Int { a + b }

let age = Age(42);
add(age, 1);                    -- ERROR: expected Int, found Age
add(Age(42), 1);                -- ERROR: expected Int, found Age
```

This matches the principle that no conversions are truly safe (bottom behavior, etc.).
Explicit wrapping is the sole path between a newtype and its representation.

### 4.3 Unsafe coercion (deferred)

If an unsafe direct coercion is needed (bypass the constructor for performance), it would be
an explicit `coerce_unsafe` or similar. This is **deferred** — not part of the core newtype
mechanism. The basic mechanism provides only explicit constructor/pattern-match coercion.

## 5. Phantom Types

A type parameter that does not appear in the representation type is phantom. The type system
tracks it at compile time; it is erased at runtime.

```ash
-- Type-level role tagging
newtype Tagged<L> = Tagged(String);

let admin_data: Tagged<Admin> = Tagged("secret");
let user_data: Tagged<User> = Tagged("public");

-- admin_data and user_data are distinct types at compile time
-- both are String at runtime
```

### 5.1 Why transparent alias fails for phantom types

A transparent alias erases phantom parameters at definitional equality:

```ash
type TaggedAlias<L> = String;   -- transparent alias

TaggedAlias<Admin> ≡ TaggedAlias<User> ≡ String   -- all equal!
```

The phantom parameter `<L>` is erased because the alias canonicalizes to its origin head
(`String`). This defeats the purpose: `TaggedAlias<Admin>` and `TaggedAlias<User>` are the
same type, so the type system cannot distinguish them.

A newtype preserves the distinction:

```ash
newtype Tagged<L> = Tagged(String);

Tagged<Admin> ≢ Tagged<User>   -- distinct nominal types
```

### 5.2 State-machine encoding

Phantom types can encode state machines at the type level:

```ash
type Unconnected;              -- bodyless: state marker
type Connected;                -- bodyless: state marker
type Authenticated;            -- bodyless: state marker

newtype Connection<S> = Connection(ConnHandle);   -- S is phantom state

-- Only callable when S = Connected
fn send<S>(conn: Connection<Connected>, msg: String) -> {Net::send} Unit { ... }

-- Transition: Connected -> Authenticated
fn authenticate(conn: Connection<Connected>) -> Connection<Authenticated> { ... }
```

The phantom parameter `S` tracks the connection state at compile time. The runtime
representation is always `ConnHandle` regardless of state.

## 6. Effect Identity Interaction

A newtype produces a distinct effect identity, exactly like any nominal type. This is the
primary motivation from NOTE-025: wrapping an existing impl type to create a new identity.

```ash
newtype CustomFs = CustomFs(PosixFs);

impl Fs for CustomFs
where requires host posix_fs
{
    fn read(path: Path) -> String { builtin(fs_read, path) }
    fn write(path: Path, contents: String) -> Unit { builtin(fs_write, path, contents) }
}
```

`CustomFs::read` is a distinct effect identity from `PosixFs::read`. The impl method body
calls `builtin(fs_read, path)` directly — it does NOT delegate to `PosixFs::read` (which
would raise `PosixFs::read`, a different effect). The newtype's identity is `CustomFs`, so
`CustomFs::read` is the raised effect identity. The wrapped representation is irrelevant to
the effect system.

### 6.1 Coherence

The strong coherence rule from NOTE-025 §7.7 applies: for any type `T`, each operation name
may be defined by at most one interface. Since `CustomFs` is a distinct type from `PosixFs`,
both can implement `Fs` without conflict — they produce distinct operation identities
(`CustomFs::read` vs `PosixFs::read`).

### 6.2 Zero-cost identity wrapping

The zero-cost property is valuable for effect identity carriers that wrap existing
infrastructure types. Wrapping `PosixFs` as `CustomFs` adds no runtime overhead — the handler
dispatches on the compile-time identity, and the runtime representation is shared.

## 7. Worked Examples

### 7.1 Domain primitives (zero-cost wrappers)

```ash
newtype Age = Age(Int);
newtype Milliseconds = Milliseconds(Float);
newtype UserId = UserId(String);

-- Cannot mix them up: distinct types, no automatic coercion
fn is_adult(age: Age) -> Bool {
    let Age(years) = age;
    years >= 18
}

let user = UserId("abc-123");
is_adult(user);                   -- ERROR: expected Age, found UserId
```

### 7.2 Phantom type tagging

```ash
newtype Tagged<L> = Tagged(String);

type Admin;
type User;

let admin_secret: Tagged<Admin> = Tagged("root-key");
let user_input: Tagged<User> = Tagged("hello");

-- The type system prevents cross-contamination
fn reveal_admin(s: Tagged<Admin>) -> String {
    let Tagged(inner) = s;
    inner
}

reveal_admin(user_input);         -- ERROR: expected Tagged<Admin>, found Tagged<User>
```

### 7.3 Effect identity carrier (zero-cost wrapping of an inhabited type)

```ash
type PosixFs;                      -- bodyless: identity-only (NOTE-025)

newtype AuditedFs = AuditedFs(PosixFs);
-- ERROR: PosixFs is uninhabited (bodyless), cannot be a representation type.
```

This is the constraint from §2.3 in action. To wrap an effect identity carrier that carries
data:

```ash
type ConfiguredFs = { root: Path, readonly: Bool };   -- data-carrying (inhabited)

newtype AuditedFs = AuditedFs(ConfiguredFs);           -- OK: inhabited representation

impl Fs for AuditedFs
where requires host posix_fs
{
    fn read(path: Path) -> String {
        let AuditedFs(ConfiguredFs { root, readonly: _ }) = self;
        builtin(fs_read, root.join(path))
    }
    fn write(path: Path, contents: String) -> Unit {
        let AuditedFs(ConfiguredFs { root, readonly }) = self;
        if readonly { panic("readonly") } else { builtin(fs_write, root.join(path), contents) }
    }

    derive handler audited_fs;
}
```

`AuditedFs::read` is a distinct effect identity. The runtime representation is
`ConfiguredFs` (a record), shared zero-cost.

### 7.4 Multiple phantom parameters

```ash
newtype PhantomAndData<L, A> = Wrapper(A);  -- L phantom, A representation

type ReadOnly;
type ReadWrite;

let ro: PhantomAndData<ReadOnly, String> = Wrapper("data");
let rw: PhantomAndData<ReadWrite, String> = Wrapper("data");

-- ro and rw are distinct types despite same representation value
fn write_data(p: PhantomAndData<ReadWrite, String>, s: String) -> Unit { ... }

write_data(ro, "new");             -- ERROR: expected ReadWrite, found ReadOnly
write_data(rw, "new");             -- OK
```

## 8. Deriving Impls (Deferred)

Haskell's `GeneralizedNewtypeDeriving` lets you derive a typeclass instance for a newtype by
lifting through the wrapper. In Ash, this would look like:

```ash
newtype CustomFs = CustomFs(PosixFs);
derive Fs for CustomFs;            -- hypothetical: lifts PosixFs's Fs impl through the wrapper
```

This is **deferred** and not part of the core newtype mechanism. The subtlety is that the
derived impl must produce `CustomFs::read` (not `PosixFs::read`) as the effect identity. The
method body cannot simply call `PosixFs::read(path)` — that would raise the wrong identity.
Derivation would need to substitute the newtype for the representation type throughout the
method bodies, which is mechanical but non-trivial. Flagged for future work.

For now, any newtype implementing an interface writes its impl method bodies explicitly.

## 9. What This Eliminates

- ~~Separate phantom-type feature~~ — phantom parameters fall out of newtype with type
  parameters not appearing in the representation.
- ~~Record wrapper for zero-cost identity~~ — `type CustomFs = { inner: R }` is no longer
  needed for distinct identity with data; `newtype CustomFs = CustomFs(R)` is zero-cost.
- ~~Transparent alias for domain primitives~~ — `type Age = Int` collapses identity; newtype
  preserves it.

## 10. Open Questions

1. **Recursive newtypes.** Haskell allows `newtype Stream = Stream(Unit -> Stream)` for
   recursive types that are uninhabited at the value level but useful at the type level.
   Should Ash support this? It requires a fixity/strictness resolution mechanism (the
   representation type is not inhabited until the newtype is defined). Deferred for now.

2. **GADT-like newtypes.** Can a newtype's representation type depend on its type
   parameters in a way that enables GADT-like patterns? E.g., `newtype Vec<N, A> = Vec(...)`
   where the representation differs per `N`. This is likely out of scope — GADTs are a
   separate feature. Noted for awareness.

3. **Coerce role system.** Haskell has a `Coercible` type class that tracks which newtypes
   can be safely coerced to their representation (and through which type parameters). Ash's
   explicit-constructor model avoids this for now, but a role system may be needed if
   derived impls or unsafe coercion are added later. Deferred.

## 11. Working Principle

```text
A newtype wraps exactly one inhabited type, giving it a distinct nominal identity.
The wrapper is erased at runtime — zero-cost, same representation as the underlying type.
A newtype is never definitionally equal to its representation (nominal, not alias).
Wrapping and unwrapping are explicit and zero-cost (constructor and pattern match).
No automatic coercion between a newtype and its representation.
Phantom parameters are type parameters not appearing in the representation type.
Multiple parameters are supported — phantom and representation roles can mix.
A newtype produces a distinct effect identity (like any nominal type).
The representation type must be inhabited — bodyless types cannot be wrapped.
Deriving impls through the wrapper is deferred (GeneralizedNewtypeDeriving analog).
```

## 12. References

Internal references:

- [NOTE-025: Effect Identity via Sorts and Impls](NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
  — §7.1 bodyless types, §7.7 coherence, effect identity model
- [NOTE-022: Effects as Interfaces — Declaration Side](NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-023: Handler Surface — Dispatch Side](NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- [SPEC-058: Canonical Type Expression IR](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
  — transparent alias canonicalization
- [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md) — type_definition, §6.6
  bodyless types
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md) — definitional
  equality
- [SPEC-098b: Target CPS IR](../spec/SPEC-098b-TARGET-IR.md)

External references:

- Marlow & Peyton Jones, "How to make ad-hoc polymorphism less ad hoc" (1988) — Haskell
  newtype derivation. https://doi.org/10.1145/73577.73578
- Wehr & Thiemann, "ScalaGI: The meaning of newtypes in Scala" (2013).
  https://doi.org/10.1007/978-3-642-41139-1_2
- "GHC User's Guide: Newtypes." https://downloads.haskell.org/ghc/latest/docs/users_guide/data_type_declarations.html#newtype

## 13. Changelog

- 2026-06-28: Initial version. Defines the `newtype` mechanism: zero-cost nominal wrapper,
  distinct identity, representation erasure. Unifies newtype wrappers and phantom types into
  one mechanism. Grammar, type-system semantics (definitional equality, coercion model),
  effect-identity interaction, worked examples (domain primitives, phantom tagging,
  state-machine encoding, effect carriers). Deriving impls deferred. Open questions:
  recursive newtypes, GADT-like newtypes, coerce role system.
