# TASK-255: Update Stale Documentation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Refresh `README.md`, `docs/API.md`, and `docs/spec/README.md` to match current implementation.

**Spec Reference:** Project documentation standards

**File Locations:**
- `README.md:45`
- `docs/API.md:52`
- `docs/spec/README.md:9`

---

## Background

The audit found stale documentation:

1. **README.md:45** - Points to `examples/multi_agent.ash` which doesn't exist
2. **docs/API.md:52** - Contains incorrect sample code (`pubuse provenance::*;`)
3. **docs/spec/README.md:9** - Maps old spec IDs (SPEC-002, SPEC-004, SPEC-005) that don't match filenames

---

## Step 1: Audit All References

```bash
# Check README examples
grep -n "examples/" README.md

# Check API.md code samples
grep -n "```" docs/API.md

# Check spec index
cat docs/spec/README.md
```

---

## Step 2: Fix README.md

### Fix Broken Example Reference

```markdown
<!-- README.md -->
## Examples

See the `examples/` directory for sample Ash workflows:

- `examples/hello.ash` - Basic workflow
- `examples/capabilities.ash` - Capability demonstration
<!-- Remove or fix: multi_agent.ash -->
```

### Update Installation Instructions (if stale)

```bash
grep -n "cargo install\|brew install\|apt" README.md
```

### Update Quick Start

Ensure quick start actually works:

```bash
cargo build --release
./target/release/ash --version
./target/release/ash run examples/hello.ash
```

---

## Step 3: Fix docs/API.md

### Fix Code Samples

```rust
// Before (broken):
pubuse provenance::*;

// After (fixed):
pub use ash_engine::provenance::*;
```

### Update API Examples

Check all code blocks compile:

```rust
use ash_engine::Engine;

let engine = Engine::builder()
    .with_http_capabilities(HttpConfig::default())
    .build()?;
```

---

## Step 4: Fix docs/spec/README.md

Update spec index to match actual filenames:

```markdown
<!-- docs/spec/README.md -->
# Specification Index

| Spec | Title | Status |
|------|-------|--------|
| SPEC-001-IR.md | Intermediate Representation | Canonical |
| SPEC-002-SURFACE.md | Surface Syntax | Canonical |
| SPEC-003-TYPE.md | Type System | Canonical |
| SPEC-004-RUNTIME.md | Runtime Semantics | Canonical |
| SPEC-005-CLI.md | CLI Interface | Canonical |
| ... | ... | ... |
```

Ensure the IDs match filenames exactly.

---

## Step 5: Check All Links

```bash
# Check internal links
grep -rn "\.md)" docs/ | grep -v "http"

# Verify each link target exists
for link in $(grep -oP '\[.*?\]\(\K[^)]+\.md' README.md docs/*.md); do
    if [ ! -f "$link" ]; then
        echo "Broken link: $link"
    fi
done
```

---

## Step 6: Commit

```bash
git add README.md
git add docs/API.md
git add docs/spec/README.md
git commit -m "docs: update stale documentation (TASK-255)

- Fix broken example reference in README.md
- Correct code samples in API.md
- Update spec index to match actual filenames
- Verify quick start instructions work
- Fix typos and broken links

Files updated:
- README.md
- docs/API.md
- docs/spec/README.md"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-255 implementation"
  context: |
    Files to verify:
    - README.md
    - docs/API.md
    - docs/spec/README.md
    
    Requirements:
    1. All example references point to existing files
    2. All code samples are syntactically valid
    3. Spec index matches actual filenames
    4. Quick start instructions work
    5. No broken internal links
    6. Documentation matches current implementation
    
    Run and report:
    1. Verify examples exist: ls examples/
    2. Check code samples compile (extract and test)
    3. cargo build --release (verify quick start)
    4. Check all .md links have valid targets
    5. Read through for obvious errors
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] All stale references identified
- [ ] README.md fixed
- [ ] docs/API.md fixed
- [ ] docs/spec/README.md fixed
- [ ] Broken links checked
- [ ] Code samples verified
- [ ] Quick start tested
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** None
**Blocks:** None
