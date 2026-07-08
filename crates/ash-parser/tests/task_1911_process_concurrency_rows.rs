use ash_parser::surface::{ComputationRowItem, Definition};

fn first_function(source: &str) -> ash_parser::surface::FnDef {
    let module = ash_parser::parse_surface_file(source).expect("module parses");
    let Definition::Function(function) = module
        .definitions
        .into_iter()
        .next()
        .expect("module has a function")
    else {
        panic!("expected function definition");
    };
    function
}

#[test]
fn canonical_process_and_channel_rows_parse_without_removed_proc_surface() {
    let function = first_function(
        r#"
        fn schedule(job: Int) -> Int
        where
            row {
                process spawn,
                process await,
                channel jobs,
                channel results
            }
        {
            job
        }
        "#,
    );
    let row = &function
        .proposition_tail
        .as_ref()
        .and_then(|tail| tail.row.as_ref())
        .expect("where row parses")
        .row;

    assert_eq!(row.items.len(), 4);
    assert!(matches!(
        &row.items[0],
        ComputationRowItem::Process {
            keyword,
            operation: Some(operation),
            ..
        } if keyword.as_ref() == "process" && operation.as_ref() == "spawn"
    ));
    assert!(matches!(
        &row.items[1],
        ComputationRowItem::Process {
            keyword,
            operation: Some(operation),
            ..
        } if keyword.as_ref() == "process" && operation.as_ref() == "await"
    ));
    assert!(matches!(
        &row.items[2],
        ComputationRowItem::Channel {
            mode: None,
            path,
            ..
        } if path.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>() == ["jobs"]
    ));
    assert!(matches!(
        &row.items[3],
        ComputationRowItem::Channel {
            mode: None,
            path,
            ..
        } if path.iter().map(std::convert::AsRef::as_ref).collect::<Vec<_>>() == ["results"]
    ));
}
