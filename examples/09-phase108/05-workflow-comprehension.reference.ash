// REFERENCE-ONLY: explicit-target Workflow comprehension.
//
// `[...]: Workflow` normalizes through the same typed-do path as do:Workflow in
// typechecker coverage. This file is reference-only until the source-file
// parse_file path elaborates workflow comprehensions before lowering.

pub fn seed() -> Workflow<Int> {
    do:Workflow {
        return 1
    }
}

pub fn via_comprehension() -> Workflow<Int> {
    [x | x <- seed()]: Workflow
}
