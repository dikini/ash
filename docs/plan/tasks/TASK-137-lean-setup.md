# TASK-137: Lean 4 Toolchain and Project Setup

## Status: ✅ Complete

## Description

Set up Lean 4 development environment and project structure for the Ash reference interpreter.

## Specification Reference

- SPEC-021: Lean Reference - Section 4 (Architecture)

## Requirements

### Functional Requirements

1. Lean 4 toolchain installed (elan, lake)
2. VS Code/Emacs editor integration
3. Project structure mirroring Rust organization
4. Build configuration (lakefile.lean)
5. Hello world test working
6. Git integration (gitignore for Lean artifacts)

### Non-functional Requirements

- Reproducible builds (lock lake-manifest.json)
- Cross-platform (Linux, macOS)
- Documented setup steps

## TDD Steps

### Step 1: Install Lean Toolchain (Green)

```bash
# Install elan (Lean version manager)
curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
source $HOME/.elan/env

# Verify installation
elan --version  # Should show version
lake --version  # Should show version
```

### Step 2: Create Project Structure (Green)

**File**: `lean_reference/lakefile.lean`

```lean
import Lake
open Lake DSL

package ash_reference {
  -- Add package configuration options here
  srcDir := "Ash"
}

lean_lib Ash {
  -- Library configuration
}

@[default_target]
lean_exe ash_ref {
  root := `Main
}

require std from git
  "https://github.com/leanprover/std4.git"
```

**File**: `lean_reference/.gitignore`

```gitignore
/build
/lake-packages
*.olean
.lake
.DS_Store
```

### Step 3: Create Directory Structure (Green)

```bash
cd lean_reference
mkdir -p Ash/Core
mkdir -p Ash/Eval
mkdir -p Ash/Differential
touch Ash.lean
touch Main.lean
```

### Step 4: Create Root Module (Green)

**File**: `lean_reference/Ash.lean`

```lean
-- Ash Reference Interpreter
-- Root module re-exporting all submodules

import Ash.Core.AST
import Ash.Core.Types
import Ash.Core.Environment
import Ash.Eval.Expr
import Ash.Eval.Pattern
import Ash.Differential.Compare
```

**File**: `lean_reference/Ash/Core/AST.lean`

```lean
-- Placeholder for AST types
namespace Ash

def hello := "Ash Reference Interpreter"

end Ash
```

### Step 5: Create Main Entry Point (Green)

**File**: `lean_reference/Main.lean`

```lean
import Ash

def main : IO Unit :=
  IO.println s!"{Ash.hello} - Version 0.1"
```

### Step 6: Build Project (Green)

```bash
cd lean_reference
lake update  # Download dependencies
lake build   # Build the project

# Expected output:
# [0/2] Building Ash.Core.AST
# [1/2] Building Main
# [2/2] Linking ash_ref
```

### Step 7: Run Executable (Green)

```bash
lake exe ash_ref
# Expected output:
# Ash Reference Interpreter - Version 0.1
```

### Step 8: Add VS Code Configuration (Green)

**File**: `.vscode/settings.json` (in project root)

```json
{
  "lean4.toolchainPath": "${userHome}/.elan/toolchains/leanprover--lean4---stable"
}
```

### Step 9: Document Setup (Green)

**File**: `lean_reference/README.md`

```markdown
# Ash Reference Interpreter

Lean 4 reference implementation of the Ash workflow language.

## Setup

1. Install Lean 4:
   ```bash
   curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
   ```

2. Build the project:
   ```bash
   lake update
   lake build
   ```

3. Run tests:
   ```bash
   lake exe test
   ```

## Structure

- `Ash/Core/` - AST and type definitions
- `Ash/Eval/` - Expression evaluation
- `Ash/Differential/` - Comparison with Rust
```

### Step 10: Verify Reproducibility (Green)

```bash
# Clean build test
rm -rf lean_reference/build
rm -rf lean_reference/lake-packages
cd lean_reference
lake update
lake build

# Should build successfully from clean state
```

## Completion Checklist

- [ ] Lean 4 toolchain installed (elan, lake)
- [ ] lakefile.lean configured
- [ ] .gitignore for Lean artifacts
- [ ] Directory structure created
- [ ] Root module (Ash.lean) created
- [ ] Main entry point builds and runs
- [ ] README with setup instructions
- [ ] Clean build test passes
- [ ] VS Code/Editor integration working

## Self-Review Questions

1. **Simplicity**: Is the structure minimal?
   - Yes: Standard Lean 4 project layout

2. **Spec drift**: Does it match SPEC-021?
   - Directory structure matches architecture section

3. **Cross-platform**: Will it work on macOS/Linux?
   - Yes: elan handles platform differences

## Estimated Effort

4 hours

## Dependencies

None - this is the first task

## Blocked By

Nothing

## Blocks

- TASK-138 (AST Types)
- All subsequent Lean tasks
