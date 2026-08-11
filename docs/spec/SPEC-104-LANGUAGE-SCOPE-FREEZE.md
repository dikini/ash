---
id: spec.ash.language-scope-freeze
title: Ash Language Scope Freeze
kind: spec
audience: [human, agent]
authority: canonical
status: active
stability: alpha
owner: language
last_verified: 2026-08-09
verified_against:
  specs:
    - docs/spec/CANONICAL-CORE.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
    - docs/spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md
  plans:
    - docs/plan/PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md
    - docs/plan/PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md
    - docs/plan/PLAN-196-APPLICATION-WORKFLOW-RUNTIME.md
    - docs/plan/PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md
---

# SPEC-104: Ash Language Scope Freeze

**Status:** Active language-scope authority. This document freezes the intended language into P1,
P2, P3+, and removed sets. Those dispositions are priorities, not implementation claims.
**Scope:** Surface language, static semantics, standard-library boundary, effects/providers,
application boundary, process runtime, testing, and trace observability.
**Depends on:** Ash Canonical Core, SPEC-095b through SPEC-100, SPEC-103, and PLAN-203.

## 1. Purpose

Ash is a general-purpose functional language specialized for automation and orchestration of AI
and other systems. Correctness and performance are parallel goals. Performance comparisons must
compare equivalent execution tiers: interpreter with interpreter, bytecode executor with bytecode
executor, and JIT with JIT.

The immediate objective is an actually executable, coherent language rather than preservation of
the accumulated feature inventory. P1 is therefore the smallest useful language that can travel
through the production route:

~~~text
Surface Ash -> checked Core -> checked CPS -> Engine CPS executor -> terminal envelope
~~~

CLI and daemon are clients of this route. There is no direct evaluator fallback.

## 2. Authority and precedence

SPEC-104 owns whether a feature is retained, its phase, and whether old syntax or behavior must be
removed. Ash Canonical Core and the more detailed target specifications own semantics only for
features retained here.

When another target, design, plan, reference, test, or implementation artifact conflicts with this
scope freeze:

1. SPEC-104 controls scope and phase.
2. Ash Canonical Core controls the semantics of retained features.
3. The most specific non-conflicting target rule controls detail.
4. Current code and tests are implementation evidence only; they do not retain a removed feature.

No backward compatibility or migration alias is required. Removed syntax must eventually reject
with a direct diagnostic, and redundant implementation and tests must be deleted.

This document introduces four stable scope rules:

| Scope rule | Boundary |
|---|---|
| REQ-SPEC104-P1-BOUNDARY-001 | The complete P1 language boundary |
| REQ-SPEC104-P2-BOUNDARY-001 | The P2 set and P1 exclusions |
| REQ-SPEC104-P3-BOUNDARY-001 | The fresh-design P3+ set |
| REQ-SPEC104-REMOVAL-001 | The removal, rejection, and deferral boundary |

These are deliberately broad disposition umbrellas, not implementation task units. Exact feature
semantics continue to refine the existing Canonical Core rules:

| Frozen domain | Existing canonical rule owners |
|---|---|
| Purpose, vocabulary, and phase/removal policy | VOCAB-TARGET-OVERVIEW-001 |
| Surface forms across the P1/P2/P3+/removal boundaries | GRAM-TARGET-MODULE-001 |
| Types, rows, effects, and boundary typing | TYPE-TARGET-ROW-001 |
| Required Core/CPS representation | CORE-CPS-SYNTAX-001 |
| Surface elaboration and lowering | LOWER-SURFACE-CORE-001 |
| Evaluation, handlers, providers, processes, and cancellation | SEM-TARGET-CORE-CPS-001 |
| Terminals, manifests, JSON boundaries, tracing, and client-visible results | OBS-TARGET-PROJECTION-001 |
| Complete production route and deletion conformance | CONF-IMPLEMENTATION-001 |

Before implementing a clause whose detailed rule is missing or contradictory, the task must first
introduce or reconcile a narrow rule under the applicable scope umbrella, mapped Canonical Core
rule, and specific target owner. Implementation work reports implementation, evidence, and parity
independently.

## 3. Phase meanings and change control

| Disposition | Meaning |
|---|---|
| P1 | Required for the first coherent, useful, end-to-end executable language. |
| P2 | Valuable next capability, excluded from the P1 critical path. |
| P3+ | Deferred design, research, tooling, or convenience work. It must not constrain P1 representation or compatibility. |
| Remove | Not part of the target language. Delete syntax, semantics, compatibility paths, code, and tests unless a small rejection diagnostic is still useful. |

A feature may enter or leave the freeze only through a SPEC-104 amendment that states its execution
route, interaction with retained features, implementation cost, deletion cost, and phase effect.
An implementation task may narrow its slice but may not silently widen the language.

P1 completion requires the selected rule to be complete across every applicable layer, with
positive, negative, and mutation evidence. An active executable route also requires CLI/daemon
parity; a route marked none or prerequisite records why client parity is not applicable. “Task
complete” does not mean “feature implemented” when a layer remains missing.

## P1 language boundary

### 4.1 Programs, applications, and modules

- Programs are composed from ordinary functions. There is no special workflow declaration.
- An application is selected by an explicit manifest; without one, the root module's ordinary
  monomorphic function named main is the entry.
- The entry has one input and one output. Its output is Result. Boundary input and output use an
  exact canonical JSON representation and must be closed, serializable, and Sendable.
- With no manifest, the entry's fully annotated parameter and Result payload types define the
  boundary schemas. CLI or daemon supplies exactly one JSON input value and validates it against
  the parameter type before execution. A manifest may restate stricter named schemas, but they
  must be compatible with the annotated entry type.
- The entry's effect row is closed. Public boundaries require explicit complete types and closed
  rows. Private/local inference is best effort only; an annotation is required when local row
  inference cannot decide.
- File-backed and inline modules normalize to the same ModuleUnit and cannot change program
  meaning. Structural modules form a rooted acyclic tree; import dependencies are acyclic in P1.
- File layout uses explicit Rust-style module paths. Source roots are deterministic. self and
  super are explicit; name resolution has no fallback search.
- Items are private by default. pub exposes them. Imports are explicit, module-wide, and
  source-order independent. Grouped imports and explicit pub use re-exports are supported. Glob
  imports are not.
- Modules may act purely as API facades assembled from submodule re-exports.
- Functions are order-independent and recursive. Cross-module recursion is rejected with import
  cycles in P1.
- Module values are immutable, explicitly typed let bindings. They are pure, initialized once per
  application, and limited to the application-reachable graph. Dependencies initialize first;
  ties use fully qualified name order. Effectful initialization and initialization cycles reject.
- Types and values share one namespace except that a nominal type may have a same-name companion
  module.
- There is no implicit prelude. Primitive type names are universal, but primitive operations are
  obtained through explicitly imported companion modules or aliases.
- Source is UTF-8 without a byte-order mark. A BOM or invalid UTF-8 rejects. LF and CRLF each count
  as one newline. Identifiers and labels are ASCII; strings and comments accept Unicode.
- Line comments use // and documentation comments use ///. There are no block comments, raw
  identifiers, or general attribute syntax in P1.

### 4.2 Application manifest and run envelope

The optional P1 application manifest is strict, versioned JSON with one exact schema. Unknown
fields reject. It is metadata only and may name:

- the application entry function;
- provider recipes and exact effect bindings;
- input and output boundary schemas;
- declared run-binding slots for secrets or host configuration.

It contains no Ash code, authority grants, or secret values. A provider entry is a requested exact
recipe selection, not an authorization. Engine admission validates that selection against
host-controlled policy and only the resulting sealed admission artifact installs provider
authority. Discovery is explicit: a CLI
--manifest path or a daemon application identity resolves exactly one manifest. There is no
adjacent-file or upward-directory search. Omitting a manifest means no manifest and root main.

The manifest is authoritative for entry, requested provider selections, schemas, and declared
slots. A run envelope
does not override or merge them. It supplies only values for declared inputs/bindings, secret
material, explicit root ProcessConfig, deadline, trace configuration, provider shutdown grace, and
other operational controls.

Paths are relative to the manifest directory. After canonicalization they must remain contained
within that directory. Absolute paths, parent escapes, and symlink targets outside the root reject.

#### 4.2.1 Canonical boundary JSON

Boundary JSON is schema-directed: the fully checked Ash boundary type determines how a JSON value
is decoded. Encoders emit UTF-8 without insignificant whitespace, sort object keys by Unicode
scalar order, reject duplicate object keys, and use the canonical primitive formatting in §4.6.

| Ash value | Canonical JSON |
|---|---|
| Unit, Bool, String | null, a JSON boolean, or a JSON string |
| Int | A decimal JSON number in the signed 64-bit range |
| finite Float other than negative zero | A shortest-roundtrip JSON number |
| NaN, positive/negative infinity, negative zero | An object with sole field $float and exact value nan, inf, -inf, or -0.0 |
| Char | A JSON string containing exactly one Unicode scalar |
| Bytes | An object with sole field $bytes containing unpadded base64url text |
| tuple or List | A JSON array in element order |
| record | A JSON object whose keys are exactly the record labels |
| nominal ADT | An object with $tag equal to the fully qualified constructor and, for a payload constructor, a $value field |
| Map<K, V> | An array of two-element [key, value] arrays in canonical ascending K order |
| transparent alias | The representation of its expanded type |

Result uses the nominal ADT rule. Opaque types, functions, dictionaries, effect/provider values,
Inbox, ProcessRef, and ProcessControl are not boundary-serializable in P1. The expected type makes
$float, $bytes, $tag, and $value unambiguous; those reserved keys are not valid source record
labels. Decoding rejects missing/extra fields, wrong tags, non-canonical tagged values,
out-of-range numbers, and duplicate keys. It does not accept multiple spellings for the same Ash
value.

### 4.3 Functions, evaluation, and control

- Evaluation is strict call-by-value and left-to-right.
- Named functions use a braced body. A bodyless signature ends with a semicolon; there is no
  = expression function body.
- Functions have explicit multiple arguments and are not curried. Partial application uses an
  explicit lambda.
- Lambda syntax is |x: A| expression. Parameter annotations may be omitted only when a complete
  expected callable type supplies them.
- Closures capture immutable values. A closure is Sendable exactly when all captures are Sendable;
  closures never cross an application boundary.
- Callable types are (A, B) -> R ! {Effects}. Omitting the row means the empty row.
- Bindings are immutable and may shadow. The let value restriction applies. Parameter, lambda, and
  let patterns must be irrefutable.
- Blocks are expressions. Their final expression is their value. A non-final bare expression must
  have Unit type; semicolon sequencing discards Unit.
- return exits the innermost callable and is a bottom expression at its use site. Bare return
  returns Unit. It bypasses ordinary handler arms; a handler return clause applies only to normal
  completion.
- Parameters are positional and fixed-arity. There are no labels, defaults, or variadics.
- Proper tail calls are stack safe. Iteration uses recursion or concrete collection operations;
  there are no loop statements.
- Surface if requires Bool, requires else, and lowers to match. Core has no separate if form.
  && and || short-circuit.
- match is exhaustive; unreachable arms reject. An empty match may eliminate an empty ADT.
  Arms end with commas.
- P1 patterns are nested wildcard, binding, non-Float literal, tuple, and qualified ADT
  constructor patterns. Binders are unique. Refutable patterns appear only in match.
- Local recursion uses one annotated monomorphic let rec. Module recursive strongly connected
  components require complete signatures.
- Generic arguments infer by default; ambiguous calls may use explicit ::<...> arguments.
- Result-only postfix ? returns through the nearest callable. The enclosing result must have the
  exact error type. It uses the imported canonical Result identity or alias; a missing import
  produces a targeted diagnostic. There is no general Try abstraction.
- There is no truthiness, exception, async function, generator, defer, destructor, if let, guard,
  or user-defined operator in P1.

### 4.4 Data and types

- Nominal algebraic data types and transparent aliases coexist with structural tuples and records.
- Every defined nominal type uses type Name = body;. Examples:

~~~ash
type Option<T> = None | Some(T);
type Never = {};
~~~

- A bodyless type declaration, type Opaque;, is abstract: the type exists, but its representation
  and values are unknown. It is not the empty type. The empty ADT has the value-level body {}.
- Constructors have either no payload or exactly one payload. A tuple payload is explicit, such as
  Make((Float, Float)); a record payload is labeled. Only | denotes alternatives.

~~~ash
type Point2D = Point2D({x: Float, y: Float});
type Axis = X(Float) | Y(Float);
~~~

  A labeled record payload is therefore not confused with a choice between field-like
  constructors.
- Constructors are qualified and first-class. A nullary constructor is a value; a payload
  constructor is a one-argument function.
- Constructors follow the type's visibility unless a public type uses a private right-hand side,
  as in pub type Token = private Make(Bytes);. Per-variant visibility is absent.
- Aliases use the distinct transparent, non-recursive form alias UserId = Int;.
- Records are unordered structural labeled products. Field order is not type or layout identity,
  while field expressions evaluate left-to-right. Duplicate labels reject.
- Open record types use {x: Int | r}, where r has kind RecordRow. A runtime record value is closed.
  An open row implicitly lacks its explicit labels.
- Record update {p with x: value} is shape-preserving. The base evaluates once; replacements
  evaluate left-to-right. Updated fields must exist and retain their types. There is no extension
  or removal update.
- () is Unit and {} is an empty block expression; there is no empty-record value.
- Tuples have arity two or greater and are accessed through patterns, not numeric projection.
- Generics are rank 1. Abstract nominal types may have ordinary type parameters.
- P1 interfaces are minimal and coherent:

~~~ash
interface CollectionShape<C> {
    type Item;
    type Index;
}

impl CollectionShape<MyCollection> {
    type Item = Int;
    type Index = Float;
}
~~~

- Interface constraints use where Eq<T>. Associated types have kind Type and are projected as
  CollectionShape<C>::Item; a bare associated name is allowed only within its interface or impl.
- An impl defines each associated type exactly once with a concrete type. Missing, duplicate,
  defaulted, or extra definitions reject.
- Interface evidence is explicit in checked Core. At the surface, coherent lookup supplies the
  default dictionary; a call may select a complete alternative dictionary with using dict.
  Associated projections cannot be overridden by a call-site dictionary.
- Interface methods are not bare first-class values; an explicit lambda adapts them.
- An interface method exposes its exact declared effect row. An impl may use a strict subset
  internally but cannot widen the exposed row.
- There is no inheritance, subtyping, Any, Dynamic, null, function overloading, newtype, associated
  equality constraint, impl overlap, specialization, negative impl, or interface default method.
- Implementations obey orphan/coherence rules and may be constrained.
- Type ascription is supported. Casts are not; conversions are named functions.

### 4.5 Effects, rows, handlers, and providers

- P1 has a minimal end-to-end algebraic-effect system. Static interfaces and effect declarations
  are distinct.
- Effect declarations contain bodyless operation signatures. Operations are called by qualified
  name. Effect applications are nominal, exact row keys:

~~~ash
effect Fs<Route> {
    fn read(path: String) -> Result<Bytes, FsError>;
}

Fs<Real>::read(path)
~~~

- A direct operation adds only its enclosing nominal effect application. Duplicate exact entries
  collapse.
- Effect identity arguments are closed and monomorphic at runtime. They need not be Sendable or
  serializable; opaque marker types are permitted. Actual operation payloads retain their own
  transport requirements.
- Callable rows are declared upper bounds; a body may use a subset. Widening is explicit.
- Direct rows and one open tail are supported: ! {Fs, Log | e}, where e: EffectRow. Elimination
  from a generic row requires an explicit exclusion constraint such as
  where e excludes {Fs, Log}.
- Rows state requirements and never grant authority. Imports expose names and never grant
  authority.
- handle is lexical, deep, total, and affine. Every operation appears exactly once and return is
  mandatory:

~~~ash
handle work() with Fs<Policy> {
    read(path) => ...,
    return(value) => ...,
}
~~~

- A handler may transform result A to B; each operation arm and the return arm produce B.
  resume(value) is scoped, non-first-class, and statically usable at most once on every path.
  Zero resumes abort the continuation. A same-effect call made in a clause forwards outward;
  resume reinstalls the handler.
- Handler clauses end with commas. Source order has no semantic effect.
- Effect declarations have rank-1 parameters and no operation-local generics.
- External and built-in functionality is visible through modules. Source has no builtin
  declaration. Pure trusted extern functions are imported from trusted runtime/standard-library
  module interfaces and linked by exact module/item identity; P1 source authors cannot declare
  them. Effectful host access is through providers. The detailed extern registry and failure rule
  must be reconciled under the mapped canonical rules before implementation.
- A **provider declaration** is trusted runtime/standard-library metadata visible through a
  module. A **provider recipe selection** is manifest metadata choosing and configuring a
  declaration for one exact effect application. A **provider binding** is the Engine-authorized
  runtime instance created only after admission. None is a first-class Ash value.
- A provider declaration may be rank-1 generic:

~~~ash
pub provider LocalFs<Route>(config: Config) for Fs<Route> ! {Clock, Log};
~~~

- A recipe configuration is one typed closed value, or Unit when absent. The manifest supplies its
  exact JSON representation. Provider dependencies form a separate closed row and an acyclic
  graph; shutdown reverses initialization.
- Provider selection uses the innermost lexical handler and then the unique manifest provider.
  Missing or ambiguous provision rejects.
- Provider instances are runtime message services. Calls are ordered per caller; calls from
  different processes may be concurrent and their completion order is traced.
- Provider cancellation is best effort. The caller never resumes, late replies are discarded, and
  no rollback is implied. P1 operations make one attempt.
- Shutdown is bounded and two-phase. A forced close becomes provider failure; if shutdown fails it
  is the primary terminal outcome and the original main/deadline cause remains diagnostic context.
- This provider-service machinery is mandatory P1 execution infrastructure, not the P2
  user-visible service/lifecycle abstraction. P1 exposes no general service declaration, health,
  reload, or supervisor profile.
- Secrets and host configuration enter only through typed declared run slots. They are redacted
  and are not directly visible to Ash code.
- Alternative policy routing uses distinct nominal effect applications and an ordinary handler,
  for example Fs<Real>, Fs<Dummy>, and Fs<Policy>. Providers are not first-class handles.

### 4.6 Primitive values and collections

- P1 Int is signed 64-bit and Float is IEEE binary64. Future fixed-width numeric types may be
  added without changing Int or Float.
- String, Char, and Bytes are distinct. Char is exactly one Unicode scalar value.
- Int arithmetic is checked. Overflow terminates the current process with
  ArithmeticError::Overflow; named checked operations return Option.
- Int division is Euclidean with 0 <= remainder < abs(divisor). Division by zero and MIN / -1 are
  arithmetic errors; checked companions return Option.
- Float preserves NaN, infinities, and negative zero; uses round-to-nearest ties-to-even; and does
  not permit reassociation or implicit fused operations. Float arithmetic does not raise
  ArithmeticError.
- Float comparison is IEEE partial comparison. Float has no lawful Eq or Ord instance and cannot
  be a Map key.
- Numeric conversions are explicit. Float::from_int may round; an exact companion returns Option.
  Float-to-Int functions name their rounding mode and return Result for non-finite or out-of-range
  values.
- Bit operations are named Int companion functions, not operators. Shifts act on the bit pattern
  and return Option for counts outside 0 through 63.
- P1 math is limited to arithmetic, comparison, classification, rounding, absolute value, and
  conversion. Powers, roots, logarithms, exponentials, and trigonometry are P2.
- Numeric literals include decimal integers/floats and hexadecimal/binary integers, with _
  separators. There is no octal or suffix syntax. Expected types decide ambiguous literals;
  defaults are Int and Float. Unary negation folds the minimum integer literal before range
  checking.
- + is numeric only. String, Bytes, and List concatenation use named companion functions.
- Primitive parsing and formatting use explicit companion functions for Int, Float, Bool, and
  Char. Parsing returns a typed Result and consumes the entire input with no whitespace or _.
  Integer text is signed decimal; Float text is decimal/scientific plus exact nan, inf, and -inf;
  Bool is exactly true or false; Char is exactly one scalar.
- Formatting is canonical and locale-independent. Float formatting is shortest-roundtrip and
  preserves -0.0. P1 has no generic formatting interface or implicit interpolation conversion.
- String equality, ordering, and hashing use Unicode scalar sequences without normalization.
  Locale-independent Unicode collation may be a P2 library. String order is lexicographic by
  scalar; Bytes order is lexicographic by byte.
- String literals support escapes and interpolation plus a non-interpolated raw multiline form.
  An interpolation expression must already have String type.
- Bytes literals contain ASCII plus \xNN escapes.
- String::uncons returns Option<(Char, String)> and a pure fold is provided. Bytes has analogous
  uncons/fold operations using Int values 0 through 255; construction validates the range.
  P1 has no String/Bytes indexing or slicing and no Byte type.
- UTF-8 encode/decode is explicit. Decode returns Result and reports the invalid byte offset.
- List and Map are canonical standard-library types, not universal names. Literals require their
  imported canonical identity or alias; a missing import gets a targeted diagnostic.
- Collections are persistent and immutable. List and Map are P1; Set is P2.
- List literals and Map literals use fixed entries with no spread. Map syntax is
  #{key => value}; #{} is empty. Entries evaluate left-to-right and a duplicate key is last-wins.
  Empty literals require an expected type; non-empty elements require an exact common type.
- Map is ordered and requires Ord<K>. Iteration is canonical ascending key order; equality is
  independent of insertion history.
- P1 offers concrete List/Map operations, not a general Iterator or lazy-adapter protocol.
  Higher-order callbacks are pure. Effectful traversal uses explicit recursion and List::uncons.
- Json is a closed ADT with explicit Result-returning decoding.

### 4.7 Assertions, incomplete code, and tests

- Domain failure uses ordinary Result or declared effects. Assertions are a separate nonrecoverable
  trap.
- assert evaluates its Bool condition first. On true it produces Unit without evaluating the
  message. On false it evaluates one pure String message and terminates with AssertionFailed.
  assert(false, message) is the unconditional trap; there is no separate panic form.
- Source comments may use // TODO:. The expression todo("static description") has bottom type and
  adapts to any expected type without fabricating a value.
- todo supports allow, warn, and deny policy. P1 defaults to warn for all clients. allow and warn
  permit compilation, but reaching the expression terminates with noncatchable TodoReached carrying
  its description and source span. deny rejects at admission. Profile-dependent defaults are P2+.
- Tests are ordinary zero-argument, Unit-returning, closed-row Ash functions selected by a
  manifest. Each runs as a fresh application through the same Engine route with an explicit
  provider profile. Discovery/registration must not initialize unrelated application modules;
  execution uses a bounded worker set.
- Later special test, law, contract, and proof forms may call ordinary Ash functions, but they do
  not belong to P1.

### 4.8 Lightweight processes

- P1 processes are isolated, lightweight, and semantically shared-nothing. Communication and
  sharing, including effects, occur only through messaging. Immutable backing storage may be
  shared internally only when unobservable.
- Initial arguments and messages must be explicitly Sendable. spawn names a process entry; it
  cannot capture a closure, handler, or ambient authority.
- A process entry receives its private Inbox<M> and own ProcessRef<M>, followed by initialization
  arguments. Inbox is abstract, process-local, non-serializable, and non-Sendable. There is no
  ambient mailbox or self lookup.
- A P1 process entry returns Unit. Typed exit values are deferred.
- ProcessRef<M> is app-local, non-serializable, and Sendable within that application. It has
  identity equality only, no ordering and no exposed numeric identifier. It remains stable after
  termination.
- spawn returns (ProcessRef<M>, ProcessControl). The reference sends and joins. ProcessControl
  requests stop and is process-local and non-Sendable.
- stop is asynchronous and idempotent, reporting Requested, AlreadyRequested, or
  AlreadyTerminated. join waits, is repeatable by many observers, and returns
  Result<ProcessOutcome, JoinError>. Self-join immediately returns SelfJoin.
- ProcessOutcome is Completed, Stopped, or Failed(ProcessFailure). Failure detail is available
  through tracing rather than an unbounded tombstone.
- Live processes are runtime-owned until exit, explicit stop, or application shutdown; dropping
  handles does not cancel them.
- On exit the stack, mailbox, runtime state, and process-local provider client/session state are
  released immediately. Application-owned provider instances remain until application shutdown.
  A compact outcome remains only while an owning handle or pending join exists. The weak registry
  reclaims it after the last owner; a process with no handles at exit leaves no tombstone.
- Process limits count live root/child processes in starting, runnable, suspended, and stopping
  states, not tombstones. Provider services have separate limits.
- spawn is atomic and nonblocking. It returns handles or SpawnError for quota/shutdown. After the
  linearization point parent and child are both runnable with no parent-first guarantee; the
  decision is traced.
- Every spawn has an explicit ProcessConfig containing positive bounded FIFO mailbox capacity and
  logical memory budget. The root configuration comes from run control. There are no implicit
  defaults.
- send is nonblocking and atomic, returning Sent, Full, Closed, or ResourceExhausted. Sent means
  enqueued, not processed. Per-sender FIFO is guaranteed; inter-sender order is nondeterministic
  and traced.
- receive suspends. receive_for returns Message or TimedOut and requires Process plus
  Clock<Monotonic>. A zero duration is an immediate poll. Message/timeout races are linearized and
  traced; a message losing the race remains queued.
- Duration is an imported abstract standard type made by validated nonnegative constructors.
  There are no duration suffix literals.
- Memory enforcement uses dual accounting: each process owns a logical retained budget including
  mailbox and heap; the application counts actual physical allocation once. A successful send
  reserves receiver budget; dequeue reclassifies it. Ordinary allocation beyond a process budget
  fails only that process with ResourceLimitExceeded.
- Main completion cancels all remaining processes, performs bounded cleanup, and produces one
  terminal outcome. Cancellation is two-phase and uncatchable.
- P1 includes explicit process-count, mailbox, logical-memory, and application-deadline limits. It
  has no CPU/fuel limit.
- A run deadline is explicitly Unlimited or After(Duration); there is no implicit deadline. The
  host monotonic clock controls it. Expiry cancels the application and yields DeadlineExceeded.
  Provider shutdown grace is separate.

### 4.9 Tracing and runtime lifecycle

- P1 trace collection is optional and control-plane-only. Ash code cannot observe it; application
  logging remains an ordinary effect.
- When enabled, P1 records enough runtime-controlled scheduling and linearization decisions for a
  future replay implementation. P1 does not implement replay or provider-outcome capture.
- Trace retention is bounded and lossless-or-fail. If the sink cannot accept an event, the
  run terminates with noncatchable TraceFailure and the trace is marked incomplete. “Observational”
  means trace contents cannot influence an otherwise healthy execution; instrumentation failure is
  the explicit exception and becomes the terminal envelope rather than an Ash-catchable effect.
- The trace format is one exact version with no compatibility or migration requirement.
- Semantic events include application/process identity and lifecycle, scheduler choices, message
  enqueue/dequeue, timer races, cancellation/shutdown transitions, and provider completion order.
  Core/CPS instruction reductions are not trace events.
- P1 trace data contains metadata and correlation identifiers, not message/provider bodies or
  fingerprints. Secrets are redacted.
- P1 has no dedicated service, health, reload, supervisor-profile, or external-actor abstraction.
  Long-lived work is an application using Unlimited deadline and ordinary processes. The daemon
  provides start, cancel, join, and status as control-plane operations.

## P2 boundary

P2 may add the following without delaying or broadening P1:

- unary higher-kinded parameters of kind Type -> Type, with a minimal Monad interface containing
  pure and flat_map, Option and Result instances, and ordinary helper functions;
- explicit constructor holes such as Result<_, Error>, with one hole and explicit kinds;
- transparent closed effect-row aliases after substitution, including value type parameters but no
  row parameters or open alias tails;
- sealed one-way associated type families, Set, Eq/Ord derivation, or-patterns, and
  locale-independent Unicode collation as a library, plus a lawful total-order Float wrapper;
- effect-polymorphic collection combinators and advanced mathematical companion functions;
- trusted user FFI and persistent user-defined provider recipes;
- provider retries with explicit evidence, resource-scope library combinators with nonescaping
  handles, and app-default provider tooling that makes the provider of Fs::read obvious without
  creating a universal or magical provider;
- typed process exit values, selectable mailbox policy expressed through interfaces/effects,
  links, supervision, and dedicated service lifecycle;
- offline temporal analysis of completed traces. Analysis cannot affect admission, scheduling, or
  the result of the recorded run;
- profile-dependent todo defaults and other tooling profiles.

P2 does not include dedicated channel syntax, macros, nominal-record shorthand, newtypes, runtime
temporal contracts, or distributed processes.

Monad laws are normative and may be property-tested; generic methods remain ambient-effect
polymorphic rather than introducing a separate computation tower.

## P3+ boundary

P3+ is a fresh-design space for:

- hygienic expression macros and library-provided surface conveniences;
- notation as aliases to functions, including infix, postfix, mixfix, and sections;
- optional explicit perform syntax, named handlers, shallow handlers, and multi-shot
  continuations;
- runtime temporal contracts;
- informational algebraic metadata, property/law-derived tests, Ash compile/link-time proofs, and
  foreign SMT/proof-assistant integration;
- higher constructors such as (Type -> Type) -> Type;
- serialization and communication across applications or daemon nodes;
- distributed actors and other distributed-runtime facilities.

Deferred work creates no P1 syntax reservation or representation compatibility requirement unless
SPEC-104 is explicitly amended.

The long-term evidence ladder is informational declarations, derived property tests, Ash proofs,
and foreign proofs. Compiler decisions must record the evidence strength they require. A project
or compiler mode may deliberately accept weaker evidence, but must not report it as a stronger
proof class. Ash proofs are compile/link-time only, may not terminate, and receive only bounded
high-level analysis.

## Removal and deferral boundary

### 7.1 Removed

The following are removed from target authority and receive no compatibility path:

- workflow declarations and the Act, Proc, and Workflow carrier tower;
- dedicated role, policy, capability, resource, channel, and heterogeneous row-item language
  families; providers and nominal algebraic effects cover retained host interaction;
- legacy interface-as-effect, default-handler, and derived-handler machinery;
- dedicated lazy, memo, and force modes. Delay is an ordinary () -> A function; memoization is an
  explicit library or provider concern. A future proposal starts fresh;
- typed do blocks and comprehension syntax. Option and Result remain useful ordinary types, and
  P2 Monad functionality uses ordinary functions;
- tuple numeric projection, one-tuples, the empty record, dedicated channel syntax, source builtin
  declarations, implicit prelude functions, automatic provider authority, ambient mailbox/self
  lookup, first-class provider handles, and environment lookup by Ash code;
- dedicated async/generator syntax, exceptions, fail/with_error, implicit truthiness, loops,
  destructors, defer, general casts, function overloading, and implicit string conversion;
- the existing requires, ensures, check obligation, law, proof, and runtime
  contract/evidence syntax. Any later contract/evidence surface is a P3+ fresh design;
- migration aliases or compatibility evaluation paths for any removed form.

An implemented removed feature is deletion work, not a reason to restore it to the target.

### 7.2 Deferred, not removed

The following are absent from P1 and P2 but may be reconsidered by a fresh P3+ proposal: newtypes,
general Iterator/lazy adapters, comprehensions as library/macro sugar, ranges, slices, general
indexing, nominal-record shorthand, record/list patterns, runtime temporal monitors, and
distributed actors.

Links, supervision, typed process exits, alternative mailbox policies, and dedicated service
lifecycle are P2 as stated in §5.

Macro and notation declarations and invocations are disabled in a conforming P1/P2 compiler and
must reject before semantic admission. Existing parser or summary support is deletion or
feature-branch evidence only; it is not accepted-but-unrunnable P1 syntax.

## Realization policy

PLAN-203 remains the executable-realization programme after its tasks are narrowed to this freeze.
Work proceeds as vertical semantic slices:

~~~text
frozen feature -> canonical rule -> typing -> Core -> CPS -> admission -> runtime
-> positive/negative/mutation tests -> CLI/daemon parity
~~~

Each slice must name explicit non-goals and delete superseded paths in the same programme where
practical. Work on P2/P3+ or removed features cannot be used to close a P1 slice.

The companion
[AUDIT-208](../plan/audits/AUDIT-208-language-scope-dispositions.md) records the repository
conflicts and cleanup implications behind this freeze. It is evidence, not authority.
