-- Act monad helpers
--
-- Phase 97 library surface for effectful computations.
-- Runtime-managed substrate: hidden ActEnv is compiler/runtime-owned and not source-denotable.
-- Public code sees only opaque Act<A> plus the operations below.

pub type Policy = String;

builtin fn __unit<A>(v: A) -> Act<A>;
builtin fn __bind<A, B>(ma: Act<A>, f: A -> Act<B>) -> Act<B>;
builtin fn __then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B>;
builtin fn __fail<A>(error: String) -> Act<A>;
builtin fn __guard<A>(p: String, ma: Act<A>) -> Act<A>;
pub builtin fn policy_check(p: String) -> Bool;

pub fn unit<A>(v: A) -> Act<A> {
    __unit(v)
}

pub fn bind<A, B>(ma: Act<A>, f: A -> Act<B>) -> Act<B> {
    __bind(ma, f)
}

pub fn then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B> {
    __then(ma, mb)
}

pub fn guard<A>(p: String, ma: Act<A>) -> Act<A> {
    act::__guard(p, ma)
}
