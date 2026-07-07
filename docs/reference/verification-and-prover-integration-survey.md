> # Verification, Prover Integration, and LLM-Driven Proving: A Literature Survey

**Status:** Draft — literature review in progress  
**Purpose:** Survey the state of the art in compiler-prover integration, type-checking-as-proving,
and LLM-driven theorem proving / program verification. Map each approach to Ash's design
philosophy: symbolic-connectionist duality, effect rows, contracts/laws/proofs, evidence rows,
and the cooperative `(compiler, prover, LLM)` model.

**Scope:** Research and reference only. This document does not specify Ash implementation; it
informs future specs and plans.

**Companion documents:**

- [NOTE-036: Gradual Verification and Proof Provider Architecture](../notes/NOTE-036-GRADUAL-VERIFICATION-AND-PROOF-PROVIDERS.md)
- [NOTE-037: Ash as a Symbolic-Connectionist Duality](../notes/NOTE-037-SYMBOLIC-CONNECTIONIST-DUALITY.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)

---

## Table of contents

1. [Scope and methodology](#1-scope-and-methodology)
2. [Taxonomy](#2-taxonomy)
3. [Compiler-integrated provers](#3-compiler-integrated-provers)
   1. [SMT-backed program verifiers](#31-smt-backed-program-verifiers)
   2. [Proof-assistant-backed verifiers](#32-proof-assistant-backed-verifiers)
   3. [Refinement-type compilers](#33-refinement-type-compilers)
   4. [Dependently typed compilers](#34-dependently-typed-compilers)
   5. [Verified compilers](#35-verified-compilers)
   6. [Rust verification ecosystem](#36-rust-verification-ecosystem)
4. [Type checking and inference as proving](#4-type-checking-and-inference-as-proving)
5. [LLM-driven proving and verification](#5-llm-driven-proving-and-verification)
   1. [Tactic and proof-script suggestion](#51-tactic-and-proof-script-suggestion)
   2. [Conjecture formation](#52-conjecture-formation)
   3. [Bug and security finding](#53-bug-and-security-finding)
   4. [End-to-end theorem proving](#54-end-to-end-theorem-proving)
6. [Comparison matrix](#6-comparison-matrix)
7. [Alignment with Ash](#7-alignment-with-ash)
   1. [What Ash can adopt](#71-what-ash-can-adopt)
   2. [What Ash should avoid](#72-what-ash-should-avoid)
   3. [Open research questions for Ash](#73-open-research-questions-for-ash)
8. [References](#8-references)

---

## 1. Scope and methodology

This survey covers three overlapping areas:

1. **Compiler-integrated provers.** Programming languages and compilers that embed or communicate
   with SMT solvers, proof assistants, or automated theorem provers to verify programs.
2. **Type checking and inference as proving.** Systems where the compiler's type system is itself
   a logic, and type checking/type inference performs proof search.
3. **LLM-driven proving and verification.** Recent work using large language models to suggest
   proofs, form conjectures, find bugs, or drive interactive theorem provers.

The survey prioritizes systems and papers that are:

- practically implemented (not purely theoretical);
- relevant to general-purpose or workflow-oriented languages;
- informative for Ash's gradual-verification and symbolic-connectionist goals.

Each section ends with a short note on relevance to Ash.

---

## 2. Taxonomy

We classify compiler-prover integration along five axes:

| Axis | Values |
|---|---|
| **Automation** | Fully automated (SMT), tactic-based (proof assistant), mixed (hammers + tactics) |
| **Embedding depth** | Shallow (annotations + external checker), deep (proof terms in IR), full (verified compiler) |
| **Trust base** | Trusted solver, trusted kernel, extracted code, untrusted assistant |
| **User interaction** | Push-button, annotated, interactive, counterexample-guided |
| **LLM involvement** | None, suggestion only, closed loop with checker, fully autonomous |

Ash's target position is a **mixed-automation, shallow-to-moderate embedding, checker-trusted,
counterexample-guided, LLM-assisted** system.

---

## 3. Compiler-integrated provers

### 3.1 SMT-backed program verifiers

#### Dafny

Dafny is a verification-aware, statically typed programming language originally developed at
Microsoft Research and now maintained by the Amazon Automated Reasoning group. It supports
imperative, functional, and object-oriented idioms within a single language whose distinguishing
feature is that specifications are first-class constructs checked by a built-in static verifier.
The verifier compiles Dafny into the Boogie intermediate verification language, which generates
first-order verification conditions dispatched to the Z3 SMT solver. Correct programs are proven
automatically; failed proofs produce counterexamples that are mapped back to Dafny source locations.

Dafny's contract syntax is close to textbook Hoare logic: methods carry `requires` preconditions
and `ensures` postconditions, loops need `invariant` annotations, and recursive or iterative code
needs a `decreases` variant for termination. The language also provides `ghost` variables,
functions, lemmas, and assertions—code that participates in proofs but is erased at runtime—along
with modules for encapsulation, inductive and coinductive datatypes, generics, and refinement.

Strengths include high automation for a broad range of programs, a gentle learning curve for users
familiar with mainstream languages, strong modular verification, and a substantial industrial track
record at Amazon. Weaknesses are the flip side of its automation: verification can be brittle and
sensitive to small specification changes, loop invariant and ghost-code annotations impose a
significant specification burden, and the toolchain is tightly coupled to Z3/Boogie.

**Relevance to Ash:** Dafny is the canonical compiler-integrated SMT verifier and the closest peer
to Ash's existing `requires`/`ensures` contract syntax. It demonstrates that annotation-driven
functional verification can be practical in production, but it also illustrates the costs Ash must
contain: annotation overhead, solver unpredictability, and backend lock-in. Ash can adopt Dafny's
contract style and ghost-code discipline while avoiding tight Z3 coupling through its planned
proof-provider architecture.

**Key citations**

- K. Rustan M. Leino, "Dafny: An Automatic Program Verifier for Functional Correctness," LPAR-16,
  2010. <https://www.microsoft.com/en-us/research/publication/dafny-automatic-program-verifier-functional-correctness-2/>
- K. Rustan M. Leino, "Accessible Software Verification with Dafny," *IEEE Software* 34(6), 2017.
  <https://doi.org/10.1109/MS.2017.4121212>
- Gudmund Grov and Vytautas Tumas, "Tactics for the Dafny Program Verifier," TACAS 2016.
  <https://doi.org/10.1007/978-3-662-49674-9_3>

#### F\*

F\* is a dependently typed, higher-order functional language that doubles as a program verifier and
proof assistant. Its core design combines a predicative hierarchy of dependent types with built-in
support for ML-style computational effects including state, exceptions, divergence, and IO. The
compiler generates verification conditions and discharges them primarily with the Z3 SMT solver,
while also allowing users to fall back to tactic-based reasoning via Meta-F\*. A distinctive feature
of F\* is its use of Dijkstra monads to give effectful computations weakest-precondition
specifications, enabling compositional Hoare-style reasoning inside a dependently typed setting.

Key features include refinement types (e.g., `x:int{x >= 0}`), SMT-driven automation for proof
obligations, user-extensible effects, Meta-F\* metaprogramming and tactics, and extraction to OCaml,
F#, C, or WebAssembly through the KaRaMeL backend. The type system distinguishes pure and effectful
computation types, and the effect lattice can be extended with user-defined effects. F\* has been
used for high-assurance systems such as the HACL\* cryptographic library, the miTLS verified TLS
implementation, and the EverParse verified parsers.

**Relevance to Ash:** F\* is the closest peer technology for a functional language with effect rows,
contracts, and SMT-backed verification. Ash's `requires`/`ensures` contracts, effect rows, and
planned monadic Hoare composition map directly onto F\*'s refinement types and Dijkstra monads. The
"Dijkstra monads for free" construction is especially pertinent: it shows how a CPS translation of a
standard monad can yield a correct-by-construction weakest-precondition semantics, which could
inform how Ash compiles effectful workflows to verification conditions without hand-crafting a
separate Hoare logic for every effect. F\*'s experience also suggests that SMT automation alone is
not enough for all proof obligations; Ash should reserve manual/tactic proof as an opt-in provider.

**Key citations**

- Swamy, Hriţcu, Keller, et al., "Dependent Types and Multi-monadic Effects in F\*." POPL 2016.
  <https://fstar-lang.org/papers/mumon/>
- Ahman, Hriţcu, Maillard, et al., "Dijkstra Monads for Free." POPL 2017.
  <https://fstar-lang.org/papers/dm4free/>
- Martínez, Ahman, Dumitrescu, et al., "Meta-F\*: Proof Automation with SMT, Tactics, and
  Metaprograms." ESOP 2019. <https://arxiv.org/abs/1803.06547>

#### Why3

Why3 is a deductive program-verification platform built around WhyML, an ML-like first-order language
that combines programming and specification in one surface syntax. A WhyML program is annotated with
preconditions, postconditions, loop invariants, and variant functions; Why3 generates verification
conditions (VCs) and dispatches them to a heterogeneous set of backends, including SMT solvers
(Alt-Ergo, CVC5, Z3), first-order theorem provers (E, Vampire), and interactive proof assistants
(Rocq/Coq, Isabelle/HOL, PVS). This multi-backend architecture lets the same specification be
attempted automatically first and escalated to interactive proof only when automation fails.

Key features include a rich standard library of logical theories—arithmetic, sets, maps, sequences,
arrays, and queues—and support for user-defined algebraic theories and abstract data types. WhyML
provides polymorphic algebraic data types, pattern matching, exceptions, and limited imperative
features, and can extract verified programs to OCaml. Why3 is also widely used as an intermediate
verification language: Frama-C/WP, SPARK 2014, and the Rust verifier Creusot all translate source
programs into WhyML to reuse its driver infrastructure.

**Relevance to Ash:** Why3 is the closest practical precedent for Ash's planned proof-provider
architecture. Its theory language and named predicates map directly to Ash's `prop` layer and named
contract predicates, while its driver-based backend selection mirrors Ash's goal of routing
obligations to SMT, proof-assistant, or LLM providers without changing the source language.

**Key citations**

- F. Bobot, J.-C. Filliâtre, C. Marché, and A. Paskevich, "Why3: Shepherd Your Herd of Provers,"
  Boogie 2011. <https://why3.lri.fr/>
- F. Bobot and A. Paskevich, "Expressing Program Properties Using First-Order Logic and Theories,"
  ESOP 2013. <https://inria.hal.science/hal-00789533v1/document>
- Why3 reference manual. <https://why3.lri.fr/doc/>

#### Viper / Silicon

Viper is a verification infrastructure centered on a permission-based separation logic and an
intermediate verification language of the same name. The Silicon verifier uses Z3 to check
verification conditions generated from Viper programs; Carbon is an alternative verifier that
produces Boogie. Viper underpins several program verifiers, including Prusti for Rust, Gobra for
Go, and Nagini for Python.

Viper's permission model tracks ownership and borrow-like sharing of heap locations, making it
well-suited to verifying heap-manipulating and concurrent programs. It separates the source
language frontend from the shared Viper intermediate language, similar in spirit to Boogie and
Why3.

**Relevance to Ash:** Viper's permission model is informative for Ash's resource and provenance
reasoning, though Ash does not currently aim at full separation-logic verification. Its frontend/
intermediate-language split reinforces the provider-model lesson from Why3.

**Key citations**

- P. Müller, M. Schwerhoff, and A. J. Summers, "Viper: A Verification Infrastructure for
  Permission-Based Reasoning." VMCAI 2016. <https://doi.org/10.1007/978-3-662-49122-5_2>

### 3.2 Proof-assistant-backed verifiers

#### VST (Verified Software Toolchain)

The Verified Software Toolchain (VST) is a Coq-based framework for proving that C programs satisfy
their specifications. It builds on CompCert's verified Clight semantics and uses separation logic
to reason about memory. VST demonstrates that proof-assistant-backed verification of real programs
is possible, but labor-intensive: users write Coq proofs about C source annotated with preconditions,
postconditions, and loop invariants.

**Relevance to Ash:** VST supports Ash's decision to use provers as external providers rather than
requiring users to write proof terms. It also shows the cost of full foundational verification and
validates Ash's choice of a gradual, provider-based model.

**Key citations**

- A. W. Appel et al., *Program Logics for Certified Compilers*. Cambridge University Press, 2014.
  <https://vst.cs.princeton.edu/>

#### Iris

Iris is a higher-order concurrent separation logic framework implemented in Coq. It provides ghost
state, invariants, and view shifts for modular reasoning about fine-grained concurrency. Iris has
been used to verify realistic concurrent algorithms and has influenced several other program logics
and verifiers.

**Relevance to Ash:** Iris is the leading framework for reasoning about concurrency and fine-grained
concurrency. It informs Ash's temporal/trace contract thinking (NOTE-035) but is likely too heavy
for direct adoption in the near term.

**Key citations**

- R. Jung et al., "Iris from the Ground Up: A Modular Foundation for Higher-Order Concurrent
  Separation Logic." *Journal of Functional Programming* 28, 2018. <https://iris-project.org/>

#### Lean 4

Lean 4 is both a dependently typed functional programming language and an interactive theorem
prover, self-hosted in Lean itself. Its logical core is a small, trusted kernel for the Calculus of
Inductive Constructions (CIC): full dependent types, inductive families, an impredicative `Prop`
universe, quotient types, and universe polymorphism. Everything outside the kernel—parsing,
elaboration, type-class resolution, tactics, and code generation—is user-extensible and written in
Lean 4. This architecture yields a very small trusted computing base while allowing deep
customization.

Metaprogramming and tactics are first-class. The `MetaM`/`TacticM` monads let users write tactics,
macros, elaborators, and even entire DSLs in Lean, with hygienic macro expansion and quasiquotation
built in. Custom tactics construct proof terms that are ultimately checked by the kernel. Automation
includes `simp`, `aesop`, `grind` (SMT-style with proof reconstruction), and `lean-blaster` (Z3
backend without reconstruction). Lean 4 also exposes its own compiler IR as Lean data structures, so
backends can be written as Lean programs.

For compilation, Lean 4 translates elaborated code to an intermediate representation and emits C,
which is then compiled to native code. It uses reference counting with a "functional but in-place"
(FBIP) memory model.

**Relevance to Ash:** Lean 4 is the leading candidate for an Ash proof-provider backend. Ash's
`by lean` proof mode can target Lean's tactic framework, while its contracts (`requires`/`ensures`)
and laws map naturally to Lean propositions. Lean's extensible elaborator could also support
Ash-specific syntax and effect-aware reasoning.

**Key citations**

- L. de Moura and S. Ullrich, "The Lean 4 Theorem Prover and Programming Language," CADE-28, 2021.
  <https://lean-lang.org/papers/lean4.pdf>
- L. de Moura and S. Ullrich, "Beyond Notations: Hygienic Macro Expansion for Theorem Proving
  Languages," IJCAR 2020. <https://doi.org/10.1007/978-3-030-51074-9_10>

### 3.3 Refinement-type compilers

#### Liquid Haskell

Liquid Haskell extends Haskell with refinement types—base types annotated with SMT-decidable
logical predicates such as `{v:Int | v > 0}` or `{v:[a] | len v > 0}`. Implemented as a GHC plugin,
it reuses the standard Haskell compilation pipeline: after GHC lowers source to Core, Liquid Haskell
lifts Core terms into a refinement logic and generates implication constraints discharged by an SMT
solver (Z3 by default). This design adds machine-checked verification to an existing, mature
language without altering its runtime semantics.

Key features include automatic refinement inference via liquid types, termination checking through
well-founded metrics, and refinement reflection, which reifies a function's definition into its
refinement type so the function can appear in subsequent proofs. Reflection lets users write
equational proofs as ordinary Haskell functions: induction is recursion, case analysis is pattern
matching, and lemmas are helper functions. Recent extensions add typeclass refinements, allowing
laws such as Monoid associativity or Functor composition to be stated as refined method signatures
and proved instance-by-instance.

**Relevance to Ash:** Liquid Haskell is a strong model for adding value-predicate contracts and law
verification to Ash without redesigning the core language. Its reflection-based proofs (programs as
proofs) align with Ash's existing contract/law syntax and with the goal of letting users supply
proofs in the same language they write workflows. Its SMT-first, reflection-for-hard-cases strategy
is also a useful reference for Ash's gradual-verification provider architecture.

**Key citations**

- N. Vazou, E. L. Seidel, R. Jhala, et al., "Refinement Types for Haskell." ICFP 2014.
  <https://goto.ucsd.edu/~nvazou/icfp14/haskell-refinements-techrep.pdf>
- N. Vazou, A. Tondwalkar, V. Choudhury, et al., "Refinement Reflection: Complete Verification with
  SMT." POPL 2018. <https://arxiv.org/abs/1711.03842>
- Liquid Haskell documentation. <https://ucsd-progsys.github.io/liquidhaskell/>

#### Stainless (Scala)

Stainless is a verification tool for Scala that checks functional correctness of a subset of Scala
programs. It translates Scala to an intermediate functional representation and discharges
verification conditions to an SMT solver (Z3 or CVC4). Stainless supports preconditions,
postconditions, invariants, and ADT reasoning, and can infer some invariants automatically.

**Relevance to Ash:** Stainless demonstrates object-functional verification and could inform how Ash
handles ADTs, algebraic laws, and extraction to runnable code.

**Key citations**

- E. Kneuss, M. Bliudze, S. Kuncak, and V. Kuncak, "Synthesis-Modulo Theories for the Verification
  of Scala Programs." <https://github.com/epfl-lara/stainless>

### 3.4 Dependently typed compilers

#### Idris / Idris 2

Idris 2 is a dependently typed, pure functional language designed for practical programming with
full dependent types and totality checking. Its compiler exposes multiple intermediate
representations and can be extended with custom code-generation backends. Idris emphasizes
compile-time reasoning about program structure and supports tactic-like elaborator reflection.

**Relevance to Ash:** Idris shows how dependent types can be practical for general-purpose
programming. Its elaborator reflection is relevant to any future Ash tactic/proof-script system.

**Key citations**

- E. Brady, "Idris 2: Quantitative Type Theory in Practice." <https://idris2.readthedocs.io/>

#### Agda

Agda is a dependently typed functional language and interactive proof assistant based on
Martin-Löf type theory, emphasizing direct, proof-term-based verification through interactive
hole-filling. It can be compiled to executables via the GHC and JavaScript backends.

**Relevance to Ash:** Agda's interactive proof mode is informative for Ash's LSP/prover integration,
though Ash aims for more automation.

**Key citations**

- U. Norell, "Dependently Typed Programming in Agda." <https://agda.readthedocs.io/>

### 3.5 Verified compilers

#### CompCert

CompCert is a formally verified optimizing compiler for a large subset of C, written and proved in
Coq. It guarantees that the generated assembly semantics preserves the source semantics, closing a
major source of compiler-introduced bugs. The proof covers the compiler pipeline from Clight
through intermediate languages to assembly.

**Relevance to Ash:** CompCert is the landmark result for compiler verification. Ash does not aim to
verify its own compiler in the near term, but CompCert informs the value and cost of such an effort.

**Key citations**

- X. Leroy, "Formal Verification of a Realistic Compiler." *Communications of the ACM* 52(7), 2009.
  <https://compcert.org/>

#### CakeML

CakeML is a verified implementation of a subset of Standard ML, including a verified compiler,
runtime, and proof-producing translation from HOL4 to CakeML. It demonstrates that a full verified
compiler stack for a functional language is achievable.

**Relevance to Ash:** CakeML is a long-term reference for Ash, not a near-term target. It shows the
upper bound of assurance achievable with a fully verified toolchain.

**Key citations**

- R. Kumar et al., "CakeML: A Verified Implementation of ML." POPL 2014. <https://cakeml.org/>

### 3.6 Rust verification ecosystem

Because Ash is implemented in Rust and targets systems-oriented workflows, the recent family of
Rust verifiers is directly relevant both technically and culturally. These tools show how Rust's
ownership, borrowing, and unsafe boundaries can be reasoned about with SMT, proof assistants, and
model checking, and they provide concrete packaging models (cargo plugins, standalone binaries,
MCP-like servers) that Ash can learn from.

#### Verus

Verus is a deductive verifier for a subset of Rust developed at Microsoft Research. It extends Rust
with `requires`, `ensures`, `invariant`, and `decreases` annotations written in a first-order
specification language, then generates SMT-LIB queries (Z3 by default) to discharge verification
conditions. Verus is designed to reason about unsafe code, concurrency, and systems-level idioms
while reusing Rust's ownership discipline. It introduces `proof` blocks and ghost variables that are
erased at runtime, and supports modular verification of trait bounds and recursive functions with
well-founded decreases clauses.

**Relevance to Ash:** Verus is perhaps the closest Rust-based peer to Ash's contract system. Its
annotation style, ghost-code discipline, and SMT backend mirror Ash's planned `by solver` provider.
The main difference is that Verus is Rust-first and reasons about a shared mutable heap, whereas
Ash is workflow-oriented and effect-row based. Still, Verus's experience with ergonomic annotation
placement, SMT stability, and incremental verification is highly transferable.

**Key citations**

- A. Lattuada, T. Hance, C. Cho, et al., "Verus: Verifying Rust Programs using Linear Ghost Types,"
  OOPSLA 2023. <https://verus-lang.github.io/verus/>
- T. Hance, A. Lattuada, C. Cho, et al., "Verifying Concurrent Programs with Verus," OOPSLA 2024.
  <https://verus-lang.github.io/verus/vrs2024.pdf>

#### Creusot

Creusot is a deductive verifier for Rust built on top of Why3. It translates Rust (via a custom
intermediate language derived from MIR) into WhyML, leveraging Why3's multi-backend driver
infrastructure. Specifications are written in PEARLite, a Rust-like specification language. Creusot
handles ownership and lifetimes through a separation-logic-inspired translation and can verify
absence of panics, functional correctness, and termination.

**Relevance to Ash:** Creusot demonstrates that a Rust verifier can be organized as a frontend over
an existing multi-backend platform (Why3). This strongly supports Ash's provider-model approach: an
Ash-to-WhyML or Ash-to-SMT-LIB translation layer can reuse existing solver drivers rather than
embedding a solver directly into the Rust runtime.

**Key citations**

- X. Denis, "Creusot: A Foundry for the Deductive Verification of Rust Programs," PhD thesis, 2022.
  <https://creusot-rs.github.io/>
- X. Denis, J.-H. Jourdan, and C. Marché, "Creusot: A Deductive Verifier for Rust," FM 2022.
  <https://doi.org/10.1007/978-3-031-15077-9_13>

#### Kani

Kani is a bounded model checker for Rust developed at AWS. It compiles Rust code to a logical model
via CBMC and checks user-supplied assertions, panics, and safety properties. Unlike SMT-based
functional verifiers, Kani excels at finding bugs in unsafe code, concurrency primitives, and
low-level algorithms, but it does not prove full functional correctness for unbounded programs.

**Relevance to Ash:** Kani is a model-checking provider rather than a deductive proof provider. It
maps to Ash's `by test`/`by fuzz` assurance layer and could serve as a falsification provider for
unsafe or concurrent Ash code. Its integration model (cargo plugin, standalone binary) is also
relevant for Ash's toolchain packaging.

**Key citations**

- Kani Rust Verifier documentation. <https://model-checking.github.io/kani/>

#### Prusti

Prusti is a verifier for Rust based on the Viper verification infrastructure. It translates Rust
programs to Viper's permission-based separation logic and uses Z3 to check contracts written in a
Rust-like specification language. As of recent years Prusti is in maintenance mode and its ideas
have been absorbed by newer tools, but it remains an important proof of concept for
separation-logic verification of Rust.

**Relevance to Ash:** Prusti shows the cost/benefit of deep separation-logic reasoning for Rust.
For Ash, it reinforces that permission/ownership reasoning is valuable but heavy; a lighter contract
layer may be preferable unless provenance or resource reasoning explicitly requires it.

**Key citations**

- V. Astrauskas, P. Müller, F. Poli, and A. J. Summers, "Leveraging Rust Types for Modular
  Specification and Verification," OOPSLA 2019. <https://www.pm.inf.ethz.ch/research/prusti.html>

---

## 4. Type checking and inference as proving

Type checking and theorem proving share a common foundation in logic. In many systems, the boundary
between the two is thin or absent:

- **Hindley-Milner type inference** can be viewed as proof search in intuitionistic logic. The
  unification algorithm reconstructs proofs (types) for programs (terms).
- **Refinement types** extend ordinary types with logical predicates, turning type checking into a
  proof obligation discharged by an SMT solver. Liquid Haskell and F\* are prominent examples.
- **Dependent types** collapse the distinction between types and terms entirely; well-typed programs
  are well-proved programs. Idris, Agda, Coq, and Lean occupy this space.
- **Bidirectional type checking** interleaves type synthesis and type checking, analogous to
  proof-search strategies that alternate between goal generation and verification.
- **Constraint-based type inference** generates and solves constraints, often using SMT or
  specialized solvers. This is common in subtype inference, effect inference, and row-polymorphic
  systems.

For Ash, the important insight is that the compiler already performs a form of proving through type
checking and effect checking. Ash's conservative proposition layer (SPEC-064) is an explicit
acknowledgment of this. The design question is how far to push the merger: keep type checking and
proving separate (Dafny-style), let types carry predicates (Liquid Haskell / F\* style), or make
types fully dependent (Lean style). Ash's current trajectory is closer to the first two than to full
dependent types.

---

## 5. LLM-driven proving and verification

### 5.1 Tactic and proof-script suggestion

Tactic-suggestion systems treat an interactive prover as a deterministic environment and predict the
next proof step from the current goal. **ProverBot9001** (Sanchez-Stern et al.) is a Coq-oriented
neural system that first predicts a tactic name and then predicts its arguments with separate
models; combined with search it proved 28% of CompCert theorems. **Tactician** (Blaauwbroek et al.)
is a user-facing Coq plugin that learns online from the user's own scripts and uses k-NN retrieval
to suggest similar tactics. **CoqGym** (Yang and Deng) introduced a large-scale Coq dataset and
**ASTactic**, an encoder-decoder model that emits tactics as abstract-syntax-tree programs.
**LeanDojo** (Yang et al.) provides a reproducible Lean environment, premise annotations, and
ReProver, a retrieval-augmented LLM that selects premises and generates tactic steps.

Key techniques include proof-state encoding as terms or ASTs, k-NN retrieval, recurrent/transformer
models, retrieval-augmented generation, and beam or tree search. Strengths are low-cost,
data-driven suggestions that are automatically checked by the prover kernel, so incorrect
predictions cannot yield invalid proofs. Weaknesses include dependence on large corpora of existing
proofs, limited transfer across logics, and the risk of hallucinating tactic names or arguments.

**Relevance to Ash:** These systems are direct precedents for the compiler-orchestrated
`by lean with llm` and `by solver with llm` modes proposed in NOTE-036. The central lesson is that
the LLM is a connectionist suggester and the prover kernel is the symbolic authority; Ash should
route every generated tactic through the active proof assistant or SMT provider and record the
resulting evidence in the evidence row.

**Key citations**

- Sanchez-Stern et al., "Generating Correctness Proofs with Neural Networks," arXiv:1907.07794.
  <https://arxiv.org/abs/1907.07794>
- Blaauwbroek et al., "Tactic Learning and Proving for the Coq Proof Assistant," arXiv:2003.09140.
  <https://arxiv.org/abs/2003.09140>
- Yang and Deng, "Learning to Prove Theorems via Interacting with Proof Assistants,"
  arXiv:1905.09381. <https://arxiv.org/abs/1905.09381>
- Yang et al., "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models,"
  arXiv:2306.15626. <https://arxiv.org/abs/2306.15626>

### 5.2 Conjecture formation

Conjecture-formation systems generate candidate lemmas, invariants, or theorems that can later be
proved and reused as auxiliary facts. **MetaGen** (Wang et al.) is a neural theorem generator for
Metamath that performs forward reasoning: it selects an existing theorem, applies substitutions, and
derives a new theorem, filtering out trivial or contradictory results. The **Self-play Theorem
Prover (STP)** (Dong and Ma) uses an LLM in two roles: a conjecturer proposes novel problems barely
within reach of the current prover, and a prover attempts to solve them; the conjecturer is trained
on "barely provable" problems, creating a curriculum. **LEMMAID** (Einarsdóttir et al.) takes a
neuro-symbolic approach for Isabelle: an LLM generates lemma templates describing the shape of a
family of lemmas, and symbolic methods fill in details and check correctness and novelty.

Key techniques include forward reasoning, self-play, curriculum generation, template-based synthesis,
and neuro-symbolic filtering. Strengths include alleviating the scarcity of training data,
discovering useful auxiliary facts, and enabling lifelong learning. Weaknesses include the risk of
generating false, trivial, or redundant conjectures, and the need for a verifier or symbolic filter.

**Relevance to Ash:** Conjecture formation maps directly to Ash's goal of synthesizing algebra laws
and interface laws. An LLM could suggest candidate laws, which Ash then checks with property tests
(`by test`), SMT (`by solver`), or a proof assistant (`by lean`). The compiler should reject
unverified conjectures and only promote those with reproducible evidence.

**Key citations**

- Wang et al., "Learning to Prove Theorems by Learning to Generate Theorems," arXiv:2002.07019.
  <https://arxiv.org/abs/2002.07019>
- Dong and Ma, "Self-play LLM Theorem Provers with Iterative Conjecturing and Proving,"
  arXiv:2502.00212. <https://arxiv.org/abs/2502.00212>
- Einarsdóttir et al., "LEMMAID: Neuro-Symbolic Lemma Conjecturing," arXiv:2504.04942.
  <https://arxiv.org/abs/2504.04942>

### 5.3 Bug and security finding

LLM-based bug finding scans code, proposes vulnerabilities, generates attack inputs, or synthesizes
verification harnesses, while symbolic-execution hybrids validate candidates with formal precision.
**SAILOR** (Shafiuzzaman et al.) combines static analysis, LLM-driven harness synthesis, and
symbolic execution: static analysis identifies likely vulnerable locations, an LLM iteratively
constructs drivers, stubs, and assertions with feedback from the compiler and symbolic engine, and
symbolic execution proves or disproves the vulnerability. **ESBMC-AI** (Charalambous et al.) pairs a
bounded model checker with an LLM in a repair loop: ESBMC detects memory-safety violations and
produces counterexamples, the LLM proposes patches, and ESBMC re-verifies the result. **Lemur** (Wu,
Barrett, and Narodytska) integrates LLMs with automated reasoners for program verification, routing
tasks to either the LLM or the formal verifier based on program complexity.

Key techniques include static-analysis triage, LLM harness/invariant synthesis, bounded model
checking, symbolic execution, and counterexample-guided repair loops. Strengths are the combination
of LLM flexibility with formal guarantees, automation of tedious harness construction, and the
ability to find deep bugs. Weaknesses include path explosion in symbolic execution, LLM-generated
harnesses that may be invalid, and the possibility of missed vulnerabilities when the LLM fails to
spot an issue.

**Relevance to Ash:** Bug finding is a natural connectionist activity: the LLM proposes candidate
violations and suspicious code paths. Ash's symbolic side—runtime monitors, property tests, and SMT
checks—confirms or refutes them. This fits the NOTE-037 duality: the compiler orchestrates both
kinds of reasoning and records evidence in the evidence row.

**Key citations**

- Shafiuzzaman et al., "Guiding Symbolic Execution with Static Analysis and LLMs for Vulnerability
  Discovery," arXiv:2604.06506. <https://arxiv.org/abs/2604.06506>
- Charalambous et al., "Automated Repair of AI Code with Large Language Models and Formal
  Verification," arXiv:2405.08848. <https://arxiv.org/abs/2405.08848>
- Wu, Barrett, and Narodytska, "Lemur: Integrating Large Language Models in Automated Program
  Verification," arXiv:2310.04870. <https://arxiv.org/abs/2310.04870>

### 5.4 End-to-end theorem proving

End-to-end systems aim to prove theorems with minimal human intervention by coupling a generative
model with a verifier in a tight loop. **GPT-f** (Polu and Sutskever) showed that a transformer
could generate Metamath proof steps; follow-up **expert-iteration** work (Polu et al.) bootstrapped a
curriculum of increasingly hard statements, reaching strong results by retraining on model-generated
proofs. **AlphaProof** (Hubert et al.) pushed this further in Lean 4: a Gemini-based model is
fine-tuned with reinforcement-learning self-play and a Lean verifier, solving four of six 2024 IMO
problems at a silver-medal level. **Draft-Sketch-Prove (DSP)** (Jiang et al.) takes a different
route: an LLM first drafts an informal proof, autoformalizes it into a high-level sketch with holes,
and an automated prover fills the gaps. **Morph Prover v0 7B** (Morph Labs, 2023) was the first
open-source conversational assistant for Lean, fine-tuned from Mistral for autoformalization and
tactic dialogue.

Key techniques include expert iteration, self-play RL, informal-to-formal translation, proof
sketching, retrieval, and large-scale search. Strengths are impressive autonomous performance on
hard problems and a closed loop that guarantees any emitted proof is kernel-checked. Weaknesses are
opacity, high compute cost, prover-specific training, and limited explainability of the search
process.

**Relevance to Ash:** These systems define the upper bound of LLM-prover cooperation. Ash should
adopt the closed-loop architecture—compiler orchestrates LLM suggestions, prover verification, and
search—but keep the compiler, not the LLM, as the authority. Every proof must be backed by
reproducible prover evidence stored in the evidence row.

**Key citations**

- Polu and Sutskever, "Generative Language Modeling for Automated Theorem Proving,"
  arXiv:2009.03393. <https://arxiv.org/abs/2009.03393>
- Polu et al., "Formal Mathematics Statement Curriculum Learning," arXiv:2202.01344.
  <https://arxiv.org/abs/2202.01344>
- Hubert et al., "Olympiad-level formal mathematical reasoning with AlphaProof," *Nature* 2025.
  <https://www.nature.com/articles/s41586-025-09833-y>
- Jiang et al., "Guiding Formal Theorem Provers with Informal Proofs," arXiv:2210.12283.
  <https://arxiv.org/abs/2210.12283>
- Morph Labs, "The personal AI proof engineer," 2023.
  <https://morph.so/blog/the-personal-ai-proof-engineer/>

---

## 6. Comparison matrix

| System | Prover | Automation | Embedding | LLM integration | Strengths | Weaknesses | Ash relevance |
|---|---|---|---|---|---|---|---|
| Dafny | Z3 / Boogie | High | Shallow annotations | None | Modular, approachable | SMT brittle, ghost-code burden | Contract syntax model |
| F\* | Z3 + tactics | Mixed | Deep | None | Effects, low-level code, extraction | Steep, proof effort, Z3 dependency | Effect + Hoare model |
| Why3 | Many backends | High | Shallow | None | Backend portability, theory library | First-order limits, less mainstream | Provider model |
| Liquid Haskell | SMT | High | Refinement types | None | Retrofit to Haskell, laws as types | SMT fragments only, reflection burden | Value predicates + laws |
| Lean 4 | Lean kernel + tactics | Mixed | Deep / metaprogramming | Emerging | Modern, programmable, small TCB | Not a push-button verifier | Proof-assistant backend |
| Viper | Z3 | High | Permission SL annotations | None | Heap/concurrency reasoning | Separation-logic expertise needed | Resource/provenance reasoning |
| CompCert | Coq | Manual | Full compiler proof | None | Foundational correctness | Enormous proof effort | Upper-bound reference |
| ProverBot9001 | Coq | Suggestion | Interactive | Neural | Data-driven tactic prediction | Needs training data, Coq-only | Tactic suggestion |
| LeanDojo | Lean | Suggestion | Interactive | LLM + retrieval | Reproducible benchmark, premise selection | Closed corpus | Lean LLM integration |
| AlphaProof | Lean | Autonomous | End-to-end | RL + LLM | Strong autonomous results | Opaque, expensive, prover-specific | Upper-bound reference |
| DSP | Isabelle/Lean | Mixed | Sketch + holes | LLM | Informal-to-formal bridge | Sketch quality varies | Hybrid proof workflow |
| Verus | Z3 | High | Rust annotations | None | Ergonomic, unsafe/concurrency reasoning | Rust-only, SMT brittleness | Closest Rust peer to Ash contracts |
| Creusot | Why3 / many | High | Rust annotations | None | Reuses Why3 driver stack | PEARLite learning curve | Frontend-over-platform model |
| Kani | CBMC | High (bounded) | Rust assertions | None | Finds deep unsafe/concurrency bugs | Bounded, not full correctness | Falsification provider model |
| Prusti | Viper / Z3 | High | Rust annotations | None | Separation logic for Rust | Maintenance mode, annotation burden | Permission reasoning reference |

---

## 7. Alignment with Ash

### 7.1 What Ash can adopt

1. **External prover providers.** Why3's multi-backend approach and F\*'s SMT integration support
   Ash's plan to connect to SMT/Lean via MCP servers (NOTE-036).
2. **Modular contracts.** Dafny's `requires`/`ensures`/`invariant` style aligns with Ash's existing
   contract syntax (PLAN-194).
3. **Effect-aware Hoare logic.** F\*'s Dijkstra monads inform Ash's monadic Hoare composition
   (NOTE-030).
4. **LLM-as-suggester checked by kernel.** LeanDojo and ProverBot9001 validate the pattern in
   NOTE-037: the LLM proposes, the prover checks.
5. **Refinement-type contracts.** Liquid Haskell shows a path for adding predicates to existing
   types without rebuilding the type system.
6. **Rust verifier packaging.** Verus and Kani demonstrate that Rust-based verifiers can be
   distributed as cargo plugins or standalone binaries without dragging the entire compiler into
   the runtime. This informs how `ash-smt` and `ash-prover-smt` could be packaged.
7. **Type checking as proving.** The insight that type checking already performs proof search
   justifies Ash's conservative proposition layer (SPEC-064) and suggests a gradual merger rather
   than a separate verification language.

### 7.2 What Ash should avoid

1. **Requiring users to write proof terms.** Systems like Coq/Agda expect significant manual proof.
   Ash should keep automatic discharge as the default and manual proof as opt-in.
2. **Opaque LLM evidence.** AlphaProof-style end-to-end proving can produce correct results without
   explainable evidence. Ash must require reproducible prover evidence.
3. **Tight coupling to one solver.** Dafny's reliance on Z3 creates fragility. Ash's provider model
   avoids this.
4. **Ignoring effect tracking.** Many verifiers treat effects implicitly. Ash should make LLM and
   prover calls explicit in rows (NOTE-037).
5. **Full compiler verification as a near-term goal.** CompCert and CakeML are valuable reference
   points, but the cost is too high for Ash's current maturity.

### 7.3 Open research questions for Ash

1. How much of Ash's type system can be discharged by SMT without becoming unpredictable?
2. Which Lean 4 fragments are practical as a first proof-backend slice?
3. How should Ash represent LLM suggestions in the Core IR / evidence model?
4. Can synthesized laws be automatically routed to the most appropriate provider (SMT, Lean, test)?
5. What is the right user interface for iterative LLM-prover search: compiler flag, LSP code action,
   or explicit `proof` body modifier?
6. How should Ash handle non-deterministic LLM effects in effect rows and module summaries?
7. What is the right balance between annotation burden and automation for Ash's target users?
8. Which Rust-verifier patterns (Verus's SMT-LIB generation, Creusot's WhyML translation, Kani's
   bounded model checking) best fit Ash's effect-row and workflow semantics?
9. Should Ash produce SMT-LIB directly or translate through an intermediate verification language
   such as Boogie or WhyML?

---

## 8. References

### Compiler-integrated provers

- Dafny: <https://dafny.org/>
- F\*: <https://www.fstar-lang.org/>
- Why3: <https://why3.lri.fr/>
- Liquid Haskell: <https://ucsd-progsys.github.io/liquidhaskell/>
- Lean 4: <https://lean-lang.org/>
- Coq: <https://coq.inria.fr/>
- Isabelle: <https://isabelle.in.tum.de/>
- CompCert: <https://compcert.org/>
- CakeML: <https://cakeml.org/>
- VST: <https://vst.cs.princeton.edu/>
- Iris: <https://iris-project.org/>
- Stainless: <https://github.com/epfl-lara/stainless>
- Viper: <https://viper.ethz.ch/>
- Verus: <https://verus-lang.github.io/verus/>
- Creusot: <https://creusot-rs.github.io/>
- Kani: <https://model-checking.github.io/kani/>
- Prusti: <https://www.pm.inf.ethz.ch/research/prusti.html>

### Selected papers

- K. Rustan M. Leino, "Dafny: An Automatic Program Verifier for Functional Correctness," LPAR-16,
  2010.
- K. Rustan M. Leino, "Accessible Software Verification with Dafny," *IEEE Software* 34(6), 2017.
- Swamy, Hriţcu, Keller, et al., "Dependent Types and Multi-monadic Effects in F\*." POPL 2016.
- Ahman, Hriţcu, Maillard, et al., "Dijkstra Monads for Free." POPL 2017.
- F. Bobot, J.-C. Filliâtre, C. Marché, and A. Paskevich, "Why3: Shepherd Your Herd of Provers."
- N. Vazou et al., "Refinement Types for Haskell." ICFP 2014.
- N. Vazou et al., "Refinement Reflection: Complete Verification with SMT." POPL 2018.
- L. de Moura and S. Ullrich, "The Lean 4 Theorem Prover and Programming Language." CADE 2021.
- X. Leroy, "Formal Verification of a Realistic Compiler." *CACM* 2009.
- R. Jung et al., "Iris from the Ground Up." *JFP* 2018.
- A. Lattuada et al., "Verus: Verifying Rust Programs using Linear Ghost Types." OOPSLA 2023.
- T. Hance et al., "Verifying Concurrent Programs with Verus." OOPSLA 2024.
- X. Denis, J.-H. Jourdan, and C. Marché, "Creusot: A Deductive Verifier for Rust." FM 2022.
- V. Astrauskas et al., "Leveraging Rust Types for Modular Specification and Verification." OOPSLA
  2019.

### LLM-for-proving papers

- Sanchez-Stern et al., "Generating Correctness Proofs with Neural Networks," arXiv:1907.07794.
- Blaauwbroek et al., "Tactic Learning and Proving for the Coq Proof Assistant," arXiv:2003.09140.
- Yang and Deng, "Learning to Prove Theorems via Interacting with Proof Assistants,"
  arXiv:1905.09381.
- Yang et al., "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models,"
  arXiv:2306.15626.
- Polu and Sutskever, "Generative Language Modeling for Automated Theorem Proving,"
  arXiv:2009.03393.
- Polu et al., "Formal Mathematics Statement Curriculum Learning," arXiv:2202.01344.
- Hubert et al., "Olympiad-level formal mathematical reasoning with AlphaProof," *Nature* 2025.
- Jiang et al., "Guiding Formal Theorem Provers with Informal Proofs," arXiv:2210.12283.
- Wang et al., "Learning to Prove Theorems by Learning to Generate Theorems," arXiv:2002.07019.
- Dong and Ma, "Self-play LLM Theorem Provers with Iterative Conjecturing and Proving,"
  arXiv:2502.00212.
- Einarsdóttir et al., "LEMMAID: Neuro-Symbolic Lemma Conjecturing," arXiv:2504.04942.
- Shafiuzzaman et al., "Guiding Symbolic Execution with Static Analysis and LLMs for Vulnerability
  Discovery," arXiv:2604.06506.
- Charalambous et al., "Automated Repair of AI Code with Large Language Models and Formal
  Verification," arXiv:2405.08848.
- Wu, Barrett, and Narodytska, "Lemur: Integrating Large Language Models in Automated Program
  Verification," arXiv:2310.04870.

---

## Changelog

| Date | Change |
|---|---|
| 2026-07-07 | Initial draft. Defined scope, taxonomy, section structure, and Ash-alignment framework. Populated compiler-integrated provers, type-checking-as-proving, and LLM-driven proving sections with literature references. |
| 2026-07-07 | Added §3.6 on the Rust verification ecosystem (Verus, Creusot, Kani, Prusti), updated comparison matrix, and added open questions on SMT-LIB generation strategy. |
