// Fixture for Phase 143 cross-language MCP lookup tests.
// The symbol names are intentionally present as Ash-facing identifiers for
// reverse Rust → Ash usage mapping.
// Comment-only Effect should not be reported.
-- Dash-comment Effect should not be reported.
/* Block-comment Effect should not be reported. */

type Effect = String
type NoEffect = String
type No-Effect = String
type Effect-type = String
let message = "Effect in a string should not be reported"
interface CapabilityProvider {
    fn provide() -> Effect
    fn fallback() -> Effect
}
type CapabilityError = String
