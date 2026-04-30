// REFERENCE-ONLY: Phase 108 workflow algebra intrinsic spelling.
//
// These forms are covered by parser/typechecker regression tests. They remain
// reference-only in the examples corpus until full source-file parse_file
// elaboration handles first-class workflow algebra expressions before lowering.

pub fn unit_example() -> Workflow<Int> {
    workflow::unit(1)
}

pub fn bind_example() -> Workflow<Int> {
    workflow::bind(workflow::unit(1), fn(x: Int) {
        workflow::unit(x)
    })
}

pub fn then_example() -> Workflow<Int> {
    workflow::then(workflow::unit(1), workflow::unit(2))
}

pub fn contract_intrinsic_example() -> Workflow<Int> {
    workflow::then(
        workflow::requires(role(admin)),
        workflow::then(workflow::ensures(result >= 1), workflow::unit(1))
    )
}
