---
status: drafting
version: v1
created: 2026-03-31
last-revised: 2026-04-01
related-plan-tasks: []
tags: [type-system, typeclasses, traits, ad-hoc-polymorphism, generics]
---

# TYPES-002: Ad-Hoc Polymorphism (Typeclasses/Traits)

## Problem Statement

Ash currently has parametric polymorphism (generics) but lacks a mechanism for ad-hoc polymorphism—the ability to define interfaces that different types can implement, enabling overloading based on type constraints.

This exploration asks: Should Ash add typeclasses (Haskell-style) or traits (Rust-style)? What are the design tradeoffs, and how do they integrate with Ash's existing type system, effect tracking, and capability model?

## Scope

- **In scope:**
  - Interface definition mechanisms (typeclass/trait syntax)
  - Instance/implementation declarations
  - Constraint propagation and inference
  - Integration with effect system and capabilities
  - Syntax for constrained functions/workflows

- **Out of scope:**
  - Higher-kinded types (for now)
  - Associated types (may be added later)
  - Type-level programming features
  - Implicit parameters (distinct from typeclasses)

- **Related but separate:**
  - TYPES-001: Tuple variant syntax (orthogonal feature)
  - SPEC-003: Type system foundations
  - MCE-003: Functions vs capabilities (may influence method-call syntax)

## Current Understanding

### What we know

- Ash has parametric polymorphism via `type Foo<T> = ...`
- Type constructors support generics (SPEC-020 Section 6.5)
- No current mechanism for constrained polymorphism (`show : Show a => a -> String`)
- The capability system is nominal and named (not structural)
- Effect tracking is static and part of the type system

### What we're uncertain about

- Should Ash use Haskell-style typeclasses or Rust-style traits?
- How do constraints interact with workflow effect signatures?
- Can capabilities implement typeclass interfaces?
- Should there be a coherence/orphan rule? Which one?
- How does this interact with the "no first-class functions" design space?

## Design Dimensions

| Dimension | Haskell Typeclasses | Rust Traits | Ash Hybrid |
|-----------|---------------------|-------------|------------|
| **Open/Closed** | Open (instances anywhere) | Closed (impl in crate) | TBD |
| **Coherence** | Requires orphan rules | Strict coherence | TBD |
| **Method syntax** | `show x` (type-directed) | `x.show()` (UFCS available) | TBD |
| **Default impls** | Yes, via class methods | Yes, trait default methods | Likely yes |
| **Effect tracking** | N/A (pure functions) | N/A (effect via types) | Must integrate |
| **Integration with caps** | No analog | Trait objects ~ trait bounds | TBD |

## Proposed Approaches

### Approach 1: Haskell-Style Typeclasses

```ash
-- Typeclass definition
class Show a where
  show : a -> String

-- Instance declaration
instance Show Int where
  show n = int_to_string(n)

-- Constrained function
print : Show a => a -> Workflow
print x = act io.Stdout.write(show x)

-- Multiple constraints
log : (Show a, Loggable a) => a -> Logger -> Workflow
```

**Pros:**
- Mature, well-understood design
- Clean separation of interface and implementation
- Type-directed dispatch is elegant

**Cons:**
- Open world assumption complicates coherence
- Global instance search can be slow
- Method syntax doesn't match Ash's action-call model

### Approach 2: Rust-Style Traits

```ash
-- Trait definition
trait Show {
  show : Self -> String
}

-- Implementation for type
impl Show for Int {
  show(n) = int_to_string(n)
}

-- Trait bounds on functions
fn print<T: Show>(x: T) -> Workflow { ... }

-- Multiple bounds
fn log<T: Show + Loggable>(x: T, logger: Logger) -> Workflow { ... }
```

**Pros:**
- Explicit implementation location (coherence by construction)
- Can use method syntax or function syntax
- Trait objects for dynamic dispatch

**Cons:**
- More verbose than typeclasses
- `Self` parameter may confuse with OOP
- Unclear how `Self` works without methods

### Approach 3: Capability-Inspired Interfaces

Extend the capability system to support interface constraints:

```ash
-- Interface definition (like capability, but for types)
interface Show<T> {
  show : T -> String
}

-- Provide implementation
provide Show<Int> {
  show(n) = int_to_string(n)
}

-- Use in workflow
workflow display<T> requires Show<T> (value: T) {
  let s = Show::show(value);
  act io.Stdout.write(s)
}

-- Or with explicit passing (dictionary passing style)
workflow display<T> (value: T, show_impl: Show<T>) {
  let s = show_impl.show(value);
  act io.Stdout.write(s)
}
```

**Pros:**
- Consistent with Ash's capability design
- Explicit evidence passing aligns with effect tracking
- No implicit instance search

**Cons:**
- Verbose if evidence must be threaded everywhere
- May diverge from familiar typeclass/trait syntax
- Unclear if this is just explicit dictionary passing

### Approach 4: Minimal Typeclasses (No Methods, Just Constraints)

Typeclasses as pure constraints without method bundles:

```ash
-- Typeclass as constraint only
class Serialize a

-- Functions declare they need the constraint
serialize : Serialize a => a -> Bytes

-- Instances provide evidence
instance Serialize Int
instance Serialize String

-- No methods; serialization logic is internal/compiler-generated
```

**Pros:**
- Simplest addition to type system
- Could be used for capability-like constraints
- Doesn't require method dispatch

**Cons:**
- Limited expressiveness
- Unclear how serialization actually happens
- May not solve real use cases

## Examples

### Example 1: Generic Container Operations

```ash
-- Eq typeclass for equality comparisons
class Eq a where
  eq : a -> a -> Bool

instance Eq Int where
  eq(x, y) = int_eq(x, y)

-- Generic list membership
member : Eq a => a -> List a -> Bool
member(x, xs) = any(eq(x), xs)

-- Usage
workflow check_member observes input:numbers {
  observe input:numbers as nums;
  if member(42, nums) then
    act notify::send("found it!")
}
```

### Example 2: Serialization/Deserialization

```ash
class Serialize a where
  to_bytes : a -> Bytes
  from_bytes : Bytes -> Option a

instance Serialize UserRecord where
  to_bytes(user) = json_encode(user)
  from_bytes(bs) = json_decode(bs)

workflow save_user sets db:users {
  let user : UserRecord = ...;
  let bytes = Serialize::to_bytes(user);
  set db:users = bytes
}
```

### Example 3: Effect-Polymorphic Workflows

```ash
-- Monad-like abstraction for effect sequencing
class Sequential m where
  bind : m a -> (a -> m b) -> m b
  return : a -> m a

-- Instance for pure computation
instance Sequential Identity where ...

-- Instance for workflows (effect-tracking)
instance Sequential Workflow where ...

-- Generic sequence function
sequence : Sequential m => List (m a) -> m (List a)
sequence(actions) = ...
```

**Question:** How does effect tracking work here? Can `Sequential` be instantiated at different effect levels?

## Open Questions

1. **Syntax preference:** Typeclass (`Show a =>`) or trait bound (`T: Show`) syntax?

2. **Coherence rules:**
   - Allow overlapping instances? (Haskell: yes with extensions)
   - Require orphan rule? (Haskell: yes; Rust: no, impl local)
   - How to handle incoherence?

3. **Effect integration:**
   - Can typeclass methods have effects?
   - How do constraints compose with effect signatures?
   - Example: `show : Show a => a -> String` vs `show : Show a => a -> Workflow String`

4. **Dictionary passing vs. type erasure:**
   - Implement via explicit dictionary passing (like GHC)?
   - Or monomorphization (like Rust)?
   - Or a hybrid?

5. **Default implementations:**
   - Allow default method implementations?
   - How to refer to other methods in defaults?

6. **Superclasses:**
   - Allow `class (Eq a) => Ord a` (Ord requires Eq)?
   - Multi-parameter typeclasses?

7. **Capability/typeclass relationship:**
   - Are capabilities just typeclasses with effect tracking?
   - Or are they completely separate mechanisms?
   - Can a capability implement a typeclass?

8. **Integration with ADTs:**
   - Deriving instances automatically?
   - Generic programming facilities?

## Foundational Literature Review

This section reviews foundational papers on type classes and related type system mechanisms, summarizing their contributions and relevance to Ash's design.

### Paper 1: How to Make Ad-Hoc Polymorphism Less Ad Hoc

**Reference:** Wadler, P., & Blott, S. (1989). How to Make Ad-Hoc Polymorphism Less Ad Hoc. *ACM SIGPLAN Notices*, Proceedings of POPL 1989.  
**URL:** https://doi.org/10.1145/75277.75283

**What it models:** The original type class proposal for Haskell. Introduces the concept of type classes as a mechanism for ad-hoc polymorphism (overloading) that extends Hindley-Milner type inference.

**Key Contributions:**
- Distinguishes *parametric polymorphism* (generic functions that work uniformly across types, like `map`) from *ad-hoc polymorphism* (functions that behave differently for different types, like `==`)
- Proposes type classes as a disciplined mechanism for ad-hoc polymorphism
- Shows how type classes subsume and unify multiple previous approaches (Standard ML's equality types, Miranda's string conversion, numeric overloading)
- Introduces the core mechanism: class declarations, instance declarations, and constraint propagation

**Relevance to Ash:**
- **Core concept foundation:** This is the seminal paper that introduced type classes. Any Ash design must understand these foundations.
- **Ad-hoc vs parametric distinction:** Critical for Ash because workflows already have parametric polymorphism (generics). The question is whether to add ad-hoc polymorphism.
- **Constraint syntax:** The paper establishes the `Eq a =>` syntax for constraints, which Approach 1 (Haskell-style) in our exploration uses.
- **Coherence concerns:** The paper notes that "there appears to be no alternative" to translation-based semantics, foreshadowing the complexity of giving direct semantics to type classes.

---

### Paper 2: Type Classes in Haskell

**Reference:** Hall, C. V., Hammond, K., Peyton Jones, S. L., & Wadler, P. L. (1996). Type Classes in Haskell. *ACM Transactions on Programming Languages and Systems (TOPLAS)*, 18(2), 109-138.  
**URL:** https://doi.org/10.1145/227699.227700

**What it models:** The complete formal semantics of Haskell type classes as of Haskell 1.3. Defines a source language with implicit typing and overloading, and a target language (System F, second-order lambda calculus) with explicit typing.

**Key Contributions:**
- Provides a complete set of type inference rules for type classes
- Uses **second-order lambda calculus** as a target language to record type information
- Demonstrates that programs with type classes can be transformed into programs typable by standard Hindley-Milner inference
- Shows how the static analysis phase of a compiler can be derived directly from the formal rules
- Documents practical issues: polymorphic class methods, monomorphism restriction, default types

**Relevance to Ash:**
- **Implementation blueprint:** The paper demonstrates that formal rules can directly guide implementation—aligns with Ash's spec-driven approach.
- **Translation strategy:** Shows type classes are "elaborated" into explicit dictionary passing. This relates to Ash's Approach 3 (Capability-Inspired Interfaces).
- **Multiple operations per class:** Unlike earlier theoretical work, this paper handles classes with multiple methods (e.g., `class Ord a where (<), (<=), (>), (>=) :: a -> a -> Bool`).
- **Outstanding issues noted:** The paper explicitly excludes polymorphic class methods and default types from its formal treatment—these are extensions that add complexity.

---

### Paper 3: Implementing Type Classes

**Reference:** Peterson, J., & Jones, M. (1993). Implementing Type Classes. *Proceedings of the ACM SIGPLAN Conference on Programming Language Design and Implementation (PLDI)*.  
**URL:** https://doi.org/10.1145/155090.155112

**What it models:** The practical implementation of type classes in the Yale Haskell compiler, focusing on the translation to dictionary passing style.

**Key Contributions:**
- Describes **dictionary passing style** (DPS): type class instances are compiled into records (dictionaries) containing method implementations
- Shows how overloaded operations become explicit dictionary parameters
- Addresses compile-time vs run-time resolution of overloading
- Demonstrates how to implement equality types and numeric overloading uniformly

**Relevance to Ash:**
- **Implementation strategy:** Dictionary passing is the standard implementation technique. Understanding it helps evaluate Ash's design options.
- **Explicit vs implicit:** The paper's translation makes implicit constraints explicit—relevant to Ash's preference for explicitness.
- **Runtime cost:** Dictionary passing has runtime cost (record lookup). Rust uses monomorphization instead. This is a key tradeoff for Ash (see Open Question 4).
- **Relationship to capabilities:** Dictionaries are essentially records of capabilities. This suggests a deep connection between type classes and Ash's capability system.

---

### Paper 4: On Understanding Types, Data Abstraction, and Polymorphism

**Reference:** Cardelli, L., & Wegner, P. (1985). On Understanding Types, Data Abstraction, and Polymorphism. *ACM Computing Surveys*, 17(4), 471-522.  
**URL:** https://doi.org/10.1145/6041.6042

**What it models:** A comprehensive survey and taxonomy of polymorphism in programming languages, predating but anticipating type classes.

**Key Contributions:**
- Establishes the **universal vs existential** quantification distinction
- Defines the four forms of polymorphism:
  1. **Parametric polymorphism:** Functions work uniformly over types (universal quantification)
  2. **Inclusion polymorphism:** Subtyping (objects belong to multiple types via inclusion)
  3. **Overloading:** Same name, different implementations (ad-hoc polymorphism)
  4. **Coercion:** Implicit type conversions
- Uses λ-calculus as a unifying framework
- Explores the relationship between types, data abstraction, and object-oriented concepts

**Relevance to Ash:**
- **Taxonomy foundation:** Provides the vocabulary for understanding what kind of polymorphism Ash has and what it might add.
- **Parametric vs ad-hoc:** Ash currently has parametric polymorphism (generics). Type classes add ad-hoc polymorphism. Understanding the distinction is essential.
- **Universal quantification:** The paper's treatment of ∀ (universal) types underlies how type class constraints work—they're essentially restricted universal quantification.
- **Pre-dates type classes:** Published before type classes were invented, but the framework anticipates them. Reading it helps understand type classes as "systematic overloading."

---

### Paper 5: COCHIS: Stable and Coherent Implicits

**Reference:** Schrijvers, T., Oliveira, B. C. D. S., Wadler, P., & Marntirosian, K. (2019). COCHIS: Stable and Coherent Implicits. *Journal of Functional Programming*, 29, e3.  
**URL:** https://doi.org/10.1017/S0956796818000242

**What it models:** A calculus of implicit programming that reconciles flexibility (local scoping, overlapping instances) with strong reasoning properties (coherence, stability).

**Key Contributions:**
- Identifies the tension between **flexibility** and **ease of reasoning** in implicit programming (type classes, Scala implicits, Rust traits, etc.)
- Defines **coherence:** The property that every valid typing derivation for a program yields the same dynamic semantics (no ambiguous instance resolution)
- Defines **stability:** Type substitutions should not change which instances are selected
- Proposes COCHIS calculus with polymorphism, local scoping, overlapping instances, first-class instances, and higher-order rules—while remaining coherent
- Uses a focusing-based technique for proof search to achieve determinism

**Relevance to Ash:**
- **Coherence is critical:** The paper proves that flexibility and coherence can coexist. This is highly relevant to Ash's design tension between explicitness and ergonomics.
- **Local scoping:** COCHIS supports local instance definitions—aligns with Rust's approach where instances are local to modules.
- **First-class instances:** COCHIS allows passing instances as values—relevant to Ash's Approach 3 (capability-inspired interfaces).
- **Modern perspective:** Published in 2019, this paper synthesizes decades of type class research and points toward the future. If Ash adds type classes, understanding COCHIS helps avoid known pitfalls.

---

### Paper 6: RepLib: A Library for Derivable Type Classes

**Reference:** Weirich, S. (2008). RepLib: A Library for Derivable Type Classes. *Proceedings of the ACM SIGPLAN Haskell Symposium*.  
**URL:** https://doi.org/10.1145/1411286.1411293

**What it models:** A GHC library that enables automatic derivation of type class instances for arbitrary type classes (not just built-in ones like `Eq`, `Show`).

**Key Contributions:**
- Uses **representation types** to encode the structure of datatypes
- Allows users to define the relationship between type structure and instance declarations via normal Haskell functions
- Instances defined this way are **extensible**—can add special cases for specific types
- Supports generic programming (programming over the structure of types)

**Relevance to Ash:**
- **Deriving mechanism:** Haskell's `deriving (Eq)` is extremely useful. Ash would benefit from automatic derivation.
- **Generic programming:** RepLib enables generic functions (folds, maps, equality checks) that work over any datatype. This is a key use case for type classes.
- **Representation types:** The technique of using types to represent type structure is a powerful pattern that might inform Ash's metaprogramming capabilities.
- **User-extensible:** RepLib shows that instance derivation can be user-defined, not just compiler-built-in.

---

### Paper 7: Improving Typeclass Relations by Being Open

**Reference:** Mohr, C., & Dreyer, D. (2018). Improving Typeclass Relations by Being Open. *Proceedings of the ACM SIGPLAN Haskell Symposium*.  
**URL:** https://doi.org/10.1145/3242744.3242751

**What it models:** The problem of expressing relations between typeclasses (e.g., "every monad is a functor") and the limitations of Haskell's superclass mechanism.

**Key Contributions:**
- Documents problems with **superclasses** (e.g., `class Functor m => Monad m`): closed nature, cascading changes, inability to add superclasses after the fact
- Proposes "open" relations between type classes that don't require modifying class definitions
- Addresses the practical issue of retrofitting mathematical abstractions into existing codebases

**Relevance to Ash:**
- **Superclass design:** If Ash adds type classes, the superclass mechanism needs careful design. This paper shows the pitfalls of Haskell's approach.
- **Open vs closed:** Ash's philosophy favors explicit, modular designs. The paper's "open" approach aligns with this.
- **Mathematical abstractions:** Monads, functors, and similar abstractions are important for effect sequencing in Ash. The paper's insights help design how these relate.

---

### Paper 8: Oxide: The Essence of Rust

**Reference:** Weiss, A., Gierczak, O., Patterson, D., & Ahmed, A. (2021). Oxide: The Essence of Rust. *arXiv preprint* arXiv:1903.00982v4.  
**URL:** https://arxiv.org/abs/1903.00982

**What it models:** A formal type system account of Rust's borrow checker and ownership model. Oxide is a formalized programming language close to source-level Rust with fully-annotated types.

**Key Contributions:**
- Presents a complete type system that captures Rust's notion of ownership and borrowing
- Introduces **regions** (sets of locations) as a new view of lifetimes that approximates the origins of references
- The type system automatically computes region information through control-flow-based substructural typing
- Demonstrates that Rust's ownership discipline can be understood as substructural typing with region-based memory management
- Provides a foundation for formal reasoning about Rust programs

**Relevance to Ash:**
- **Formal foundations:** If Ash wants to understand how Rust achieves memory safety through types, Oxide provides the blueprint.
- **Substructural typing:** Oxide shows that ownership is fundamentally a substructural type system issue (linear/affine types). Ash's capability system already has linear/affine aspects.
- **Region-based memory:** The "region as set of locations" approach might inform how Ash tracks provenance and lifetime of workflow data.
- **Trait system context:** While Oxide focuses on ownership, it situates traits within this ownership framework—traits must respect the ownership discipline.

---

### Paper 9: An Interactive Debugger for Rust Trait Errors

**Reference:** Gray, G., Crichton, W., & Krishnamurthi, S. (2025). An Interactive Debugger for Rust Trait Errors. *Proceedings of the ACM on Programming Languages (PACMPL)*, 9(PLDI), Article 199.  
**URL:** https://doi.org/10.1145/3729302

**What it models:** The complexity of debugging type inference failures in Rust's trait system, and a system (Argus) for interactively visualizing trait inference search trees.

**Key Contributions:**
- Documents that "compiler diagnostics for type inference failures are notoriously bad, and type classes only make the problem worse"
- Shows how trait resolution introduces a complex search process that can lead to "wholly inscrutable or useless errors"
- Presents Argus, an interactive debugger that provides multiple views on the trait inference search tree
- Demonstrates that programmers using Argus localized faults 2.2× more often and 3.3× faster than without it
- Identifies key design insights: different debugging goals require different views; sensible defaults (hiding full paths, sorting obligations by complexity) improve productivity

**Relevance to Ash:**
- **Error message design:** The paper reveals how complex trait resolution is. If Ash adds type classes/traits, error messages need careful design.
- **User experience:** The complexity of type class inference can harm usability. Ash might prioritize simplicity over expressiveness.
- **Debugging infrastructure:** Argus shows that sophisticated tooling can mitigate complexity. Ash should plan for similar debugging tools if adopting traits.
- **Search complexity:** The paper highlights that trait resolution is a search problem with exponential worst-case behavior. Ash's resolution algorithm should be designed with this in mind.

---

### Paper 10: Tabled Typeclass Resolution

**Reference:** Selsam, D., Ullrich, S., & de Moura, L. (2020). Tabled Typeclass Resolution. *arXiv preprint* arXiv:2001.04301.  
**URL:** https://arxiv.org/abs/2001.04301

**What it models:** The performance problems of typeclass resolution (exponential time with diamonds, divergence with cycles) and a solution using tabling (memoization).

**Key Contributions:**
- Identifies two major limitations of traditional typeclass resolution:
  1. **Exponential running times** in the presence of "diamonds" (multiple paths to the same instance)
  2. **Divergence** in the presence of cycles
- Proposes **tabled typeclass resolution** that uses memoization (tabling) to solve both problems
- Implemented for Lean 4 and shown to be exponentially faster than existing systems in the presence of diamonds
- The procedure is lightweight and could be implemented in other systems

**Relevance to Ash:**
- **Algorithm design:** If Ash implements typeclass resolution, tabling is a technique to consider for performance.
- **Complexity awareness:** The paper documents that typeclass resolution has inherent complexity issues (exponential time, non-termination). These need to be addressed.
- **Diamond problem:** The "diamond" issue (multiple inheritance paths) is common in type hierarchies. Ash's design should consider how to handle this.
- **Practical implementation:** The paper shows that even "academically simple" algorithms need optimization for real-world use.

---

### Paper 11: Traits for Correct-by-Construction Programming

**Reference:** Runge, T., Potanin, A., Thüm, T., & Schaefer, I. (2022). Traits for Correct-by-Construction Programming. *arXiv preprint* arXiv:2204.05644.  
**URL:** https://arxiv.org/abs/2204.05644

**What it models:** Using traits to support correctness-by-construction (CbC) in programming languages, specifically as an alternative to traditional refinement-based CbC.

**Key Contributions:**
- Proposes **TraitCbC**, an incremental program construction procedure using traits
- Shows that traits provide a natural way to support CbC without specialized refinement rules
- Enables program construction by trait composition instead of refinement
- Demonstrates that traits can encode both the implementation and the specification (proofs) together
- Implemented as a Scala DSL

**Relevance to Ash:**
- **Specification integration:** The paper shows that traits can carry not just implementations but also proofs/specifications. This could be relevant to Ash's effect and capability tracking.
- **Compositional reasoning:** Trait composition enables building correct programs from correct components—aligns with Ash's workflow composition model.
- **Alternative to refinement:** Ash might use traits as a way to structure correctness arguments without separate refinement calculi.
- **Capability/typeclass unification:** The paper's approach of using traits for specification suggests that Ash's capabilities (which track effects) could also carry correctness information.

---

### Paper 12: An Executable Operational Semantics for Rust

**Reference:** (from arXiv:1804.07608). *An Executable Operational Semantics for Rust with the Formalization of Ownership and Borrowing.*

**What it models:** RustSEM, an executable operational semantics for Rust that covers a large subset of the language including ownership and borrowing.

**Key Contributions:**
- Provides an operational semantics for ownership and borrowing at the memory level
- Implemented in the K-Framework, making it executable and testable
- Evaluated against the Rust compiler using ~700 tests
- More powerful than the standard borrow checker in detecting memory errors

**Relevance to Ash:**
- **Operational semantics:** If Ash needs to formalize its runtime behavior, this paper shows how to do it for ownership-heavy languages.
- **Testing semantics:** The use of executable semantics (K-Framework) allows testing the specification against reality—a technique Ash could adopt.
- **Memory model:** Understanding how Rust models memory helps design Ash's value and reference semantics.

---

## Rust Traits Literature: Key Insights

From the Rust-specific papers, several insights emerge:

1. **Traits are complex to resolve:** The trait resolution algorithm involves complex search that can produce confusing error messages (Gray et al.). Tooling is essential.

2. **Performance matters:** Typeclass/trait resolution can have exponential worst-case behavior. Tabling (Selsam et al.) is one solution; careful language design to avoid diamonds is another.

3. **Ownership and traits interact:** Oxide shows that traits must respect the ownership discipline. Any Ash design must consider how traits/capabilities interact with effects and ownership.

4. **Traits can carry specifications:** TraitCbC demonstrates that traits can encode not just implementations but correctness arguments—relevant to Ash's goals of verified workflows.

5. **Coherence by construction:** Rust's trait coherence (avoiding overlapping impls) is simpler than Haskell's open world with orphan rules. This aligns with Ash's preference for explicit, predictable behavior.

---

## Synthesis: Key Insights for Ash

From this literature review, several insights emerge for Ash's type class design:

1. **Ad-hoc polymorphism is distinct from parametric polymorphism:** Ash has the latter; adding the former is a major design decision. The distinction matters for effect tracking.

2. **Dictionary passing is the standard implementation:** But monomorphization (Rust) avoids runtime overhead. Given Ash's performance consciousness, monomorphization may be preferable.

3. **Coherence is non-negotiable:** The COCHIS paper shows that incoherence leads to unpredictable semantics. Ash must ensure that instance resolution is deterministic and unambiguous.

4. **Superclasses are problematic:** If Ash adds type classes, consider alternative mechanisms for expressing class relationships (e.g., explicit subtyping, open relations).

5. **Deriving is essential:** Automatic instance derivation dramatically reduces boilerplate. Any Ash type class system should include this from the start.

6. **Connection to capabilities:** Dictionaries are records of capabilities. Ash might unify capabilities and type classes: a capability is a type class with effect tracking, and a type class instance is a capability provider.

## Prior Art Analysis

| Language | Mechanism | Key Features | Relevance to Ash |
|----------|-----------|--------------|------------------|
| Haskell | Typeclasses | Open world, coherence via orphan rules | Well-studied, but global instance search conflicts with Ash's explicitness |
| Rust | Traits | Closed world, coherence by construction, trait objects | Explicit impl location aligns with Ash philosophy |
| Scala | Implicits | Implicit evidence passing | Too implicit for Ash's design goals |
| Swift | Protocols | Value types, extension-based | OOP heritage doesn't match Ash |
| OCaml | Modules/Functors | Module-level abstraction | Too heavy, different abstraction level |
| Koka | Effect handlers | Effect polymorphism via row types | Effect model inspiration, but different approach |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-31 | Exploration created | Need for ad-hoc polymorphism identified during work on generic data structures |
| 2026-03-31 | Haskell literature review added | Reviewed 7 foundational papers on typeclasses to inform Ash design decisions |
| 2026-03-31 | Rust literature review added | Reviewed 5 papers on Rust traits (Oxide, trait errors, tabled resolution, TraitCbC, operational semantics) |

## Reasoning Trace: From Typeclasses to Associated Effects

This section documents the exploration process that led from initial questions about typeclasses to the current design direction. The path matters as much as the destination.

### Initial Conflation: Capabilities as Typeclasses

The exploration began with noticing similarities between Haskell typeclass dictionaries and Ash capability providers. Both involve:
- A definition (typeclass/capability)
- An implementation (instance/provider)
- A lookup mechanism (instance search/capability registry)

**Initial (incorrect) proposal:** Treat capabilities as typeclasses, where a capability provider is a typeclass instance.

### First Correction: Three Levels of Abstraction

User clarification introduced crucial distinctions:

| Level | Concept | Example |
|-------|---------|---------|
| **Abstraction** | Generic over type parameters | `class Show a` abstracts over `a` |
| **Definition** | Interface specification | Methods in the class/trait |
| **Implementation** | Concrete instance for fixed types | `instance Show Int` |

**Key insight:** Capabilities are **ambient authority**, not typeclass instances. Typeclasses abstract over **data types**; capabilities provide **runtime authority**. These are orthogonal dimensions.

### Second Correction: Effects Annotate Computations, Not Values

Early examples incorrectly wrote `Epistemic<T>` (effect wrapping value). Correction:

- **Wrong:** `observe : Query -> Epistemic<Result>`
- **Correct:** `observe : Query ->{Epistemic} Result`

Effects annotate the **arrow** (computation type), not the **carrier** (value type). This aligns with the literature on effect systems (Koka, Eff).

### Third Question: Can Effects Themselves Be Typeclasses?

Explored whether `Effect<E>` could be a typeclass:

```rust
// Awkward sketch
trait Effect {
    extern call(self);
}
```

**Conclusion:** Effects are better modeled as **kinds** or **type-level indices** rather than typeclasses. Typeclasses abstract over data types; effects classify computation shapes.

### Fourth Exploration: Effect Quantification

User introduced existential vs universal quantification:

```haskell
-- Universal: polymorphic in effect
forall e: Effect. a -{e}-> b

-- Existential: hides specific effect
exists e: Effect. a -{e}-> b
```

**Clarification:** The existential `exists e` on the RHS of a type definition means "there exists a specific effect"; the universal `forall e` means "works for any effect." These are dual.

### Fifth Step: The Curried Kind

Moved toward a curried kind for the arrow:

```haskell
Arrow : Effect -> (Type -> Type -> Type)

-- So:
Arrow Epistemic : Type -> Type -> Type
Arrow Epistemic Int String : Type
```

Effect is the **first** argument, determining the "flavor" of arrow.

### Sixth Step: Richer Arrow Annotations

Explored whether the arrow carries more than effects:

```haskell
-- Option: capabilities in the arrow
Arrow (e: Effect) (caps: Capabilities) (a: Type) (b: Type)

-- Option: provenance tracking
Arrow (e: Effect) (prov: Provenance) (a: Type) (b: Type)

-- Option: unified annotation structure
Arrow (ann: { effect: Effect, requires: Capabilities, ... }) (a: Type) (b: Type)
```

**Question raised:** How do other dimensions (capabilities, provenance, security labels) fit into this framework?

### Seventh Step: Typeclasses with Associated Effects

Current direction: Typeclasses can declare **associated effects** that instances specify:

```rust
capability Storage<S> {
    effect ReadEffect;      -- Associated effect (not type)
    effect WriteEffect;
    
    read : S ->{ReadEffect} Bytes,
    write : S -> Bytes ->{WriteEffect} (),
}

impl Storage<FileSystem> {
    effect ReadEffect = Epistemic;    -- Concrete effect
    effect WriteEffect = Operational; -- Concrete effect
    
    read(fs) = observe fs:read(...),
    write(fs, bytes) = act fs:write(bytes),
}
```

**Benefits:**
- Effect polymorphism: Generic code works with any `Storage`, effect determined by instance
- Testability: Can swap real filesystem (Operational) for mock (Pure)
- Type inference solves for both type parameters and effect parameters

### Current Understanding: Constraint-Based Inference

The type system solves constraints across multiple dimensions simultaneously:

| Dimension | Constraint | Solves For |
|-----------|------------|------------|
| Data types | `T : TypeClass` | Implementation |
| Effects | `effect = E` | Effect annotation |
| Capabilities | `requires C` | Needed authority |
| Provenance | `provenance P` | Origin tracking |

Typeclasses provide **interface abstraction** (data dimension); other dimensions are **tracked automatically** by the compiler.

---

## Associated Effects: Design Proposal

Based on the reasoning trace, the current design direction for Ash:

### Syntax

```ash
-- Typeclass definition with associated effects
capability Serializer<T> {
    effect SerializeEffect;
    effect DeserializeEffect;
    
    serialize : T ->{SerializeEffect} Bytes,
    deserialize : Bytes ->{DeserializeEffect} Option<T>,
}

-- Instance provides concrete effects
impl Serializer<Json, UserRecord> {
    effect SerializeEffect = Pure;        -- Pure for JSON
    effect DeserializeEffect = Pure;
    
    serialize(user) = json_encode(user),
    deserialize(bytes) = json_decode(bytes),
}

impl Serializer<Database, UserRecord> {
    effect SerializeEffect = Operational; -- DB write
    effect DeserializeEffect = Epistemic; -- DB read
    
    serialize(user) = act db:insert(user),
    deserialize(bytes) = observe db:fetch(bytes),  -- bytes is ID
}
```

### Usage

```ash
-- Generic over serializer, effect determined by instance
workflow persist<T, S>
  requires Serializer<T, S>
  (item: T, storage: S)
{
  let bytes = Serializer::serialize(item);  -- Effect: SerializeEffect
  ...
}
```

### Type Inference

The compiler infers:
1. **Type parameters**: `T = UserRecord`, `S = Json` or `S = Database`
2. **Effect parameters**: `SerializeEffect = Pure` or `Operational`

Both are anchored at values (the `Serializer` instance being used).

---

## Open Question: Other Dimensions

How do capabilities, provenance, and other dimensions fit?

### Option 1: Separate Dimensions

```ash
-- Effects annotate arrow
query : Query ->{Epistemic} Result

-- Capabilities are separate (ambient authority)
workflow example observes db:Database { ... }

-- Provenance tracked separately
workflow example produces provenance:AuditTrail { ... }
```

### Option 2: Unified Arrow Annotation

```ash
-- Rich arrow type
query : Arrow { effect = Epistemic, requires = [Database], provenance = Logged } Query Result

-- Or with row polymorphism
query : Arrow (| effect: Epistemic, .. |) Query Result
```

### Option 3: Associated Types for Other Dimensions

```rust
capability Backend<B> {
    effect QueryEffect;
    type ProvenanceType;     -- Associated type
    type CapabilityRequirements;  -- Associated capabilities
    
    query : Query ->{QueryEffect} Result,
}
```

**Unresolved:** Which approach best serves Ash's goals of explicitness, verifiability, and ergonomics?

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-31 | Exploration created | Need for ad-hoc polymorphism identified during work on generic data structures |
| 2026-03-31 | Haskell literature review added | Reviewed 7 foundational papers on typeclasses to inform Ash design decisions |
| 2026-03-31 | Rust literature review added | Reviewed 5 papers on Rust traits (Oxide, trait errors, tabled resolution, TraitCbC, operational semantics) |
| 2026-03-31 | Reasoning trace added | Documented exploration from initial conflation to associated effects design |
| 2026-03-31 | Associated effects proposed | Typeclasses can declare associated effects that instances specify; enables effect polymorphism |

## Next Steps

- [x] Survey Rust trait system design decisions (coherence, orphan rules)
- [x] Analyze Haskell typeclass pitfalls (incoherence, slow instance search)
- [x] Work through reasoning trace (concepts → corrections → current proposal)
- [ ] Evaluate other dimensions (capabilities, provenance) in this framework
- [ ] Prototype syntax examples for associated effects
- [ ] Determine relationship to capability system (separate or unified?)
- [ ] Draft specification for chosen approach

---

## Additional Resources: Constraint-Based Type Processing

**Type Processing by Constraint Reasoning** (Pottier, 2001; APLAS 2003)

These papers present a constraint-based approach to type inference, where type checking is formulated as solving systems of constraints. This approach is highly relevant to Ash's multi-dimensional type system:

- **Constraints as the unifying mechanism**: Instead of separate algorithms for types, effects, and capabilities, all dimensions generate constraints that are solved together
- **Anchoring at values**: Constraints are generated from and anchored at value-level terms
- **Deferred resolution**: Constraints can be collected and solved lazily, enabling more flexible inference

This suggests a possible implementation strategy where:
- `T : TypeClass` generates a constraint for dictionary lookup
- `effect = E` generates an effect annotation constraint
- `requires C` generates a capability availability constraint

All constraints are collected during type checking and solved as a system, potentially using similar techniques to those in the paper.

**Note: Ash Already Uses Constraints**

Obligations and requirements are already treated as constraints in Ash's current design:

- **Capability constraints**: `observes sensor:temp`, `sets actuator:position`
- **Obligation constraints**: `check temperature_reading`
- **Policy constraints**: `decide access_policy`

These existing constraints interact with the typeclass and effect constraints:
- Constraints may modify effect availability (a capability constraint enables specific effects)
- Z3 will validate constraint satisfiability for the Ash capabilities system
- The syntax and expressive power for capability constraints is a key design consideration

This means the constraint-based approach isn't theoretical—it's already partially implemented. The unified constraint solver would handle all dimensions: types, effects, capabilities, obligations, and policies.

---

## Effect Systems Literature Review

This section surveys foundational work on effect systems, from monadic effects through algebraic effects to modern row-based effect typing. Understanding this evolution informs Ash's effect system design.

### Paper 1: Programming with Algebraic Effects and Handlers (Eff)

**Reference:** Bauer, A., & Pretnar, M. (2012). Programming with Algebraic Effects and Handlers. *arXiv preprint* arXiv:1203.1539.  
**URL:** https://arxiv.org/abs/1203.1539

**What it models:** The Eff programming language, which provides first-class algebraic effects and handlers. Effects are viewed as algebraic operations, and handlers as homomorphisms from free algebras.

**Key Contributions:**
- Introduces algebraic effects as operations of an algebraic theory
- Separates effect *declaration* (operations) from effect *handling* (interpretation)
- Supports first-class effects: effects can be created dynamically, passed as values
- Shows how handlers subsume many constructs: exception handlers, transactions, backtracking
- Demonstrates composability: effects compose via disjoint union of operations

**Core Concepts:**
```
Effect   = Set of Operations + Equational Theory
Handler  = Homomorphism from free algebra (interpretation of operations)
```

**Relevance to Ash:**
- **Separation of concerns:** Ash already separates capability *definition* from capability *provision*. This mirrors effect/handler separation.
- **Composability:** Ash's effect lattice (`Epistemic ⊔ Operational = Operational`) is a form of effect composition. Could we generalize to arbitrary effect union?
- **First-class effects:** Ash capabilities are currently ambient. Could they become first-class values like Eff's effects?

---

### Paper 2: Handling Algebraic Effects

**Reference:** Plotkin, G. D., & Pretnar, M. (2013). Handling Algebraic Effects. *Logical Methods in Computer Science*, 9(4:23), 1-36.  
**URL:** https://doi.org/10.2168/LMCS-9(4:23)2013

**What it models:** A comprehensive theoretical treatment of algebraic effects and their handlers. Generalizes exception handling to arbitrary algebraic effects.

**Key Contributions:**
- Formal semantics for effect handlers as homomorphisms induced by universal property of free models
- Shows that algebraic effects include: exceptions, state, nondeterminism, I/O, time
- Demonstrates that handler constructs include: relabeling (CCS), timeout, rollback, stream redirection
- Provides equational theory for reasoning about effectful programs
- Establishes connection between algebraic effects and monads (free model induces computational monad)

**Key Insight:**
> "Each computation either returns a value or performs an operation with an outcome that determines a continuation."

**Relevance to Ash:**
- **Equational reasoning:** Ash workflows could support equational reasoning about effects if we formalize the capability operations.
- **Handler generality:** Ash's `observe`/`act` distinction could be seen as a specific handler pattern. Could users define custom handlers?
- **Monadic foundation:** The paper establishes that algebraic effects induce monads. Ash's workflow type is essentially a monad.

---

### Paper 3: Frank - Bidirectional Effect Types

**Reference:** Lindley, S., McBride, C., & McLaughlin, C. (2017). Do Be Do Be Do. *Proceedings of the 44th ACM SIGPLAN Symposium on Principles of Programming Languages (POPL)*.  
**URL:** https://arxiv.org/abs/1611.09259

**What it models:** The Frank programming language, which eliminates the distinction between effect handling and function application through "bidirectional effect types."

**Key Contributions:**
- Introduces **multihandlers**: handlers that interpret multiple commands simultaneously
- **Bidirectional typing**: effect types flow both ways ( caller → callee and callee → caller)
- Eliminates need for separate `handle` construct—handlers are just functions that interpret commands
- Functions are special case of handlers that interpret no commands
- **Abilities**: Frank's term for effect types (what commands are permitted)

**Syntax Comparison:**
```
-- Traditional (Eff, Koka)
handle computation with handler

-- Frank
operator computation  -- handler is just an operator
```

**Relevance to Ash:**
- **Bidirectional flow:** Ash's capability tracking is already bidirectional—requirements flow down, effects flow up.
- **Multihandlers:** Could Ash support handling multiple capabilities simultaneously? This is similar to the `requires` list in workflows.
- **Unified abstraction:** Frank unifies functions and handlers. Ash could unify pure functions and workflows.

---

### Paper 4: Monadic Effects (Background)

**Reference:** Moggi, E. (1991). Notions of Computation and Monads. *Information and Computation*, 93(1), 55-92.  
**URL:** https://doi.org/10.1016/0890-5401(91)90052-4

**What it models:** The foundational work on using monads to model computational effects in functional programming. Moggi's seminal paper that precedes algebraic effects.

**Key Contributions:**
- Proposes monads as uniform representation of computational effects
- Shows how exceptions, state, nondeterminism, I/O can be modeled as monads
- Introduces monad transformers for composing effects
- Establishes categorical semantics for effectful computation

**The Monad Pattern:**
```haskell
class Monad m where
  return :: a -> m a           -- Pure computation
  (>>=)  :: m a -> (a -> m b) -> m b  -- Sequential composition
```

**Relevance to Ash:**
- **Historical context:** Ash's workflow type is essentially a monad. Understanding monads helps understand the design space.
- **Composition:** Monad transformers compose effects sequentially. Algebraic effects compose via union. Ash currently uses a lattice—neither exactly.
- **Limitations:** Monads don't commute arbitrarily. This is why monad transformer order matters. Algebraic effects avoid this problem.

---

### Paper 5: Row-Based Effect Typing (Koka)

**Reference:** Leijen, D. (2014). Koka: Programming with Row Polymorphic Effect Types. *Proceedings of MSFP 2014*.  
**URL:** https://www.microsoft.com/en-us/research/publication/koka-programming-row-polymorphic-effect-types/

**What it models:** The Koka programming language, which uses row polymorphism for effect typing. Effects are tracked as rows (extensible records) of effect labels.

**Key Contributions:**
- **Row polymorphism**: Effects are rows (extensible records), enabling structural subtyping
- **Effect inference**: Most effect types are inferred, not annotated
- **Effect polymorphism**: Functions can be generic over their effects
- **No monad syntax**: Direct-style programming with effects tracked implicitly

**Row Types:**
```koka
// Effect row: <console,io|e> means "at least console and io, possibly more e"
function printTwice(x : a) : console <console,io|e> ()
```

**Relevance to Ash:**
- **Row polymorphism vs lattice:** Koka uses rows (structural). Ash uses a lattice (nominal). Rows are more flexible; lattices are simpler.
- **Effect inference:** Koka infers most effects. Ash currently requires explicit capability declarations. Could Ash infer more?
- **Polymorphism:** Koka's effect polymorphism (`<e>`) is similar to our proposed associated effects.

---

## Effect Systems: Key Insights for Ash

From this literature survey, several design insights emerge:

### 1. Three Approaches to Effects

| Approach | Mechanism | Flexibility | Complexity |
|----------|-----------|-------------|------------|
| **Monads** | Typeclass + bind/return | Low (ordered transformers) | High (composition) |
| **Algebraic Effects** | Operations + Handlers | High (union of effects) | Medium |
| **Row Polymorphism** | Structural rows | High (extensible) | Medium |

### 2. Effect Tracking Location

| Location | Example | Ash Current |
|----------|---------|-------------|
| **Return type** | `IO a` | No |
| **Arrow annotation** | `a -{E}> b` | Yes (implicit via capability) |
| **Row in arrow** | `a <E> b` | No |

### 3. Handler Patterns

| Pattern | Description | Ash Equivalent |
|---------|-------------|----------------|
| **Interpreter** | Handle all operations | Workflow body handles capabilities |
| **Relay** | Handle some, pass others | Capability delegation |
| **Forwarding** | Transparent handling | `with` workflow form |

### 4. The Effect Composition Question

How do effects compose?
- **Monads:** Sequential composition via bind (order matters)
- **Algebraic effects:** Disjoint union of operations (order doesn't matter)
- **Lattice (Ash):** Join operation (⊔) gives upper bound

Ash's lattice approach is simpler but less expressive than full algebraic effects. Is this the right tradeoff?

---

## Effect Systems vs Typeclasses: Synthesis

The two surveys (typeclasses and effects) reveal a convergence:

| Feature | Typeclasses | Effect Systems | Combined (Ash Direction) |
|---------|-------------|----------------|--------------------------|
| **Abstraction** | Over data types | Over effect shapes | Over both? |
| **Implementation** | Dictionary | Handler | Provider? |
| **Composition** | Intersection | Union | Lattice join? |
| **Resolution** | Compile-time | Runtime | Both? |

**The key insight:** Typeclasses abstract over *what* (data types); effects abstract over *how* (computation). Ash needs both:
- Typeclasses for data abstraction (`Serializable`, `Queryable`)
- Effects for capability safety (`Epistemic`, `Operational`)
- **Combined**: Typeclasses with associated effects (our proposed direction)

---

## Decision Log (Updated)

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-31 | Exploration created | Need for ad-hoc polymorphism identified during work on generic data structures |
| 2026-03-31 | Haskell literature review added | Reviewed 7 foundational papers on typeclasses to inform Ash design decisions |
| 2026-03-31 | Rust literature review added | Reviewed 5 papers on Rust traits (Oxide, trait errors, tabled resolution, TraitCbC, operational semantics) |
| 2026-03-31 | Reasoning trace added | Documented exploration from initial conflation to associated effects design |
| 2026-03-31 | Associated effects proposed | Typeclasses can declare associated effects that instances specify; enables effect polymorphism |
| 2026-04-01 | Effect systems literature review added | Reviewed foundational papers on algebraic effects (Eff, Plotkin/Pretnar, Frank) and row-based effects (Koka) |

---

## Review Note (2026-04-01)

This document is intentionally preserved as a reasoning trace. The syntax shifts, partial
conflations, and dead ends are useful because they show how the exploration moved rather than
only where it landed.

That said, the current review suggests a few strong guardrails for future work:

- Keep **data-level ad-hoc polymorphism**, **effects**, and **runtime authority** conceptually separate unless a later design proves that unification is worth the semantic cost.
- Avoid reusing `capability` syntax for interface abstraction unless Ash's capability semantics are explicitly widened; the current specs treat capability values as authorization witnesses, not method-dispatch receivers.
- Prefer **Ash-native workloads and examples** over imported FP examples that assume higher-kinded types, ordinary first-class functions, or a settled function/workflow boundary.
- Treat **associated effects** as a promising branch, not yet as the default conclusion of this exploration.
- Evaluate candidate designs against explicit **decision-driving workloads** rather than mainly against syntax familiarity.

For a cleaner continuation of the same exploration, see the sibling document
`TYPES-002-ad-hoc-polymorphism-v2.md`, which preserves the open design space while pruning
several obvious dead ends and reorganizing the material around Ash-native examples and design
pressures.
