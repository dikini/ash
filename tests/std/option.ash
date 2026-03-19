-- Test Option type usage

workflow test_option_is_some {
    let some_val = Some { value: 42 };
    let none_val: Option<Int> = None;
    
    observe test with {} as _ {
        assert is_some(some_val);
        assert !is_some(none_val);
    }
    
    ret Done;
}

workflow test_option_is_none {
    let some_val = Some { value: 42 };
    let none_val: Option<Int> = None;
    
    observe test with {} as _ {
        assert !is_none(some_val);
        assert is_none(none_val);
    }
    
    ret Done;
}

workflow test_option_unwrap {
    let some_val = Some { value: 42 };
    
    observe test with {} as _ {
        assert unwrap(some_val) == 42;
    }
    
    ret Done;
}

workflow test_option_unwrap_or {
    let some_val = Some { value: 42 };
    let none_val: Option<Int> = None;
    
    observe test with {} as _ {
        assert unwrap_or(some_val, 0) == 42;
        assert unwrap_or(none_val, 0) == 0;
    }
    
    ret Done;
}

workflow test_option_map {
    let some_val = Some { value: 21 };
    let none_val: Option<Int> = None;
    
    let doubled = map(some_val, fn(x) => x * 2);
    let mapped_none = map(none_val, fn(x) => x * 2);
    
    observe test with {} as _ {
        assert unwrap(doubled) == 42;
        assert is_none(mapped_none);
    }
    
    ret Done;
}

workflow test_option_and {
    let a = Some { value: 1 };
    let b = Some { value: 2 };
    let n: Option<Int> = None;
    
    observe test with {} as _ {
        assert unwrap(and(a, b)) == 2;
        assert is_none(and(a, n));
        assert is_none(and(n, b));
        assert is_none(and(n, n));
    }
    
    ret Done;
}

workflow test_option_or {
    let a = Some { value: 1 };
    let b = Some { value: 2 };
    let n: Option<Int> = None;
    
    observe test with {} as _ {
        assert unwrap(or(a, b)) == 1;
        assert unwrap(or(a, n)) == 1;
        assert unwrap(or(n, b)) == 2;
        assert is_none(or(n, n));
    }
    
    ret Done;
}

workflow test_option_ok_or {
    let some_val = Some { value: 42 };
    let none_val: Option<Int> = None;
    
    let ok_res = ok_or(some_val, "error");
    let err_res = ok_or(none_val, "error");
    
    observe test with {} as _ {
        assert is_ok(ok_res);
        assert unwrap(ok_res) == 42;
        assert is_err(err_res);
    }
    
    ret Done;
}
