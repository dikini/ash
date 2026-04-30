// Phase 108 executable example: explicit-target Workflow comprehension.
// ash check examples/09-phase108/05-workflow-comprehension.ash
//
// `[...]: Workflow` normalizes through the same typed-do path as do:Workflow.

pub fn seed() -> Workflow<Int> {
    do:Workflow {
        return 1
    }
}

pub fn via_comprehension() -> Workflow<Int> {
    [x | x <- seed()]: Workflow
}
