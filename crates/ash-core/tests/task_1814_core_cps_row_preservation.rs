//! TASK-1814 Core/CPS row preservation evidence.

use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_lower::{CoreLoweringContext, lower_core_program_with_context};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{ContRef, EffectItemKind, Term};

fn path(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_owned())
}

fn current_row_with_supported_families() -> CoreRow {
    CoreRow::closed(vec![
        CoreRowItem::operation(path(&["fs"]), "read"),
        CoreRowItem::Resource {
            path: path(&["File"]),
            mode: "read".to_owned(),
        },
        CoreRowItem::Contract {
            contract: "Signed".to_owned(),
        },
        CoreRowItem::Channel {
            path: path(&["inbox"]),
            mode: "recv".to_owned(),
            payload_type: Box::new(base("String")),
        },
        CoreRowItem::Process {
            operation: "spawn".to_owned(),
        },
        CoreRowItem::Failure {
            ty: Some(Box::new(base("String"))),
        },
        CoreRowItem::Evidence {
            path: path(&["read_allowed"]),
        },
        CoreRowItem::EffectGroupRef {
            path: path(&["FsReads"]),
        },
    ])
}

#[test]
fn core_current_row_supported_families_survive_cps_lowering() {
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitUnit)))
        .expect("Core atom program validates");
    let lowered = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(
            ContRef::Label("exit".to_owned()),
            current_row_with_supported_families(),
        ),
    )
    .expect("supported closed Core row lowers");

    let Term::Jump { row, .. } = lowered else {
        panic!("atom program should lower to Jump");
    };
    let families = row
        .items
        .iter()
        .map(|item| (item.namespace.as_str(), item.name.as_str(), item.kind))
        .collect::<Vec<_>>();

    assert!(families.contains(&("cap", "fs.read", EffectItemKind::Capability)));
    assert!(families.contains(&("resource", "File.read", EffectItemKind::Resource)));
    assert!(families.contains(&("contract", "Signed", EffectItemKind::Contract)));
    assert!(families.contains(&("channel", "inbox.recv", EffectItemKind::Channel)));
    assert!(families.contains(&("process", "spawn", EffectItemKind::Process)));
    assert!(families.contains(&("fail", "String", EffectItemKind::Failure)));
    assert!(families.contains(&("evidence", "read_allowed", EffectItemKind::Evidence)));
    assert!(families.contains(&("group", "FsReads", EffectItemKind::Group)));
}
