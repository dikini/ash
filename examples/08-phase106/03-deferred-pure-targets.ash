// Phase 106 deferred semantic targets.
//
// These examples document the intended future shape for pure data monads. They
// are NOT implementation claims for Phase 106. Phase 133 later implemented
// explicit-target Option and Result evidence; List Monad/comprehension execution,
// target inference, guards, and arbitrary user Monad execution remain follow-up work.

// Future shape only:
// pub fn list_pairs(xs: List<Int>, ys: List<Int>) -> List<Int> {
//     [x + y | x <- xs, y <- ys]: List
// }

// Phase 133 implemented explicit-target Option and Result evidence; these
// shapes are no longer pure-data deferrals, though arbitrary user Monad
// execution, guards, inference, and full List Monad semantics remain follow-up.
// pub fn maybe_increment(x: Option<Int>) -> Option<Int> {
//     [y + 1 | y <- x]: Option
// }
//
// pub fn parse_two(a: Result<Int, ParseError>, b: Result<Int, ParseError>) -> Result<Int, ParseError> {
//     [x + y | x <- a, y <- b]: Result<_, ParseError>
// }

// Bare boolean guards are also deferred; use explicit monadic operations once a
// target provides them:
// [x | x <- xs, _ <- guard(x > 0)]: List
