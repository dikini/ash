-- Act monad helpers
--
-- Phase 97 library surface for effectful computations.
-- Runtime-managed substrate:
--   builtin type ActEnv
--   type Act<A> = ActEnv -> (ActEnv, A)

builtin type ActEnv;
type Act<A> = ActEnv -> (ActEnv, A);
pub type Policy = String;

builtin fn __unit<A>(v: A) -> Act<A>;
builtin fn __bind<A, B>(ma: Act<A>, f: A -> Act<B>) -> Act<B>;
builtin fn __then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B>;
builtin fn __fail<A>(error: String) -> Act<A>;
pub builtin fn policy_check(p: Policy) -> Bool;

pub fn unit<A>(v: A) -> Act<A> {
    __unit(v)
}

pub fn bind<A, B>(ma: Act<A>, f: A -> Act<B>) -> Act<B> {
    __bind(ma, f)
}

pub fn then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B> {
    __then(ma, mb)
}

pub fn guard<A>(p: Policy, ma: Act<A>) -> Act<A> {
    if act::policy_check(p) then
        ma
    else
        __fail("policy denied")
}
