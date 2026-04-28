// Phase 106 monad comprehension syntax: explicit Act target.
// This is source-level sugar over the same typed-do machinery as:
//
// do:Act {
//     raw <- act::unit("42");
//     parsed <- act::unit(raw);
//     return parsed
// }

pub fn parse_action() -> Act<String> {
    [parsed | raw <- act::unit("42"), parsed <- act::unit(raw)]: Act
}
