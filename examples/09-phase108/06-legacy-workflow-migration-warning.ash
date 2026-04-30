// Phase 108 executable migration example.
// ash check examples/09-phase108/06-legacy-workflow-migration-warning.ash
//
// The legacy declaration below remains accepted for compatibility, but ash
// check emits DeprecatedLegacyWorkflowDeclaration. Prefer the first-class
// Workflow<Int> function form shown first.

pub fn guarded_value() -> Workflow<Int> {
    do:Workflow {
        requires: role(admin);
        ensures: result >= 1;
        return 1
    }
}

pub workflow legacy_guarded
    requires: role(admin)
{
    done
}
