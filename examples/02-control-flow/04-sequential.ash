// Sequential - validation before later computation.
//
// The historical example used capability declarations and action syntax that are
// not part of this small checkable example. This keeps the teaching point:
// later bindings can depend on earlier bindings.

workflow main {
    let timeout_seconds = 30
    let retry_count = 3
    let base_delay = 2
    let retry_wait = retry_count * base_delay
    let total_wait = timeout_seconds + retry_wait

    ret total_wait
}
