//! TASK-1905 process/channel row carrier evidence.

use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_lower::{CoreLoweringContext, lower_core_program_with_context};
use ash_core::core_ash_text::{format_row_item, parse_row_item};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{ContRef, EffectItemKind, Term};

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_owned())
}

#[test]
fn process_row_fact_uses_process_namespace_when_lowered_to_cps() {
    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitUnit)))
        .expect("Core atom program validates");
    let lowered = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(
            ContRef::Label("exit".to_owned()),
            CoreRow::closed(vec![
                CoreRowItem::process("spawn"),
                CoreRowItem::process("join"),
                CoreRowItem::process("await"),
                CoreRowItem::process("cancel"),
            ]),
        ),
    )
    .expect("closed process row lowers");

    let Term::Jump { row, .. } = lowered else {
        panic!("atom program should lower to Jump");
    };
    let facts = row
        .items
        .iter()
        .map(|item| (item.namespace.as_str(), item.name.as_str(), item.kind))
        .collect::<Vec<_>>();

    assert!(facts.contains(&("process", "spawn", EffectItemKind::Process)));
    assert!(facts.contains(&("process", "join", EffectItemKind::Process)));
    assert!(facts.contains(&("process", "await", EffectItemKind::Process)));
    assert!(facts.contains(&("process", "cancel", EffectItemKind::Process)));
    assert!(
        !facts
            .iter()
            .any(|(namespace, _, kind)| *namespace == "proc" && *kind == EffectItemKind::Process),
        "Phase 201 process facts must not lower through the removed proc namespace"
    );
}

#[test]
fn channel_row_fact_helper_preserves_mode_path_and_payload() {
    let item = CoreRowItem::channel(["jobs", "priority"], "send", base("Job"));

    let CoreRowItem::Channel {
        path,
        mode,
        payload_type,
    } = item
    else {
        panic!("channel helper should build a channel row item");
    };

    assert_eq!(path, vec!["jobs".to_owned(), "priority".to_owned()]);
    assert_eq!(mode, "send");
    assert_eq!(*payload_type, base("Job"));
}

#[test]
fn core_text_rejects_proc_alias_and_formats_canonical_process() {
    assert!(parse_row_item("proc spawn").is_err());
    let canonical =
        parse_row_item("process spawn").expect("canonical process spelling should parse");

    assert_eq!(canonical, CoreRowItem::process("spawn"));
    assert_eq!(format_row_item(&canonical), "process spawn");
}
