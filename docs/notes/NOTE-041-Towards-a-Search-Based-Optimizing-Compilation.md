# Towards a Search-Based Optimizing Compiler for Algebraic-Effect Languages

## Abstract

This note explores a compiler architecture for a language with algebraic effects, row polymorphism, and CPS-based execution. The central idea is to move beyond a fixed sequence of compiler passes and instead treat optimization as a guided search through a space of semantics-preserving program representations.

The compiler maintains multiple useful views of a program:

- an effect-oriented representation (Freer/algebraic-effect style),
- a continuation-oriented representation (CPS),
- lower-level execution representations.

Transformations between these views are themselves optimization moves. A cost model evaluates candidate trajectories, and machine learning can learn which transformations are likely to improve a program for a given objective.

The long-term vision is a compiler that can develop specialized optimization personalities for different domains: AOT, JIT, embedded systems, databases, web services, security hardening, and application-specific workloads.

---

# 1. Starting Point: Algebraic Effects and Row Types

A modern effect-oriented language can expose computations such as:

```
perform Read
perform Write
perform State.get
perform Exception.raise
```

with effect typing:

```
A -> B ! {State, IO, Exception | r}
```

The row describes the possible effects of a computation.

Rows provide:

- effect polymorphism,
- modular composition,
- handler reasoning,
- static capability information.

They answer:

> "What effects may occur?"

They do not answer:

> "How should the computation execute?"

For execution, the compiler needs another representation.

---

# 2. Effect Representation: Freer / Algebraic Form

A Freer-like representation models a computation as:

```
Pure value

or

Effect operation + continuation
```

Conceptually:

```
Op
 |
 +-- continuation
        |
        +-- next operation
```

Example:

```
Read
 |
 +-- Write
       |
       +-- Return
```

Advantages:

- effect structure is explicit,
- handlers are easy to transform,
- operations can be fused or eliminated,
- multiple interpreters are possible,
- semantic optimization is natural.

Typical transformations:

- handler fusion,
- effect elimination,
- row simplification,
- effect specialization,
- commuting independent effects.

Disadvantage:

Freer-like structures are not ideal as final execution representations.

They can introduce:

- allocation overhead,
- indirect calls,
- poor locality.

---

# 3. CPS Representation

CPS makes continuation flow explicit:

Normal form:

```
let x = f()
g(x)
```

becomes:

```
f(k)
```

where:

```
k = what happens next
```

For effects:

```
perform Read
```

becomes:

```
perform Read continuation
```

The relationship:

Freer:

```
Op continuation
```

CPS:

```
operation invokes continuation
```

They represent the same fundamental structure, but differently.

Freer:

> continuation as data

CPS:

> continuation as control flow

---

# 4. Why Both Representations Matter

Different transformations are naturally expressed in different representations.

## Effect/Freer-oriented optimizations

Better for:

- handler fusion,
- capability reduction,
- effect elimination,
- reasoning about algebraic laws,
- effect specialization.

Example:

Before:

```
State.get >>= State.set
```

After:

```
State.modify
```

---

## CPS-oriented optimizations

Better for:

- inlining,
- tail-call optimization,
- closure elimination,
- continuation shrinking,
- control-flow simplification.

Example:

Before:

```
continuation closure
```

After:

```
direct jump
```

---

# 5. Alternating Optimization Rather Than Fixed Pipeline

Traditional compiler:

```
Source
 |
 v
IR1
 |
 v
IR2
 |
 v
Machine
```

Proposed approach:

```
             Effect IR

                 |
        optimize effects

                 |
                 v

              CPS IR

                 |
        optimize control

                 |
                 v

          reify effects

                 |
                 v

              repeat
```

The transformations themselves become optimization moves.

The compiler searches:

```
P0 -> P1 -> P2 -> ... -> Pn
```

where each step reduces a cost function.

---

# 6. Optimization as Search in a Cost Space

A program state can be described by a cost vector:

```
C =
(
 effect complexity,
 CPS complexity,
 memory cost,
 operation count,
 code size,
 latency,
 energy
)
```

A transformation:

```
T(P) = P'
```

is evaluated by:

```
ΔC = C(P') - C(P)
```

The optimizer selects transformations that improve the objective.

A simple model:

\[
V^2 =
w_e eff^2 +
w_c cps^2 +
w_m mem^2 +
w_o ops^2
\]

The optimizer follows trajectories:

```
P0
 |
 v
P1
 |
 v
P2
```

until no useful descent remains.

---

# 7. Multiple Optimization Personalities

The same transformation system can support different objectives.

A personality is primarily a cost function.

## AOT personality

Priorities:

- predictable execution,
- small binary,
- compile-time efficiency.

Weights:

```
code size
memory
runtime
compile cost
```

---

## JIT personality

Priorities:

- hot-path performance,
- runtime specialization.

Weights:

```
runtime
allocation
latency
```

---

## Embedded / IoT personality

Priorities:

- memory,
- energy,
- code size.

---

## Server personality

Priorities:

- throughput,
- latency,
- scalability.

---

The compiler becomes:

```
Transformation engine
        +
Objective function
        +
Optimization policy
```

rather than separate compilers.

---

# 8. Learning the Optimization Policy

The compiler transformations remain verified.

Machine learning does not invent arbitrary programs.

Instead:

The ML model learns:

> Given this program state and objective, which legal transformation is likely to improve the cost?

State:

```
IR graph
effect rows
CPS structure
profile data
current cost
target personality
```

Actions:

```
inline continuation
fuse handler
lower effect
specialize
reify CPS
```

Reward:

```
improvement in cost vector
```

---

# 9. Suitable ML Architectures

## Graph Neural Networks

Most natural starting point.

Programs are graphs:

- control flow,
- data flow,
- effect flow.

Node features:

```
operation
type
effect row
handler depth
continuation properties
```

Output:

```
probability of useful transformations
```

---

## Graph Transformers

Useful for large programs where distant relationships matter.

Examples:

- handlers far from use sites,
- escaping continuations,
- global effect patterns.

---

## Reinforcement Learning

Natural formulation:

State:

```
program + profile + cost
```

Actions:

```
transformations
```

Reward:

```
cost reduction
```

Possible methods:

- PPO,
- policy gradients,
- Monte Carlo Tree Search.

---

## Search + Learned Policy

A practical architecture:

```
                 Program

                    |

            Feature extraction

                    |

          +---------+---------+

          |                   |

     Cost model          ML policy

          |                   |

          +---------+---------+

                    |

            Transformation search

                    |

             Better program
```

The ML model guides the search; it does not replace compiler correctness.

---

# 10. Training Data

Training data can be generated automatically.

Sources:

## Synthetic programs

Generate:

- deep handler nesting,
- effect combinations,
- continuation patterns,
- database-like workloads,
- service-like workloads.

---

## Standard library

Libraries provide realistic idioms.

Examples:

Database library:

```
Query
Transaction
Cache
Serialization
```

Web library:

```
Request
Authentication
Routing
Response
```

The optimizer learns common patterns.

---

## Execution traces

Runtime data provides:

- hot paths,
- allocation behaviour,
- latency,
- memory pressure,
- failures.

Training records:

```
program state
chosen transformation
resulting cost change
runtime outcome
```

---

# 11. Domain and Program Specialization

The same framework can learn specialized optimization strategies.

Examples:

## Database systems

Learn:

- query fusion,
- batching,
- transaction specialization,
- caching.

---

## Web services

Learn:

- middleware fusion,
- serialization optimization,
- request pipeline specialization.

---

## Data processing

Learn:

- stream fusion,
- parallel execution,
- scheduling.

---

The optimizer becomes aware of program classes.

---

# 12. Security and Resilience Optimization

The objective function need not be performance.

Security can define:

```
risk =
 capability exposure
 attack surface
 failure behaviour
 resource exhaustion
```

The optimizer searches for safer programs.

Examples:

Effect analysis:

Before:

```
{Network, FileSystem, Database}
```

After:

```
{Database}
```

The capability surface is reduced.

---

## Adversarial optimization

The environment can simulate:

- memory exhaustion,
- malformed inputs,
- network failure,
- dependency failure,
- resource starvation.

Reward:

```
availability
recovery time
fault containment
```

The optimizer learns resilient designs.

---

# 13. Long-Term Architecture

A possible architecture:

```
                 Source Language

                       |

             Typed Effect Core

                       |

             Effect/CPS IR system

                       |

        +--------------+--------------+

        |                             |

 Effect transformations       CPS transformations


        |                             |

        +--------------+--------------+

                       |

              Optimization Search

                       |

            Learned Policy Engine

                       |

               Target Backend

                    MLIR

                      |

                   LLVM
```

---

# Conclusion

The central idea is to treat compilation as navigation through a semantic transformation landscape.

Algebraic effects provide a rich representation of intent.

CPS provides an efficient representation of execution.

A cost model provides a notion of improvement.

Machine learning provides a strategy for exploring the transformation space.

The resulting compiler is not a fixed pipeline but an adaptive optimizer that can learn different strategies for different goals:

- faster programs,
- smaller programs,
- safer programs,
- lower-energy programs,
- domain-specialized programs.

The key engineering principle remains:

**Transformations must preserve semantics; learning chooses the path.**
