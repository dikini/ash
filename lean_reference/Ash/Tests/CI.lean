-- Ash CI Integration
-- CI-friendly test output

namespace Ash.Tests.CI

/-- Exit code for CI (0 = success, 1 = failure) -/
def ciExitCode (numPassed numFailed : Nat) : UInt32 :=
  if numFailed > 0 then 1 else 0

end Ash.Tests.CI
