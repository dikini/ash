-- Proc library helpers
--
-- Phase 98 library surface for process-structured computations.
-- This slice now includes single-handle observation via `proc::await`.
-- Child process admission, wait-for-all observation, and scheduler yield remain
-- owned by later PLAN-098 tasks.

pub builtin fn unit<A>(v: A) -> Proc<A>;
pub builtin fn bind<A, B>(ma: Proc<A>, f: A -> Proc<B>) -> Proc<B>;
pub builtin fn then<A, B>(ma: Proc<A>, mb: Proc<B>) -> Proc<B>;
pub builtin fn await<A>(handle: P<A>) -> Proc<A>;
