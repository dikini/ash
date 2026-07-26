//! TASK-2001 RED contract for the existing module-summary lowering boundary.
//!
//! This does not introduce a parallel declaration model: it exercises the
//! public `lower_module_type_metadata` handoff and the existing core-owned
//! `ModuleSemanticSummary` slots.  Effect-row and handler summaries require
//! dedicated core schema slots before an equivalent executable assertion can
//! exist; see the task report accompanying this test.

use std::path::Path;

use ash_core::{
    module_graph::ModuleId,
    semantic_summary::{ModuleIdentity, ModuleSourceOrigin, SourceOrigin},
};
use ash_parser::lower::lower_module_type_metadata;

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(2001),
        vec!["task_2001".to_string()],
        ModuleSourceOrigin::File("task-2001.ash".to_string()),
    )
}

#[test]
fn task_2001_lowers_public_newtype_to_nominal_type_and_constructor_summary_with_origin() {
    let module = ash_parser::parse_surface_file_with_path(
        "pub newtype OrderId = OrderId(Int);",
        Some(Path::new("task-2001.ash")),
    )
    .expect("canonical newtype parses before lowering");

    let lowered = lower_module_type_metadata(&module, module_identity());
    let newtype = lowered
        .summary
        .exported_types
        .iter()
        .find(|summary| summary.exported_name.as_str() == "OrderId")
        .expect("newtype must cross the existing core module-summary handoff");

    assert_eq!(newtype.id.module, module_identity());
    assert_eq!(newtype.source_anchor.label, "newtype OrderId");
    assert_eq!(
        newtype.source_anchor.origin,
        SourceOrigin::File("task-2001.ash".to_string())
    );
    assert!(
        lowered
            .summary
            .exported_constructors
            .iter()
            .any(|constructor| {
                constructor.exported_name.as_str() == "OrderId" && constructor.parent == newtype.id
            }),
        "the nominal constructor must be exported with the same origin identity"
    );
}
