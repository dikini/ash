-- Proc library helpers
--
-- Phase 98 library surface for process-structured computations.
-- This slice now includes single-handle observation via `proc::await`
-- and cooperative scheduler yield via `proc::yield`.
-- Child process admission and wait-for-all observation remain owned by
-- later PLAN-098 tasks.

pub builtin fn unit<A>(v: A) -> Proc<A>;
pub builtin fn bind<A, B>(ma: Proc<A>, f: A -> Proc<B>) -> Proc<B>;
pub builtin fn then<A, B>(ma: Proc<A>, mb: Proc<B>) -> Proc<B>;
pub builtin fn await<A>(handle: P<A>) -> Proc<A>;
pub builtin fn yield() -> Proc<Unit>;
