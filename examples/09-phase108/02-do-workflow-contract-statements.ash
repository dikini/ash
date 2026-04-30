// Phase 108 executable example: workflow contract injection statements.
// ash check examples/09-phase108/02-do-workflow-contract-statements.ash

pub fn guarded_value() -> Workflow<Int> {
    do:Workflow {
        requires: any_role([Reviewer, Approver]);
        ensures: result >= 1;
        return 1
    }
}
