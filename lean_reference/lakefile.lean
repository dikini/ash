import Lake
open Lake DSL

package ash_reference {
  -- Package configuration
}

lean_lib Ash {
  -- Library configuration - srcDir defaults to "Ash"
  -- Lake will discover modules automatically
}

@[default_target]
lean_exe ash_ref {
  root := `Main
  srcDir := "."
}

-- Test executable
lean_exe test {
  root := `Ash.Tests.Runner
  srcDir := "."
}

-- Dependencies
require std from git
  "https://github.com/leanprover/std4.git" @ "v4.28.0"

-- Plausible for property-based testing
require plausible from git
  "https://github.com/leanprover-community/plausible.git" @ "v4.28.0"
