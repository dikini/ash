//! TASK-2004: checked Core-to-CPS admission boundary tests.

use ash_core::core_ash::{CoreAtom, CoreExpr};
use ash_core::core_ash_lower::lower_core_program;
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

#[test]
fn malformed_raw_core_is_rejected_before_it_can_be_lowered_to_cps() {
    let raw = RawCoreProgram::new(CoreExpr::Force {
        thunk: CoreAtom::LitInt(7),
        name: "forced".to_string(),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("forced".to_string()))),
    });

    let error = validate_core_program(raw)
        .expect_err("unchecked Core must not cross the Core-to-CPS admission boundary");

    assert_eq!(
        error,
        CoreValidationError::ForceRequiresVariableThunk {
            atom: "LitInt(7)".to_string(),
        },
        "invalid Core must stop at validation instead of reaching the Core-to-CPS lowerer"
    );
}

#[test]
fn validated_core_is_the_only_lowering_input_type() {
    let valid = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(7))))
        .expect("well-formed Core admits through validation");

    let term = lower_core_program(valid).expect("only validated Core reaches lowering");

    assert!(matches!(term, ash_core::cps::Term::Jump { .. }));
}
