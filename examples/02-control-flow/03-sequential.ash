// Sequential - ordered workflow composition in the current syntax.
//
// This version demonstrates that later bindings can depend on earlier ones.
// Capability-backed data fetching from the historical sketch is deferred until
// the corresponding providers are available through executable examples.

workflow main {
    let users = 3
    let orders = users * 4
    let inventory = 20
    let remaining = inventory - orders

    if remaining > 0 then ret remaining else ret 0
}
