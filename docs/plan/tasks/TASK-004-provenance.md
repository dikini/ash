# TASK-004: Provenance and Trace Types

## Status: ✅ Complete

## Description

Implement provenance tracking and trace event types for audit trails.

## Specification Reference

- SPEC-001: IR - Section 3 Provenance

## Requirements

### Functional Requirements

1. `WorkflowId` newtype around UUID
   - Generate new IDs
   - Display/Debug formatting

2. `Provenance` struct
   - `workflow_id: WorkflowId`
   - `parent: Option<WorkflowId>`
   - `lineage: Box<[WorkflowId]>`
   - Methods: `new()`, `fork()`

3. `TraceEvent` struct/enum
   - `event_type: EventType`
   - `timestamp: DateTime<Utc>`
   - `workflow_id: WorkflowId`
   - `details: EventDetails`

4. `EventType` enum
   - Observation, Orientation, Decision, Proposal, Action, ObligationCheck, Error

5. `EventDetails` - details for each event type

### Property Requirements (proptest)

```rust
// Lineage integrity
// - Fork increases lineage by 1
// - Parent relationship is correct

// Event ordering
// - Timestamps are monotonic within a workflow
```

## TDD Steps

### Step 1: Implement WorkflowId (Green)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for WorkflowId {
    fn default() -> Self { Self::new() }
}
```

### Step 2: Implement Provenance (Green)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub workflow_id: WorkflowId,
    pub parent: Option<WorkflowId>,
    pub lineage: Box<[WorkflowId]>,
}

impl Provenance {
    pub fn new() -> Self { ... }
    pub fn fork(&self) -> Self { ... }
}
```

### Step 3: Implement TraceEvent (Green)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub workflow_id: WorkflowId,
    pub details: EventDetails,
}
```

### Step 4: Add Tests (Green)

- Provenance fork test
- Lineage integrity test
- Serialization roundtrip

### Step 5: Refactor (Refactor)

- Review Box usage for lineage
- Ensure efficient cloning

## Completion Checklist

- [ ] WorkflowId newtype with UUID
- [ ] Provenance struct with lineage
- [ ] TraceEvent and EventType
- [ ] EventDetails for each event type
- [ ] Fork lineage tests
- [ ] Serialization roundtrip tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-001 (WorkflowId uses UUID pattern)

## Blocked By

Nothing

## Blocks

- TASK-003 (Act variant uses Provenance)
- TASK-038 through TASK-041 (Provenance system)
- All interpreter tasks (trace recording)
