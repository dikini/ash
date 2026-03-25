# Design Note: Quorum Patterns for Collective Decision-Making

**Status:** Preliminary Design  
**Scope:** Future Library Component  
**Related:** SPEC-023 Proxy Workflows, TASK-239

---

## 1. Problem Summary

Distributed systems and multi-party workflows often require **collective decision-making** where multiple participants must agree before an action is taken. Examples include:

- **Board approvals**: 3 of 5 members must approve a decision
- **Multi-sig wallets**: M-of-N signatures required to spend funds
- **Distributed consensus**: Majority of nodes must agree
- **Byzantine fault tolerance**: 2f+1 nodes in a 3f+1 system

Currently, Ash supports this via manual vote counting (see SPEC-023 Section 5.2 example), but a **library abstraction** could provide:
- Type-safe quorum specifications
- Reusable vote collection patterns
- Timeout and failure handling
- Composable decision strategies

---

## 2. Possible Quorum Specifications

### 2.1 Core Quorum Types

```rust
/// Quorum specification for collective decision-making
pub enum QuorumSpec {
    /// All participants must approve
    Unanimous,
    
    /// More than half must approve (> 50%)
    Majority,
    
    /// At least N participants must approve
    AtLeast(u32),
    
    /// At least P% must approve (0-100)
    Percentage(u8),
    
    /// Weighted quorum: sum of approval weights must meet threshold
    Weighted { threshold: u64 },
    
    /// M-of-N fixed quorum (e.g., 3-of-5 multisig)
    Fixed { required: u32, total: u32 },
    
    /// Custom predicate over vote set
    Custom(Box<dyn Fn(&VoteSet) -> bool>),
}
```

### 2.2 Decision Types

```rust
/// Individual participant decision
pub enum Decision {
    Approve,
    Reject,
    Abstain,  // Doesn't count for or against
}

/// Collective quorum result
pub enum QuorumResult {
    Approved,
    Rejected { reason: RejectionReason },
    Inconclusive,  // Not yet determined
}

pub enum RejectionReason {
    /// Too many rejections to possibly meet quorum
    Impossible,
    /// Timeout waiting for votes
    Timeout,
    /// Custom predicate failed
    Custom(String),
}
```

---

## 3. Examples

### 3.1 Simple Majority Vote

```ash
-- Define quorum policy
let approval_policy = quorum::majority();

-- Collect votes from committee
let result = par {
    yield role(member1) proposal resume v1 => { v1 },
    yield role(member2) proposal resume v2 => { v2 },
    yield role(member3) proposal resume v3 => { v3 }
} collect votes
|> quorum::evaluate(approval_policy);

match result {
    Approved => resume Accepted : Response,
    Rejected(_) => resume Rejected : Response
}
```

### 3.2 Board Approval (3-of-5)

```ash
let board_quorum = quorum::fixed(required: 3, total: 5);

-- With timeout
let result = quorum::collect_votes(
    roles: [director1, director2, director3, director4, director5],
    proposal: merger_proposal,
    spec: board_quorum,
    timeout: 7d
);
```

### 3.3 Weighted Token Voting

```rust
// Each voter has token-weighted influence
let spec = QuorumSpec::Weighted { threshold: 10000 };

let vote = Vote {
    voter: "alice".to_string(),
    decision: Decision::Approve,
    weight: 5000,  // Alice holds 5000 tokens
};
```

### 3.4 Nested Quorum (Geographic + Functional)

```ash
-- Need majority from each region AND majority from each function
let regional = quorum::majority();
let functional = quorum::majority();

let combined = quorum::all_of([
    quorum::by_group(votes, group_by: region, spec: regional),
    quorum::by_group(votes, group_by: function, spec: functional)
]);
```

---

## 4. Design Directions

### 4.1 Integration Patterns

| Approach | Pros | Cons |
|----------|------|------|
| **Pure library** (no lang changes) | Simple, flexible | Verbose syntax |
| **Macro/quasi-quote** | Cleaner syntax | More complex impl |
| **Native `par ... collect` extension** | Natural fit with Ash | Tight coupling |

**Recommendation:** Start as pure library, evaluate native extension later.

### 4.2 Key Design Questions

1. **Async vs Sync**: Should vote collection be async (with timeout) or sync blocking?
   - *Lean toward async with explicit timeout handling*

2. **Partial results**: Expose intermediate vote counts or only final result?
   - *Expose via stream/observer pattern for transparency*

3. **Vote privacy**: Support private (encrypted) votes or only public?
   - *Start public; private votes need crypto primitives*

4. **Revocation**: Can votes be changed before quorum met?
   - *Yes, with linear obligation semantics*

5. **Composability**: How to combine multiple quorum rules?
   - *Use `all_of`, `any_of`, `at_least_n` combinators*

### 4.3 Type Safety Considerations

```rust
/// Phantom type tracking vote collection state
pub struct VoteCollector<S: VoteState> {
    votes: Vec<Vote>,
    spec: QuorumSpec,
    _state: PhantomData<S>,
}

pub trait VoteState {}
pub struct Collecting; impl VoteState for Collecting {}
pub struct Complete; impl VoteState for Complete {}

// Type-safe transitions only
impl VoteCollector<Collecting> {
    pub fn add_vote(self, vote: Vote) -> Self { ... }
    pub fn finalize(self) -> VoteCollector<Complete> { ... }
}

impl VoteCollector<Complete> {
    pub fn result(&self) -> QuorumResult { ... }
}
```

---

## 5. Implementation Sketch

```rust
/// Core quorum library API
pub mod quorum {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    
    /// Builder for vote collection
    pub struct VoteCollector {
        spec: QuorumSpec,
        votes: Vec<Vote>,
        deadline: Option<Instant>,
    }
    
    impl VoteCollector {
        pub fn new(spec: QuorumSpec) -> Self { ... }
        pub fn with_timeout(mut self, duration: Duration) -> Self { ... }
        pub fn add_vote(&mut self, vote: Vote) -> &mut Self { ... }
        
        /// Check current status without consuming
        pub fn check(&self) -> QuorumStatus { ... }
        
        /// Finalize and get result
        pub fn finalize(self) -> Result<QuorumResult, QuorumError> { ... }
    }
    
    /// Async vote collection from proxy workflows
    pub async fn collect_from_roles(
        roles: &[Role],
        proposal: Value,
        spec: QuorumSpec,
        timeout: Duration,
    ) -> Result<QuorumResult, QuorumError> {
        // Implementation using yield/resume
    }
    
    /// Convenience constructors
    pub fn unanimous() -> QuorumSpec { QuorumSpec::Unanimous }
    pub fn majority() -> QuorumSpec { QuorumSpec::Majority }
    pub fn at_least(n: u32) -> QuorumSpec { QuorumSpec::AtLeast(n) }
    pub fn percentage(p: u8) -> QuorumSpec { QuorumSpec::Percentage(p) }
    pub fn fixed(required: u32, total: u32) -> QuorumSpec { 
        QuorumSpec::Fixed { required, total } 
    }
}
```

---

## 6. Open Questions

1. **Should quorum be effect-polymorphic?** (e.g., `Quorum<E: Effect>`)
2. **Integration with capability system**: Does voting require special capabilities?
3. **Audit trail**: How to log quorum decisions for compliance?
4. **Conflict resolution**: Tie-breaking rules for split votes?
5. **Performance**: Efficient vote aggregation for large groups (1000+ voters)?

---

## 7. Related Work

- **Raft/Paxos**: Consensus algorithms for distributed systems
- **Multisig wallets**: M-of-N cryptography (Bitcoin, Ethereum)
- **Board governance**: Robert's Rules of Order procedures
- **Liquid democracy**: Delegative voting systems

---

## 8. Next Steps

1. **Validate design** with concrete use cases from Ash workflows
2. **Prototype** pure library implementation (no language changes)
3. **Test** with real proxy workflow scenarios
4. **Evaluate** whether language integration is justified
5. **Specify** formally if adopted into core Ash

---

*This is a preliminary design note. Implementation should wait until concrete use cases emerge from proxy workflow adoption.*
