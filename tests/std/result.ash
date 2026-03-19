-- Test Result type usage

workflow test_result_is_ok {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert is_ok(ok_val);
        assert !is_ok(err_val);
    }
    
    ret Done;
}

workflow test_result_is_err {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert !is_err(ok_val);
        assert is_err(err_val);
    }
    
    ret Done;
}

workflow test_result_unwrap {
    let ok_val = Ok { value: 42 };
    
    observe test with {} as _ {
        assert unwrap(ok_val) == 42;
    }
    
    ret Done;
}

workflow test_result_unwrap_err {
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert unwrap_err(err_val) == "oops";
    }
    
    ret Done;
}

workflow test_result_unwrap_or {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert unwrap_or(ok_val, 0) == 42;
        assert unwrap_or(err_val, 0) == 0;
    }
    
    ret Done;
}

workflow test_result_map {
    let ok_val = Ok { value: 21 };
    let err_val = Err { error: "oops" };
    
    let doubled = map(ok_val, fn(x) => x * 2);
    let mapped_err = map(err_val, fn(x) => x * 2);
    
    observe test with {} as _ {
        assert unwrap(doubled) == 42;
        assert is_err(mapped_err);
    }
    
    ret Done;
}

workflow test_result_map_err {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    let prefix_err = map_err(err_val, fn(e) => "error: " ++ e);
    let mapped_ok = map_err(ok_val, fn(e) => "error: " ++ e);
    
    observe test with {} as _ {
        assert is_ok(mapped_ok);
        assert is_err(prefix_err);
        assert unwrap_err(prefix_err) == "error: oops";
    }
    
    ret Done;
}

workflow test_result_and_then {
    let ok_val = Ok { value: 21 };
    let err_val = Err { error: "oops" };
    
    let doubled = and_then(ok_val, fn(x) => Ok { value: x * 2 });
    let chained_err = and_then(err_val, fn(x) => Ok { value: x * 2 });
    
    observe test with {} as _ {
        assert unwrap(doubled) == 42;
        assert is_err(chained_err);
    }
    
    ret Done;
}

workflow test_result_ok {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert is_some(ok(ok_val));
        assert unwrap(ok(ok_val)) == 42;
        assert is_none(ok(err_val));
    }
    
    ret Done;
}

workflow test_result_err {
    let ok_val = Ok { value: 42 };
    let err_val = Err { error: "oops" };
    
    observe test with {} as _ {
        assert is_none(err(ok_val));
        assert is_some(err(err_val));
        assert unwrap(err(err_val)) == "oops";
    }
    
    ret Done;
}
