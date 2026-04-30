// Conditionals - current Ash if/then/else syntax.
//
// Ash workflow conditionals use `if <condition> then <workflow> else <workflow>`.
// This example keeps the branches expression-free and returns a simple value so
// it remains a checkable conformance example.

workflow main {
    let score = 85
    let honors_cutoff = 90
    let pass_cutoff = 70

    if score >= honors_cutoff then ret "honors" else if score >= pass_cutoff then ret "pass" else ret "retry"
}
