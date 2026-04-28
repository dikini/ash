// Phase 106 deferred semantic targets.
//
// These examples document the intended future shape for pure data monads. They
// are NOT implementation claims for Phase 106: List, Option, and Result
// comprehensions remain deferred until Ash has user-defined Monad dictionaries,
// pure data dictionaries, and constructor-hole support such as Result<_, E>.

// Future shape only:
// pub fn list_pairs(xs: List<Int>, ys: List<Int>) -> List<Int> {
//     [x + y | x <- xs, y <- ys]: List
// }

// Future shape only:
// pub fn maybe_increment(x: Option<Int>) -> Option<Int> {
//     [y + 1 | y <- x]: Option
// }

// Future shape only; requires one-hole target syntax support:
// pub fn parse_two(a: Result<Int, ParseError>, b: Result<Int, ParseError>) -> Result<Int, ParseError> {
//     [x + y | x <- a, y <- b]: Result<_, ParseError>
// }

// Bare boolean guards are also deferred; use explicit monadic operations once a
// target provides them:
// [x | x <- xs, _ <- guard(x > 0)]: List
