// REFERENCE-ONLY: explicit lifts from lower tower carriers into Workflow.
//
// Phase 108 has no implicit Act<A> or Proc<A> to Workflow<A> conversion. Use
// workflow::from_proc(...) or workflow::from_act(...) explicitly. These forms
// are covered by typechecker regression tests; this file is reference-only until
// full source-file first-class workflow expression elaboration is available.

pub fn proc_step() -> Proc<Int> {
    do:Proc {
        return 1
    }
}

pub fn workflow_from_proc() -> Workflow<Int> {
    do:Workflow {
        x <- workflow::from_proc(proc_step());
        return x
    }
}

// Reference shape only: Act-producing definitions currently depend on the
// surrounding effectful context/provider surface.
pub fn workflow_from_act(action: Act<Int>) -> Workflow<Int> {
    do:Workflow {
        x <- workflow::from_act(action);
        return x
    }
}
