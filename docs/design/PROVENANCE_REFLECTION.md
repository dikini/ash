# Proposal: Provenance Reflection and Actor Projections

**Status**: Proposal / Research
**Collaborators**: User & Gemini CLI
**Date**: March 2026

## Overview

This document describes a design for **Provenance Reflection**, a mechanism that allows a running Ash workflow to query its own history. This capability, combined with **Actor-Specific Projections**, bridges the gap between low-level execution data and high-level reasoning for humans, AI agents, and auditors.

## 1. TraceEvent Extensions

To support semantic reflection, we extend the `TraceEvent` enum in `ash-core` to include high-level summary events.

### The `Milestone` Event
A milestone summarizes a logical phase of the workflow.

```rust
pub enum TraceEvent {
    // ... existing variants (Obs, Orient, Decide, Act, Oblig) ...
    
    /// A high-level semantic checkpoint
    Milestone {
        name: String,
        status: MilestoneStatus,
        timestamp: DateTime<Utc>,
        metadata: Vec<(String, bool)>, 
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Reached,   // All internal checks passed
    Failed,    // Attempted but conditions not met
    Pending,   // Requirement not yet satisfied
}
```

## 2. Reflexivity: The `reflect` Capability

We propose a built-in `reflect` effect. This allows the workflow code to query its own `Provenance` trace without creating a recording recursion (reading the trace does not necessarily record a primary operational event, or it records a specific `Reflect` event).

### Ash Syntax Example
```ash
-- A workflow checks its own progress to decide on its next action
let milestones = reflect history(filter: milestone);
if "SECURITY_REVIEW_DONE" in milestones then {
    act dispatch_keys;
}
```

## 3. Actor-Specific Projections

Provenance data is often too verbose for direct consumption. The `ash-provenance` crate should provide a **Projection Layer** that transforms raw events into formats optimized for different types of actors.

### A. AI Narrative Projection (The "Story")
Transforms the trace into a structured JSON "story" for LLM context windows. It filters for `Milestone`, `Decide` (with results), and `Act` events, while eliding low-level data.

```json
{
  "workflow_id": "...",
  "current_status": "Milestone: KYC_COMPLETED reached.",
  "narrative": [
    "Observed user identity document.",
    "Decided to permit verification based on policy 'id_match'.",
    "Reached milestone: KYC_COMPLETED."
  ]
}
```

### B. Human Timeline Projection (The "Progress Bar")
Summarizes the trace into a linear sequence of milestones for UI rendering.

`[✓] Input Received -> [✓] Security Check -> [●] Manager Approval Pending -> [ ] Execution`

### C. Auditor Full Trace (The "Merkle-Proof")
The complete, verbatim sequence of every event, including low-level `Obs` and `Orient` operations, cryptographically linked to ensure integrity.

## 4. Proposed Architecture

The implementation would be distributed across three components:

1.  **`TraceStore` (ash-provenance)**: A queryable interface for the historical record of a `WorkflowId`.
2.  **`ProjectionProvider` (ash-provenance)**: A trait for implementing custom summarizers (Milestone-only, full-audit, AI-narrative).
3.  **`Reflector` (ash-interp)**: A capability provider that allows the interpreter to inject parts of the `TraceStore` back into the workflow execution context.

## 5. Security and Governance

Reflection and projection are themselves **Governed Capabilities**. 
- A policy can restrict *which* roles are allowed to see *which* projections (e.g., "The `customer` role can only see the `Human Timeline`, not the `Full Trace`").
- Redaction can be applied at the projection level to hide sensitive data (secrets, PII) from the "Narrative" projection.

## Conclusion

By enabling a workflow to reflect on its own history and providing actor-specific views of that history, Ash moves beyond a "black-box" execution engine and becomes a transparent, "self-aware" governance framework.
