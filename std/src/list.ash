-- List operations implemented in pure Ash
-- List<T> is defined as an algebraic data type: Nil | Cons { head: T, tail: List<T> }
--
-- Previously these were Rust builtins. Now they are ordinary Ash functions.

pub fn len<a>(list: List<a>) -> Int {
    match list {
        [] => 0,
        [_, ..rest] => 1 + len(rest)
    }
}

pub fn head<a>(list: List<a>) -> a {
    match list {
        [h, .._] => h,
        [] => panic "head of empty list"
    }
}

pub fn tail<a>(list: List<a>) -> List<a> {
    match list {
        [_, ..rest] => rest,
        [] => panic "tail of empty list"
    }
}

pub fn append<a>(list: List<a>, item: a) -> List<a> {
    match list {
        [] => [item],
        [h, ..rest] => Cons { head: h, tail: append(rest, item) }
    }
}

pub fn concat<a>(left: List<a>, right: List<a>) -> List<a> {
    match left {
        [] => right,
        [h, ..rest] => Cons { head: h, tail: concat(rest, right) }
    }
}

pub fn map<a, b>(list: List<a>, f: (a) -> b) -> List<b> {
    match list {
        [] => [],
        [h, ..rest] => Cons { head: f(h), tail: map(rest, f) }
    }
}

pub fn filter<a>(list: List<a>, pred: (a) -> Bool) -> List<a> {
    match list {
        [] => [],
        [h, ..rest] =>
            if pred(h) then Cons { head: h, tail: filter(rest, pred) }
            else filter(rest, pred)
    }
}

pub fn index<a>(list: List<a>, n: Int) -> a {
    match list {
        [] => panic "index out of bounds",
        [h, ..rest] =>
            if n == 0 then h
            else index(rest, n - 1)
    }
}

pub fn take<a>(n: Int, list: List<a>) -> List<a> {
    if n <= 0 then []
    else match list {
        [] => [],
        [h, ..rest] => Cons { head: h, tail: take(n - 1, rest) }
    }
}

pub fn drop<a>(n: Int, list: List<a>) -> List<a> {
    if n <= 0 then list
    else match list {
        [] => [],
        [_, ..rest] => drop(n - 1, rest)
    }
}

pub fn reverse<a>(list: List<a>) -> List<a> {
    reverse_helper(list, [])
}

fn reverse_helper<a>(list: List<a>, acc: List<a>) -> List<a> {
    match list {
        [] => acc,
        [h, ..rest] => reverse_helper(rest, Cons { head: h, tail: acc })
    }
}

pub fn prepend<a>(item: a, list: List<a>) -> List<a> {
    Cons { head: item, tail: list }
}

pub fn is_empty<a>(list: List<a>) -> Bool {
    match list {
        [] => true,
        _ => false
    }
}

-- Algebraic structure instances

pub fn list_functor_map<a, b>(list: List<a>, f: (a) -> b) -> List<b> {
    map(list, f)
}

pub fn list_semigroup_append<a>(left: List<a>, right: List<a>) -> List<a> {
    concat(left, right)
}

pub fn list_monoid_empty<a>() -> List<a> {
    []
}
