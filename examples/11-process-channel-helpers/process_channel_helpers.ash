use process::{spawn_join_plan, bounded_worker_pool, channel_loop_plan, cancellation_cleanup, sendability_guard, channel_diagnostic, process_trace}

fn main() -> Bool {
  do {
    let spawn_plan = spawn_join_plan("parallel fetch", 2);
    let pool_plan = bounded_worker_pool("workers", 4, 16);
    let stream_plan = channel_loop_plan("events", "updates", 32);
    let cleanup_plan = cancellation_cleanup("shutdown", 2);
    let guard = sendability_guard("payload", true, true);
    let diagnostic = channel_diagnostic("updates", "closed");
    let trace = process_trace("join", true);

    return spawn_plan.preserves_sendability;
  }
}
