//! TASK-1993 regression and property tests for frame-ordered operation lookup.

use ash_core::cps::{
    ContMultiplicity, EffectItem, EffectItemKind, EffectOp, EffectRow, HandlerChain, HandlerClause,
    HandlerFrame, HandlerFrameMatch, ResumeRowMetadata, Term, TrapReason,
};
use proptest::prelude::*;

fn operation(name: impl Into<String>) -> EffectOp {
    EffectOp {
        item: EffectItem {
            namespace: "task-1993".to_owned(),
            name: name.into(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_owned()],
        result_type: "String".to_owned(),
    }
}

fn shallow_frame(op: EffectOp, marker: usize) -> HandlerFrame {
    HandlerFrame::Shallow {
        clause: HandlerClause {
            op,
            params: vec![format!("argument_{marker}")],
            resume: format!("resume_{marker}"),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom(format!("body_{marker}")),
            }),
            row: EffectRow::default(),
            resume_row: ResumeRowMetadata::InheritFromTarget,
            resume_multiplicity: ContMultiplicity::Affine,
        },
    }
}

fn provider_frame(op: EffectOp, marker: usize) -> HandlerFrame {
    HandlerFrame::Provider {
        op,
        handler: format!("provider_{marker}"),
    }
}

#[test]
fn operation_lookup_is_absent_when_no_frame_matches() {
    let target = operation("target");
    let mut chain = HandlerChain::new();
    chain.push(shallow_frame(operation("other-handler"), 0));
    chain.push(provider_frame(operation("other-provider"), 1));

    assert_eq!(chain.find_operation_frame(&target), None);
}

#[test]
fn operation_lookup_mutation_sentinel_rejects_outermost_first_search() {
    let target = operation("target");
    let mut chain = HandlerChain::new();
    chain.push(shallow_frame(target.clone(), 0));
    chain.push(provider_frame(operation("not-target"), 1));
    chain.push(provider_frame(target.clone(), 2));

    let selected = chain
        .find_operation_frame(&target)
        .expect("the inner provider should match");
    let selected_index = match selected {
        HandlerFrameMatch::Shallow { frame_index, .. }
        | HandlerFrameMatch::Deep { frame_index, .. }
        | HandlerFrameMatch::Provider { frame_index, .. } => frame_index,
    };
    let deliberately_outermost_first = chain
        .frames
        .iter()
        .enumerate()
        .find_map(|(index, frame)| match frame {
            HandlerFrame::Shallow { clause } if clause.op == target => Some(index),
            HandlerFrame::Provider { op, .. } if *op == target => Some(index),
            _ => None,
        })
        .expect("the outer handler should match in the mutation model");

    assert_eq!(selected_index, 2, "the innermost matching frame wins");
    assert_eq!(deliberately_outermost_first, 0);
    assert_ne!(selected_index, deliberately_outermost_first);
}

proptest! {
    #[test]
    fn operation_lookup_selects_the_greatest_matching_index_and_its_exact_payload(
        frame_kinds in proptest::collection::vec((any::<bool>(), any::<bool>()), 0..32),
    ) {
        let target = operation("target");
        let mut chain = HandlerChain::new();

        for (index, (matches_target, is_provider)) in frame_kinds.iter().copied().enumerate() {
            let frame_op = if matches_target {
                target.clone()
            } else {
                operation(format!("other_{index}"))
            };
            chain.push(if is_provider {
                provider_frame(frame_op, index)
            } else {
                shallow_frame(frame_op, index)
            });
        }

        let expected = frame_kinds
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (matches_target, _))| *matches_target)
            .map(|(index, (_, is_provider))| (index, *is_provider));

        match (chain.find_operation_frame(&target), expected) {
            (None, None) => {},
            (Some(HandlerFrameMatch::Shallow { clause, frame_index }), Some((index, false))) => {
                prop_assert_eq!(frame_index, index);
                prop_assert!(frame_index < chain.frames.len());
                prop_assert_eq!(&clause.op, &target);
                prop_assert_eq!(&clause.params, &vec![format!("argument_{index}")]);
                prop_assert_eq!(&clause.resume, &format!("resume_{index}"));
            },
            (Some(HandlerFrameMatch::Provider { handler, frame_index }), Some((index, true))) => {
                prop_assert_eq!(frame_index, index);
                prop_assert!(frame_index < chain.frames.len());
                prop_assert_eq!(handler, format!("provider_{index}"));
                match &chain.frames[frame_index] {
                    HandlerFrame::Provider { op, handler: source_handler } => {
                        prop_assert_eq!(op, &target);
                        prop_assert_eq!(handler, source_handler);
                    },
                    HandlerFrame::Shallow { .. } => prop_assert!(false, "provider match must retain provider provenance"),
                    HandlerFrame::Deep { .. } => prop_assert!(false, "provider match must retain provider provenance"),
                }
            },
            (Some(HandlerFrameMatch::Deep { .. }), _) => {
                prop_assert!(false, "this shallow/provider generator cannot yield a deep frame")
            },
            (actual, expected) => prop_assert!(false, "lookup result {actual:?} disagrees with innermost model {expected:?}"),
        }
    }
}
