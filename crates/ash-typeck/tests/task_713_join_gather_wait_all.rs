use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

#[test]
fn proc_join_builtin_is_registered_as_wait_for_all_pair_observer() {
    let env = TypeEnv::with_builtin_types();

    let ty = env
        .lookup_variable("proc::join")
        .expect("TypeEnv should register proc::join");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };

    assert_eq!(params.len(), 2);
    let left_result = match &params[0] {
        Type::Constructor { name, args, .. } if name.name == "P" && args.len() == 1 => {
            args[0].clone()
        }
        other => panic!("expected left handle parameter, got {other:?}"),
    };
    let right_result = match &params[1] {
        Type::Constructor { name, args, .. } if name.name == "P" && args.len() == 1 => {
            args[0].clone()
        }
        other => panic!("expected right handle parameter, got {other:?}"),
    };

    match *ret {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            match &args[0] {
                Type::Record(fields) => {
                    assert_eq!(
                        fields.len(),
                        2,
                        "join should return exactly two ordered child values"
                    );
                    assert_eq!(fields[0].0.as_ref(), "_0");
                    assert_eq!(fields[0].1, left_result);
                    assert_eq!(fields[1].0.as_ref(), "_1");
                    assert_eq!(fields[1].1, right_result);
                }
                other => {
                    panic!("expected tuple-record payload for proc::join result, got {other:?}")
                }
            }
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}

#[test]
fn proc_gather_builtin_is_registered_as_wait_for_all_handle_list_observer() {
    let env = TypeEnv::with_builtin_types();

    let ty = env
        .lookup_variable("proc::gather")
        .expect("TypeEnv should register proc::gather");
    let Type::Fn(params, ret) = ty else {
        panic!("expected function type");
    };

    assert_eq!(params.len(), 1);
    let item_ty = match &params[0] {
        Type::List(item) => match item.as_ref() {
            Type::Constructor { name, args, .. } if name.name == "P" && args.len() == 1 => {
                args[0].clone()
            }
            other => panic!("expected List<P<A>> input, got {other:?}"),
        },
        Type::Constructor { name, args, .. } if name.name == "List" && args.len() == 1 => {
            match &args[0] {
                Type::Constructor { name, args, .. } if name.name == "P" && args.len() == 1 => {
                    args[0].clone()
                }
                other => panic!("expected List<P<A>> input, got {other:?}"),
            }
        }
        other => panic!("expected List<P<A>> input, got {other:?}"),
    };

    match *ret {
        Type::Constructor { name, args, .. } if name.name == "Proc" && args.len() == 1 => {
            match &args[0] {
                Type::List(inner) => assert_eq!(inner.as_ref(), &item_ty),
                Type::Constructor { name, args, .. } if name.name == "List" && args.len() == 1 => {
                    assert_eq!(args[0], item_ty)
                }
                other => panic!("expected Proc<List<A>>, got {other:?}"),
            }
        }
        other => panic!("expected Proc<...>, got {other:?}"),
    }
}
