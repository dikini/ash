// Phase 108 executable example: first-class Workflow value with unit-like result.
// ash check examples/09-phase108/01-do-workflow-unit.ash

pub fn approved_value() -> Workflow<Int> {
    do:Workflow {
        return 1
    }
}
