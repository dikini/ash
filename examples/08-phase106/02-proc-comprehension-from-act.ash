// Phase 106 monad comprehension syntax: explicit Proc target.
// Proc comprehensions do not implicitly lift Act work. The lift is explicit via
// proc::from_act, matching the Phase 105 do:Proc tower rule.

pub fn parse_process() -> Proc<String> {
    [parsed | raw <- proc::from_act(act::unit("42")), parsed <- proc::unit(raw)]: Proc
}
