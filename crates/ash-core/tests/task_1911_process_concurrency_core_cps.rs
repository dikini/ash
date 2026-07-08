//! TASK-1911 Core/CPS process concurrency fixture coverage.

use ash_core::core_ash::{CoreAtom, CoreExpr, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_lower::{CoreLoweringContext, lower_core_program_with_context};
use ash_core::core_ash_text::{format_row_item, parse_row_item};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{ContRef, EffectItemKind, Term};

#[test]
fn core_text_process_channel_rows_round_trip_and_lower_to_cps_facts() {
    let process_spawn = parse_row_item("process spawn").expect("process row parses");
    let channel_send = parse_row_item("channel jobs send Unit").expect("channel row parses");

    assert_eq!(format_row_item(&process_spawn), "process spawn");
    assert_eq!(format_row_item(&channel_send), "channel jobs send Unit");

    let program = validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitUnit)))
        .expect("Core atom program validates");
    let lowered = lower_core_program_with_context(
        program,
        CoreLoweringContext::new(
            ContRef::Label("exit".to_owned()),
            CoreRow::closed(vec![
                process_spawn,
                channel_send,
                CoreRowItem::channel(["results"], "recv", CoreType::Base("Int".into())),
            ]),
        ),
    )
    .expect("process/channel row lowers");

    let Term::Jump { row, .. } = lowered else {
        panic!("atom program should lower to Jump");
    };
    let facts = row
        .items
        .iter()
        .map(|item| (item.namespace.as_str(), item.name.as_str(), item.kind))
        .collect::<Vec<_>>();

    assert!(facts.contains(&("process", "spawn", EffectItemKind::Process)));
    assert!(facts.contains(&("channel", "jobs.send", EffectItemKind::Channel)));
    assert!(facts.contains(&("channel", "results.recv", EffectItemKind::Channel)));
    assert!(
        !facts
            .iter()
            .any(|(namespace, _, kind)| *namespace == "proc" && *kind == EffectItemKind::Process),
        "canonical process rows must not lower through the removed proc namespace"
    );
}
