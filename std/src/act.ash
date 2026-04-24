-- Act monad helpers
--
-- Phase 97 library surface for effectful computations.
-- These remain ordinary Ash functions in the stdlib surface.

pub fn unit<A>(value: A) -> Act<A> {
    |env| => Ok { value: (value, env) }
}

pub fn bind<A, B>(ma: Act<A>, f: Fn(A) -> Act<B>) -> Act<B> {
    |env| => match ma(env) {
        Ok { value: (a, next_env) } => f(a)(next_env),
        Err { error: e } => Err { error: e }
    }
}

pub fn then<A, B>(ma: Act<A>, mb: Act<B>) -> Act<B> {
    bind(ma, |_a| => mb)
}

pub fn guard<A>(policy: Policy, ma: Act<A>) -> Act<A> {
    |env| => match env.policies.check(policy) {
        Deny(reason) => Err { error: PolicyViolation(reason) },
        Allow => ma(env)
    }
}
