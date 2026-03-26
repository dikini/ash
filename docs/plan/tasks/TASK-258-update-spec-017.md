# TASK-258: Update SPEC-017 with Constraint Syntax

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Update SPEC-017 to include constraint refinement syntax from SPEC-024.

**Spec Reference:** SPEC-017 (Capability Integration), SPEC-024

**File Locations:**
- Modify: `docs/spec/SPEC-017-CAPABILITY.md`

---

## Background

SPEC-017 defines capability integration. It needs updating to include:
- `@ { constraints }` syntax
- Constraint semantics
- Relationship to SPEC-024

---

## Step 1: Review Current SPEC-017

```bash
head -100 docs/spec/SPEC-017-CAPABILITY.md
```

Identify sections needing updates.

---

## Step 2: Add Constraint Section

Add to SPEC-017:

```markdown
## 5. Constraint Refinement

Capabilities can be refined with constraints at declaration site:

```ash
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}

-- Usage with constraints
workflow processor capabilities: [
    file @ { 
        paths: ["/var/log/*", "/tmp/llm-*"],
        read: true,
        write: false
    }
]
```

### 5.1 Constraint Semantics

Constraints narrow the capability grant:
- `paths`: Limits accessible file paths
- `permissions`: Narrows available operations
- Additional fields depend on capability type

### 5.2 Constraint Checking

Constraints are checked:
1. **Statically:** At compile time when possible
2. **Dynamically:** At runtime for dynamic values

### 5.3 Relationship to Roles

Roles bundle capabilities with constraints:
```ash
role log_processor {
    capabilities: [
        file @ { paths: ["/var/log/*"], read: true }
    ]
}
```

See SPEC-024 for full syntax specification.
```

---

## Step 3: Update References

Add to SPEC-017:

```markdown
## References

- SPEC-024: Capability-Role-Workflow Syntax (Reduced)
```

---

## Step 4: Verify Consistency

Check SPEC-017 and SPEC-024 are consistent:

```bash
diff <(grep "capability.*@" docs/spec/SPEC-017-CAPABILITY.md) <(grep "capability.*@" docs/spec/SPEC-024-CAPABILITY-ROLE-REDUCED.md)
```

---

## Step 5: Commit

```bash
git add docs/spec/SPEC-017-CAPABILITY.md
git commit -m "docs: update SPEC-017 with constraint syntax (TASK-258)

- Add section 5: Constraint Refinement
- Document @ { constraints } syntax
- Explain constraint semantics
- Describe static and dynamic checking
- Add reference to SPEC-024
- Ensure consistency with reduced syntax spec"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-258 SPEC-017 update"
  context: |
    Files to verify:
    - docs/spec/SPEC-017-CAPABILITY.md
    
    Requirements:
    1. Constraint syntax documented
    2. Semantics explained clearly
    3. Consistent with SPEC-024
    4. Examples valid
    5. References added
    
    Run and report:
    1. Read constraint section
    2. Check consistency with SPEC-024
    3. Verify examples are valid syntax
    4. Check all references exist
    5. cargo doc --no-deps
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] SPEC-017 reviewed
- [ ] Constraint section added
- [ ] Semantics documented
- [ ] Examples included
- [ ] References updated
- [ ] Consistency verified
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 4
**Blocked by:** TASK-257
**Blocks:** Phase 46 implementation tasks
