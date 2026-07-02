use ash_parser::surface::{
    ComputationRow, ComputationRowItem, Definition, PropositionWhereRow, Type,
};

fn parse_module(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module should parse")
}

fn first_function(module: &ash_parser::surface::ModuleFile) -> &ash_parser::surface::FnDef {
    let Definition::Function(function) = &module.definitions[0] else {
        panic!("expected first definition to be a function");
    };

    function
}

fn row_path_text(row_path: &[ash_parser::surface::Name]) -> String {
    row_path
        .iter()
        .map(|segment| segment.as_ref())
        .collect::<Vec<_>>()
        .join("::")
}

fn assert_single_op_row_item(row: &ComputationRow, expected: &str) {
    let [item] = &row.items[..] else {
        panic!("expected exactly one row item");
    };

    match item {
        ComputationRowItem::Operation { path, .. } => {
            assert_eq!(row_path_text(path), expected);
        }
        other => panic!("expected operation row item, got {other:?}"),
    }
}

#[test]
fn task_1809_parses_inline_callable_row_after_arrow() {
    let module = parse_module("fn read(path: Path) -> {PosixFs::read} String { path }");
    let function = first_function(&module);

    let Type::Fn(_, row, ret) = function
        .return_type
        .as_ref()
        .expect("function should have return type")
    else {
        panic!("expected function return type to be callable");
    };
    let row = row.as_ref().expect("inline row should be present");
    assert_single_op_row_item(row, "PosixFs::read");
    let Type::Name(name) = &**ret else {
        panic!("expected return type name, got {ret:?}");
    };
    assert_eq!(name.as_ref(), "String");
}

#[test]
fn task_1809_parses_where_row_expansion_for_callable_type() {
    let module = parse_module("fn read(path: Path) -> String where row { PosixFs::read } { path }");
    let function = first_function(&module);

    let return_type = function
        .return_type
        .as_ref()
        .expect("function should have return type");
    let Type::Name(name) = return_type else {
        panic!("expected return type name, got {return_type:?}");
    };
    assert_eq!(name.as_ref(), "String");

    let PropositionWhereRow { row, .. } = function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .expect("expected where row clause");
    assert_single_op_row_item(row, "PosixFs::read");
}

#[test]
fn task_1809_parses_row_variable_in_inline_fn_parameters_and_result() {
    let module = parse_module("fn map<A, B, r>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { xs }");
    let function = first_function(&module);

    let Type::Fn(params, row, _ret) = &function.params[1].ty else {
        panic!("expected higher-order parameter to be callable");
    };
    assert_eq!(params.len(), 1, "higher-order parameter has unary domain");
    assert_eq!(
        params[0],
        Type::Name("A".into()),
        "higher-order parameter domain should be `A`"
    );

    let row = row
        .as_ref()
        .expect("higher-order parameter should have inline row");
    assert_single_op_row_item(row, "r");

    let Type::Fn(_result_params, result_row, result_ret) = function
        .return_type
        .as_ref()
        .expect("function should have return type")
    else {
        panic!("expected function return type to be callable");
    };
    let result_row = result_row
        .as_ref()
        .expect("function return should carry row variable");
    assert_single_op_row_item(result_row, "r");
    let Type::Constructor { name, args } = &**result_ret else {
        panic!("expected return type constructor, got {result_ret:?}");
    };
    assert_eq!(name.as_ref(), "List");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], Type::Name("B".into()));
}

#[test]
fn task_1809_parses_mixed_row_family_items_with_tail() {
    let module = parse_module(
        "fn validate(req: Int) -> Int where row {
            resource fs,
            role app.admin,
            policy compliance,
            fail RuntimeError,
            evidence response_contract,
            | r
        } { req }",
    );
    let function = first_function(&module);
    let PropositionWhereRow { row, .. } = function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .expect("expected where row");

    assert_eq!(row.items.len(), 6);
    assert!(matches!(
        &row.items[0],
        ComputationRowItem::Resource {
            path,
            ..
        } if row_path_text(path) == "fs"
    ));
    assert!(matches!(
        &row.items[1],
        ComputationRowItem::Role {
            path,
            ..
        } if row_path_text(path) == "app::admin"
    ));
    assert!(matches!(
        &row.items[2],
        ComputationRowItem::Policy {
            path,
            ..
        } if row_path_text(path) == "compliance"
    ));
    assert!(matches!(
        &row.items[3],
        ComputationRowItem::Fail {
            path: Some(path),
            ..
        } if row_path_text(path) == "RuntimeError"
    ));
    assert!(matches!(
        &row.items[4],
        ComputationRowItem::Evidence {
            path,
            ..
        } if row_path_text(path) == "response_contract"
    ));
    assert!(matches!(
        &row.items[5],
        ComputationRowItem::Tail {
            variable,
            ..
        } if variable.as_ref() == "r"
    ));
}
