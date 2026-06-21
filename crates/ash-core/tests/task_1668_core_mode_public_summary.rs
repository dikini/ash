use ash_core::core_ash::{CoreEvalMode, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_typecheck::{
    CorePublicFunctionSummary, CorePublicSummaryError, summarize_core_public_function_type,
};

fn app(name: &str, args: Vec<CoreType>) -> CoreType {
    CoreType::App {
        name: name.to_owned(),
        args,
    }
}

fn base(name: &str) -> CoreType {
    CoreType::Base(name.to_owned())
}

fn chan(path: &[&str], mode: &str, payload: CoreType) -> CoreRowItem {
    CoreRowItem::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: Box::new(payload),
    }
}

fn mode_ty(inner: CoreType, mode: CoreEvalMode, latent_row: CoreRow) -> CoreType {
    CoreType::Mode {
        mode,
        inner: Box::new(inner),
        latent_row: Some(latent_row),
    }
}

#[test]
fn public_function_summary_collects_mode_wrappers_and_latent_row_type_constructors() {
    let function_type = CoreType::Function {
        params: vec![mode_ty(
            app("ModeParamPayload", vec![base("Int")]),
            CoreEvalMode::Memo,
            CoreRow::closed(vec![chan(
                &["jobs"],
                "send",
                app("ModeLatentPayload", vec![]),
            )]),
        )],
        result: Box::new(mode_ty(
            base("Unit"),
            CoreEvalMode::Lazy,
            CoreRow::closed(vec![chan(
                &["audit"],
                "recv",
                app("ModeResultPayload", vec![base("String")]),
            )]),
        )),
        row: CoreRow::default(),
    };

    let summary: CorePublicFunctionSummary =
        summarize_core_public_function_type("mode_summary", &function_type, &[], &[])
            .expect("public summary accepts mode types");

    assert_eq!(summary.exported_name(), "mode_summary");
    assert_eq!(
        summary.params(),
        &[mode_ty(
            app("ModeParamPayload", vec![base("Int")]),
            CoreEvalMode::Memo,
            CoreRow::closed(vec![chan(
                &["jobs"],
                "send",
                app("ModeLatentPayload", vec![])
            )])
        )]
    );
    assert_eq!(
        summary.result(),
        &mode_ty(
            base("Unit"),
            CoreEvalMode::Lazy,
            CoreRow::closed(vec![chan(
                &["audit"],
                "recv",
                app("ModeResultPayload", vec![base("String")])
            )])
        )
    );

    let constructors = summary.type_constructors();
    let constructor_names: Vec<_> = constructors.iter().map(|c| c.name()).collect();

    assert!(constructor_names.contains(&"ModeParamPayload"));
    assert!(constructor_names.contains(&"ModeLatentPayload"));
    assert!(constructor_names.contains(&"ModeResultPayload"));
}

#[test]
fn public_summary_rejects_private_row_refs_inside_mode_latent_rows() {
    let function_type = CoreType::Function {
        params: vec![mode_ty(
            base("Int"),
            CoreEvalMode::Memo,
            CoreRow::closed(vec![CoreRowItem::EffectGroupRef {
                path: vec!["private".into(), "secret".into()],
            }]),
        )],
        result: Box::new(base("Unit")),
        row: CoreRow::default(),
    };

    let err = summarize_core_public_function_type("bad_mode_summary", &function_type, &[], &[])
        .expect_err("private row refs in mode metadata should reject public summaries");

    assert_eq!(
        err,
        CorePublicSummaryError::PrivateRowReference {
            path: vec!["private".into(), "secret".into()],
            public_item: None,
            detail:
                "private effect group private.secret must be expanded or exported before summary"
                    .into(),
        }
    );
}
