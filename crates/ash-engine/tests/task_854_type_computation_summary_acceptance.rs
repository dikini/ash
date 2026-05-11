//! TASK-854: SPEC-062 acceptance/non-interference focused engine checks.

use ash_core::type_ir::TypeComputationHeadId;
use ash_engine::module_loader::{LoadedOrdinaryFile, load_ordinary_file};

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

fn type_function_names(loaded: &LoadedOrdinaryFile) -> Vec<String> {
    let mut names = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|type_function| type_function.exported_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn type_function_heads(loaded: &LoadedOrdinaryFile) -> Vec<(String, TypeComputationHeadId)> {
    let mut heads = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .map(|type_function| {
            (
                type_function.exported_name.clone(),
                type_function.head.clone(),
            )
        })
        .collect::<Vec<_>>();
    heads.sort_by(|left, right| left.0.cmp(&right.0));
    heads
}

const PROVIDER: &str = r"
pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }
pub sealed type domain Unrelated { Other; }

pub type fn Helper(xs: TypeList) -> TypeList { case Helper<xs> = xs; }
pub type fn UseHelper(xs: TypeList) -> TypeList { case UseHelper<xs> = Helper<xs>; }
pub type fn Sibling(xs: TypeList) -> TypeList { case Sibling<xs> = xs; }
";

#[test]
fn named_import_keeps_only_selected_head_source_visible_while_dependency_closure_is_transported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    let helper_caller = dir.path().join("helper_caller.ash");
    let sibling_caller = dir.path().join("sibling_caller.ash");

    write_file(&provider, PROVIDER);
    write_file(&facade, "pub use provider::{UseHelper};\n");
    write_file(
        &caller,
        "use facade::{UseHelper}\nworkflow main { ret 0 }\n",
    );
    write_file(
        &helper_caller,
        "use facade::{Helper}\nworkflow main { ret 0 }\n",
    );
    write_file(
        &sibling_caller,
        "use facade::{Sibling}\nworkflow main { ret 0 }\n",
    );

    let loaded = load_ordinary_file(&caller).expect("selected named import loads");

    // Current source syntax does not yet support applying an imported type-function
    // name from a downstream local type-fn RHS. The honest acceptance evidence here
    // is therefore loader/import visibility plus dependency-closure transport:
    // UseHelper is selected/source-visible, Helper is transported under an
    // internal dependency metadata name for the normalizer, and Sibling/unrelated
    // domains do not leak.
    assert_eq!(
        type_function_names(&loaded),
        vec!["$ash_dependency$Helper", "UseHelper"]
    );
    assert_eq!(
        loaded
            .imported_type_function_heads
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["UseHelper"]
    );
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .all(|type_def| type_def.name != "Helper" && type_def.name != "Sibling"),
        "dependency helper and unrelated sibling must not appear as ordinary source-visible types"
    );
    assert!(
        loaded
            .imported_semantic_summaries
            .iter()
            .flat_map(|summary| summary.exported_sealed_domains.iter())
            .all(|domain| domain.exported_name != "Unrelated"),
        "unrelated sealed-domain closure must not leak"
    );

    let err = load_ordinary_file(&helper_caller)
        .expect_err("dependency helper is normalizer-available but not re-export source-visible");
    assert!(
        err.to_string().contains("item 'Helper' not found"),
        "unexpected error: {err}"
    );
    let err = load_ordinary_file(&sibling_caller).expect_err("sibling is not source-visible");
    assert!(
        err.to_string().contains("item 'Sibling' not found"),
        "unexpected error: {err}"
    );
}

#[test]
fn glob_import_and_pub_use_preserve_canonical_heads_and_equation_order() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let glob = dir.path().join("glob.ash");
    let reexported = dir.path().join("reexported.ash");

    write_file(
        &provider,
        r"
pub sealed type domain Bits { Zero; One; }
pub type fn Prefer(x: Bits) -> Bits {
    case Prefer<Zero> = One;
    case Prefer<x> = x;
}
pub type fn Id(x: Bits) -> Bits { case Id<x> = x; }
",
    );
    write_file(&facade, "pub use provider::{Prefer};\n");
    write_file(&glob, "use provider::*\nworkflow main { ret 0 }\n");
    write_file(
        &reexported,
        "use facade::{Prefer}\nworkflow main { ret 0 }\n",
    );

    let glob_loaded = load_ordinary_file(&glob).expect("glob import loads");
    assert_eq!(type_function_names(&glob_loaded), vec!["Id", "Prefer"]);

    let reexported_loaded = load_ordinary_file(&reexported).expect("re-exported import loads");
    let glob_prefer = glob_loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .find(|type_function| type_function.exported_name == "Prefer")
        .expect("glob-imported Prefer summary");
    let reexported_prefer = reexported_loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_type_functions.iter())
        .find(|type_function| type_function.exported_name == "Prefer")
        .expect("re-exported Prefer summary");

    assert_eq!(reexported_prefer.head, glob_prefer.head);
    assert_eq!(
        reexported_prefer
            .equations
            .iter()
            .map(|equation| equation.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(reexported_prefer.equations, glob_prefer.equations);
}

#[test]
fn repeated_glob_imports_are_deterministic_and_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let first = dir.path().join("first.ash");
    let second = dir.path().join("second.ash");

    write_file(&provider, PROVIDER);
    write_file(&first, "use provider::*\nworkflow main { ret 0 }\n");
    write_file(
        &second,
        "use provider::*\nuse provider::*\nworkflow main { ret 0 }\n",
    );

    let first = load_ordinary_file(&first).expect("first glob import loads");
    let second = load_ordinary_file(&second).expect("repeated glob import loads");

    assert_eq!(
        type_function_names(&first),
        vec!["Helper", "Sibling", "UseHelper"]
    );
    assert_eq!(type_function_names(&first), type_function_names(&second));
    assert_eq!(type_function_heads(&first), type_function_heads(&second));
}
