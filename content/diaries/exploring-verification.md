---
title: "Exploring verification in Ash"
description: "An introduction to propositions, evidence, and compile-time proof construction in Ash's emerging verification design."
author: "Vlado"
author_uri: "https://github.com/dikini"
draft: false
published: "2026-10-23"
updated: "2026-10-23"
topics:
  - verification
  - language-design
section: "Developer Diaries"
route: "diaries/exploring-verification"
---

<!-- Imported from https://github.com/dikini/ash-blog/blob/a534f9d9abe0b63ba17896e0e729df8acdfcbd0f/entries/exploring-verification/index.md; publication metadata set for ash-site. -->

Ash grew out of the conviction that program verification is central for future development when the team consists of both humans and AI. Agents are far more productive, in raw terms than humans, but often produce churn. For various reasons, ranging from innate capabilities and knowledge through misinterpreted specification, to incomplete or wrongfull specs. One of the avenues to improve the results is verification - not just QA of the resulting code, but of the specs themselves.

Traditionally this has lead to type driven design and sufficiently strong type systems. Another, complementary avenue is proof assistants and dependntly typed languages. They are capable of expressing most of the problems, but, from experience, they make life difficult in  how you express, prove, check. If you want to solve real worlf problems. They are focused on proofs, not so much on execution of computations.

We also have examples like rust. Practical languages, whose type system is strong enough to express and verify the correctness of real-life concerns, not just the fabled memory safety.

So how do we do it?

There is no single good answer. At the moment ash is being developed to reach a stage where we can play with these concepts. It has some basic primitive types, ADTs, parametric and ad-hoc polymorphism whic help with the expressiveness of the language. It has effect and resource typing, with the intent to open up the visibility into the hidden world of the machine, without descending into assembly, but lifting them up a notch. The aim is to make machine behaviour visible enough to reason about. These gives a basis from which to speculate, explore, play and eventually settle on a design.

To start exploring, we can take one small claim about a program and ask what it would mean to verify it.

Suppose an operation has the right type for combining two values. What tells us that we can regroup a sequence of those operations without changing the result?

The type alone may leave that question unanswered. What properties should the program have, do they hold? For example - associativity. And once we have stated it, we need to ask what supports it. Did we assume it? Test it on some inputs? Prove it under a particular set of assumptions?

This gives us three things to keep track of: the program, the claim about it, and the evidence supporting that claim.

## Stating a claim

For an operation `combine`, associativity says:

```text
combine(combine(x, y), z) = combine(x, combine(y, z))
```

The claim is that this equality holds for every permitted choice of `x`, `y`, and `z`.

We can call this a property of the operation. When it forms part of the behaviour expected from an abstraction, we often call it a law. Its logical content is a proposition: a statement for which we can ask for justification.

Other claims play different roles. A contract can describe what a function requires and what it guarantees. An invariant can describe a condition preserved at particular boundaries in a program. We want to understand how these roles fit together without making each one an unrelated verification mechanism.

Even this small example has details to settle. What values belong to the domain? What does equality mean for them? Can the operation fail or perform effects? A useful verification design needs to keep the meaning of the claim connected to the program it concerns.

## What supports the claim?

Writing down associativity does not establish it. Giving the statement a name such as `law` does not establish it either.

A test can check the equation for selected inputs. More tests can explore more cases and find counterexamples we would otherwise miss. But a successful collection of tests does not, by itself, establish a statement about every input in an unbounded domain.

A proof addresses the general statement under its premises. An assumption makes the statement available as a premise without supplying that derivation. These are different ways a claim can enter our reasoning, and we want those differences to remain visible.

An early intuition was to arrange axioms, tests, and proofs along a tower of increasing assurance. That raises useful questions, but a single scale leaves too much out. A proof may depend on an assumption about an external component. Two test suites may cover different cases. Evidence needs a scope and a context before we can sensibly compare it.

We need to distinguish the proposition from its evidence and from the assumptions that evidence depends on.

## Ordinary code at compile time

Could we use ordinary Ash code to construct the evidence? We already have functions, abstractions, algorithms, and recursion. What if we could use them during compilation to search for proofs and put them together?

But successfully running that code tells us only that it ran. Did it actually prove anything? If yes, how do we use the proofs? How do we trust the proofs or their conclusions?

That gives us a few things to untangle:

- **Expression:** How we write the computation, using ordinary Ash syntax and semantics.
- **Execution:** Where it runs, in a compile-time environment with restricted capabilities.
- **Certification:** How we determine whether the evidence it produces establishes the proposition.

We can separate these concerns without requiring three different languages. Certification addresses whether the result establishes the claim. How that checked result can be used elsewhere in the program is a further design question, including which assumptions must travel with it.

A search can terminate with an invalid candidate. A constructor can contain a bug. The validation process must judge the evidence against the claim; the constructor's successful execution cannot stand in for that judgment. This is the trust boundary we want to preserve.

Sandboxing addresses a different concern. Compile-time code needs an appropriate execution environment, with limits on what it can access and do. Those restrictions protect the build environment. They do not establish the logical correctness of whatever the code produces.

## How does this relate to types?

Verification naturally brings us back to the type system. Types already let us express and check constraints on programs. Richer type systems can express propositions and represent proofs as terms inhabiting them. That raises a design question for Ash: how much machinery should typing and verification share, and where do they need different rules?

Should types and propositions be distinguished within a common framework? Or should verification have a more separate logical layer?

Running proof construction at compile time does not decide that question. A system based on propositions as types can also check proofs during compilation. The distinction we have established concerns the computation that constructs evidence and the judgment that accepts it. Several logical architectures could support that distinction.

## A foundation to work from

The direction is to state propositions, associate obligations with them, and explore ordinary sandboxed Ash computation as a way to produce evidence. Validation determines whether that evidence establishes the claim.

That leaves concrete questions: which expressions can form propositions? How does their meaning relate to Ash programs? What does evidence look like, and which assumptions can validation use?

A `proof` declaration could associate an obligation with a compile-time computation that attempts to discharge it. In that arrangement, we would ask for a claim to be established and supply code to work on it, without treating `proof` as a special kind of function.
